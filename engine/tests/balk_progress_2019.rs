use bbsim_engine::accounting::derive_accounting;
use bbsim_engine::{
    apply, BalkCause, BalkDisposition, BalkFacts, BalkProgressFacts, BalkResolution, Base, Credit,
    CreditKind, DeliveryContext, Delta, Destination, Event, EventKind, EventTransition,
    FieldingReview, GameState, Half, HandlingAdvance, HandlingAssessment, NamedBaseTouch,
    Participants, PersonnelChange, RecoveryReview, Runner, RunnerMove, SingleDeliveryFacts,
    TeamSide, TeamState, ThirdOut, ThirdOutKind, TransitionError,
};

fn credit(id: &str, kind: CreditKind) -> Credit {
    Credit {
        player_id: id.into(),
        kind,
    }
}

// Every before runner advances, including runners not forced by a first award.
// Touches are supplied separately; final destinations do not create them.
fn fixture(half: Half, outs: u8, mask: u8, reach: u8) -> (GameState, Event) {
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
    let bases: [Option<Runner>; 3] = std::array::from_fn(|i| {
        (mask & (1 << i) != 0).then(|| Runner {
            player_id: teams[offense].batting_order[5 + i].clone(),
            responsible_pitcher: pitcher.clone(),
        })
    });
    let mut moves: Vec<_> = bases
        .iter()
        .enumerate()
        .filter_map(|(i, runner)| {
            runner.as_ref().map(|r| RunnerMove {
                runner: r.clone(),
                from: [Base::First, Base::Second, Base::Third][i],
                to: [Destination::Second, Destination::Third, Destination::Home][i],
            })
        })
        .collect();
    let touches = moves
        .iter()
        .map(|m| NamedBaseTouch {
            runner_id: m.runner.player_id.clone(),
            base: m.to,
        })
        .collect();
    moves.push(RunnerMove {
        runner: Runner {
            player_id: batter.clone(),
            responsible_pitcher: pitcher.clone(),
        },
        from: Base::Plate,
        to: Destination::First,
    });
    let mut credits = vec![credit(&batter, CreditKind::PlateAppearance)];
    match reach {
        0 => credits.extend([
            credit(&batter, CreditKind::Hit),
            credit(&batter, CreditKind::Single),
        ]),
        1 => credits.extend([
            credit(&batter, CreditKind::Walk),
            credit(&batter, CreditKind::OrdinaryWalk),
        ]),
        2 => credits.push(credit(&batter, CreditKind::HitByPitch)),
        3 => credits.push(credit(&batter, CreditKind::Strikeout)),
        4 => {}
        _ => panic!("unknown synthetic reach"),
    }
    if let Some(runner) = &bases[2] {
        credits.push(credit(&runner.player_id, CreditKind::Run));
    }
    let event = Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(batter.clone()),
            pitcher,
        },
        fielding_review: FieldingReview::Unknown,
        two_out_contact: None,
        single_delivery: None,
        balk: Some(BalkFacts {
            cause: BalkCause::PitcherAction602a,
            disposition: BalkDisposition::PlayProceedsWithoutReference,
            resolution: BalkResolution::ActualMinimumProgress(BalkProgressFacts {
                batter_first_touch: NamedBaseTouch {
                    runner_id: batter,
                    base: Destination::First,
                },
                runner_next_touches: touches,
            }),
        }),
        delta: Delta {
            moves,
            outs: 0,
            runs: u8::from(bases[2].is_some()),
            credits,
            changes: vec![],
            third_out: None,
        },
    };
    (
        GameState {
            inning: 2,
            half,
            outs,
            score: [0, 0],
            teams,
            bases,
            over: false,
        },
        event,
    )
}

fn accepted(before: &GameState, event: Event) -> EventTransition {
    let mut legacy = event.clone();
    legacy.balk = None;
    let physical = apply(before, legacy).unwrap();
    let saved = before.clone();
    let direct =
        EventTransition::try_new(before.clone(), event.clone(), physical.after().clone()).unwrap();
    let reduced = apply(before, event).unwrap();
    assert_eq!(before, &saved);
    assert_eq!(direct, reduced);
    assert_eq!(reduced.after(), physical.after());
    let bytes = reduced.to_bytes();
    assert_eq!(&bytes[..5], b"BBE18");
    assert_eq!(EventTransition::from_bytes(&bytes).unwrap(), reduced);
    reduced
}

