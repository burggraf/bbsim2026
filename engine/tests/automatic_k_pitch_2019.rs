use bbsim_engine::accounting::{derive_accounting, BatterLine, FielderLine, PitcherLine};
use bbsim_engine::{
    apply, Base, Credit, CreditKind, DeliveryContext, Delta, Destination, Event, EventKind,
    EventTransition, FieldingReview, GameState, Half, HandlingAdvance, HandlingAssessment,
    Participants, PreDeliverySteal, RecoveryReview, Runner, RunnerMove, SingleDeliveryFacts,
    TeamState, ThirdOut, ThirdOutKind, TransitionError,
};

fn credit(id: &str, kind: CreditKind) -> Credit {
    Credit {
        player_id: id.into(),
        kind,
    }
}

fn fixture(half: Half, outs: u8, mask: u8, advancing: u8, passed: bool) -> (GameState, Event) {
    let teams = ["away", "home"].map(|prefix| TeamState {
        batting_order: std::array::from_fn(|i| format!("{prefix}{i}")),
        next_batter: 0,
        pitcher: format!("{prefix}_pitcher"),
        dh_slot: Some(8),
        dh_must_bat: false,
        bench: vec![format!("{prefix}_relief")],
        removed: vec![],
    });
    let offense = half.offense();
    let pitcher = teams[1 - offense].pitcher.clone();
    let batter = teams[offense].batting_order[0].clone();
    let catcher = teams[1 - offense].batting_order[1].clone();
    let before = GameState {
        inning: 2,
        half,
        outs,
        score: [0, 0],
        over: false,
        bases: std::array::from_fn(|i| {
            (mask & (1 << i) != 0).then(|| Runner {
                player_id: teams[offense].batting_order[5 + i].clone(),
                responsible_pitcher: pitcher.clone(),
            })
        }),
        teams,
    };
    let mut event = Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(batter.clone()),
            pitcher: pitcher.clone(),
        },
        fielding_review: if passed {
            FieldingReview::Unknown
        } else {
            FieldingReview::NoMisplay
        },
        two_out_contact: None,
        balk: None,
        single_delivery: Some(SingleDeliveryFacts {
            context: DeliveryContext::UncaughtThirdStrike,
            handling: if passed {
                HandlingAssessment::CatcherPassed
            } else {
                HandlingAssessment::PitcherWild
            },
            catcher_id: catcher.clone(),
            advances: vec![],
            pre_delivery_steals: Some(vec![]),
            recovery: RecoveryReview::NoRecoveryPutout,
        }),
        delta: Delta {
            moves: vec![RunnerMove {
                runner: Runner {
                    player_id: batter.clone(),
                    responsible_pitcher: pitcher.clone(),
                },
                from: Base::Plate,
                to: Destination::Out,
            }],
            outs: 1,
            runs: 0,
            changes: vec![],
            third_out: None,
            credits: vec![
                credit(&batter, CreditKind::PlateAppearance),
                credit(&batter, CreditKind::Strikeout),
                credit(
                    if passed { &catcher } else { &pitcher },
                    if passed {
                        CreditKind::PassedBall
                    } else {
                        CreditKind::WildPitch
                    },
                ),
            ],
        },
    };
    for i in 0..3 {
        if advancing & (1 << i) != 0 {
            let runner = before.bases[i]
                .clone()
                .expect("advancing actor must occupy before base");
            let to = [Destination::Second, Destination::Third, Destination::Home][i];
            event
                .single_delivery
                .as_mut()
                .unwrap()
                .advances
                .push(HandlingAdvance {
                    runner_id: runner.player_id.clone(),
                    reached: to,
                });
            if to == Destination::Home {
                event.delta.runs += 1;
                event
                    .delta
                    .credits
                    .push(credit(&runner.player_id, CreditKind::Run));
            }
            event.delta.moves.push(RunnerMove {
                runner,
                from: [Base::First, Base::Second, Base::Third][i],
                to,
            });
        }
    }
    (before, event)
}

// Build only the unchanged physical control, never normalize a disputed event.
fn physical_after(before: &GameState, event: &Event) -> GameState {
    let mut control = event.clone();
    control.single_delivery = None;
    control
        .delta
        .credits
        .retain(|c| !matches!(c.kind, CreditKind::WildPitch | CreditKind::PassedBall));
    let transition = apply(before, control.clone()).expect("legacy physical control must be legal");
    assert_eq!(
        EventTransition::try_new(before.clone(), control, transition.after().clone()).unwrap(),
        transition
    );
    assert_eq!(
        EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
        transition
    );
    transition.after().clone()
}

fn accepted(before: &GameState, event: Event) -> EventTransition {
    let saved = before.clone();
    let after = physical_after(before, &event);
    let reduced = apply(before, event.clone());
    let direct = EventTransition::try_new(before.clone(), event, after);
    assert_eq!(*before, saved);
    assert_eq!(
        (reduced.as_ref().map(|_| ()), direct.as_ref().map(|_| ())),
        (Ok(()), Ok(()))
    );
    let reduced = reduced.unwrap();
    assert_eq!(direct.unwrap(), reduced);
    assert_eq!(&reduced.to_bytes()[..5], b"BBE17");
    assert_eq!(
        EventTransition::from_bytes(&reduced.to_bytes()).unwrap(),
        reduced
    );
    reduced
}

