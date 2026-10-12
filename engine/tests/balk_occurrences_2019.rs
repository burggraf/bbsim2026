use bbsim_engine::accounting::derive_accounting;
use bbsim_engine::{
    apply, BalkCause, BalkDisposition, BalkFacts, BalkResolution, Base, Credit, CreditKind,
    DeliveryContext, Delta, Destination, Event, EventKind, EventTransition, FieldingReview,
    GameState, Half, HandlingAdvance, HandlingAssessment, Participants, RecoveryReview, Runner,
    RunnerEvent, RunnerMove, SingleDeliveryFacts, TeamState, ThirdOut, ThirdOutKind,
    TransitionError,
};

fn credit(id: &str, kind: CreditKind) -> Credit {
    Credit {
        player_id: id.into(),
        kind,
    }
}

fn fixture(half: Half, outs: u8, mask: u8) -> (GameState, Event) {
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
    let moves: Vec<_> = bases
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
    let mut credits = vec![credit(&pitcher, CreditKind::Balk)];
    if let Some(lead) = &bases[2] {
        credits.push(credit(&lead.player_id, CreditKind::Run));
    }
    let event = Event {
        kind: EventKind::RunnerOnly(RunnerEvent::Balk),
        participants: Participants {
            batter: Some(batter),
            pitcher,
        },
        fielding_review: FieldingReview::Unknown,
        two_out_contact: None,
        single_delivery: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        ordinary_ground_throw: None,
        balk: Some(BalkFacts {
            cause: BalkCause::PitcherAction602a,
            disposition: BalkDisposition::PenaltyApplied,
            resolution: BalkResolution::ExactProtectedAwards,
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
    let saved = before.clone();
    let transition = apply(before, event.clone()).unwrap();
    assert_eq!(*before, saved);
    assert_eq!(
        EventTransition::try_new(before.clone(), event, transition.after().clone()).unwrap(),
        transition
    );
    let bytes = transition.to_bytes();
    assert_eq!(&bytes[..5], b"BBE24");
    assert_eq!(EventTransition::from_bytes(&bytes).unwrap(), transition);
    transition
}

#[test]
fn one_imposed_ordinary_occurrence_counts_once_for_all_exact_award_masks() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..=2 {
            for mask in 1..8 {
                for reviewed in [false, true] {
                    for reverse in [false, true] {
                        let (before, mut event) = fixture(half, outs, mask);
                        let pitcher = event.participants.pitcher.clone();
                        if reviewed {
                            event.fielding_review = FieldingReview::NoMisplay;
                        }
                        if reverse {
                            event.delta.moves.reverse();
                            event.delta.credits.reverse();
                        }
                        let transition = accepted(&before, event);
                        assert_eq!(transition.after().teams, before.teams);
                        assert_eq!(transition.after().outs, outs);
                        assert_eq!(
                            transition.after().score[half.offense()],
                            u16::from(mask & 4 != 0)
                        );
                        assert_eq!(
                            transition.after().bases,
                            [None, before.bases[0].clone(), before.bases[1].clone()]
                        );
                        let accounting = derive_accounting(&[transition]).unwrap();
                        let line = &accounting.pitchers[1 - half.offense()][&pitcher];
                        assert_eq!(
                            (line.credited_balks, line.batters_faced, line.outs_recorded),
                            (1, 0, 0)
                        );
                        assert!(!accounting.batters[half.offense()].contains_key(&pitcher));
                        assert_eq!(
                            accounting.batters[half.offense()].len(),
                            usize::from(mask & 4 != 0)
                        );
                        assert!(!accounting.complete_game);
                    }
                }
            }
        }
    }
}

#[test]
fn current_pitcher_balk_and_inherited_scoring_responsibility_are_separate() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..=2 {
            let (mut before, mut event) = fixture(half, outs, 5);
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
            assert_eq!(accounting.pitchers[defense][&current].credited_balks, 1);
            assert_eq!(accounting.pitchers[defense][&current].runs_allowed, 0);
            assert_eq!(accounting.pitchers[defense][&current].batters_faced, 0);
            assert_eq!(
                accounting.pitchers[defense]["prior_pitcher"].credited_balks,
                0
            );
            assert_eq!(
                accounting.pitchers[defense]["prior_pitcher"].runs_allowed,
                1
            );
            assert_eq!(
                accounting.pitchers[defense]["prior_pitcher"].batters_faced,
                0
            );
            assert_eq!(
                accounting.pitchers[defense]["prior_pitcher"].earned_runs,
                None
            );
            assert!(!accounting.batters[half.offense()].contains_key(&current));
            assert!(!accounting.batters[half.offense()].contains_key("prior_pitcher"));
        }
    }
}

