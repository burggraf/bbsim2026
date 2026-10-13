use bbsim_engine::accounting::{derive_accounting, PitcherLine};
use bbsim_engine::{
    apply, Base, Credit, CreditKind, DeliveryContext, Delta, Destination, Event, EventKind,
    EventTransition, FieldingReview, GameState, Half, HandlingAdvance, HandlingAssessment,
    Participants, PreDeliverySteal, RecoveryReview, Runner, RunnerEvent, RunnerMove,
    SingleDeliveryFacts, TeamState, ThirdOut, ThirdOutKind, TransitionError,
};

fn credit(id: &str, kind: CreditKind) -> Credit {
    Credit {
        player_id: id.into(),
        kind,
    }
}

fn fixture(half: Half, outs: u8) -> (GameState, Event) {
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
    let runner = Runner {
        player_id: teams[offense].batting_order[5].clone(),
        responsible_pitcher: pitcher.clone(),
    };
    let before = GameState {
        inning: 2,
        half,
        outs,
        score: [0, 0],
        teams,
        bases: [Some(runner.clone()), None, None],
        over: false,
    };
    let event = Event {
        kind: EventKind::RunnerOnly(RunnerEvent::WildPitch),
        participants: Participants {
            batter: Some(batter),
            pitcher: pitcher.clone(),
        },
        fielding_review: FieldingReview::NoMisplay,
        two_out_contact: None,
        balk: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        ordinary_air_catch: None,
        muffed_ground_ball: None,
        ordinary_ground_throw: None,
        single_delivery: Some(SingleDeliveryFacts {
            context: DeliveryContext::RunnerOnly,
            handling: HandlingAssessment::PitcherWild,
            catcher_id: catcher,
            advances: vec![HandlingAdvance {
                runner_id: runner.player_id.clone(),
                reached: Destination::Second,
            }],
            pre_delivery_steals: Some(vec![]),
            recovery: RecoveryReview::NoRecoveryPutout,
        }),
        delta: Delta {
            moves: vec![RunnerMove {
                runner,
                from: Base::First,
                to: Destination::Second,
            }],
            outs: 0,
            runs: 0,
            credits: vec![credit(&pitcher, CreditKind::WildPitch)],
            changes: vec![],
            third_out: None,
        },
    };
    (before, event)
}

fn accepted(before: &GameState, event: Event) -> EventTransition {
    let saved = before.clone();
    let transition = apply(before, event.clone()).unwrap();
    assert_eq!(*before, saved);
    assert_eq!(
        EventTransition::try_new(before.clone(), event, transition.after().clone()).unwrap(),
        transition
    );
    let bytes = transition.to_bytes();
    assert_eq!(&bytes[..5], b"BBE25");
    assert_eq!(EventTransition::from_bytes(&bytes).unwrap(), transition);
    transition
}

#[test]
fn one_delivery_credits_one_pitcher_wp_without_a_batting_line_or_pa() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..=2 {
            let (before, event) = fixture(half, outs);
            let pitcher = event.participants.pitcher.clone();
            let transition = accepted(&before, event);
            assert_eq!(transition.after().teams, before.teams);
            assert_eq!(transition.after().outs, outs);
            assert_eq!(transition.after().bases[1], before.bases[0]);
            let accounting = derive_accounting(&[transition]).unwrap();
            assert_eq!(
                accounting.pitchers[1 - half.offense()][&pitcher],
                PitcherLine {
                    credited_wild_pitches: 1,
                    ..PitcherLine::default()
                }
            );
            assert!(accounting.batters[half.offense()].is_empty());
            assert!(!accounting.complete_game);
        }
    }
}

#[test]
fn several_handling_advances_are_one_wp_not_one_per_runner_or_carried_charge() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..=2 {
            let (mut before, mut event) = fixture(half, outs);
            let offense = half.offense();
            let current = event.participants.pitcher.clone();
            let old = "prior_pitcher";
            let lead = Runner {
                player_id: before.teams[offense].batting_order[7].clone(),
                responsible_pitcher: old.into(),
            };
            before.bases[2] = Some(lead.clone());
            event.delta.moves.push(RunnerMove {
                runner: lead.clone(),
                from: Base::Third,
                to: Destination::Home,
            });
            event.delta.runs = 1;
            event
                .delta
                .credits
                .push(credit(&lead.player_id, CreditKind::Run));
            event
                .single_delivery
                .as_mut()
                .unwrap()
                .advances
                .push(HandlingAdvance {
                    runner_id: lead.player_id.clone(),
                    reached: Destination::Home,
                });
            let transition = accepted(&before, event);
            let accounting = derive_accounting(&[transition]).unwrap();
            assert_eq!(accounting.runs[offense], 1);
            assert_eq!(
                accounting.pitchers[1 - offense][&current].credited_wild_pitches,
                1
            );
            assert_eq!(accounting.pitchers[1 - offense][&current].runs_allowed, 0);
            assert_eq!(accounting.pitchers[1 - offense][&current].batters_faced, 0);
            assert_eq!(
                accounting.pitchers[1 - offense][old].credited_wild_pitches,
                0
            );
            assert_eq!(accounting.pitchers[1 - offense][old].runs_allowed, 1);
            assert_eq!(accounting.pitchers[1 - offense][old].earned_runs, None);
            assert!(!accounting.batters[offense].contains_key(&current));
        }
    }
}

fn legacy_after(before: &GameState, event: &Event) -> GameState {
    let mut legacy = event.clone();
    legacy.single_delivery = None;
    legacy
        .delta
        .credits
        .retain(|c| c.kind != CreditKind::WildPitch);
    accepted(before, legacy).after().clone()
}

fn rejected(before: &GameState, event: Event, after: &GameState, error: TransitionError) {
    let saved = before.clone();
    assert_eq!(apply(before, event.clone()).map(|_| ()), Err(error.clone()));
    assert_eq!(*before, saved);
    assert_eq!(
        EventTransition::try_new(before.clone(), event, after.clone()).map(|_| ()),
        Err(error)
    );
}

#[test]
fn positive_wp_needs_occurrence_facts_but_a_generic_label_stays_partial() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..=2 {
            let (before, mut event) = fixture(half, outs);
            let after = legacy_after(&before, &event);
            event.single_delivery = None;
            rejected(&before, event, &after, TransitionError::UnsupportedRule);
        }
    }
}

#[test]
fn missing_local_steal_or_recovery_review_does_not_infer_qualification() {
    for half in [Half::Top, Half::Bottom] {
        for missing_steals in [false, true] {
            let (before, mut event) = fixture(half, 0);
            let after = legacy_after(&before, &event);
            if missing_steals {
                event.single_delivery.as_mut().unwrap().pre_delivery_steals = None;
            } else {
                event.single_delivery.as_mut().unwrap().recovery = RecoveryReview::Unknown;
            }
            rejected(&before, event, &after, TransitionError::UnsupportedRule);
        }
    }
}

#[test]
fn explicit_one_delivery_duplicate_or_opposite_units_precede_missing_reviews() {
    for half in [Half::Top, Half::Bottom] {
        for duplicate in [false, true] {
            let (before, mut event) = fixture(half, 0);
            let after = legacy_after(&before, &event);
            let facts = event.single_delivery.as_mut().unwrap();
            facts.pre_delivery_steals = None;
            facts.recovery = RecoveryReview::Unknown;
            if duplicate {
                event.delta.credits.push(event.delta.credits[0].clone());
            } else {
                event.fielding_review = FieldingReview::Unknown;
                event.delta.credits = vec![credit(&facts.catcher_id, CreditKind::PassedBall)];
            }
            rejected(&before, event, &after, TransitionError::IllegalCredit);
        }
    }
}

#[test]
fn an_explicit_qualified_delivery_cannot_omit_its_unit() {
    for half in [Half::Top, Half::Bottom] {
        let (before, mut event) = fixture(half, 0);
        let after = legacy_after(&before, &event);
        event.delta.credits.clear();
        rejected(&before, event, &after, TransitionError::IllegalCredit);
    }
}