#[test]
fn explicit_minimum_progress_conserves_one_pa_and_no_balk_for_all_safe_frames() {
    let mut frames = 0;
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..=2 {
            for mask in 1..8 {
                for reach in 0..5 {
                    if reach == 3 && outs < 2 && mask & 1 != 0 {
                        continue;
                    }
                    for review in [FieldingReview::Unknown, FieldingReview::NoMisplay] {
                        for reverse in [false, true] {
                            let (before, mut event) = fixture(half, outs, mask, reach);
                            event.fielding_review = review;
                            if reverse {
                                event.delta.moves.reverse();
                                event.delta.credits.reverse();
                                let BalkResolution::ActualMinimumProgress(facts) =
                                    &mut event.balk.as_mut().unwrap().resolution
                                else {
                                    unreachable!()
                                };
                                facts.runner_next_touches.reverse();
                            }
                            let pitcher = event.participants.pitcher.clone();
                            let batter = event.participants.batter.clone().unwrap();
                            let transition = accepted(&before, event);
                            assert_eq!(transition.after().outs, outs);
                            assert_eq!(transition.after().teams[half.offense()].next_batter, 1);
                            assert_eq!(
                                transition.after().score[half.offense()],
                                u16::from(mask & 4 != 0)
                            );
                            assert!(!transition.after().over);
                            let accounting = derive_accounting(&[transition]).unwrap();
                            let line = &accounting.pitchers[1 - half.offense()][&pitcher];
                            let batting = &accounting.batters[half.offense()][&batter];
                            assert_eq!(
                                (line.batters_faced, line.outs_recorded, line.credited_balks),
                                (1, 0, 0)
                            );
                            assert_eq!(batting.plate_appearances, 1);
                            let ab = match reach {
                                0 | 3 => Some(1),
                                1 | 2 => Some(0),
                                _ => None,
                            };
                            assert_eq!(batting.at_bats, ab);
                            assert_eq!(line.at_bats_against, ab);
                            assert_eq!(batting.credited_strikeouts, u32::from(reach == 3));
                            assert!(!accounting.batters[half.offense()].contains_key(&pitcher));
                            assert!(!accounting.complete_game);
                            frames += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(frames, 776);
}

fn progress(event: &mut Event) -> &mut BalkProgressFacts {
    let BalkResolution::ActualMinimumProgress(facts) = &mut event.balk.as_mut().unwrap().resolution
    else {
        panic!("expected explicit progress fixture")
    };
    facts
}

// Only the physical control is normalized. The disputed event is never altered.
fn physical_after(before: &GameState, event: &Event) -> GameState {
    let mut control = event.clone();
    control.balk = None;
    control.single_delivery = None;
    control.fielding_review = FieldingReview::Unknown;
    control.delta.changes.clear();
    control.delta.outs = control
        .delta
        .moves
        .iter()
        .filter(|m| m.to == Destination::Out)
        .count() as u8;
    control.delta.runs = control
        .delta
        .moves
        .iter()
        .filter(|m| m.to == Destination::Home)
        .count() as u8;
    control.delta.credits.retain(|c| {
        !matches!(
            c.kind,
            CreditKind::PlateAppearance
                | CreditKind::Run
                | CreditKind::Strikeout
                | CreditKind::Balk
                | CreditKind::WildPitch
                | CreditKind::PassedBall
                | CreditKind::Error
                | CreditKind::StolenBase
        )
    });
    if matches!(
        control.kind,
        EventKind::PlateAppearance | EventKind::OutOfParkHomeRun | EventKind::FieldersChoice
    ) {
        control.delta.credits.push(credit(
            control.participants.batter.as_ref().unwrap(),
            CreditKind::PlateAppearance,
        ));
    }
    for m in &control.delta.moves {
        if m.to == Destination::Home {
            control
                .delta
                .credits
                .push(credit(&m.runner.player_id, CreditKind::Run));
        }
    }
    apply(before, control).unwrap().after().clone()
}

fn rejected(before: &GameState, event: Event, error: TransitionError) {
    let after = physical_after(before, &event);
    rejected_with_after(before, event, after, error);
}

fn rejected_with_after(before: &GameState, event: Event, after: GameState, error: TransitionError) {
    let saved = before.clone();
    let direct = EventTransition::try_new(before.clone(), event.clone(), after);
    let reduced = apply(before, event);
    assert_eq!(before, &saved);
    assert_eq!((direct, reduced), (Err(error.clone()), Err(error)));
}

#[test]
fn favorable_endpoints_and_awards_never_replace_missing_whole_or_individual_touches() {
    for mask in 1..8 {
        let (before, event) = fixture(Half::Top, 0, mask, 4);
        let mut missing = event.clone();
        missing.balk.as_mut().unwrap().resolution = BalkResolution::Unknown;
        rejected(&before, missing, TransitionError::UnsupportedRule);
        for i in 0..progress(&mut event.clone()).runner_next_touches.len() {
            let mut missing = event.clone();
            progress(&mut missing).runner_next_touches.remove(i);
            rejected(&before, missing, TransitionError::UnsupportedRule);
        }
    }
}

#[test]
fn wrong_named_touch_actors_are_known_even_with_unknown_cause_or_disposition() {
    for unknown in 0..3 {
        for actor in 0..4 {
            let (before, mut event) = fixture(Half::Top, 0, 1, 4);
            match unknown {
                1 => event.balk.as_mut().unwrap().cause = BalkCause::Unknown,
                2 => event.balk.as_mut().unwrap().disposition = BalkDisposition::Unknown,
                _ => {}
            }
            match actor {
                0 => progress(&mut event).batter_first_touch.runner_id = "outsider".into(),
                1 => progress(&mut event).batter_first_touch.runner_id = "away5".into(),
                2 => progress(&mut event).runner_next_touches[0].runner_id = "outsider".into(),
                3 => progress(&mut event).runner_next_touches[0].runner_id = "away0".into(),
                _ => unreachable!(),
            }
            rejected(&before, event, TransitionError::IllegalCredit);
        }
    }
}

#[test]
fn threshold_fields_bind_first_and_each_computed_next_base_not_an_award_or_highest_reach() {
    for mask in 1..8 {
        for malformed in 0..4 {
            let (before, mut event) = fixture(Half::Top, 0, mask, 4);
            match malformed {
                0 => progress(&mut event).batter_first_touch.base = Destination::Second,
                1 => progress(&mut event).batter_first_touch.base = Destination::Out,
                2 => progress(&mut event).runner_next_touches[0].base = Destination::First,
                3 => progress(&mut event).runner_next_touches[0].base = Destination::Out,
                _ => unreachable!(),
            }
            rejected(&before, event, TransitionError::IllegalCredit);
        }
    }
}

#[test]
fn one_qualification_assertion_per_named_before_runner_is_not_a_real_touch_history_cap() {
    for mask in 1..8 {
        let (before, mut event) = fixture(Half::Top, 0, mask, 4);
        let repeated = progress(&mut event).runner_next_touches[0].clone();
        progress(&mut event).runner_next_touches.push(repeated);
        event.balk.as_mut().unwrap().cause = BalkCause::Unknown;
        rejected(&before, event, TransitionError::IllegalCredit);
    }
}

#[test]
fn ordinary_minimum_progress_origin_requires_a_before_runner_not_a_new_batter_runner() {
    let (before, event) = fixture(Half::Top, 0, 0, 4);
    rejected(&before, event, TransitionError::IllegalCredit);
}

#[test]
fn discarded_balk_credit_and_known_run_actor_error_precede_incomplete_progress() {
    let (before, mut event) = fixture(Half::Top, 0, 4, 4);
    progress(&mut event).runner_next_touches.clear();
    let mut positive = event.clone();
    positive
        .delta
        .credits
        .push(credit(&positive.participants.pitcher, CreditKind::Balk));
    rejected(&before, positive, TransitionError::IllegalCredit);
    let batter = event.participants.batter.clone().unwrap();
    event
        .delta
        .credits
        .iter_mut()
        .find(|c| c.kind == CreditKind::Run)
        .unwrap()
        .player_id = batter;
    rejected(&before, event, TransitionError::IllegalCredit);
}

#[test]
fn automatic_uncaught_k_uses_before_first_occupancy_even_when_every_runner_vacates() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..2 {
            for mask in [1, 3, 5, 7] {
                let (before, event) = fixture(half, outs, mask, 3);
                rejected(&before, event, TransitionError::IllegalStateChange);
            }
        }
    }
}

fn retired_batter(event: &mut Event, before: &GameState, kind: ThirdOutKind) {
    let batter = event.participants.batter.clone().unwrap();
    event
        .delta
        .moves
        .iter_mut()
        .find(|m| m.from == Base::Plate)
        .unwrap()
        .to = Destination::Out;
    event.delta.outs = 1;
    if before.outs == 2 {
        event.delta.third_out = Some(ThirdOut {
            runner_id: batter,
            kind,
            home_before_out: vec![],
        });
    }
}

#[test]
fn explicit_before_first_retirement_contradicts_named_first_touch_before_unknown_binding() {
    for unknown in 0..3 {
        let (before, mut event) = fixture(Half::Top, 2, 1, 4);
        retired_batter(&mut event, &before, ThirdOutKind::BatterBeforeFirst);
        match unknown {
            1 => event.balk.as_mut().unwrap().cause = BalkCause::Unknown,
            2 => event.balk.as_mut().unwrap().disposition = BalkDisposition::Unknown,
            _ => {}
        }
        rejected(&before, event, TransitionError::IllegalCredit);
    }
}

#[test]
fn lawful_after_first_retirement_return_and_terminal_histories_need_richer_facts() {
    let (before, mut after_first) = fixture(Half::Top, 2, 1, 4);
    retired_batter(
        &mut after_first,
        &before,
        ThirdOutKind::BatterAfterFirstTiming,
    );
    rejected(&before, after_first, TransitionError::UnsupportedRule);
    let (before, mut returned) = fixture(Half::Top, 0, 2, 4);
    returned
        .delta
        .moves
        .iter_mut()
        .find(|m| m.from == Base::Second)
        .unwrap()
        .to = Destination::Second;
    rejected(&before, returned, TransitionError::UnsupportedRule);
    let (mut before, event) = fixture(Half::Bottom, 0, 4, 4);
    before.inning = 9;
    rejected(&before, event, TransitionError::UnsupportedRule);
}

#[test]
fn unknown_or_unbound_disposition_and_cause_are_not_inferred_from_complete_touches() {
    for cause in [
        BalkCause::Unknown,
        BalkCause::SqueezeOrStealHomeInterference601g,
        BalkCause::PitchingProhibition602d,
    ] {
        let (before, mut event) = fixture(Half::Top, 0, 1, 4);
        event.balk.as_mut().unwrap().cause = cause;
        rejected(&before, event, TransitionError::UnsupportedRule);
    }
    for disposition in [BalkDisposition::Unknown, BalkDisposition::PenaltyApplied] {
        let (before, mut event) = fixture(Half::Top, 0, 1, 4);
        event.balk.as_mut().unwrap().disposition = disposition;
        rejected(&before, event, TransitionError::UnsupportedRule);
    }
}

#[test]
fn existing_participant_pa_review_role_and_state_errors_stay_known_before_progress() {
    let (before, event) = fixture(Half::Top, 0, 1, 4);
    let after = physical_after(&before, &event);
    let mut participant = event.clone();
    participant.participants.pitcher = "home_relief".into();
    assert_eq!(
        apply(&before, participant.clone()),
        Err(TransitionError::InvalidParticipants)
    );
    assert_eq!(
        EventTransition::try_new(before.clone(), participant, after.clone()),
        Err(TransitionError::InvalidParticipants)
    );
    let mut invalid = before.clone();
    invalid.outs = 3;
    assert_eq!(
        apply(&invalid, event.clone()),
        Err(TransitionError::InvalidState)
    );
    assert_eq!(
        EventTransition::try_new(invalid, event.clone(), after),
        Err(TransitionError::InvalidState)
    );
    let mut pa = event.clone();
    pa.delta.credits.clear();
    rejected(&before, pa, TransitionError::IllegalCredit);
    let mut review = event.clone();
    review.fielding_review = FieldingReview::NoMisplay;
    review
        .delta
        .credits
        .push(credit("home1", CreditKind::Error));
    rejected(&before, review, TransitionError::IllegalCredit);
    let mut role = event;
    role.delta.credits.push(credit("home8", CreditKind::Error));
    rejected(&before, role, TransitionError::IllegalCredit);
}

#[test]
fn represented_out_and_run_conservation_errors_precede_missing_progress_witnesses() {
    for whole_missing in [false, true] {
        for contradiction in 0..3 {
            let (before, mut event) = fixture(Half::Top, 0, 1, 4);
            if whole_missing {
                event.balk.as_mut().unwrap().resolution = BalkResolution::Unknown;
            } else {
                progress(&mut event).runner_next_touches.clear();
            }
            match contradiction {
                0 => event.delta.outs = 1,
                1 => event.delta.runs = 1,
                2 => {
                    event
                        .delta
                        .moves
                        .iter_mut()
                        .find(|m| m.from == Base::Plate)
                        .unwrap()
                        .to = Destination::Out;
                    // Actual out claim with a represented zero-out delta is contradictory.
                }
                _ => unreachable!(),
            }
            rejected(&before, event, TransitionError::DeltaMismatch);
        }
    }
}

#[test]
fn represented_personnel_change_in_a_pa_is_known_not_missing_balk_chronology() {
    let (before, mut event) = fixture(Half::Top, 0, 1, 4);
    progress(&mut event).runner_next_touches.clear();
    event.delta.changes.push(PersonnelChange::Pitcher {
        team: TeamSide::Home,
        outgoing: "home_pitcher".into(),
        incoming: "home_relief".into(),
    });
    rejected(&before, event, TransitionError::IllegalStateChange);
}

#[test]
fn complete_touches_and_absent_balk_unit_do_not_bind_pitch_error_or_steal_episodes() {
    for supplied in [CreditKind::Error, CreditKind::StolenBase] {
        let (before, mut event) = fixture(Half::Top, 0, 2, 4);
        let actor = if supplied == CreditKind::Error {
            "home1"
        } else {
            "away6"
        };
        event.delta.credits.push(credit(actor, supplied));
        let mut legacy = event.clone();
        legacy.balk = None;
        accepted(&before, legacy);
        rejected(&before, event, TransitionError::UnsupportedRule);
    }
    for handling in [
        HandlingAssessment::PitcherWild,
        HandlingAssessment::CatcherPassed,
    ] {
        let (before, mut event) = fixture(Half::Top, 0, 2, 1);
        event.single_delivery = Some(SingleDeliveryFacts {
            context: DeliveryContext::DeliveredFourthBall,
            handling,
            catcher_id: "home1".into(),
            advances: vec![HandlingAdvance {
                runner_id: "away6".into(),
                reached: Destination::Third,
            }],
            pre_delivery_steals: Some(vec![]),
            recovery: RecoveryReview::NoRecoveryPutout,
        });
        event
            .delta
            .credits
            .push(if handling == HandlingAssessment::PitcherWild {
                credit(&event.participants.pitcher, CreditKind::WildPitch)
            } else {
                credit("home1", CreditKind::PassedBall)
            });
        let mut legacy = event.clone();
        legacy.balk = None;
        accepted(&before, legacy);
        rejected(&before, event, TransitionError::UnsupportedRule);
    }
    // A fully suppressed/protected delivery still needs binding, with no WP/PB/Balk unit.
    let (before, mut event) = fixture(Half::Top, 0, 1, 1);
    event.single_delivery = Some(SingleDeliveryFacts {
        context: DeliveryContext::DeliveredFourthBall,
        handling: HandlingAssessment::PitcherWild,
        catcher_id: "home1".into(),
        advances: vec![HandlingAdvance {
            runner_id: "away5".into(),
            reached: Destination::Second,
        }],
        pre_delivery_steals: Some(vec![]),
        recovery: RecoveryReview::NoRecoveryPutout,
    });
    let mut legacy = event.clone();
    legacy.balk = None;
    accepted(&before, legacy);
    rejected(&before, event, TransitionError::UnsupportedRule);
}

fn unknown_balk_field(event: &mut Event, unknown: u8) {
    match unknown {
        0 => {}
        1 => event.balk.as_mut().unwrap().cause = BalkCause::Unknown,
        2 => event.balk.as_mut().unwrap().disposition = BalkDisposition::Unknown,
        _ => panic!("unknown synthetic field case"),
    }
}

#[test]
fn automatic_k_retirement_cannot_supply_a_legal_first_touch_even_with_unknown_balk_facts() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..2 {
            for mask in [1, 3, 5, 7] {
                for unknown in 0..3 {
                    let (before, mut event) = fixture(half, outs, mask, 3);
                    retired_batter(&mut event, &before, ThirdOutKind::BatterBeforeFirst);
                    unknown_balk_field(&mut event, unknown);
                    // K/Out itself is legal; the supplied LEGAL first acquisition is false.
                    let mut legacy = event.clone();
                    legacy.balk = None;
                    let control = apply(&before, legacy).unwrap();
                    rejected_with_after(
                        &before,
                        event,
                        control.after().clone(),
                        TransitionError::IllegalCredit,
                    );
                }
            }
        }
    }
}

#[test]
fn eligible_retired_k_with_named_touch_stays_unsupported_without_retirement_history() {
    for half in [Half::Top, Half::Bottom] {
        for (outs, mask) in [(0, 2), (0, 4), (1, 6), (2, 1), (2, 3)] {
            for unknown in 0..3 {
                let (before, mut event) = fixture(half, outs, mask, 3);
                retired_batter(&mut event, &before, ThirdOutKind::BatterAfterFirstTiming);
                unknown_balk_field(&mut event, unknown);
                let mut legacy = event.clone();
                legacy.balk = None;
                let control = apply(&before, legacy).unwrap();
                rejected_with_after(
                    &before,
                    event,
                    control.after().clone(),
                    TransitionError::UnsupportedRule,
                );
            }
        }
    }
}

fn corrupt_before_origin(event: &mut Event, malformed: u8) {
    let index = event
        .delta
        .moves
        .iter()
        .position(|m| m.from == Base::First)
        .unwrap();
    match malformed {
        0 => event.delta.moves.push(event.delta.moves[index].clone()),
        1 => {
            let prefix = if event
                .participants
                .batter
                .as_ref()
                .unwrap()
                .starts_with("away")
            {
                "away"
            } else {
                "home"
            };
            event.delta.moves[index].runner.player_id = format!("{prefix}6");
        }
        2 => {
            event.delta.moves[index].runner.responsible_pitcher =
                event.participants.pitcher.clone();
        }
        _ => panic!("unknown synthetic origin corruption"),
    }
}

fn missing_progress_bad_origin(malformed: u8) {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..=2 {
            for unknown in 0..3 {
                let (mut before, mut event) = fixture(half, outs, 1, 4);
                if malformed == 2 {
                    before.bases[0].as_mut().unwrap().responsible_pitcher = "carried_prior".into();
                    event
                        .delta
                        .moves
                        .iter_mut()
                        .find(|m| m.from == Base::First)
                        .unwrap()
                        .runner = before.bases[0].as_ref().unwrap().clone();
                }
                // Keep the original VALID after-state. A malformed control must not
                // be sent through physical_after and fail before the disputed assertion.
                let mut legacy = event.clone();
                legacy.balk = None;
                let after = apply(&before, legacy).unwrap().after().clone();
                progress(&mut event).runner_next_touches.clear();
                unknown_balk_field(&mut event, unknown);
                corrupt_before_origin(&mut event, malformed);
                rejected_with_after(&before, event, after, TransitionError::DeltaMismatch);
            }
        }
    }
}