// Derive a valid physical after-state from the legacy partial view, not from
// unimplemented balk qualification or malformed bytes.
fn physical_after(before: &GameState, event: &Event) -> GameState {
    let mut partial = event.clone();
    partial.balk = None;
    partial.single_delivery = None;
    partial.fielding_review = FieldingReview::Unknown;
    let is_pa = matches!(
        partial.kind,
        EventKind::PlateAppearance | EventKind::OutOfParkHomeRun | EventKind::FieldersChoice
    );
    partial.delta.credits.retain(|c| {
        !matches!(
            c.kind,
            CreditKind::Balk
                | CreditKind::WildPitch
                | CreditKind::PassedBall
                | CreditKind::StolenBase
                | CreditKind::Error
        ) && (c.kind != CreditKind::PlateAppearance || is_pa)
    });
    apply(before, partial).unwrap().after().clone()
}
fn rejected(before: &GameState, event: Event, expected: TransitionError) {
    let after = physical_after(before, &event);
    let saved = before.clone();
    assert_eq!(apply(before, event.clone()), Err(expected.clone()));
    assert_eq!(*before, saved);
    assert_eq!(
        EventTransition::try_new(before.clone(), event, after),
        Err(expected)
    );
    assert_eq!(*before, saved);
}
fn without_unit(event: &mut Event) {
    event.delta.credits.retain(|c| c.kind != CreditKind::Balk);
}

#[test]
fn missing_or_unknown_occurrence_facts_cannot_qualify_positive_balk_units() {
    for uncertain in 0..4 {
        let (before, mut event) = fixture(Half::Top, 0, 1);
        match uncertain {
            0 => event.balk = None,
            1 => event.balk.as_mut().unwrap().cause = BalkCause::Unknown,
            2 => event.balk.as_mut().unwrap().disposition = BalkDisposition::Unknown,
            _ => event.balk.as_mut().unwrap().resolution = BalkResolution::Unknown,
        }
        let mut partial = event.clone();
        without_unit(&mut partial);
        let control = accepted(&before, partial);
        assert_eq!(
            derive_accounting(&[control]).unwrap().pitchers[1]["home_pitcher"].credited_balks,
            0
        );
        rejected(&before, event, TransitionError::UnsupportedRule);
    }
}

#[test]
fn explicitly_qualified_imposed_occurrence_cannot_omit_its_one_unit() {
    for half in [Half::Top, Half::Bottom] {
        for mask in 1..8 {
            let (before, mut event) = fixture(half, 1, mask);
            without_unit(&mut event);
            rejected(&before, event, TransitionError::IllegalCredit);
        }
    }
}

#[test]
fn duplicate_units_are_illegal_for_one_typed_occurrence_not_a_universal_history_cap() {
    let (before, event) = fixture(Half::Top, 0, 1);
    for unknown in [false, true] {
        let mut duplicate = event.clone();
        duplicate
            .delta
            .credits
            .push(duplicate.delta.credits[0].clone());
        if unknown {
            duplicate.balk.as_mut().unwrap().disposition = BalkDisposition::Unknown;
        }
        rejected(&before, duplicate, TransitionError::IllegalCredit);
    }
    let mut unbound = event;
    unbound.balk = None;
    unbound.delta.credits.push(unbound.delta.credits[0].clone());
    rejected(&before, unbound, TransitionError::UnsupportedRule);
}

#[test]
fn wrong_balk_actor_is_known_before_missing_occurrence_context() {
    for unknown in 0..4 {
        let (before, mut event) = fixture(Half::Top, 0, 1);
        event.delta.credits[0].player_id = "prior_pitcher".into();
        match unknown {
            0 => event.balk = None,
            1 => event.balk.as_mut().unwrap().cause = BalkCause::Unknown,
            2 => event.balk.as_mut().unwrap().disposition = BalkDisposition::Unknown,
            _ => event.balk.as_mut().unwrap().resolution = BalkResolution::Unknown,
        }
        rejected(&before, event, TransitionError::IllegalCredit);
    }
}