#[test]
fn catcher_assertion_must_be_defensively_eligible_before_review_uncertainty() {
    for half in [Half::Top, Half::Bottom] {
        for invalid in 0..4 {
            let (before, mut event) = fixture(half, 0);
            let after = legacy_after(&before, &event);
            let defense = &before.teams[1 - half.offense()];
            let facts = event.single_delivery.as_mut().unwrap();
            facts.catcher_id = match invalid {
                0 => String::new(),
                1 => defense.pitcher.clone(),
                2 => defense.batting_order[8].clone(),
                _ => defense.bench[0].clone(),
            };
            facts.pre_delivery_steals = None;
            facts.recovery = RecoveryReview::Unknown;
            rejected(&before, event, &after, TransitionError::IllegalCredit);
        }
    }
}

#[test]
fn explicit_pb_unit_belongs_to_the_asserted_catcher_not_another_eligible_fielder() {
    for half in [Half::Top, Half::Bottom] {
        let (before, mut event) = fixture(half, 0);
        event.kind = EventKind::RunnerOnly(RunnerEvent::PassedBall);
        event.fielding_review = FieldingReview::Unknown;
        let other = before.teams[1 - half.offense()].batting_order[2].clone();
        event.delta.credits = vec![credit(&other, CreditKind::PassedBall)];
        let after = legacy_after(&before, &event);
        let facts = event.single_delivery.as_mut().unwrap();
        facts.handling = HandlingAssessment::CatcherPassed;
        facts.pre_delivery_steals = None;
        facts.recovery = RecoveryReview::Unknown;
        rejected(&before, event, &after, TransitionError::IllegalCredit);
    }
}

#[test]
fn legacy_multiple_pb_units_are_not_a_universal_one_event_cap() {
    for half in [Half::Top, Half::Bottom] {
        let (before, mut event) = fixture(half, 0);
        event.kind = EventKind::RunnerOnly(RunnerEvent::PassedBall);
        event.fielding_review = FieldingReview::Unknown;
        event.delta.moves[0].to = Destination::Third;
        let facts = event.single_delivery.as_mut().unwrap();
        facts.handling = HandlingAssessment::CatcherPassed;
        facts.advances[0].reached = Destination::Third;
        let catcher = facts.catcher_id.clone();
        event.delta.credits = vec![credit(&catcher, CreditKind::PassedBall); 2];
        let mut legacy = event.clone();
        legacy.single_delivery = None;
        let control = accepted(&before, legacy);
        let accounting = derive_accounting(std::slice::from_ref(&control)).unwrap();
        assert_eq!(
            accounting.fielders[1 - half.offense()][&catcher].credited_passed_balls,
            2
        );
        rejected(
            &before,
            event,
            control.after(),
            TransitionError::IllegalCredit,
        );
    }
}

#[test]
fn passed_handling_contradicts_no_misplay_even_when_steal_suppresses_pb_unit() {
    for half in [Half::Top, Half::Bottom] {
        let (before, mut event) = fixture(half, 0);
        let runner = event.delta.moves[0].runner.player_id.clone();
        event.kind = EventKind::RunnerOnly(RunnerEvent::StealAttempt);
        event.delta.credits = vec![credit(&runner, CreditKind::StolenBase)];
        let facts = event.single_delivery.as_mut().unwrap();
        facts.handling = HandlingAssessment::CatcherPassed;
        facts.pre_delivery_steals = Some(vec![PreDeliverySteal {
            runner_id: runner,
            target: Destination::Second,
        }]);
        let after = legacy_after(&before, &event);
        rejected(&before, event, &after, TransitionError::IllegalCredit);
    }
}

#[test]
fn known_delivery_context_and_advance_contradictions_precede_uncertainty() {
    for half in [Half::Top, Half::Bottom] {
        for invalid in 0..4 {
            let (before, mut event) = fixture(half, 0);
            let after = legacy_after(&before, &event);
            let facts = event.single_delivery.as_mut().unwrap();
            facts.pre_delivery_steals = None;
            facts.recovery = RecoveryReview::Unknown;
            match invalid {
                0 => facts.context = DeliveryContext::UncaughtThirdStrike,
                1 => facts.context = DeliveryContext::DeliveredFourthBall,
                2 => {
                    facts.advances[0].runner_id =
                        before.teams[half.offense()].batting_order[0].clone()
                }
                _ => facts.advances[0].reached = Destination::Out,
            }
            rejected(&before, event, &after, TransitionError::IllegalCredit);
        }
    }
}

#[test]
fn wrong_wp_actor_cannot_hide_behind_inherited_batter_charge_ambiguity() {
    for half in [Half::Top, Half::Bottom] {
        let (mut before, mut event) = fixture(half, 0);
        before.bases[0].as_mut().unwrap().responsible_pitcher = "prior_pitcher".into();
        let batter = event.participants.batter.clone().unwrap();
        let current = event.participants.pitcher.clone();
        event.kind = EventKind::PlateAppearance;
        event.fielding_review = FieldingReview::Unknown;
        event.single_delivery = None;
        event.delta.moves = vec![
            RunnerMove {
                runner: before.bases[0].clone().unwrap(),
                from: Base::First,
                to: Destination::Out,
            },
            RunnerMove {
                runner: Runner {
                    player_id: batter.clone(),
                    responsible_pitcher: current,
                },
                from: Base::Plate,
                to: Destination::First,
            },
        ];
        event.delta.outs = 1;
        event.delta.credits = vec![
            credit(&batter, CreditKind::PlateAppearance),
            credit(&batter, CreditKind::Hit),
        ];
        let control = accepted(&before, event.clone());
        event.delta.credits = vec![
            credit(&batter, CreditKind::PlateAppearance),
            credit(&batter, CreditKind::AtBat),
            credit("prior_pitcher", CreditKind::WildPitch),
        ];
        rejected(
            &before,
            event,
            control.after(),
            TransitionError::IllegalCredit,
        );
    }
}

fn unique_offset(bytes: &[u8], needle: &[u8]) -> usize {
    let matches: Vec<_> = bytes
        .windows(needle.len())
        .enumerate()
        .filter_map(|(i, part)| (part == needle).then_some(i))
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "codec fixture needle must identify exactly one field"
    );
    matches[0]
}

#[test]
fn well_formed_codec_omission_reaches_semantic_qualification_not_byte_corruption() {
    let (before, event) = fixture(Half::Top, 0);
    let current = event.participants.pitcher.clone();
    let control = accepted(&before, event);
    let mut bytes = control.to_bytes();
    let mut one_credit = 1u64.to_le_bytes().to_vec();
    one_credit.extend_from_slice(&(current.len() as u64).to_le_bytes());
    one_credit.extend_from_slice(current.as_bytes());
    one_credit.push(23); // Existing tags unchanged; new WP unit.
    let offset = unique_offset(&bytes, &one_credit);
    bytes.splice(offset..offset + one_credit.len(), 0u64.to_le_bytes());
    assert_eq!(
        EventTransition::from_bytes(&bytes).map(|_| ()),
        Err(TransitionError::IllegalCredit)
    );
}