#[test]
fn missing_progress_does_not_mask_duplicate_origins() {
    missing_progress_bad_origin(0);
}

#[test]
fn missing_progress_does_not_mask_wrong_before_runner_identity() {
    missing_progress_bad_origin(1);
}

#[test]
fn missing_progress_does_not_mask_wrong_before_runner_carried_responsibility() {
    missing_progress_bad_origin(2);
}

fn wire_string(value: &str, out: &mut Vec<u8>) {
    out.extend_from_slice(&(value.len() as u64).to_le_bytes());
    out.extend_from_slice(value.as_bytes());
}
fn wire_progress(facts: &BalkProgressFacts) -> Vec<u8> {
    let mut out = vec![];
    let touch = |value: &NamedBaseTouch, out: &mut Vec<u8>| {
        wire_string(&value.runner_id, out);
        out.push(match value.base {
            Destination::First => 0,
            Destination::Second => 1,
            Destination::Third => 2,
            Destination::Home => 3,
            Destination::Out => 4,
        });
    };
    touch(&facts.batter_first_touch, &mut out);
    out.extend_from_slice(&(facts.runner_next_touches.len() as u64).to_le_bytes());
    for value in &facts.runner_next_touches {
        touch(value, &mut out);
    }
    out
}
fn forged_progress(before: &GameState, event: &Event) -> Vec<u8> {
    let mut legacy = event.clone();
    legacy.balk = None;
    let bytes = apply(before, legacy).unwrap().to_bytes();
    // Complete known event tail, not an ID/tag substring or stale header needle.
    let mut tail = (event.delta.credits.len() as u64).to_le_bytes().to_vec();
    for c in &event.delta.credits {
        wire_string(&c.player_id, &mut tail);
        tail.push(match c.kind {
            CreditKind::PlateAppearance => 0,
            CreditKind::Strikeout => 1,
            CreditKind::Run => 4,
            _ => panic!("unsupported synthetic wire credit"),
        });
    }
    tail.extend_from_slice(&0u64.to_le_bytes()); // changes
    tail.extend_from_slice(&[0, 0, 0, 0]); // third-out, contact, delivery, balk None
    let offsets: Vec<_> = bytes
        .windows(tail.len())
        .enumerate()
        .filter_map(|(i, window)| (window == tail).then_some(i))
        .collect();
    assert_eq!(offsets.len(), 1, "complete event tail must be unique");
    let offset = offsets[0] + tail.len() - 1;
    let facts = event.balk.as_ref().unwrap();
    let BalkResolution::ActualMinimumProgress(witness) = &facts.resolution else {
        panic!("expected explicit wire witness")
    };
    let mut inserted = vec![
        1,
        match facts.cause {
            BalkCause::Unknown => 0,
            BalkCause::PitcherAction602a => 1,
            BalkCause::SqueezeOrStealHomeInterference601g => 2,
            BalkCause::PitchingProhibition602d => 3,
        },
        match facts.disposition {
            BalkDisposition::Unknown => 0,
            BalkDisposition::PenaltyApplied => 1,
            BalkDisposition::PlayProceedsWithoutReference => 2,
        },
        2,
    ];
    inserted.extend(wire_progress(witness));
    let mut forged = bytes;
    forged.splice(offset..=offset, inserted);
    forged
}