#[test]
fn exact_before_state_awards_require_runners_but_a_bare_label_is_partial() {
    let (before, event) = fixture(Half::Top, 0, 0);
    let mut partial = event.clone();
    partial.balk = None;
    without_unit(&mut partial);
    accepted(&before, partial);
    rejected(&before, event, TransitionError::IllegalCredit);
}

#[test]
fn an_ordinary_disregarded_occurrence_cannot_supply_balk_but_needs_progress_proof() {
    let (before, mut event) = fixture(Half::Top, 0, 1);
    event.balk.as_mut().unwrap().disposition = BalkDisposition::PlayProceedsWithoutReference;
    event.balk.as_mut().unwrap().resolution = BalkResolution::Unknown;
    rejected(&before, event.clone(), TransitionError::IllegalCredit);
    without_unit(&mut event);
    rejected(&before, event, TransitionError::UnsupportedRule);
}

#[test]
fn extra_held_missing_or_retired_effects_contradict_exact_witness_not_all_balk_histories() {
    for effect in 0..4 {
        let (before, mut event) = fixture(Half::Top, 0, 1);
        match effect {
            0 => event.delta.moves[0].to = Destination::Third,
            1 => event.delta.moves[0].to = Destination::First,
            2 => {
                event.delta.moves[0].to = Destination::Out;
                event.delta.outs = 1;
            }
            _ => event.delta.moves.clear(),
        }
        rejected(&before, event.clone(), TransitionError::IllegalCredit);
        event.balk.as_mut().unwrap().resolution = BalkResolution::Unknown;
        rejected(&before, event, TransitionError::UnsupportedRule);
    }
}

#[test]
fn same_award_steal_is_contradictory_but_unbound_independent_history_is_unsupported() {
    let (before, mut event) = fixture(Half::Top, 0, 1);
    let runner = before.bases[0].as_ref().unwrap().player_id.clone();
    event
        .delta
        .credits
        .push(credit(&runner, CreditKind::StolenBase));
    rejected(&before, event.clone(), TransitionError::IllegalCredit);
    event.delta.moves[0].to = Destination::Third;
    event.balk.as_mut().unwrap().resolution = BalkResolution::Unknown;
    rejected(&before, event, TransitionError::UnsupportedRule);
}

#[test]
fn error_chance_or_pitch_episode_binding_is_not_inferred_from_a_balk_label() {
    let (before, mut event) = fixture(Half::Top, 0, 1);
    event.delta.credits.push(credit("home1", CreditKind::Error));
    rejected(&before, event.clone(), TransitionError::UnsupportedRule);
    event.fielding_review = FieldingReview::NoMisplay;
    rejected(&before, event, TransitionError::IllegalCredit);
    let (before, mut event) = fixture(Half::Top, 0, 1);
    event.single_delivery = Some(SingleDeliveryFacts {
        context: DeliveryContext::RunnerOnly,
        handling: HandlingAssessment::PitcherWild,
        catcher_id: "home1".into(),
        advances: vec![HandlingAdvance {
            runner_id: "away5".into(),
            reached: Destination::Second,
        }],
        pre_delivery_steals: Some(vec![]),
        recovery: RecoveryReview::NoRecoveryPutout,
    });
    rejected(&before, event, TransitionError::UnsupportedRule);
}

fn other_cause_pa(cause: BalkCause) -> (GameState, Event) {
    let (before, mut event) = fixture(Half::Top, 0, 4);
    event.kind = EventKind::PlateAppearance;
    event.delta.moves.push(RunnerMove {
        runner: Runner {
            player_id: "away0".into(),
            responsible_pitcher: "home_pitcher".into(),
        },
        from: Base::Plate,
        to: Destination::First,
    });
    event
        .delta
        .credits
        .push(credit("away0", CreditKind::PlateAppearance));
    event.delta.credits.push(credit(
        "away0",
        if cause == BalkCause::PitchingProhibition602d {
            CreditKind::Walk
        } else {
            CreditKind::InterferenceAward
        },
    ));
    event.balk.as_mut().unwrap().cause = cause;
    event.balk.as_mut().unwrap().resolution = BalkResolution::Unknown;
    (before, event)
}

#[test]
fn other_balk_causes_with_batter_awards_are_unfinished_not_universally_illegal() {
    for cause in [
        BalkCause::SqueezeOrStealHomeInterference601g,
        BalkCause::PitchingProhibition602d,
    ] {
        let (before, event) = other_cause_pa(cause);
        rejected(&before, event, TransitionError::UnsupportedRule);
    }
}