fn rejected(before: &GameState, event: Event, after: GameState, error: TransitionError) {
    let saved = before.clone();
    let reduced = apply(before, event.clone()).map(|_| ());
    let direct = EventTransition::try_new(before.clone(), event, after).map(|_| ());
    assert_eq!(*before, saved);
    assert_eq!((reduced, direct), (Err(error.clone()), Err(error)));
}

fn positive_matrix(passed: bool) {
    let mut frames = 0;
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..2 {
            for mask in [1, 3, 5, 7] {
                for advancing in 1..8 {
                    if advancing & !mask != 0
                        || (0..2).any(|i| {
                            advancing & (1 << i) != 0
                                && mask & (1 << (i + 1)) != 0
                                && advancing & (1 << (i + 1)) == 0
                        })
                    {
                        continue;
                    }
                    for reverse in [false, true] {
                        let (before, mut event) = fixture(half, outs, mask, advancing, passed);
                        let pitcher = event.participants.pitcher.clone();
                        let batter = event.participants.batter.clone().unwrap();
                        let catcher = event.single_delivery.as_ref().unwrap().catcher_id.clone();
                        event
                            .delta
                            .credits
                            .push(credit(&catcher, CreditKind::Putout));
                        if reverse {
                            event.delta.moves.reverse();
                            event.delta.credits.reverse();
                            event.single_delivery.as_mut().unwrap().advances.reverse();
                        }
                        let transition = accepted(&before, event);
                        assert_eq!(transition.after().outs, outs + 1);
                        assert_eq!(transition.after().teams[half.offense()].next_batter, 1);
                        let accounting =
                            derive_accounting(std::slice::from_ref(&transition)).unwrap();
                        assert_eq!(
                            accounting.batters[half.offense()][&batter],
                            BatterLine {
                                plate_appearances: 1,
                                credited_strikeouts: 1,
                                at_bats: Some(1),
                                ..BatterLine::default()
                            }
                        );
                        let runs = u32::from(advancing & 4 != 0);
                        assert_eq!(
                            accounting.pitchers[1 - half.offense()][&pitcher],
                            PitcherLine {
                                outs_recorded: 1,
                                batters_faced: 1,
                                at_bats_against: Some(1),
                                runs_allowed: runs,
                                earned_runs: if runs == 0 { Some(0) } else { None },
                                credited_strikeouts: 1,
                                credited_wild_pitches: u32::from(!passed),
                                ..PitcherLine::default()
                            }
                        );
                        assert_eq!(
                            accounting.fielders[1 - half.offense()][&catcher],
                            FielderLine {
                                credited_putouts: 1,
                                credited_passed_balls: u32::from(passed),
                                ..FielderLine::default()
                            }
                        );
                        assert_eq!(accounting.runs[half.offense()], runs);
                        assert_eq!(accounting.innings[&2][half.offense()], Some(runs as u16));
                        assert!(!accounting.complete_game);
                        assert!(!accounting.batters[half.offense()].contains_key(&pitcher));
                        assert!(!accounting.batters[half.offense()].contains_key(&catcher));
                        frames += 1;
                    }
                }
            }
        }
    }
    assert_eq!(frames, 72);
}

#[test]
fn automatic_k_wp_has_one_current_pitcher_unit_and_exact_literal_participation() {
    positive_matrix(false);
}

#[test]
fn automatic_k_pb_has_one_actual_catcher_unit_and_exact_literal_participation() {
    positive_matrix(true);
}

#[test]
fn missing_causal_witness_preserves_positive_uncertainty_and_no_unit_partial_view() {
    for passed in [false, true] {
        let (before, mut event) = fixture(Half::Top, 0, 1, 1, passed);
        let after = physical_after(&before, &event);
        event.single_delivery.as_mut().unwrap().advances.clear();
        rejected(
            &before,
            event.clone(),
            after.clone(),
            TransitionError::UnsupportedRule,
        );
        event
            .delta
            .credits
            .retain(|c| !matches!(c.kind, CreditKind::WildPitch | CreditKind::PassedBall));
        assert_eq!(accepted(&before, event).after(), &after);
    }
}

fn without_units(event: &mut Event) {
    event
        .delta
        .credits
        .retain(|c| !matches!(c.kind, CreditKind::WildPitch | CreditKind::PassedBall));
}

fn missing_review(event: &mut Event, missing: u8) {
    let facts = event.single_delivery.as_mut().unwrap();
    if missing != 0 {
        facts.recovery = RecoveryReview::Unknown;
    }
    if missing == 2 {
        facts.pre_delivery_steals = None;
    }
}