fn wire_moves(moves: &[RunnerMove]) -> Vec<u8> {
    let mut out = (moves.len() as u64).to_le_bytes().to_vec();
    for movement in moves {
        wire_string(&movement.runner.player_id, &mut out);
        wire_string(&movement.runner.responsible_pitcher, &mut out);
        out.push(match movement.from {
            Base::Plate => 0,
            Base::First => 1,
            Base::Second => 2,
            Base::Third => 3,
        });
        out.push(match movement.to {
            Destination::First => 0,
            Destination::Second => 1,
            Destination::Third => 2,
            Destination::Home => 3,
            Destination::Out => 4,
        });
    }
    out
}

#[test]
fn automatic_k_touch_contradictions_are_semantic_decoder_errors() {
    for half in [Half::Top, Half::Bottom] {
        for unknown in 0..3 {
            let (before, mut event) = fixture(half, 0, 1, 3);
            retired_batter(&mut event, &before, ThirdOutKind::BatterBeforeFirst);
            unknown_balk_field(&mut event, unknown);
            assert_eq!(
                EventTransition::from_bytes(&forged_progress(&before, &event)),
                Err(TransitionError::IllegalCredit)
            );
        }
    }
}

#[test]
fn partial_progress_movement_forgeries_reuse_canonical_origin_errors() {
    for malformed in 0..3 {
        for unknown in 0..3 {
            let (mut before, mut event) = fixture(Half::Top, 0, 1, 4);
            if malformed == 2 {
                before.bases[0].as_mut().unwrap().responsible_pitcher = "carried_prior".into();
                event
                    .delta
                    .moves
                    .iter_mut()
                    .find(|m| m.from == Base::First)
                    .unwrap()
                    .runner = before.bases[0].as_ref().unwrap().clone();
            }
            progress(&mut event).runner_next_touches.clear();
            unknown_balk_field(&mut event, unknown);
            let mut bytes = forged_progress(&before, &event);
            assert_eq!(
                EventTransition::from_bytes(&bytes),
                Err(TransitionError::UnsupportedRule),
                "original well-formed partial progress must reach semantic validation"
            );
            let original = wire_moves(&event.delta.moves);
            let offsets: Vec<_> = bytes
                .windows(original.len())
                .enumerate()
                .filter_map(|(i, window)| (window == original).then_some(i))
                .collect();
            assert_eq!(offsets.len(), 1, "complete movement vector must be unique");
            corrupt_before_origin(&mut event, malformed);
            let offset = offsets[0];
            bytes.splice(
                offset..offset + original.len(),
                wire_moves(&event.delta.moves),
            );
            assert_eq!(
                EventTransition::from_bytes(&bytes),
                Err(TransitionError::DeltaMismatch),
                "well-formed movement corruption must not be masked by missing touches"
            );
        }
    }
}