#[test]
fn explicit_interference_cause_contradicts_no_misplay_even_without_interference_credit() {
    let (before, mut event) = other_cause_pa(BalkCause::SqueezeOrStealHomeInterference601g);
    event.fielding_review = FieldingReview::NoMisplay;
    event
        .delta
        .credits
        .retain(|c| c.kind != CreditKind::InterferenceAward);
    event.balk.as_mut().unwrap().disposition = BalkDisposition::Unknown;
    rejected(&before, event, TransitionError::IllegalCredit);
}

#[test]
fn separate_imposed_occurrences_accumulate_but_do_not_extend_earned_run_proof() {
    let (before, event) = fixture(Half::Top, 0, 1);
    let first = accepted(&before, event);
    let mut log = vec![first];
    for from in [Base::Second, Base::Third] {
        let state = log.last().unwrap().after();
        let mut event = log[0].event().clone();
        let index = if from == Base::Second { 1 } else { 2 };
        event.delta.moves[0].from = from;
        event.delta.moves[0].runner = state.bases[index].clone().unwrap();
        event.delta.moves[0].to = if from == Base::Second {
            Destination::Third
        } else {
            Destination::Home
        };
        event.delta.runs = u8::from(from == Base::Third);
        if from == Base::Third {
            event.delta.credits.push(credit("away5", CreditKind::Run));
        }
        log.push(accepted(state, event));
    }
    assert!(log.windows(2).all(|w| w[1].follows(&w[0]).is_ok()));
    let accounting = derive_accounting(&log).unwrap();
    assert_eq!(accounting.pitchers[1]["home_pitcher"].credited_balks, 3);
    assert_eq!(accounting.pitchers[1]["home_pitcher"].batters_faced, 0);
    assert_eq!(accounting.pitchers[1]["home_pitcher"].earned_runs, None);
    assert!(!accounting.complete_game);
}

fn wire_credits(credits: &[Credit]) -> Vec<u8> {
    let mut bytes = (credits.len() as u64).to_le_bytes().to_vec();
    for c in credits {
        bytes.extend((c.player_id.len() as u64).to_le_bytes());
        bytes.extend(c.player_id.as_bytes());
        bytes.push(match c.kind {
            CreditKind::Balk => 24,
            CreditKind::PlateAppearance => 0,
            CreditKind::Strikeout => 1,
            CreditKind::Run => 4,
            CreditKind::StolenBase => 6,
            CreditKind::Error => 10,
            CreditKind::PassedBall => 11,
            CreditKind::WildPitch => 23,
            _ => panic!("unsupported credit in this wire fixture"),
        });
    }
    bytes
}
fn unique_offset(bytes: &[u8], needle: &[u8]) -> usize {
    let offsets: Vec<_> = bytes
        .windows(needle.len())
        .enumerate()
        .filter_map(|(i, b)| (b == needle).then_some(i))
        .collect();
    assert_eq!(
        offsets.len(),
        1,
        "wire fixture must select one complete record"
    );
    offsets[0]
}

#[test]
fn well_formed_balk_fact_and_credit_forgeries_use_shared_semantic_guards() {
    let (before, event) = fixture(Half::Top, 0, 1);
    let control = accepted(&before, event.clone());
    let original = control.to_bytes();
    let mut marker = wire_credits(&event.delta.credits);
    marker.extend(0u64.to_le_bytes()); // changes
    marker.extend([0, 0, 0, 1, 1, 1, 1]); // third/contact/delivery None; Balk Some/cause/disposition/resolution
    let offset = unique_offset(&original, &marker);
    let end = offset + marker.len();
    for (distance, tag, expected) in [
        (3, 0, TransitionError::UnsupportedRule),
        (2, 0, TransitionError::UnsupportedRule),
        (1, 0, TransitionError::UnsupportedRule),
        (2, 2, TransitionError::IllegalCredit),
    ] {
        let mut bytes = original.clone();
        bytes[end - distance] = tag;
        assert_eq!(EventTransition::from_bytes(&bytes), Err(expected));
    }
    let mut missing = original.clone();
    missing.splice(end - 4..end, [0]); // valid Balk None, retain physical snapshots
    assert_eq!(
        EventTransition::from_bytes(&missing),
        Err(TransitionError::UnsupportedRule)
    );
    for count in [0, 2] {
        let credits = vec![event.delta.credits[0].clone(); count];
        let mut bytes = original.clone();
        bytes.splice(
            offset..offset + wire_credits(&event.delta.credits).len(),
            wire_credits(&credits),
        );
        assert_eq!(
            EventTransition::from_bytes(&bytes),
            Err(TransitionError::IllegalCredit)
        );
    }
    let mut bad_tag = original.clone();
    bad_tag[end - 3] = 255;
    assert_eq!(
        EventTransition::from_bytes(&bad_tag),
        Err(TransitionError::InvalidEncoding)
    );
    assert_eq!(EventTransition::from_bytes(&original).unwrap(), control);
}