fn pa_delivery(half: Half, outs: u8, mask: u8, walk: bool) -> (GameState, Event) {
    let (mut before, mut event) = fixture(half, outs);
    let offense = half.offense();
    let pitcher = event.participants.pitcher.clone();
    let batter = event.participants.batter.clone().unwrap();
    before.bases = std::array::from_fn(|i| {
        (mask & (1 << i) != 0).then(|| Runner {
            player_id: before.teams[offense].batting_order[5 + i].clone(),
            responsible_pitcher: pitcher.clone(),
        })
    });
    event.kind = EventKind::PlateAppearance;
    event.delta.moves = vec![RunnerMove {
        runner: Runner {
            player_id: batter.clone(),
            responsible_pitcher: pitcher.clone(),
        },
        from: Base::Plate,
        to: Destination::First,
    }];
    for i in 0..3 {
        if (0..=i).all(|slot| before.bases[slot].is_some()) {
            event.delta.moves.push(RunnerMove {
                runner: before.bases[i].clone().unwrap(),
                from: [Base::First, Base::Second, Base::Third][i],
                to: [Destination::Second, Destination::Third, Destination::Home][i],
            });
        }
    }
    event.delta.credits = vec![
        credit(&batter, CreditKind::PlateAppearance),
        credit(
            &batter,
            if walk {
                CreditKind::Walk
            } else {
                CreditKind::Strikeout
            },
        ),
    ];
    if walk {
        event
            .delta
            .credits
            .push(credit(&batter, CreditKind::OrdinaryWalk));
    } else {
        event
            .delta
            .credits
            .push(credit(&pitcher, CreditKind::WildPitch));
    }
    event.delta.runs = 0;
    for movement in &event.delta.moves {
        if movement.to == Destination::Home {
            event.delta.runs += 1;
            event
                .delta
                .credits
                .push(credit(&movement.runner.player_id, CreditKind::Run));
        }
    }
    let facts = event.single_delivery.as_mut().unwrap();
    facts.context = if walk {
        DeliveryContext::DeliveredFourthBall
    } else {
        DeliveryContext::UncaughtThirdStrike
    };
    facts.advances = event
        .delta
        .moves
        .iter()
        .map(|m| HandlingAdvance {
            runner_id: m.runner.player_id.clone(),
            reached: m.to,
        })
        .collect();
    (before, event)
}

#[test]
fn safe_uncaught_k_first_reach_qualifies_one_wp_with_exact_participation() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..=2 {
            for mask in 0..8 {
                if outs < 2 && mask & 1 != 0 {
                    continue;
                }
                let (before, event) = pa_delivery(half, outs, mask, false);
                let after = legacy_after(&before, &event);
                let batter = event.participants.batter.clone().unwrap();
                let pitcher = event.participants.pitcher.clone();
                let transition = accepted(&before, event);
                assert_eq!(transition.after(), &after);
                let accounting = derive_accounting(&[transition]).unwrap();
                assert_eq!(
                    accounting.batters[half.offense()][&batter].plate_appearances,
                    1
                );
                assert_eq!(accounting.batters[half.offense()][&batter].at_bats, Some(1));
                let line = &accounting.pitchers[1 - half.offense()][&pitcher];
                assert_eq!(
                    (
                        line.batters_faced,
                        line.outs_recorded,
                        line.credited_strikeouts,
                        line.credited_wild_pitches
                    ),
                    (1, 0, 1, 1)
                );
                assert!(!accounting.batters[half.offense()].contains_key(&pitcher));
            }
        }
    }
}

#[test]
fn fourth_ball_forced_awards_alone_suppress_wp_without_an_at_bat() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..=2 {
            for mask in 0..8 {
                let (before, mut event) = pa_delivery(half, outs, mask, true);
                let pitcher = event.participants.pitcher.clone();
                let batter = event.participants.batter.clone().unwrap();
                let control = accepted(&before, event.clone());
                let accounting = derive_accounting(std::slice::from_ref(&control)).unwrap();
                let line = &accounting.pitchers[1 - half.offense()][&pitcher];
                assert_eq!(
                    (
                        line.batters_faced,
                        line.at_bats_against,
                        line.credited_walks_allowed,
                        line.credited_wild_pitches
                    ),
                    (1, Some(0), 1, 0)
                );
                assert_eq!(accounting.batters[half.offense()][&batter].at_bats, Some(0));
                event
                    .delta
                    .credits
                    .push(credit(&pitcher, CreditKind::WildPitch));
                rejected(
                    &before,
                    event,
                    control.after(),
                    TransitionError::IllegalCredit,
                );
            }
        }
    }
}

#[test]
fn fourth_ball_handling_excess_or_unforced_advance_qualifies_one_wp() {
    for half in [Half::Top, Half::Bottom] {
        for scenario in 0..3 {
            let mask = [0, 1, 4][scenario];
            let (before, mut event) = pa_delivery(half, 0, mask, true);
            match scenario {
                0 => event.delta.moves[0].to = Destination::Second,
                1 => event.delta.moves[1].to = Destination::Third,
                _ => {
                    let lead = before.bases[2].clone().unwrap();
                    event.delta.moves.push(RunnerMove {
                        runner: lead.clone(),
                        from: Base::Third,
                        to: Destination::Home,
                    });
                    event.delta.runs = 1;
                    event
                        .delta
                        .credits
                        .push(credit(&lead.player_id, CreditKind::Run));
                }
            }
            let pitcher = event.participants.pitcher.clone();
            event
                .delta
                .credits
                .push(credit(&pitcher, CreditKind::WildPitch));
            event.single_delivery.as_mut().unwrap().advances = event
                .delta
                .moves
                .iter()
                .map(|m| HandlingAdvance {
                    runner_id: m.runner.player_id.clone(),
                    reached: m.to,
                })
                .collect();
            let after = legacy_after(&before, &event);
            let transition = accepted(&before, event);
            assert_eq!(transition.after(), &after);
            let accounting = derive_accounting(&[transition]).unwrap();
            let line = &accounting.pitchers[1 - half.offense()][&pitcher];
            assert_eq!(
                (
                    line.batters_faced,
                    line.credited_walks_allowed,
                    line.credited_wild_pitches
                ),
                (1, 1, 1)
            );
        }
    }
}

#[test]
fn pre_delivery_steal_only_suppresses_but_extra_or_other_advance_allows_wp() {
    for half in [Half::Top, Half::Bottom] {
        for scenario in 0..3 {
            let (mut before, mut event) = fixture(half, 0);
            let runner = event.delta.moves[0].runner.player_id.clone();
            let pitcher = event.participants.pitcher.clone();
            event.kind = EventKind::RunnerOnly(RunnerEvent::StealAttempt);
            event
                .delta
                .credits
                .push(credit(&runner, CreditKind::StolenBase));
            event.single_delivery.as_mut().unwrap().pre_delivery_steals =
                Some(vec![PreDeliverySteal {
                    runner_id: runner,
                    target: Destination::Second,
                }]);
            match scenario {
                0 => {
                    event
                        .delta
                        .credits
                        .retain(|c| c.kind != CreditKind::WildPitch);
                }
                1 => {
                    event.delta.moves[0].to = Destination::Third;
                    event.single_delivery.as_mut().unwrap().advances[0].reached =
                        Destination::Third;
                }
                _ => {
                    let lead = Runner {
                        player_id: before.teams[half.offense()].batting_order[7].clone(),
                        responsible_pitcher: pitcher.clone(),
                    };
                    before.bases[2] = Some(lead.clone());
                    event.delta.moves.push(RunnerMove {
                        runner: lead.clone(),
                        from: Base::Third,
                        to: Destination::Home,
                    });
                    event
                        .delta
                        .credits
                        .push(credit(&lead.player_id, CreditKind::Run));
                    event.delta.runs = 1;
                    event
                        .single_delivery
                        .as_mut()
                        .unwrap()
                        .advances
                        .push(HandlingAdvance {
                            runner_id: lead.player_id,
                            reached: Destination::Home,
                        });
                }
            }
            let after = legacy_after(&before, &event);
            let transition = accepted(&before, event.clone());
            let accounting = derive_accounting(std::slice::from_ref(&transition)).unwrap();
            assert_eq!(
                accounting.pitchers[1 - half.offense()][&pitcher].credited_wild_pitches,
                u32::from(scenario != 0)
            );
            assert_eq!(transition.after(), &after);
            if scenario == 0 {
                event
                    .delta
                    .credits
                    .push(credit(&pitcher, CreditKind::WildPitch));
                rejected(
                    &before,
                    event,
                    transition.after(),
                    TransitionError::IllegalCredit,
                );
            }
        }
    }
}