#[test]
fn complete_record_wire_forgeries_share_known_touch_actor_target_and_duplicate_guards() {
    for malformed in 0..4 {
        let (before, mut event) = fixture(Half::Top, 0, 1, 4);
        match malformed {
            0 => progress(&mut event).batter_first_touch.runner_id = "outsider".into(),
            1 => progress(&mut event).runner_next_touches[0].base = Destination::Out,
            2 => {
                let repeated = progress(&mut event).runner_next_touches[0].clone();
                progress(&mut event).runner_next_touches.push(repeated);
            }
            3 => progress(&mut event).runner_next_touches[0].runner_id = "away0".into(),
            _ => unreachable!(),
        }
        assert_eq!(
            EventTransition::from_bytes(&forged_progress(&before, &event)),
            Err(TransitionError::IllegalCredit)
        );
    }
}

#[test]
fn parsed_partial_progress_is_unsupported_and_wire_corruption_remains_invalid_encoding() {
    let (before, mut event) = fixture(Half::Top, 0, 1, 4);
    progress(&mut event).runner_next_touches.clear();
    let partial = forged_progress(&before, &event);
    assert_eq!(
        EventTransition::from_bytes(&partial),
        Err(TransitionError::UnsupportedRule)
    );
    let mut previous = partial.clone();
    previous[..5].copy_from_slice(b"BBE16");
    assert_eq!(
        EventTransition::from_bytes(&previous),
        Err(TransitionError::InvalidEncoding)
    );
    assert_eq!(
        EventTransition::from_bytes(&partial[..partial.len() - 1]),
        Err(TransitionError::InvalidEncoding)
    );
}