#[test]
fn definite_credit_actor_pa_and_review_errors_precede_unknown_balk_resolution() {
    for contradiction in 0..3 {
        let (before, mut event) = fixture(Half::Top, 0, 1);
        event.balk.as_mut().unwrap().resolution = BalkResolution::Unknown;
        match contradiction {
            0 => event
                .delta
                .credits
                .push(credit("away0", CreditKind::StolenBase)),
            1 => {
                event.fielding_review = FieldingReview::NoMisplay;
                event.delta.credits.push(credit("home1", CreditKind::Error));
            }
            _ => event
                .delta
                .credits
                .push(credit("away0", CreditKind::PlateAppearance)),
        }
        rejected(&before, event, TransitionError::IllegalCredit);
    }
}

#[test]
fn ordinary_exact_award_witness_cannot_also_claim_batter_pa_completion() {
    let (before, mut event) = fixture(Half::Top, 0, 1);
    event.kind = EventKind::PlateAppearance;
    event.delta.moves.push(RunnerMove {
        runner: Runner {
            player_id: "away0".into(),
            responsible_pitcher: "home_pitcher".into(),
        },
        from: Base::Plate,
        to: Destination::First,
    });
    event
        .delta
        .credits
        .push(credit("away0", CreditKind::PlateAppearance));
    event.delta.credits.push(credit("away0", CreditKind::Walk));
    rejected(&before, event, TransitionError::IllegalCredit);
}

#[test]
fn known_current_participant_mismatch_precedes_unknown_balk_resolution() {
    let (before, event) = fixture(Half::Top, 0, 1);
    let after = physical_after(&before, &event);
    for mismatch in 0..2 {
        let mut bad = event.clone();
        bad.balk.as_mut().unwrap().resolution = BalkResolution::Unknown;
        if mismatch == 0 {
            bad.participants.pitcher = "prior_pitcher".into();
        } else {
            bad.participants.batter = None;
        }
        assert_eq!(
            apply(&before, bad.clone()),
            Err(TransitionError::InvalidParticipants)
        );
        assert_eq!(
            EventTransition::try_new(before.clone(), bad, after.clone()),
            Err(TransitionError::InvalidParticipants)
        );
    }
}

#[test]
fn known_invalid_before_state_precedes_balk_history_uncertainty_in_both_paths() {
    let (before, event) = fixture(Half::Top, 0, 1);
    let after = physical_after(&before, &event);
    for invalid in 0..6 {
        let mut bad_before = before.clone();
        let mut bad_event = event.clone();
        bad_event.balk.as_mut().unwrap().resolution = BalkResolution::Unknown;
        match invalid {
            0 => bad_before.inning = 0,
            1 => bad_before.outs = 3,
            2 => {
                bad_before.teams[0].next_batter = 5;
                bad_event.participants.batter = Some("away5".into());
            }
            3 => bad_before.bases[0]
                .as_mut()
                .unwrap()
                .responsible_pitcher
                .clear(),
            4 => bad_before.bases[1] = bad_before.bases[0].clone(),
            _ => bad_before.over = true,
        }
        let saved = bad_before.clone();
        assert_eq!(
            apply(&bad_before, bad_event.clone()),
            Err(TransitionError::InvalidState)
        );
        assert_eq!(bad_before, saved);
        assert_eq!(
            EventTransition::try_new(bad_before.clone(), bad_event, after.clone()),
            Err(TransitionError::InvalidState)
        );
    }
}