#[test]
fn automatic_batter_cannot_claim_a_safe_handling_acquisition_despite_missing_reviews() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..2 {
            for passed in [false, true] {
                for missing in 0..3 {
                    for reached in [
                        Destination::First,
                        Destination::Second,
                        Destination::Third,
                        Destination::Home,
                    ] {
                        let (before, mut event) = fixture(half, outs, 1, 1, passed);
                        let after = physical_after(&before, &event);
                        missing_review(&mut event, missing);
                        event
                            .single_delivery
                            .as_mut()
                            .unwrap()
                            .advances
                            .push(HandlingAdvance {
                                runner_id: event.participants.batter.clone().unwrap(),
                                reached,
                            });
                        rejected(&before, event, after, TransitionError::IllegalCredit);
                    }
                }
            }
        }
    }
}

#[test]
fn qualified_automatic_unit_omission_is_known_even_with_incomplete_steal_review() {
    for passed in [false, true] {
        for missing_steals in [false, true] {
            let (before, mut event) = fixture(Half::Top, 0, 1, 1, passed);
            event.delta.moves[1].to = Destination::Third;
            event.single_delivery.as_mut().unwrap().advances[0].reached = Destination::Third;
            if missing_steals {
                event.single_delivery.as_mut().unwrap().pre_delivery_steals = None;
            }
            let after = physical_after(&before, &event);
            without_units(&mut event);
            rejected(&before, event, after, TransitionError::IllegalCredit);
        }
    }
}

#[test]
fn automatic_unit_actor_opposite_duplicate_and_pb_review_errors_precede_missing_recovery() {
    for passed in [false, true] {
        for malformed in 0..4 {
            let (before, mut event) = fixture(Half::Top, 0, 1, 1, passed);
            let after = physical_after(&before, &event);
            missing_review(&mut event, 2);
            match malformed {
                0 => event.delta.credits[2].player_id = "wrong_fielder".into(),
                1 => {
                    let repeated = event.delta.credits[2].clone();
                    event.delta.credits.push(repeated);
                }
                2 => {
                    event.delta.credits[2] = credit(
                        if passed {
                            &event.participants.pitcher
                        } else {
                            &event.single_delivery.as_ref().unwrap().catcher_id
                        },
                        if passed {
                            CreditKind::WildPitch
                        } else {
                            CreditKind::PassedBall
                        },
                    );
                }
                3 => {
                    let facts = event.single_delivery.as_mut().unwrap();
                    facts.handling = HandlingAssessment::CatcherPassed;
                    event.delta.credits[2] = credit(&facts.catcher_id, CreditKind::PassedBall);
                    event.fielding_review = FieldingReview::NoMisplay;
                }
                _ => unreachable!(),
            }
            rejected(&before, event, after, TransitionError::IllegalCredit);
        }
    }
}

fn known_errors(cases: &[u8]) {
    for passed in [false, true] {
        for case in cases {
            let (mut before, mut event) =
                fixture(Half::Top, 0, 5, if *case == 13 { 5 } else { 1 }, passed);
            if *case == 10 {
                before.bases[0].as_mut().unwrap().responsible_pitcher = "carried_prior".into();
                event.delta.moves[1].runner = before.bases[0].clone().unwrap();
            }
            let after = physical_after(&before, &event);
            missing_review(&mut event, 2);
            let error = match case {
                0 => {
                    before.inning = 0;
                    TransitionError::InvalidState
                }
                1 => {
                    before.outs = 3;
                    TransitionError::InvalidState
                }
                2 => {
                    before.over = true;
                    TransitionError::InvalidState
                }
                3 => {
                    before.bases[0].as_mut().unwrap().player_id = "not_in_lineup".into();
                    TransitionError::InvalidState
                }
                4 => {
                    event.participants.pitcher = "not_current".into();
                    TransitionError::InvalidParticipants
                }
                5 => {
                    event.participants.batter = None;
                    TransitionError::InvalidParticipants
                }
                6 => {
                    event
                        .delta
                        .changes
                        .push(bbsim_engine::PersonnelChange::Pitcher {
                            team: if passed {
                                bbsim_engine::TeamSide::Home
                            } else {
                                bbsim_engine::TeamSide::Away
                            },
                            outgoing: "old".into(),
                            incoming: "new".into(),
                        });
                    TransitionError::IllegalStateChange
                }
                7 => {
                    let repeated = event.delta.moves[1].clone();
                    event.delta.moves.push(repeated);
                    TransitionError::DeltaMismatch
                }
                8 => {
                    event.delta.moves.remove(0);
                    TransitionError::DeltaMismatch
                }
                9 => {
                    event.delta.moves[1].runner.player_id = "wrong_runner".into();
                    TransitionError::DeltaMismatch
                }
                10 => {
                    event.delta.moves[1].runner.responsible_pitcher =
                        event.participants.pitcher.clone();
                    TransitionError::DeltaMismatch
                }
                11 => {
                    event.delta.moves[0].runner.responsible_pitcher = "not_current_charge".into();
                    TransitionError::DeltaMismatch
                }
                12 => {
                    event.delta.outs = 0;
                    TransitionError::DeltaMismatch
                }
                13 => {
                    event
                        .delta
                        .credits
                        .iter_mut()
                        .find(|c| c.kind == CreditKind::Run)
                        .unwrap()
                        .player_id = event.participants.batter.clone().unwrap();
                    TransitionError::IllegalCredit
                }
                14 => {
                    event.delta.runs = 1;
                    TransitionError::DeltaMismatch
                }
                15 => {
                    event.delta.moves[1].to = Destination::Third;
                    event.single_delivery.as_mut().unwrap().advances[0].reached =
                        Destination::Third;
                    TransitionError::DeltaMismatch // before Third is held by a different runner
                }
                16 => {
                    event.delta.third_out = Some(ThirdOut {
                        runner_id: event.participants.batter.clone().unwrap(),
                        kind: ThirdOutKind::Timing,
                        home_before_out: vec![],
                    });
                    TransitionError::IllegalStateChange // represented total is only one
                }
                _ => unreachable!(),
            };
            rejected(&before, event, after, error);
        }
    }
}