#[test]
fn extra_safe_advances_and_multiple_scorers_use_supplied_touches_without_inferring_hit_value() {
    for half in [Half::Top, Half::Bottom] {
        let (before, mut event) = fixture(half, 1, 6, 4);
        event
            .delta
            .moves
            .iter_mut()
            .find(|m| m.from == Base::Second)
            .unwrap()
            .to = Destination::Home;
        event
            .delta
            .moves
            .iter_mut()
            .find(|m| m.from == Base::Plate)
            .unwrap()
            .to = Destination::Second;
        event.delta.runs = 2;
        event.delta.credits.push(credit(
            &before.bases[1].as_ref().unwrap().player_id,
            CreditKind::Run,
        ));
        let transition = accepted(&before, event);
        let accounting = derive_accounting(&[transition]).unwrap();
        assert_eq!(accounting.runs[half.offense()], 2);
        assert_eq!(
            accounting.batters[half.offense()][&before.teams[half.offense()].batting_order[0]]
                .at_bats,
            None
        );
    }
}

#[test]
fn nonterminal_ninth_and_extra_innings_are_supported_not_an_inning_less_than_nine_proxy() {
    for inning in [9, 10] {
        for half in [Half::Top, Half::Bottom] {
            for outs in 0..=2 {
                for mask in 1..8 {
                    let (mut before, event) = fixture(half, outs, mask, 4);
                    before.inning = inning;
                    before.score = [10, 0];
                    let transition = accepted(&before, event);
                    assert!(!transition.after().over);
                    assert_eq!(transition.after().inning, inning);
                    assert_eq!(transition.after().half, half);
                }
            }
        }
    }
}