#[test]
fn unit_omission_duplication_and_wrong_actor_are_conserved_across_all_exact_frames() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..=2 {
            for mask in 1..8 {
                for review in [FieldingReview::Unknown, FieldingReview::NoMisplay] {
                    for reverse in [false, true] {
                        let (before, mut event) = fixture(half, outs, mask);
                        event.fielding_review = review;
                        if reverse {
                            event.delta.moves.reverse();
                            event.delta.credits.reverse();
                        }
                        for contradiction in 0..3 {
                            let mut bad = event.clone();
                            match contradiction {
                                0 => without_unit(&mut bad),
                                1 => {
                                    let unit = bad
                                        .delta
                                        .credits
                                        .iter()
                                        .find(|c| c.kind == CreditKind::Balk)
                                        .unwrap()
                                        .clone();
                                    bad.delta.credits.push(unit);
                                    bad.balk.as_mut().unwrap().disposition =
                                        BalkDisposition::Unknown;
                                }
                                _ => {
                                    bad.delta
                                        .credits
                                        .iter_mut()
                                        .find(|c| c.kind == CreditKind::Balk)
                                        .unwrap()
                                        .player_id = "prior_pitcher".into();
                                    bad.balk.as_mut().unwrap().resolution = BalkResolution::Unknown;
                                }
                            }
                            rejected(&before, bad, TransitionError::IllegalCredit);
                        }
                    }
                }
            }
        }
    }
}

fn unresolved_facts(case: u8) -> BalkFacts {
    BalkFacts {
        cause: if case & 1 == 0 {
            BalkCause::PitcherAction602a
        } else {
            BalkCause::Unknown
        },
        disposition: if case & 2 == 0 {
            BalkDisposition::PenaltyApplied
        } else {
            BalkDisposition::Unknown
        },
        resolution: BalkResolution::Unknown,
    }
}

fn pitch_with_unbound_balk(passed: bool) -> (GameState, Event) {
    let (before, mut event) = fixture(Half::Top, 0, 1);
    without_unit(&mut event);
    event.balk = Some(unresolved_facts(0));
    event.kind = EventKind::RunnerOnly(if passed {
        RunnerEvent::PassedBall
    } else {
        RunnerEvent::WildPitch
    });
    event.delta.credits.push(credit(
        if passed { "home1" } else { "home_pitcher" },
        if passed {
            CreditKind::PassedBall
        } else {
            CreditKind::WildPitch
        },
    ));
    event.single_delivery = Some(SingleDeliveryFacts {
        context: DeliveryContext::RunnerOnly,
        handling: if passed {
            HandlingAssessment::CatcherPassed
        } else {
            HandlingAssessment::PitcherWild
        },
        catcher_id: "home1".into(),
        advances: vec![HandlingAdvance {
            runner_id: "away5".into(),
            reached: Destination::Second,
        }],
        pre_delivery_steals: Some(vec![]),
        recovery: RecoveryReview::NoRecoveryPutout,
    });
    (before, event)
}

#[test]
fn omitted_balk_does_not_bind_an_asserted_delivery_or_legacy_pb_episode() {
    for passed in [false, true] {
        let (before, event) = pitch_with_unbound_balk(passed);
        let mut legacy = event.clone();
        legacy.balk = None;
        accepted(&before, legacy);
        for unknown in 0..4 {
            let mut bad = event.clone();
            bad.balk = Some(unresolved_facts(unknown));
            rejected(&before, bad.clone(), TransitionError::UnsupportedRule);
            if passed {
                bad.single_delivery = None;
                rejected(&before, bad, TransitionError::UnsupportedRule);
            }
        }
    }
}

#[test]
fn omitted_balk_does_not_bind_an_asserted_error_chance() {
    let (before, mut event) = fixture(Half::Top, 0, 1);
    without_unit(&mut event);
    event.delta.credits.push(credit("home1", CreditKind::Error));
    let mut legacy = event.clone();
    legacy.balk = None;
    accepted(&before, legacy);
    for unknown in 0..4 {
        event.balk = Some(unresolved_facts(unknown));
        rejected(&before, event.clone(), TransitionError::UnsupportedRule);
    }
}

#[test]
fn omitted_balk_does_not_bind_a_completed_independent_steal() {
    let (before, mut event) = fixture(Half::Top, 0, 1);
    without_unit(&mut event);
    event.delta.moves[0].to = Destination::Third;
    event
        .delta
        .credits
        .push(credit("away5", CreditKind::StolenBase));
    let mut legacy = event.clone();
    legacy.balk = None;
    accepted(&before, legacy);
    for unknown in 0..4 {
        event.balk = Some(unresolved_facts(unknown));
        rejected(&before, event.clone(), TransitionError::UnsupportedRule);
    }
}