#[test]
fn malformed_before_state_is_not_masked_by_missing_automatic_delivery_review() {
    known_errors(&[0, 1, 2, 3]);
}
#[test]
fn participants_and_pa_personnel_shape_remain_known_before_missing_review() {
    known_errors(&[4, 5, 6]);
}
#[test]
fn canonical_origins_and_actual_carried_charges_precede_missing_review() {
    known_errors(&[7, 8, 9, 10, 11]);
}
#[test]
fn represented_outs_and_named_run_conservation_precede_missing_review() {
    known_errors(&[12, 13, 14, 16]);
}
#[test]
fn final_destination_collision_remains_a_known_delta_error() {
    known_errors(&[15]);
}

#[test]
fn an_empty_causal_vector_does_not_certify_no_intermediate_advance_and_return() {
    for passed in [false, true] {
        let (before, mut event) = fixture(Half::Top, 1, 1, 0, passed);
        let after = physical_after(&before, &event);
        rejected(
            &before,
            event.clone(),
            after.clone(),
            TransitionError::UnsupportedRule,
        );
        without_units(&mut event);
        assert_eq!(accepted(&before, event).after(), &after);
    }
}

#[test]
fn independent_qualifying_cause_can_bind_the_episode_without_inventing_another_runner_cause() {
    for passed in [false, true] {
        let (before, mut event) = fixture(Half::Top, 0, 5, 5, passed);
        let first = before.bases[0].as_ref().unwrap().player_id.clone();
        event
            .single_delivery
            .as_mut()
            .unwrap()
            .advances
            .retain(|a| a.runner_id != first);
        accepted(&before, event);
    }
}

#[test]
fn explicit_intermediate_reach_cannot_be_hidden_by_another_qualifier() {
    for passed in [false, true] {
        let (before, mut event) = fixture(Half::Top, 0, 3, 3, passed);
        event
            .delta
            .moves
            .iter_mut()
            .find(|m| m.from == Base::Second)
            .unwrap()
            .to = Destination::Home;
        event.delta.runs = 1;
        event.delta.credits.push(credit(
            &before.bases[1].as_ref().unwrap().player_id,
            CreditKind::Run,
        ));
        let after = physical_after(&before, &event);
        // First→Second independently qualifies, but the asserted Third reach lacks Home chronology.
        rejected(
            &before,
            event.clone(),
            after.clone(),
            TransitionError::UnsupportedRule,
        );
        event.delta.moves.reverse();
        event.single_delivery.as_mut().unwrap().advances.reverse();
        rejected(&before, event, after, TransitionError::UnsupportedRule);
    }
}

#[test]
fn reviewed_steal_only_suppression_and_another_runner_qualifier_stay_distinct() {
    for passed in [false, true] {
        for another in [false, true] {
            let (before, mut event) = fixture(
                Half::Top,
                0,
                if another { 5 } else { 1 },
                if another { 5 } else { 1 },
                passed,
            );
            let first = before.bases[0].as_ref().unwrap().player_id.clone();
            event.single_delivery.as_mut().unwrap().pre_delivery_steals =
                Some(vec![PreDeliverySteal {
                    runner_id: first.clone(),
                    target: Destination::Second,
                }]);
            event
                .delta
                .credits
                .push(credit(&first, CreditKind::StolenBase));
            let after = physical_after(&before, &event);
            if !another {
                rejected(
                    &before,
                    event.clone(),
                    after.clone(),
                    TransitionError::IllegalCredit,
                );
                without_units(&mut event);
            }
            let accounting = derive_accounting(&[accepted(&before, event)]).unwrap();
            assert_eq!(
                accounting.batters[Half::Top.offense()][&first].credited_steals,
                1
            );
        }
    }
}

#[test]
fn unrelated_earlier_error_does_not_erase_qualified_cause_or_required_unit() {
    for passed in [false, true] {
        let (before, mut event) = fixture(Half::Top, 0, 1, 1, passed);
        event.fielding_review = FieldingReview::Unknown;
        let fielder = before.teams[1].batting_order[2].clone();
        event
            .delta
            .credits
            .push(credit(&fielder, CreditKind::Error));
        let transition = accepted(&before, event.clone());
        assert_eq!(
            derive_accounting(&[transition]).unwrap().fielders[1][&fielder].credited_errors,
            1
        );
        let after = physical_after(&before, &event);
        without_units(&mut event);
        rejected(&before, event, after, TransitionError::IllegalCredit);
    }
}