#[test]
fn nonterminal_out_of_park_homer_keeps_actual_first_touches_and_existing_hit_classification() {
    for half in [Half::Top, Half::Bottom] {
        for review in [FieldingReview::Unknown, FieldingReview::NoMisplay] {
            let (before, mut event) = fixture(half, 0, 7, 4);
            event.kind = EventKind::OutOfParkHomeRun;
            event.fielding_review = review;
            let batter = event.participants.batter.clone().unwrap();
            event.delta.credits.push(credit(&batter, CreditKind::Hit));
            event.delta.credits.retain(|c| c.kind != CreditKind::Run);
            for m in &mut event.delta.moves {
                m.to = Destination::Home;
                event
                    .delta
                    .credits
                    .push(credit(&m.runner.player_id, CreditKind::Run));
            }
            event.delta.runs = 4;
            let transition = accepted(&before, event);
            let accounting = derive_accounting(&[transition]).unwrap();
            let line = &accounting.batters[half.offense()][&batter];
            assert_eq!(line.plate_appearances, 1);
            assert_eq!(line.at_bats, Some(1));
            assert_eq!(line.credited_hit_breakdown.as_ref().unwrap().home_runs, 1);
            assert_eq!(accounting.runs[half.offense()], 4);
        }
    }
}

#[test]
fn invalid_progress_tags_and_touch_destination_are_encoding_failures_not_semantic_unknowns() {
    let (before, mut event) = fixture(Half::Top, 0, 1, 4);
    let bytes = forged_progress(&before, &event);
    assert_eq!(
        EventTransition::from_bytes(&bytes).unwrap(),
        apply(&before, event.clone()).unwrap()
    );
    let mut marker = vec![1, 1, 2, 2];
    marker.extend(wire_progress(progress(&mut event)));
    let offsets: Vec<_> = bytes
        .windows(marker.len())
        .enumerate()
        .filter_map(|(i, window)| (window == marker).then_some(i))
        .collect();
    assert_eq!(offsets.len(), 1, "complete progress record must be unique");
    let offset = offsets[0];
    let first_destination = 4 + 8 + progress(&mut event).batter_first_touch.runner_id.len();
    for field in [1, 2, 3, first_destination] {
        let mut bad = bytes.clone();
        bad[offset + field] = 255;
        assert_eq!(
            EventTransition::from_bytes(&bad),
            Err(TransitionError::InvalidEncoding)
        );
    }
}