#[test]
fn omitted_balk_does_not_disprove_a_steal_before_final_retirement() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..=2 {
            let (before, mut event) = fixture(half, outs, 1);
            let runner = before.bases[0].as_ref().unwrap().player_id.clone();
            without_unit(&mut event);
            event.delta.moves[0].to = Destination::Out;
            event.delta.outs = 1;
            event
                .delta
                .credits
                .push(credit(&runner, CreditKind::StolenBase));
            for unknown in 0..4 {
                event.balk = Some(unresolved_facts(unknown));
                rejected(&before, event.clone(), TransitionError::UnsupportedRule);
            }
        }
    }
}

fn retired_batter(outs: u8, mask: u8, third: Option<ThirdOutKind>) -> (GameState, Event) {
    let (before, mut event) = fixture(Half::Top, outs, mask);
    event.kind = EventKind::PlateAppearance;
    event.balk = Some(unresolved_facts(0));
    event.delta.moves.push(RunnerMove {
        runner: Runner {
            player_id: "away0".into(),
            responsible_pitcher: "home_pitcher".into(),
        },
        from: Base::Plate,
        to: Destination::Out,
    });
    event.delta.outs = 1;
    event.delta.credits.extend([
        credit("away0", CreditKind::PlateAppearance),
        credit("away0", CreditKind::Strikeout),
        credit("away0", CreditKind::StolenBase),
    ]);
    event.delta.third_out = third.map(|kind| ThirdOut {
        runner_id: "away0".into(),
        kind,
        home_before_out: vec![],
    });
    (before, event)
}

#[test]
fn automatic_k_batter_steal_is_known_before_missing_balk_history_or_unit() {
    for outs in 0..2 {
        for supplied in [true, false] {
            let (before, mut event) = retired_batter(outs, 1, None);
            if !supplied {
                without_unit(&mut event);
            }
            for unknown in 0..4 {
                event.balk = Some(unresolved_facts(unknown));
                rejected(&before, event.clone(), TransitionError::IllegalCredit);
            }
        }
    }
}

#[test]
fn explicit_before_first_batter_steal_is_known_before_missing_balk_history_or_unit() {
    for supplied in [true, false] {
        for strikeout in [true, false] {
            let (before, mut event) = retired_batter(2, 1, Some(ThirdOutKind::BatterBeforeFirst));
            if !supplied {
                without_unit(&mut event);
            }
            if !strikeout {
                event
                    .delta
                    .credits
                    .retain(|c| c.kind != CreditKind::Strikeout);
            }
            for unknown in 0..4 {
                event.balk = Some(unresolved_facts(unknown));
                rejected(&before, event.clone(), TransitionError::IllegalCredit);
            }
        }
    }
}

#[test]
fn eligible_or_after_first_batter_retirement_keeps_unknown_steal_chronology() {
    for (outs, mask, third) in [
        (0, 2, None),
        (2, 1, Some(ThirdOutKind::BatterAfterFirstTiming)),
    ] {
        for supplied in [true, false] {
            let (before, mut event) = retired_batter(outs, mask, third);
            if !supplied {
                without_unit(&mut event);
            }
            rejected(&before, event, TransitionError::UnsupportedRule);
        }
    }
}