#[test]
fn inherited_scorer_and_current_delivery_participation_have_literal_separate_lines() {
    for half in [Half::Top, Half::Bottom] {
        for passed in [false, true] {
            let (mut before, mut event) = fixture(half, 1, 5, 5, passed);
            before.bases[2].as_mut().unwrap().responsible_pitcher = "prior_pitcher".into();
            event
                .delta
                .moves
                .iter_mut()
                .find(|m| m.from == Base::Third)
                .unwrap()
                .runner = before.bases[2].clone().unwrap();
            let pitcher = event.participants.pitcher.clone();
            let accounting = derive_accounting(&[accepted(&before, event)]).unwrap();
            assert_eq!(
                accounting.pitchers[1 - half.offense()]["prior_pitcher"],
                PitcherLine {
                    runs_allowed: 1,
                    earned_runs: None,
                    ..PitcherLine::default()
                }
            );
            assert_eq!(
                accounting.pitchers[1 - half.offense()][&pitcher],
                PitcherLine {
                    outs_recorded: 1,
                    batters_faced: 1,
                    at_bats_against: Some(1),
                    credited_strikeouts: 1,
                    credited_wild_pitches: u32::from(!passed),
                    ..PitcherLine::default()
                }
            );
        }
    }
}

#[test]
fn ordinary_single_winning_run_and_extra_innings_use_existing_ending_logic() {
    for inning in [9, 10] {
        for passed in [false, true] {
            let (mut before, event) = fixture(Half::Bottom, 0, 5, 4, passed);
            before.inning = inning;
            let transition = accepted(&before, event.clone());
            assert!(transition.after().over);
            assert_eq!(transition.after().score, [0, 1]);
            assert_eq!(transition.after().outs, 1);
            before.score = [2, 0];
            assert!(!accepted(&before, event).after().over);
        }
    }
}

#[test]
fn eligible_retired_and_mixed_recovery_histories_are_not_automatic_safe_reach_contradictions() {
    for passed in [false, true] {
        let (before, mut eligible) = fixture(Half::Top, 0, 4, 4, passed);
        let after = physical_after(&before, &eligible);
        eligible
            .single_delivery
            .as_mut()
            .unwrap()
            .advances
            .push(HandlingAdvance {
                runner_id: eligible.participants.batter.clone().unwrap(),
                reached: Destination::First,
            });
        rejected(&before, eligible, after, TransitionError::UnsupportedRule);

        let (before, mut mixed) = fixture(Half::Top, 0, 5, 5, passed);
        mixed.delta.moves[1].to = Destination::Out;
        mixed.delta.outs = 2;
        let retired = before.bases[0].as_ref().unwrap().player_id.clone();
        let advancer = before.bases[2].as_ref().unwrap().player_id.clone();
        let facts = mixed.single_delivery.as_mut().unwrap();
        facts.advances.retain(|a| a.runner_id != retired);
        facts.recovery = RecoveryReview::AfterAdvance {
            runner_id: retired,
            advancer_id: advancer,
        };
        let after = physical_after(&before, &mixed);
        rejected(&before, mixed, after, TransitionError::UnsupportedRule);
    }
}

#[test]
fn missing_recovery_or_sole_next_base_steal_review_remains_unknown_not_zero() {
    for passed in [false, true] {
        for missing in [0, 1] {
            let (before, mut event) = fixture(Half::Top, 0, 1, 1, passed);
            if missing == 0 {
                event.single_delivery.as_mut().unwrap().recovery = RecoveryReview::Unknown;
            } else {
                event.single_delivery.as_mut().unwrap().pre_delivery_steals = None;
            }
            let after = physical_after(&before, &event);
            rejected(
                &before,
                event.clone(),
                after.clone(),
                TransitionError::UnsupportedRule,
            );
            without_units(&mut event);
            assert_eq!(accepted(&before, event).after(), &after);
        }
    }
}

#[test]
fn two_legal_simultaneous_k_delivery_episodes_accumulate_without_cross_event_deduplication() {
    for half in [Half::Top, Half::Bottom] {
        for passed in [false, true] {
            let (before, first_event) = fixture(half, 0, 5, 4, passed);
            let pitcher = first_event.participants.pitcher.clone();
            let catcher = first_event
                .single_delivery
                .as_ref()
                .unwrap()
                .catcher_id
                .clone();
            let first = accepted(&before, first_event);
            let next_before = first.after().clone();
            let (_, mut next_event) = fixture(half, 1, 1, 1, passed);
            let batter = next_before.teams[half.offense()].batting_order[1].clone();
            next_event.participants.batter = Some(batter.clone());
            next_event.delta.moves[0].runner.player_id = batter.clone();
            for c in &mut next_event.delta.credits {
                if matches!(c.kind, CreditKind::PlateAppearance | CreditKind::Strikeout) {
                    c.player_id = batter.clone();
                }
            }
            let second = accepted(&next_before, next_event);
            second.follows(&first).unwrap();
            let accounting = derive_accounting(&[first, second]).unwrap();
            assert_eq!(
                accounting.pitchers[1 - half.offense()][&pitcher],
                PitcherLine {
                    outs_recorded: 2,
                    batters_faced: 2,
                    at_bats_against: Some(2),
                    credited_strikeouts: 2,
                    credited_wild_pitches: if passed { 0 } else { 2 },
                    runs_allowed: 1,
                    earned_runs: None,
                    ..PitcherLine::default()
                }
            );
            if passed {
                assert_eq!(
                    accounting.fielders[1 - half.offense()][&catcher].credited_passed_balls,
                    2
                );
            }
        }
    }
}