fn recovery_play(half: Half, outs: u8, after_advance: bool, passed: bool) -> (GameState, Event) {
    let (mut before, mut event) = fixture(half, outs);
    let lead = Runner {
        player_id: before.teams[half.offense()].batting_order[7].clone(),
        responsible_pitcher: event.participants.pitcher.clone(),
    };
    before.bases[2] = Some(lead.clone());
    let retired = event.delta.moves[0].runner.player_id.clone();
    event.delta.moves[0].to = Destination::Out;
    event.delta.moves.push(RunnerMove {
        runner: lead.clone(),
        from: Base::Third,
        to: Destination::Home,
    });
    event.delta.outs = 1;
    event.delta.runs = u8::from(outs < 2 || after_advance);
    let facts = event.single_delivery.as_mut().unwrap();
    facts.advances = vec![HandlingAdvance {
        runner_id: lead.player_id.clone(),
        reached: Destination::Home,
    }];
    facts.recovery = if after_advance {
        RecoveryReview::AfterAdvance {
            runner_id: retired.clone(),
            advancer_id: lead.player_id.clone(),
        }
    } else {
        RecoveryReview::BeforeAnyAdvance {
            runner_id: retired.clone(),
        }
    };
    if passed {
        facts.handling = HandlingAssessment::CatcherPassed;
        event.fielding_review = FieldingReview::Unknown;
        event.kind = EventKind::RunnerOnly(RunnerEvent::PassedBall);
    }
    event.delta.credits = vec![credit(&facts.catcher_id, CreditKind::Putout)];
    if after_advance {
        event.delta.credits.push(credit(
            if passed {
                &facts.catcher_id
            } else {
                &event.participants.pitcher
            },
            if passed {
                CreditKind::PassedBall
            } else {
                CreditKind::WildPitch
            },
        ));
    }
    if event.delta.runs != 0 {
        event
            .delta
            .credits
            .push(credit(&lead.player_id, CreditKind::Run));
    }
    event.delta.third_out = (outs == 2).then(|| ThirdOut {
        runner_id: retired,
        kind: ThirdOutKind::Timing,
        home_before_out: if after_advance {
            vec![lead.player_id]
        } else {
            vec![]
        },
    });
    (before, event)
}

#[test]
fn recovery_before_any_advance_suppresses_while_later_recovery_preserves_one_unit() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..=2 {
            for passed in [false, true] {
                for after in [false, true] {
                    let (before, mut event) = recovery_play(half, outs, after, passed);
                    let catcher = event.single_delivery.as_ref().unwrap().catcher_id.clone();
                    let pitcher = event.participants.pitcher.clone();
                    let control_after = legacy_after(&before, &event);
                    event.delta.moves.reverse(); // Facts, not vector order, control timing.
                    let control = accepted(&before, event.clone());
                    assert_eq!(control.after(), &control_after);
                    let accounting = derive_accounting(std::slice::from_ref(&control)).unwrap();
                    assert_eq!(
                        accounting.pitchers[1 - half.offense()][&pitcher].outs_recorded,
                        1
                    );
                    assert_eq!(
                        accounting.pitchers[1 - half.offense()][&pitcher].batters_faced,
                        0
                    );
                    let units = if passed {
                        accounting.fielders[1 - half.offense()][&catcher].credited_passed_balls
                    } else {
                        accounting.pitchers[1 - half.offense()][&pitcher].credited_wild_pitches
                    };
                    assert_eq!(units, u32::from(after));
                    if !after {
                        event.delta.credits.push(credit(
                            if passed { &catcher } else { &pitcher },
                            if passed {
                                CreditKind::PassedBall
                            } else {
                                CreditKind::WildPitch
                            },
                        ));
                        rejected(
                            &before,
                            event,
                            control.after(),
                            TransitionError::IllegalCredit,
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn recovered_third_strike_before_first_has_separate_fc_suppression() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..=2 {
            let (before, mut event) = pa_delivery(half, outs, 4, false);
            let batter = event.participants.batter.clone().unwrap();
            let pitcher = event.participants.pitcher.clone();
            let lead = before.bases[2].clone().unwrap();
            event.delta.moves[0].to = Destination::Out;
            event.delta.moves.push(RunnerMove {
                runner: lead.clone(),
                from: Base::Third,
                to: Destination::Home,
            });
            event.delta.outs = 1;
            event.delta.runs = u8::from(outs < 2);
            event
                .delta
                .credits
                .retain(|c| c.kind != CreditKind::WildPitch);
            let facts = event.single_delivery.as_mut().unwrap();
            facts.advances = vec![HandlingAdvance {
                runner_id: lead.player_id.clone(),
                reached: Destination::Home,
            }];
            facts.recovery = RecoveryReview::RecoveredThirdStrikeBatterBeforeFirst;
            event
                .delta
                .credits
                .push(credit(&facts.catcher_id, CreditKind::Putout));
            if outs < 2 {
                event
                    .delta
                    .credits
                    .push(credit(&lead.player_id, CreditKind::Run));
            }
            event.delta.third_out = (outs == 2).then(|| ThirdOut {
                runner_id: batter,
                kind: ThirdOutKind::BatterBeforeFirst,
                home_before_out: vec![],
            });
            let control = accepted(&before, event.clone());
            event
                .delta
                .credits
                .push(credit(&pitcher, CreditKind::WildPitch));
            rejected(
                &before,
                event,
                control.after(),
                TransitionError::IllegalCredit,
            );
        }
    }
}

#[test]
fn recovery_actor_and_explicit_third_out_timing_contradictions_are_known() {
    for half in [Half::Top, Half::Bottom] {
        for wrong_actor in [false, true] {
            let (before, mut event) = recovery_play(half, 2, false, false);
            let after = legacy_after(&before, &event);
            let facts = event.single_delivery.as_mut().unwrap();
            facts.pre_delivery_steals = None;
            if wrong_actor {
                facts.recovery = RecoveryReview::BeforeAnyAdvance {
                    runner_id: "absent".into(),
                };
            } else {
                facts.recovery = RecoveryReview::AfterAdvance {
                    runner_id: event.delta.moves[0].runner.player_id.clone(),
                    advancer_id: event.delta.moves[1].runner.player_id.clone(),
                };
            }
            rejected(&before, event, &after, TransitionError::IllegalCredit);
        }
    }
}

#[test]
fn intermediate_causal_reach_is_unsupported_even_after_an_independent_qualifier() {
    for half in [Half::Top, Half::Bottom] {
        for reverse in [false, true] {
            let (mut before, mut event) = fixture(half, 0);
            let extra = Runner {
                player_id: before.teams[half.offense()].batting_order[6].clone(),
                responsible_pitcher: event.participants.pitcher.clone(),
            };
            before.bases[1] = Some(extra.clone());
            event.delta.moves.push(RunnerMove {
                runner: extra.clone(),
                from: Base::Second,
                to: Destination::Home,
            });
            event
                .delta
                .credits
                .push(credit(&extra.player_id, CreditKind::Run));
            event.delta.runs = 1;
            event
                .single_delivery
                .as_mut()
                .unwrap()
                .advances
                .push(HandlingAdvance {
                    runner_id: extra.player_id,
                    reached: Destination::Third,
                });
            if reverse {
                event.single_delivery.as_mut().unwrap().advances.reverse();
            }
            let after = legacy_after(&before, &event);
            rejected(&before, event, &after, TransitionError::UnsupportedRule);
        }
    }
}

#[test]
fn separate_validated_deliveries_accumulate_without_cross_event_deduplication() {
    for half in [Half::Top, Half::Bottom] {
        let (before, event) = fixture(half, 0);
        let pitcher = event.participants.pitcher.clone();
        let first = accepted(&before, event.clone());
        let mut later = event;
        later.delta.moves[0].from = Base::Second;
        later.delta.moves[0].to = Destination::Third;
        later.single_delivery.as_mut().unwrap().advances[0].reached = Destination::Third;
        let second = accepted(first.after(), later);
        assert!(second.follows(&first).is_ok());
        let accounting = derive_accounting(&[first, second]).unwrap();
        assert_eq!(
            (
                accounting.pitchers[1 - half.offense()][&pitcher].credited_wild_pitches,
                accounting.pitchers[1 - half.offense()][&pitcher].batters_faced
            ),
            (2, 0)
        );
        assert!(!accounting.batters[half.offense()].contains_key(&pitcher));
    }
}

#[test]
fn a_qualified_pb_uses_the_asserted_catcher_without_a_pitcher_wp() {
    for half in [Half::Top, Half::Bottom] {
        let (before, mut event) = fixture(half, 0);
        let pitcher = event.participants.pitcher.clone();
        let facts = event.single_delivery.as_mut().unwrap();
        facts.handling = HandlingAssessment::CatcherPassed;
        let catcher = facts.catcher_id.clone();
        event.kind = EventKind::RunnerOnly(RunnerEvent::PassedBall);
        event.fielding_review = FieldingReview::Unknown;
        event.delta.credits = vec![credit(&catcher, CreditKind::PassedBall)];
        let control = accepted(&before, event);
        let accounting = derive_accounting(&[control]).unwrap();
        assert_eq!(
            accounting.fielders[1 - half.offense()][&catcher].credited_passed_balls,
            1
        );
        assert_eq!(
            accounting.pitchers[1 - half.offense()][&pitcher].credited_wild_pitches,
            0
        );
        assert!(!accounting.batters[half.offense()].contains_key(&catcher));
    }
}

#[test]
fn automatic_k_positive_qualification_is_supported_and_legacy_pb_remains_partial() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..2 {
            let (before, mut event) = pa_delivery(half, outs, 1, false);
            event.delta.moves[0].to = Destination::Out;
            event.delta.outs = 1;
            event.single_delivery.as_mut().unwrap().advances.remove(0);
            let after = legacy_after(&before, &event);
            assert_eq!(accepted(&before, event.clone()).after(), &after);
            let catcher = event.single_delivery.as_ref().unwrap().catcher_id.clone();
            event.single_delivery = None;
            event.fielding_review = FieldingReview::Unknown;
            event
                .delta
                .credits
                .retain(|c| c.kind != CreditKind::WildPitch);
            event
                .delta
                .credits
                .push(credit(&catcher, CreditKind::PassedBall));
            assert_eq!(accepted(&before, event).after(), &after);
        }
    }
}

#[test]
fn impossible_safe_automatic_k_is_not_masked_by_missing_local_reviews() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..2 {
            let (before, mut event) = pa_delivery(half, outs, 1, false);
            let (_, control) = pa_delivery(half, outs, 1, true);
            let after = accepted(&before, control).after().clone();
            let facts = event.single_delivery.as_mut().unwrap();
            facts.pre_delivery_steals = None;
            facts.recovery = RecoveryReview::Unknown;
            rejected(&before, event, &after, TransitionError::IllegalStateChange);
        }
    }
}