// Forge only a complete credit vector and optional Balk record in an accepted
// legacy physical control. The full event tail must occur exactly once;
// unchanged snapshots/pitch/retirement fields are retained.
fn forged_balk_bytes(before: &GameState, event: &Event) -> Vec<u8> {
    let mut control = event.clone();
    control.balk = None;
    control
        .delta
        .credits
        .retain(|c| !matches!(c.kind, CreditKind::Balk | CreditKind::StolenBase));
    let transition = accepted(before, control.clone());
    let mut bytes = transition.to_bytes();
    let mut marker = wire_credits(&control.delta.credits);
    marker.extend(0u64.to_le_bytes());
    if let Some(third) = &control.delta.third_out {
        marker.push(1);
        marker.extend((third.runner_id.len() as u64).to_le_bytes());
        marker.extend(third.runner_id.as_bytes());
        marker.push(match third.kind {
            ThirdOutKind::BatterBeforeFirst => 0,
            ThirdOutKind::BatterAfterFirstTiming => 4,
            _ => panic!("unsupported third-out fixture"),
        });
        assert!(third.home_before_out.is_empty());
        marker.extend(0u64.to_le_bytes());
    } else {
        marker.push(0);
    }
    assert!(control.two_out_contact.is_none());
    marker.push(0);
    if let Some(facts) = &control.single_delivery {
        assert_eq!(facts.context, DeliveryContext::RunnerOnly);
        assert_eq!(facts.pre_delivery_steals, Some(vec![]));
        assert_eq!(facts.recovery, RecoveryReview::NoRecoveryPutout);
        marker.extend([
            1,
            0,
            u8::from(facts.handling == HandlingAssessment::CatcherPassed),
        ]);
        marker.extend((facts.catcher_id.len() as u64).to_le_bytes());
        marker.extend(facts.catcher_id.as_bytes());
        marker.extend((facts.advances.len() as u64).to_le_bytes());
        for advance in &facts.advances {
            marker.extend((advance.runner_id.len() as u64).to_le_bytes());
            marker.extend(advance.runner_id.as_bytes());
            assert_eq!(advance.reached, Destination::Second);
            marker.push(1);
        }
        marker.push(1);
        marker.extend(0u64.to_le_bytes());
        marker.push(1);
    } else {
        marker.push(0);
    }
    marker.push(0); // legacy Balk None
    let offset = unique_offset(&bytes, &marker);
    let facts = event.balk.as_ref().unwrap();
    let end = offset + marker.len();
    bytes.splice(
        end - 1..end,
        [
            1,
            if facts.cause == BalkCause::Unknown {
                0
            } else {
                1
            },
            if facts.disposition == BalkDisposition::Unknown {
                0
            } else {
                1
            },
            0,
        ],
    );
    bytes.splice(
        offset..offset + wire_credits(&control.delta.credits).len(),
        wire_credits(&event.delta.credits),
    );
    bytes
}

#[test]
fn semantic_codec_cannot_bypass_pitch_or_error_binding_by_omitting_balk() {
    for passed in [false, true] {
        let (before, mut event) = pitch_with_unbound_balk(passed);
        for unknown in 0..4 {
            event.balk = Some(unresolved_facts(unknown));
            assert_eq!(
                EventTransition::from_bytes(&forged_balk_bytes(&before, &event)),
                Err(TransitionError::UnsupportedRule)
            );
        }
    }
    let (before, mut event) = fixture(Half::Top, 0, 1);
    without_unit(&mut event);
    event.balk = Some(unresolved_facts(0));
    event.delta.credits.push(credit("home1", CreditKind::Error));
    assert_eq!(
        EventTransition::from_bytes(&forged_balk_bytes(&before, &event)),
        Err(TransitionError::UnsupportedRule)
    );
}

#[test]
fn semantic_codec_preserves_unbound_safe_and_retired_steal_history_without_balk_unit() {
    for retired in [false, true] {
        let (before, mut event) = fixture(Half::Top, 0, 5);
        without_unit(&mut event);
        event.balk = Some(unresolved_facts(0));
        event.delta.moves[0].to = if retired {
            Destination::Out
        } else {
            Destination::Third
        };
        event.delta.outs = u8::from(retired);
        event
            .delta
            .credits
            .push(credit("away5", CreditKind::StolenBase));
        assert_eq!(
            EventTransition::from_bytes(&forged_balk_bytes(&before, &event)),
            Err(TransitionError::UnsupportedRule)
        );
    }
}

#[test]
fn semantic_codec_rejects_known_before_first_batter_steal_before_balk_uncertainty() {
    for (outs, third) in [(0, None), (2, Some(ThirdOutKind::BatterBeforeFirst))] {
        let (before, event) = retired_batter(outs, 1, third);
        assert_eq!(
            EventTransition::from_bytes(&forged_balk_bytes(&before, &event)),
            Err(TransitionError::IllegalCredit)
        );
    }
}

#[test]
fn a_final_retirement_does_not_disprove_prior_independent_steal_in_unknown_balk_history() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..=2 {
            let (before, mut event) = fixture(half, outs, 1);
            let runner = before.bases[0].as_ref().unwrap().player_id.clone();
            event.delta.moves[0].to = Destination::Out;
            event.delta.outs = 1;
            event.balk.as_mut().unwrap().resolution = BalkResolution::Unknown;
            event
                .delta
                .credits
                .push(credit(&runner, CreditKind::StolenBase));
            rejected(&before, event, TransitionError::UnsupportedRule);
        }
    }
}