fn wire_string(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}
fn wire_destination(value: Destination) -> u8 {
    match value {
        Destination::First => 0,
        Destination::Second => 1,
        Destination::Third => 2,
        Destination::Home => 3,
        Destination::Out => 4,
    }
}
fn wire_delivery(facts: &SingleDeliveryFacts) -> Vec<u8> {
    let mut bytes = vec![
        1,
        1,
        if facts.handling == HandlingAssessment::PitcherWild {
            0
        } else {
            1
        },
    ];
    assert_eq!(facts.context, DeliveryContext::UncaughtThirdStrike);
    wire_string(&mut bytes, &facts.catcher_id);
    bytes.extend_from_slice(&(facts.advances.len() as u64).to_le_bytes());
    for a in &facts.advances {
        wire_string(&mut bytes, &a.runner_id);
        bytes.push(wire_destination(a.reached));
    }
    match &facts.pre_delivery_steals {
        None => bytes.push(0),
        Some(starts) => {
            bytes.push(1);
            bytes.extend_from_slice(&(starts.len() as u64).to_le_bytes());
            for s in starts {
                wire_string(&mut bytes, &s.runner_id);
                bytes.push(wire_destination(s.target));
            }
        }
    }
    match &facts.recovery {
        RecoveryReview::Unknown => bytes.push(0),
        RecoveryReview::NoRecoveryPutout => bytes.push(1),
        RecoveryReview::BeforeAnyAdvance { runner_id } => {
            bytes.push(2);
            wire_string(&mut bytes, runner_id);
        }
        RecoveryReview::AfterAdvance {
            runner_id,
            advancer_id,
        } => {
            bytes.push(3);
            wire_string(&mut bytes, runner_id);
            wire_string(&mut bytes, advancer_id);
        }
        RecoveryReview::RecoveredThirdStrikeBatterBeforeFirst => bytes.push(4),
    }
    bytes
}
// Complete current-layout event selection, not an ambiguous singleton enum/string needle.
fn wire_event(event: &Event) -> Vec<u8> {
    assert_eq!(event.kind, EventKind::PlateAppearance);
    assert!(
        event.delta.changes.is_empty()
            && event.delta.third_out.is_none()
            && event.two_out_contact.is_none()
            && event.balk.is_none()
    );
    let mut bytes = vec![0]; // PlateAppearance
    match &event.participants.batter {
        Some(id) => {
            bytes.push(1);
            wire_string(&mut bytes, id);
        }
        None => bytes.push(0),
    }
    wire_string(&mut bytes, &event.participants.pitcher);
    bytes.push(u8::from(event.fielding_review == FieldingReview::NoMisplay));
    bytes.extend_from_slice(&(event.delta.moves.len() as u64).to_le_bytes());
    for m in &event.delta.moves {
        wire_string(&mut bytes, &m.runner.player_id);
        wire_string(&mut bytes, &m.runner.responsible_pitcher);
        bytes.push(match m.from {
            Base::Plate => 0,
            Base::First => 1,
            Base::Second => 2,
            Base::Third => 3,
        });
        bytes.push(wire_destination(m.to));
    }
    bytes.extend_from_slice(&[event.delta.outs, event.delta.runs]);
    bytes.extend_from_slice(&(event.delta.credits.len() as u64).to_le_bytes());
    for c in &event.delta.credits {
        wire_string(&mut bytes, &c.player_id);
        bytes.push(match c.kind {
            CreditKind::PlateAppearance => 0,
            CreditKind::Strikeout => 1,
            CreditKind::Run => 4,
            CreditKind::StolenBase => 6,
            CreditKind::Putout => 8,
            CreditKind::Error => 10,
            CreditKind::PassedBall => 11,
            CreditKind::WildPitch => 23,
            _ => panic!("credit outside bounded automatic-K wire fixtures"),
        });
    }
    bytes.extend_from_slice(&0u64.to_le_bytes()); // no changes
    bytes.extend_from_slice(&[0, 0]); // no third_out/contact
    match &event.single_delivery {
        Some(facts) => bytes.extend_from_slice(&wire_delivery(facts)),
        None => bytes.push(0),
    }
    bytes.push(0); // no balk
    bytes
}
fn unique_offset(bytes: &[u8], needle: &[u8]) -> usize {
    let offsets: Vec<_> = bytes
        .windows(needle.len())
        .enumerate()
        .filter_map(|(i, w)| (w == needle).then_some(i))
        .collect();
    assert_eq!(
        offsets.len(),
        1,
        "complete fixture record must be uniquely selected"
    );
    offsets[0]
}
fn forged(before: &GameState, original: &Event, changed: &Event) -> Vec<u8> {
    let mut control = original.clone();
    control.single_delivery = None;
    without_units(&mut control);
    let transition = apply(before, control).expect("original physical control must validate");
    let mut bytes = transition.to_bytes();
    assert_eq!(EventTransition::from_bytes(&bytes).unwrap(), transition);
    let old = wire_event(transition.event());
    let offset = unique_offset(&bytes, &old);
    bytes.splice(offset..offset + old.len(), wire_event(changed));
    bytes
}