#[test]
fn a_wrong_k_actor_is_known_even_without_the_new_delivery_record() {
    for half in [Half::Top, Half::Bottom] {
        let (before, mut event) = pa_delivery(half, 0, 0, false);
        let after = legacy_after(&before, &event);
        event.single_delivery = None;
        event
            .delta
            .credits
            .iter_mut()
            .find(|c| c.kind == CreditKind::Strikeout)
            .unwrap()
            .player_id = "other_batter".into();
        rejected(&before, event, &after, TransitionError::IllegalCredit);
    }
}

#[test]
fn reviewed_steal_start_actor_and_next_target_are_not_inferred_from_missing_recovery() {
    for half in [Half::Top, Half::Bottom] {
        for invalid in 0..3 {
            let (before, mut event) = fixture(half, 0);
            let after = legacy_after(&before, &event);
            let facts = event.single_delivery.as_mut().unwrap();
            facts.recovery = RecoveryReview::Unknown;
            facts.pre_delivery_steals = Some(vec![PreDeliverySteal {
                runner_id: if invalid == 0 {
                    event.participants.batter.clone().unwrap()
                } else {
                    event.delta.moves[0].runner.player_id.clone()
                },
                target: if invalid == 2 {
                    Destination::Out
                } else {
                    Destination::Third
                },
            }]);
            rejected(&before, event, &after, TransitionError::IllegalCredit);
        }
    }
}

#[test]
fn qualified_family_unit_omissions_cannot_become_known_zeroes() {
    for half in [Half::Top, Half::Bottom] {
        for family in 0..5 {
            let (before, mut event) = match family {
                0 => pa_delivery(half, 0, 0, false),
                1 => {
                    let (before, mut event) = pa_delivery(half, 0, 0, true);
                    event.delta.moves[0].to = Destination::Second;
                    event.single_delivery.as_mut().unwrap().advances[0].reached =
                        Destination::Second;
                    event
                        .delta
                        .credits
                        .push(credit(&event.participants.pitcher, CreditKind::WildPitch));
                    (before, event)
                }
                2 => {
                    let (before, mut event) = fixture(half, 0);
                    let runner = event.delta.moves[0].runner.player_id.clone();
                    event.kind = EventKind::RunnerOnly(RunnerEvent::StealAttempt);
                    event.delta.moves[0].to = Destination::Third;
                    let facts = event.single_delivery.as_mut().unwrap();
                    facts.advances[0].reached = Destination::Third;
                    facts.pre_delivery_steals = Some(vec![PreDeliverySteal {
                        runner_id: runner.clone(),
                        target: Destination::Second,
                    }]);
                    event
                        .delta
                        .credits
                        .push(credit(&runner, CreditKind::StolenBase));
                    (before, event)
                }
                3 => recovery_play(half, 0, true, false),
                _ => {
                    let (before, mut event) = fixture(half, 0);
                    let facts = event.single_delivery.as_mut().unwrap();
                    facts.handling = HandlingAssessment::CatcherPassed;
                    event.delta.credits = vec![credit(&facts.catcher_id, CreditKind::PassedBall)];
                    event.kind = EventKind::RunnerOnly(RunnerEvent::PassedBall);
                    event.fielding_review = FieldingReview::Unknown;
                    (before, event)
                }
            };
            let control = accepted(&before, event.clone());
            event
                .delta
                .credits
                .retain(|c| !matches!(c.kind, CreditKind::WildPitch | CreditKind::PassedBall));
            rejected(
                &before,
                event,
                control.after(),
                TransitionError::IllegalCredit,
            );
        }
    }
}

fn wire_string(bytes: &mut Vec<u8>, id: &str) {
    bytes.extend_from_slice(&(id.len() as u64).to_le_bytes());
    bytes.extend_from_slice(id.as_bytes());
}

#[test]
fn well_formed_codec_delivery_fact_forgeries_use_the_same_semantic_guards() {
    let (before, event) = fixture(Half::Top, 0);
    let facts = event.single_delivery.as_ref().unwrap().clone();
    let pitcher = event.participants.pitcher.clone();
    let mut record = vec![1, 0, 0]; // Some, RunnerOnly, PitcherWild.
    wire_string(&mut record, &facts.catcher_id);
    record.extend_from_slice(&1u64.to_le_bytes());
    wire_string(&mut record, &facts.advances[0].runner_id);
    let destination = record.len();
    record.push(1); // Second.
    record.push(1); // Reviewed steal starts Some([]).
    record.extend_from_slice(&0u64.to_le_bytes());
    record.push(1); // NoRecoveryPutout.
    let control = accepted(&before, event);
    let original = control.to_bytes();
    let offset = unique_offset(&original, &record);
    for forgery in 0..5 {
        let mut bytes = original.clone();
        match forgery {
            0 => bytes[offset + 1] = 1,           // Uncaught K context on non-PA.
            1 => bytes[offset + 2] = 1,           // Passed assessment contradicts WP/NoMisplay.
            2 => bytes[offset + destination] = 4, // Out is not a safe reach.
            3 => bytes[offset + record.len() - 1] = 0, // Unknown recovery is not absence.
            _ => {
                let mut wrong = vec![];
                wire_string(&mut wrong, &pitcher);
                bytes.splice(offset + 3..offset + 3 + 8 + facts.catcher_id.len(), wrong);
            }
        }
        assert_eq!(
            EventTransition::from_bytes(&bytes).map(|_| ()),
            Err(if forgery == 3 {
                TransitionError::UnsupportedRule
            } else {
                TransitionError::IllegalCredit
            })
        );
    }
}