#[test]
fn play_proceeds_run_charge_stays_carried_while_pa_belongs_to_actual_pitcher() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..=2 {
            let (mut before, mut event) = fixture(half, outs, 4, 0);
            let current = event.participants.pitcher.clone();
            before.bases[2].as_mut().unwrap().responsible_pitcher = "prior_pitcher".into();
            event
                .delta
                .moves
                .iter_mut()
                .find(|m| m.from == Base::Third)
                .unwrap()
                .runner = before.bases[2].clone().unwrap();
            let transition = accepted(&before, event);
            let accounting = derive_accounting(&[transition]).unwrap();
            let defense = 1 - half.offense();
            assert_eq!(accounting.pitchers[defense][&current].batters_faced, 1);
            assert_eq!(accounting.pitchers[defense][&current].runs_allowed, 0);
            assert_eq!(accounting.pitchers[defense][&current].credited_balks, 0);
            assert_eq!(
                accounting.pitchers[defense]["prior_pitcher"].batters_faced,
                0
            );
            assert_eq!(
                accounting.pitchers[defense]["prior_pitcher"].runs_allowed,
                1
            );
            assert_eq!(
                accounting.pitchers[defense]["prior_pitcher"].credited_balks,
                0
            );
            assert_eq!(
                accounting.pitchers[defense]["prior_pitcher"].earned_runs,
                None
            );
        }
    }
}