#[test]
fn complete_record_semantic_forgeries_share_automatic_touch_unit_and_origin_guards() {
    for passed in [false, true] {
        for malformed in 0..5 {
            let (before, original) = fixture(Half::Top, 0, 1, 1, passed);
            let mut event = original.clone();
            let error = match malformed {
                0 => {
                    event
                        .single_delivery
                        .as_mut()
                        .unwrap()
                        .advances
                        .push(HandlingAdvance {
                            runner_id: event.participants.batter.clone().unwrap(),
                            reached: Destination::First,
                        });
                    missing_review(&mut event, 2);
                    TransitionError::IllegalCredit
                }
                1 => {
                    without_units(&mut event);
                    TransitionError::IllegalCredit
                }
                2 => {
                    event.delta.moves.push(event.delta.moves[1].clone());
                    missing_review(&mut event, 2);
                    TransitionError::DeltaMismatch
                }
                3 => {
                    event.delta.moves[1].runner.responsible_pitcher = "wrong_charge".into();
                    missing_review(&mut event, 2);
                    TransitionError::DeltaMismatch
                }
                4 => {
                    event.delta.outs = 0;
                    missing_review(&mut event, 2);
                    TransitionError::DeltaMismatch
                }
                _ => unreachable!(),
            };
            assert_eq!(
                EventTransition::from_bytes(&forged(&before, &original, &event)),
                Err(error)
            );
        }
    }
}

#[test]
fn missing_cause_wire_is_semantically_partial_and_invalid_tags_are_encoding_errors() {
    for passed in [false, true] {
        let (before, original) = fixture(Half::Top, 0, 1, 1, passed);
        let mut event = original.clone();
        event.single_delivery.as_mut().unwrap().advances.clear();
        let bytes = forged(&before, &original, &event);
        assert_eq!(
            EventTransition::from_bytes(&bytes),
            Err(TransitionError::UnsupportedRule)
        );
        without_units(&mut event);
        let partial = EventTransition::from_bytes(&forged(&before, &original, &event)).unwrap();
        let valid = partial.to_bytes();
        assert_eq!(EventTransition::from_bytes(&valid).unwrap(), partial);
        let facts = wire_delivery(event.single_delivery.as_ref().unwrap());
        let mut invalid = forged(&before, &original, &event);
        let offset = unique_offset(&invalid, &facts);
        invalid[offset + 1] = 255;
        assert_eq!(
            EventTransition::from_bytes(&invalid),
            Err(TransitionError::InvalidEncoding)
        );
        assert_eq!(
            EventTransition::from_bytes(&valid[..valid.len() - 1]),
            Err(TransitionError::InvalidEncoding)
        );
    }
}

#[test]
fn held_before_runner_can_have_an_unrepresented_legal_intermediate_reach_and_return() {
    for passed in [false, true] {
        for explicit_held in [false, true] {
            let (before, original) = fixture(Half::Top, 0, 5, 4, passed);
            let mut event = original.clone();
            let held = before.bases[0].as_ref().unwrap();
            if explicit_held {
                event.delta.moves.push(RunnerMove {
                    runner: held.clone(),
                    from: Base::First,
                    to: Destination::First,
                });
            }
            let after = physical_after(&before, &event);
            event
                .single_delivery
                .as_mut()
                .unwrap()
                .advances
                .push(HandlingAdvance {
                    runner_id: held.player_id.clone(),
                    reached: Destination::Second,
                });
            rejected(
                &before,
                event.clone(),
                after.clone(),
                TransitionError::UnsupportedRule,
            );
            assert_eq!(
                EventTransition::from_bytes(&forged(&before, &original, &event)),
                Err(TransitionError::UnsupportedRule)
            );
            without_units(&mut event);
            assert_eq!(accepted(&before, event).after(), &after);
        }
    }
}

// Home is a legal scoring acquisition, unlike an intermediate Second/Third
// reach followed by an unrepresented return. Keep the physical control legal.
fn legal_home_return_case(
    half: Half,
    outs: u8,
    passed: bool,
    endpoint: u8,
    independent: bool,
) -> (GameState, Event, Event) {
    let (before, mut original) = fixture(
        half,
        outs,
        if independent { 5 } else { 1 },
        if independent { 4 } else { 0 },
        passed,
    );
    let held = before.bases[0].as_ref().unwrap();
    if endpoint != 0 {
        original.delta.moves.push(RunnerMove {
            runner: held.clone(),
            from: Base::First,
            to: if endpoint == 1 {
                Destination::First
            } else {
                Destination::Second
            },
        });
    }
    let mut changed = original.clone();
    changed
        .single_delivery
        .as_mut()
        .unwrap()
        .advances
        .push(HandlingAdvance {
            runner_id: held.player_id.clone(),
            reached: Destination::Home,
        });
    (before, original, changed)
}