#[test]
fn handling_class_changes_charge_actor_not_physical_effects_or_participation() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..=2 {
            for mask in 0..8 {
                for walk in [false, true] {
                    if !walk && outs < 2 && mask & 1 != 0 {
                        continue;
                    }
                    let (before, event) = pa_delivery(half, outs, mask, walk);
                    let pitcher = event.participants.pitcher.clone();
                    let wild_units = event
                        .delta
                        .credits
                        .iter()
                        .filter(|c| c.kind == CreditKind::WildPitch)
                        .count() as u32;
                    let wild = accepted(&before, event.clone());
                    let mut passed = event;
                    let facts = passed.single_delivery.as_mut().unwrap();
                    facts.handling = HandlingAssessment::CatcherPassed;
                    let catcher = facts.catcher_id.clone();
                    passed.fielding_review = FieldingReview::Unknown;
                    for c in &mut passed.delta.credits {
                        if c.kind == CreditKind::WildPitch {
                            c.kind = CreditKind::PassedBall;
                            c.player_id = catcher.clone();
                        }
                    }
                    let pb = accepted(&before, passed);
                    assert_eq!(wild.after(), pb.after());
                    let wp_box = derive_accounting(&[wild]).unwrap();
                    let pb_box = derive_accounting(&[pb]).unwrap();
                    assert_eq!(
                        pb_box.pitchers[1 - half.offense()][&pitcher].credited_wild_pitches,
                        0
                    );
                    assert_eq!(
                        pb_box.fielders[1 - half.offense()]
                            .get(&catcher)
                            .map_or(0, |f| f.credited_passed_balls),
                        wild_units
                    );
                    assert_eq!(
                        wp_box.pitchers[1 - half.offense()][&pitcher].credited_wild_pitches,
                        wild_units
                    );
                    assert_eq!(pb_box.batters, wp_box.batters);
                    assert_eq!(
                        (
                            pb_box.pitchers[1 - half.offense()][&pitcher].batters_faced,
                            pb_box.pitchers[1 - half.offense()][&pitcher].outs_recorded
                        ),
                        (1, 0)
                    );
                    assert!(!pb_box.batters[half.offense()].contains_key(&catcher));
                }
            }
        }
    }
}

// Review regressions mutate only supplied scorer facts/credits, retaining valid snapshots.
fn passed_handling(event: &mut Event) {
    let facts = event.single_delivery.as_mut().unwrap();
    facts.handling = HandlingAssessment::CatcherPassed;
    event.fielding_review = FieldingReview::Unknown;
    if matches!(event.kind, EventKind::RunnerOnly(_)) {
        event.kind = EventKind::RunnerOnly(RunnerEvent::PassedBall);
    }
    for c in &mut event.delta.credits {
        if c.kind == CreditKind::WildPitch {
            c.kind = CreditKind::PassedBall;
            c.player_id = facts.catcher_id.clone();
        }
    }
}

fn wire_destination(destination: Destination) -> u8 {
    match destination {
        Destination::First => 0,
        Destination::Second => 1,
        Destination::Third => 2,
        Destination::Home => 3,
        Destination::Out => 4,
    }
}

fn wire_delivery(facts: &SingleDeliveryFacts) -> Vec<u8> {
    let mut bytes = vec![
        1, // Some(single_delivery).
        match facts.context {
            DeliveryContext::RunnerOnly => 0,
            DeliveryContext::UncaughtThirdStrike => 1,
            DeliveryContext::DeliveredFourthBall => 2,
        },
        match facts.handling {
            HandlingAssessment::PitcherWild => 0,
            HandlingAssessment::CatcherPassed => 1,
        },
    ];
    wire_string(&mut bytes, &facts.catcher_id);
    bytes.extend_from_slice(&(facts.advances.len() as u64).to_le_bytes());
    for advance in &facts.advances {
        wire_string(&mut bytes, &advance.runner_id);
        bytes.push(wire_destination(advance.reached));
    }
    if let Some(starts) = &facts.pre_delivery_steals {
        bytes.push(1);
        bytes.extend_from_slice(&(starts.len() as u64).to_le_bytes());
        for start in starts {
            wire_string(&mut bytes, &start.runner_id);
            bytes.push(wire_destination(start.target));
        }
    } else {
        bytes.push(0);
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
        RecoveryReview::ThirdStrikeBatterAfterFirstThrow {
            putout_fielder_id,
            at,
        } => {
            bytes.push(5);
            wire_string(&mut bytes, putout_fielder_id);
            bytes.push(wire_destination(*at));
        }
    }
    bytes
}

fn wire_review_fixture_credits(credits: &[Credit]) -> Vec<u8> {
    let mut bytes = (credits.len() as u64).to_le_bytes().to_vec();
    for c in credits {
        wire_string(&mut bytes, &c.player_id);
        bytes.push(match c.kind {
            CreditKind::PlateAppearance => 0,
            CreditKind::Strikeout => 1,
            CreditKind::Walk => 2,
            CreditKind::Run => 4,
            CreditKind::StolenBase => 6,
            CreditKind::Putout => 8,
            CreditKind::PassedBall => 11,
            CreditKind::OrdinaryWalk => 20,
            CreditKind::WildPitch => 23,
            _ => panic!("credit outside these bounded review fixtures"),
        });
    }
    bytes
}

fn forged_scorer_bytes(control: &EventTransition, event: &Event) -> Vec<u8> {
    let mut bytes = control.to_bytes();
    if event.delta.credits != control.event().delta.credits {
        let old = wire_review_fixture_credits(&control.event().delta.credits);
        let offset = unique_offset(&bytes, &old);
        bytes.splice(
            offset..offset + old.len(),
            wire_review_fixture_credits(&event.delta.credits),
        );
    }
    if event.single_delivery != control.event().single_delivery {
        let old = wire_delivery(control.event().single_delivery.as_ref().unwrap());
        let offset = unique_offset(&bytes, &old);
        bytes.splice(
            offset..offset + old.len(),
            wire_delivery(event.single_delivery.as_ref().unwrap()),
        );
    }
    bytes
}