#[test]
fn legal_home_with_no_net_move_cannot_remain_on_first_without_a_run() {
    let (before, original, mut event) = legal_home_return_case(Half::Top, 0, false, 0, false);
    without_units(&mut event);
    let after = physical_after(&before, &original);
    assert_eq!(after.score, before.score);
    assert_eq!(after.bases[0], before.bases[0]);
    rejected(&before, event, after, TransitionError::DeltaMismatch);
}

#[test]
fn legal_home_with_no_net_move_is_a_semantic_wire_contradiction() {
    let (before, original, mut event) = legal_home_return_case(Half::Top, 0, false, 0, false);
    without_units(&mut event);
    assert_eq!(
        EventTransition::from_bytes(&forged(&before, &original, &event)),
        Err(TransitionError::DeltaMismatch)
    );
}

#[test]
fn legal_home_final_state_contradictions_precede_occurrence_uncertainty_in_both_apis() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..2 {
            for passed in [false, true] {
                for endpoint in 0..3 {
                    for independent in [false, true] {
                        for missing in 0..3 {
                            for supplied in [false, true] {
                                let (before, original, mut event) = legal_home_return_case(
                                    half,
                                    outs,
                                    passed,
                                    endpoint,
                                    independent,
                                );
                                let after = physical_after(&before, &original);
                                missing_review(&mut event, missing);
                                if !supplied {
                                    without_units(&mut event);
                                }
                                rejected(&before, event, after, TransitionError::DeltaMismatch);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn legal_home_final_state_contradictions_are_rechecked_on_complete_wire_records() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..2 {
            for passed in [false, true] {
                for endpoint in 0..3 {
                    for independent in [false, true] {
                        for missing in 0..3 {
                            for supplied in [false, true] {
                                let (before, original, mut event) = legal_home_return_case(
                                    half,
                                    outs,
                                    passed,
                                    endpoint,
                                    independent,
                                );
                                missing_review(&mut event, missing);
                                if !supplied {
                                    without_units(&mut event);
                                }
                                assert_eq!(
                                    EventTransition::from_bytes(&forged(
                                        &before, &original, &event
                                    )),
                                    Err(TransitionError::DeltaMismatch)
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn extra_home_advancement_of_several_runners_remains_one_delivery_unit() {
    for half in [Half::Top, Half::Bottom] {
        for passed in [false, true] {
            let (before, mut event) = fixture(half, 0, 7, 7, passed);
            event.delta.credits.retain(|c| c.kind != CreditKind::Run);
            for movement in event
                .delta
                .moves
                .iter_mut()
                .filter(|m| m.from != Base::Plate)
            {
                movement.to = Destination::Home;
                event
                    .delta
                    .credits
                    .push(credit(&movement.runner.player_id, CreditKind::Run));
            }
            for advance in &mut event.single_delivery.as_mut().unwrap().advances {
                advance.reached = Destination::Home;
            }
            event.delta.runs = 3;
            let pitcher = event.participants.pitcher.clone();
            let catcher = event.single_delivery.as_ref().unwrap().catcher_id.clone();
            let transition = accepted(&before, event);
            assert!(transition.after().bases.iter().all(Option::is_none));
            let accounting = derive_accounting(&[transition]).unwrap();
            assert_eq!(accounting.runs[half.offense()], 3);
            assert_eq!(
                accounting.pitchers[1 - half.offense()][&pitcher],
                PitcherLine {
                    outs_recorded: 1,
                    batters_faced: 1,
                    at_bats_against: Some(1),
                    credited_strikeouts: 1,
                    credited_wild_pitches: u32::from(!passed),
                    runs_allowed: 3,
                    earned_runs: None,
                    ..PitcherLine::default()
                }
            );
            if passed {
                assert_eq!(
                    accounting.fielders[1 - half.offense()][&catcher].credited_passed_balls,
                    1
                );
            }
        }
    }
}

#[test]
fn held_runner_missing_net_move_does_not_legalize_foreign_actors_or_nonadvancing_targets() {
    for passed in [false, true] {
        for malformed in 0..3 {
            let (before, mut event) = fixture(Half::Top, 0, 5, 4, passed);
            let after = physical_after(&before, &event);
            missing_review(&mut event, 2);
            event
                .single_delivery
                .as_mut()
                .unwrap()
                .advances
                .push(HandlingAdvance {
                    runner_id: if malformed == 0 {
                        before.teams[0].batting_order[6].clone()
                    } else {
                        before.bases[0].as_ref().unwrap().player_id.clone()
                    },
                    reached: match malformed {
                        0 => Destination::Second,
                        1 => Destination::First,
                        2 => Destination::Out,
                        _ => unreachable!(),
                    },
                });
            rejected(
                &before,
                event.clone(),
                after,
                TransitionError::IllegalCredit,
            );
            assert_eq!(
                EventTransition::from_bytes(&forged(
                    &before,
                    &fixture(Half::Top, 0, 5, 4, passed).1,
                    &event
                )),
                Err(TransitionError::IllegalCredit)
            );
        }
    }
}