fn reject_sb_cause(walk: bool, codec: bool) {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..=2 {
            for passed in [false, true] {
                for unknown_recovery in [false, true] {
                    let (before, mut event) = if walk {
                        pa_delivery(half, outs, 1, true)
                    } else {
                        fixture(half, outs)
                    };
                    if passed {
                        passed_handling(&mut event);
                    }
                    let control = accepted(&before, event.clone());
                    let runner = before.bases[0].as_ref().unwrap().player_id.clone();
                    event
                        .delta
                        .credits
                        .push(credit(&runner, CreditKind::StolenBase));
                    if unknown_recovery {
                        event.single_delivery.as_mut().unwrap().recovery = RecoveryReview::Unknown;
                    }
                    if codec {
                        assert_eq!(
                            EventTransition::from_bytes(&forged_scorer_bytes(&control, &event))
                                .map(|_| ()),
                            Err(TransitionError::IllegalCredit)
                        );
                    } else {
                        rejected(
                            &before,
                            event,
                            control.after(),
                            TransitionError::IllegalCredit,
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn reviewed_handling_only_next_base_reach_cannot_also_supply_a_steal() {
    reject_sb_cause(false, false);
}

#[test]
fn protected_fourth_ball_award_cannot_also_supply_a_steal() {
    reject_sb_cause(true, false);
}

#[test]
fn semantic_codec_rejects_sb_on_reviewed_handling_only_reach() {
    reject_sb_cause(false, true);
}

#[test]
fn semantic_codec_rejects_sb_on_protected_fourth_ball_award() {
    reject_sb_cause(true, true);
}

fn special_recovery_play(half: Half, outs: u8) -> (GameState, Event) {
    let (before, mut event) = pa_delivery(half, outs, 4, false);
    let batter = event.participants.batter.clone().unwrap();
    let lead = before.bases[2].clone().unwrap();
    event.delta.moves[0].to = Destination::Out;
    event.delta.moves.push(RunnerMove {
        runner: lead.clone(),
        from: Base::Third,
        to: Destination::Home,
    });
    event.delta.outs = 1;
    event.delta.runs = u8::from(outs < 2);
    event
        .delta
        .credits
        .retain(|c| c.kind != CreditKind::WildPitch);
    let facts = event.single_delivery.as_mut().unwrap();
    facts.advances = vec![HandlingAdvance {
        runner_id: lead.player_id.clone(),
        reached: Destination::Home,
    }];
    facts.recovery = RecoveryReview::RecoveredThirdStrikeBatterBeforeFirst;
    event
        .delta
        .credits
        .push(credit(&facts.catcher_id, CreditKind::Putout));
    if outs < 2 {
        event
            .delta
            .credits
            .push(credit(&lead.player_id, CreditKind::Run));
    }
    event.delta.third_out = (outs == 2).then(|| ThirdOut {
        runner_id: batter,
        kind: ThirdOutKind::BatterBeforeFirst,
        home_before_out: vec![],
    });
    (before, event)
}

fn reject_suppressed_same_actor_reach(special_k: bool, codec: bool) {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..=2 {
            for passed in [false, true] {
                for missing_steals in [false, true] {
                    for reverse in [false, true] {
                        let (before, mut event) = if special_k {
                            special_recovery_play(half, outs)
                        } else {
                            recovery_play(half, outs, false, passed)
                        };
                        if passed {
                            passed_handling(&mut event);
                        }
                        if missing_steals {
                            event.single_delivery.as_mut().unwrap().pre_delivery_steals = None;
                        }
                        if reverse {
                            event.delta.moves.reverse();
                        }
                        // Other actors' later reaches remain valid, even with missing steal review.
                        let control = accepted(&before, event.clone());
                        let actor = event
                            .delta
                            .moves
                            .iter()
                            .find(|m| m.to == Destination::Out)
                            .unwrap()
                            .runner
                            .player_id
                            .clone();
                        let facts = event.single_delivery.as_mut().unwrap();
                        facts.advances.push(HandlingAdvance {
                            runner_id: actor,
                            reached: if special_k {
                                Destination::First
                            } else {
                                Destination::Second
                            },
                        });
                        if reverse {
                            facts.advances.reverse();
                        }
                        if codec {
                            assert_eq!(
                                EventTransition::from_bytes(&forged_scorer_bytes(&control, &event))
                                    .map(|_| ()),
                                Err(TransitionError::IllegalCredit)
                            );
                        } else {
                            rejected(
                                &before,
                                event,
                                control.after(),
                                TransitionError::IllegalCredit,
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn retirement_before_any_advance_contradicts_that_actors_safe_causal_reach() {
    reject_suppressed_same_actor_reach(false, false);
}

#[test]
fn recovered_k_before_first_contradicts_a_safe_batter_first_reach() {
    reject_suppressed_same_actor_reach(true, false);
}

#[test]
fn semantic_codec_rejects_before_any_retirement_with_same_actor_safe_reach() {
    reject_suppressed_same_actor_reach(false, true);
}

#[test]
fn semantic_codec_rejects_recovered_k_before_first_with_safe_batter_first_reach() {
    reject_suppressed_same_actor_reach(true, true);
}

#[test]
fn sb_plus_handling_extra_keeps_partial_sb_omission_and_both_charge_classes() {
    for half in [Half::Top, Half::Bottom] {
        for passed in [false, true] {
            for omit_sb in [false, true] {
                let (before, mut event) = fixture(half, 0);
                let runner = event.delta.moves[0].runner.player_id.clone();
                event.kind = EventKind::RunnerOnly(RunnerEvent::StealAttempt);
                event.delta.moves[0].to = Destination::Third;
                let facts = event.single_delivery.as_mut().unwrap();
                facts.advances[0].reached = Destination::Third;
                facts.pre_delivery_steals = Some(vec![PreDeliverySteal {
                    runner_id: runner.clone(),
                    target: Destination::Second,
                }]);
                if !omit_sb {
                    event
                        .delta
                        .credits
                        .push(credit(&runner, CreditKind::StolenBase));
                }
                if passed {
                    passed_handling(&mut event);
                }
                accepted(&before, event);
            }
        }
    }
}

#[test]
fn lawful_safe_reach_then_retirement_does_not_get_a_blanket_illegal_ruling() {
    for half in [Half::Top, Half::Bottom] {
        for passed in [false, true] {
            for unknown in [false, true] {
                let (before, mut event) = recovery_play(half, 0, true, passed);
                let retired = event.delta.moves[0].runner.player_id.clone();
                event
                    .single_delivery
                    .as_mut()
                    .unwrap()
                    .advances
                    .push(HandlingAdvance {
                        runner_id: retired.clone(),
                        reached: Destination::Second,
                    });
                let after = legacy_after(&before, &event);
                if unknown {
                    event.single_delivery.as_mut().unwrap().recovery = RecoveryReview::Unknown;
                } else {
                    event.single_delivery.as_mut().unwrap().recovery =
                        RecoveryReview::AfterAdvance {
                            runner_id: retired.clone(),
                            advancer_id: retired,
                        };
                }
                rejected(&before, event, &after, TransitionError::UnsupportedRule);
            }
        }
    }
}

#[test]
fn sb_with_unrepresented_intermediate_cause_stays_unsupported_not_illegal() {
    for half in [Half::Top, Half::Bottom] {
        for passed in [false, true] {
            for unknown_starts in [false, true] {
                let (before, mut event) = fixture(half, 0);
                let runner = event.delta.moves[0].runner.player_id.clone();
                event.delta.moves[0].to = Destination::Third;
                event.single_delivery.as_mut().unwrap().advances[0].reached = Destination::Third;
                if passed {
                    passed_handling(&mut event);
                }
                let control = accepted(&before, event.clone());
                event
                    .delta
                    .credits
                    .push(credit(&runner, CreditKind::StolenBase));
                if unknown_starts {
                    event.single_delivery.as_mut().unwrap().pre_delivery_steals = None;
                }
                rejected(
                    &before,
                    event.clone(),
                    control.after(),
                    TransitionError::UnsupportedRule,
                );
                assert_eq!(
                    EventTransition::from_bytes(&forged_scorer_bytes(&control, &event)).map(|_| ()),
                    Err(TransitionError::UnsupportedRule)
                );
            }
        }
    }
}

#[test]
fn wrong_sb_actor_precedes_missing_delivery_recovery_review() {
    for half in [Half::Top, Half::Bottom] {
        for passed in [false, true] {
            let (before, mut event) = fixture(half, 0);
            if passed {
                passed_handling(&mut event);
            }
            let control = accepted(&before, event.clone());
            event
                .delta
                .credits
                .push(credit("absent", CreditKind::StolenBase));
            event.single_delivery.as_mut().unwrap().recovery = RecoveryReview::Unknown;
            rejected(
                &before,
                event.clone(),
                control.after(),
                TransitionError::IllegalCredit,
            );
            assert_eq!(
                EventTransition::from_bytes(&forged_scorer_bytes(&control, &event)).map(|_| ()),
                Err(TransitionError::IllegalCredit)
            );
        }
    }
}

fn reject_prestart_to_protected_award(codec: bool) {
    for half in [Half::Top, Half::Bottom] {
        for passed in [false, true] {
            for reverse in [false, true] {
                let (before, mut event) = pa_delivery(half, 0, 1, true);
                let runner = before.bases[0].as_ref().unwrap().player_id.clone();
                event.delta.moves[1].to = Destination::Third;
                let facts = event.single_delivery.as_mut().unwrap();
                facts.advances[1].reached = Destination::Third;
                facts.pre_delivery_steals = Some(vec![PreDeliverySteal {
                    runner_id: runner.clone(),
                    target: Destination::Second,
                }]);
                event
                    .delta
                    .credits
                    .push(credit(&event.participants.pitcher, CreditKind::WildPitch));
                if passed {
                    passed_handling(&mut event);
                }
                if reverse {
                    event.delta.moves.reverse();
                    event.single_delivery.as_mut().unwrap().advances.reverse();
                }
                // Without the unbound SB claim, the extra handling reach is a valid one-unit play.
                let control = accepted(&before, event.clone());
                event
                    .delta
                    .credits
                    .push(credit(&runner, CreditKind::StolenBase));
                if codec {
                    assert_eq!(
                        EventTransition::from_bytes(&forged_scorer_bytes(&control, &event))
                            .map(|_| ()),
                        Err(TransitionError::UnsupportedRule)
                    );
                } else {
                    rejected(
                        &before,
                        event,
                        control.after(),
                        TransitionError::UnsupportedRule,
                    );
                }
            }
        }
    }
}

#[test]
fn prestart_to_protected_walk_award_does_not_certify_an_independent_steal() {
    reject_prestart_to_protected_award(false);
}

#[test]
fn semantic_codec_defers_unbound_steal_at_protected_walk_award() {
    reject_prestart_to_protected_award(true);
}

fn reject_retired_sb_without_intermediate_history(codec: bool) {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..=2 {
            for passed in [false, true] {
                for reverse in [false, true] {
                    for unknown_recovery in [false, true] {
                        let (before, mut event) = recovery_play(half, outs, true, passed);
                        let retired = event.delta.moves[0].runner.player_id.clone();
                        event.single_delivery.as_mut().unwrap().pre_delivery_steals =
                            Some(vec![PreDeliverySteal {
                                runner_id: retired.clone(),
                                target: Destination::Second,
                            }]);
                        if reverse {
                            event.delta.moves.reverse();
                        }
                        let control = accepted(&before, event.clone());
                        event
                            .delta
                            .credits
                            .push(credit(&retired, CreditKind::StolenBase));
                        if unknown_recovery {
                            event.single_delivery.as_mut().unwrap().recovery =
                                RecoveryReview::Unknown;
                        }
                        if codec {
                            assert_eq!(
                                EventTransition::from_bytes(&forged_scorer_bytes(&control, &event))
                                    .map(|_| ()),
                                Err(TransitionError::UnsupportedRule)
                            );
                        } else {
                            rejected(
                                &before,
                                event,
                                control.after(),
                                TransitionError::UnsupportedRule,
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn sb_then_final_retirement_needs_intermediate_arrival_and_holding_history() {
    reject_retired_sb_without_intermediate_history(false);
}

#[test]
fn semantic_codec_defers_retired_actor_sb_without_intermediate_history() {
    reject_retired_sb_without_intermediate_history(true);
}

#[test]
fn before_any_advance_retirement_cannot_also_credit_that_actors_completed_steal() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..=2 {
            for passed in [false, true] {
                let (before, mut event) = recovery_play(half, outs, false, passed);
                let retired = event.delta.moves[0].runner.player_id.clone();
                let control = accepted(&before, event.clone());
                event
                    .delta
                    .credits
                    .push(credit(&retired, CreditKind::StolenBase));
                rejected(
                    &before,
                    event.clone(),
                    control.after(),
                    TransitionError::IllegalCredit,
                );
                assert_eq!(
                    EventTransition::from_bytes(&forged_scorer_bytes(&control, &event)).map(|_| ()),
                    Err(TransitionError::IllegalCredit)
                );
            }
        }
    }
}

#[test]
fn fourth_ball_nonforced_prestart_still_supports_sb_with_other_handling_advance() {
    for half in [Half::Top, Half::Bottom] {
        for passed in [false, true] {
            let (before, mut event) = pa_delivery(half, 0, 2, true);
            let runner = before.bases[1].clone().unwrap();
            event.delta.moves[0].to = Destination::Second;
            event.delta.moves.push(RunnerMove {
                runner: runner.clone(),
                from: Base::Second,
                to: Destination::Third,
            });
            let facts = event.single_delivery.as_mut().unwrap();
            facts.advances = event
                .delta
                .moves
                .iter()
                .map(|m| HandlingAdvance {
                    runner_id: m.runner.player_id.clone(),
                    reached: m.to,
                })
                .collect();
            facts.pre_delivery_steals = Some(vec![PreDeliverySteal {
                runner_id: runner.player_id.clone(),
                target: Destination::Third,
            }]);
            event.delta.credits.extend([
                credit(&runner.player_id, CreditKind::StolenBase),
                credit(&event.participants.pitcher, CreditKind::WildPitch),
            ]);
            if passed {
                passed_handling(&mut event);
            }
            accepted(&before, event);
        }
    }
}

fn reject_sb_on_batter_known_retired_before_first(automatic: bool) {
    for half in [Half::Top, Half::Bottom] {
        for passed in [false, true] {
            let (before, mut event) = if automatic {
                let (before, mut event) = pa_delivery(half, 0, 1, false);
                event.delta.moves[0].to = Destination::Out;
                event.delta.outs = 1;
                let batter = event.participants.batter.clone().unwrap();
                event
                    .single_delivery
                    .as_mut()
                    .unwrap()
                    .advances
                    .retain(|a| a.runner_id != batter);
                event
                    .delta
                    .credits
                    .retain(|c| c.kind != CreditKind::WildPitch);
                (before, event)
            } else {
                special_recovery_play(half, 2)
            };
            let facts = event.single_delivery.as_mut().unwrap();
            facts.recovery = RecoveryReview::Unknown;
            facts.pre_delivery_steals = None;
            if passed {
                passed_handling(&mut event);
            }
            // Unknown qualification with no positive WP/PB stays a partial view.
            let control = accepted(&before, event.clone());
            let batter = event.participants.batter.clone().unwrap();
            event
                .delta
                .credits
                .push(credit(&batter, CreditKind::StolenBase));
            rejected(
                &before,
                event.clone(),
                control.after(),
                TransitionError::IllegalCredit,
            );
            assert_eq!(
                EventTransition::from_bytes(&forged_scorer_bytes(&control, &event)).map(|_| ()),
                Err(TransitionError::IllegalCredit)
            );
        }
    }
}

#[test]
fn automatic_k_retirement_cannot_supply_a_batter_steal_despite_unknown_recovery() {
    reject_sb_on_batter_known_retired_before_first(true);
}

#[test]
fn explicit_batter_before_first_third_out_cannot_supply_a_steal_despite_unknown_recovery() {
    reject_sb_on_batter_known_retired_before_first(false);
}

#[test]
fn known_credit_and_review_contradictions_do_not_hide_behind_k_uncertainty() {
    for half in [Half::Top, Half::Bottom] {
        for invalid in 0..4 {
            let (before, mut event) = pa_delivery(half, 0, 0, false);
            let after = legacy_after(&before, &event);
            let batter = event.participants.batter.clone().unwrap();
            match invalid {
                0 => event
                    .delta
                    .credits
                    .extend(vec![credit(&batter, CreditKind::AtBat); 2]),
                1 => {
                    event
                        .delta
                        .credits
                        .iter_mut()
                        .find(|c| c.kind == CreditKind::Strikeout)
                        .unwrap()
                        .player_id = "other_batter".into()
                }
                2 => event
                    .delta
                    .credits
                    .push(credit(&event.participants.pitcher, CreditKind::Putout)),
                _ => event
                    .delta
                    .credits
                    .push(credit(&event.participants.pitcher, CreditKind::Error)),
            }
            event.single_delivery.as_mut().unwrap().pre_delivery_steals = None;
            event.single_delivery.as_mut().unwrap().recovery = RecoveryReview::Unknown;
            rejected(&before, event, &after, TransitionError::IllegalCredit);
        }
    }
}
