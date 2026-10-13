use bbsim_engine::accounting::derive_accounting;
use bbsim_engine::{
    apply, Base, ContactClass, ContactContinuity, Credit, CreditKind, Delta, Destination, Event,
    EventKind, EventTransition, FieldingReview, ForceLoss, GameState, Half, Participants,
    RetirementCause, RetirementFact, ReviewedErrorPlacement, Runner, RunnerMove, TeamState,
    ThirdOut, ThirdOutKind, TransitionError, TwoOutContactFacts,
};

fn credit(id: &str, kind: CreditKind) -> Credit {
    Credit {
        player_id: id.into(),
        kind,
    }
}

fn fixture(half: Half, outs: u8, reverse_force: bool, crossing: bool) -> (GameState, Event) {
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
    let batter = teams[offense].batting_order[0].clone();
    let pitcher = teams[1 - offense].pitcher.clone();
    let runner = Runner {
        player_id: teams[offense].batting_order[5].clone(),
        responsible_pitcher: pitcher.clone(),
    };
    let lead = crossing.then(|| Runner {
        player_id: teams[offense].batting_order[7].clone(),
        responsible_pitcher: pitcher.clone(),
    });
    let before = GameState {
        inning: 2,
        half,
        outs,
        score: [0, 0],
        teams,
        bases: [Some(runner.clone()), None, lead.clone()],
        over: false,
    };
    let batter_out = RetirementFact {
        runner_id: batter.clone(),
        cause: RetirementCause::BatterPutoutBeforeFirstTouch,
    };
    let runner_out = RetirementFact {
        runner_id: runner.player_id.clone(),
        cause: if reverse_force {
            RetirementCause::Tag {
                at: Destination::Second,
                force_loss: ForceLoss::RemovedByFirstOut,
            }
        } else {
            RetirementCause::Force {
                at: Destination::Second,
            }
        },
    };
    let (first_out, second_out) = if reverse_force {
        (batter_out, runner_out)
    } else {
        (runner_out, batter_out)
    };
    let runs = u8::from(crossing && (outs == 0 || reverse_force));
    let mut moves = vec![
        RunnerMove {
            runner: Runner {
                player_id: batter.clone(),
                responsible_pitcher: pitcher.clone(),
            },
            from: Base::Plate,
            to: Destination::Out,
        },
        RunnerMove {
            runner,
            from: Base::First,
            to: Destination::Out,
        },
    ];
    let mut credits = vec![
        credit(&batter, CreditKind::PlateAppearance),
        credit(&batter, CreditKind::GroundedIntoDoublePlay),
    ];
    if let Some(lead) = lead {
        if runs == 1 {
            credits.push(credit(&lead.player_id, CreditKind::Run));
        }
        moves.push(RunnerMove {
            runner: lead,
            from: Base::Third,
            to: Destination::Home,
        });
    }
    let third_out = (outs == 1).then(|| ThirdOut {
        runner_id: second_out.runner_id.clone(),
        kind: if reverse_force {
            ThirdOutKind::Timing
        } else {
            ThirdOutKind::BatterBeforeFirst
        },
        home_before_out: if reverse_force && crossing {
            vec![before.bases[2].as_ref().unwrap().player_id.clone()]
        } else {
            vec![]
        },
    });
    let event = Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(batter),
            pitcher,
        },
        fielding_review: FieldingReview::Unknown,
        single_delivery: None,
        balk: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        ordinary_air_catch: None,
        muffed_ground_ball: None,
        ordinary_ground_throw: None,
        two_out_contact: Some(TwoOutContactFacts {
            contact: ContactClass::Grounded,
            first_out,
            second_out,
            continuity: ContactContinuity::Continuous,
            error_placement: Some(ReviewedErrorPlacement {
                between_outs: vec![],
                outside_interval: vec![],
            }),
        }),
        delta: Delta {
            moves,
            outs: 2,
            runs,
            credits,
            changes: vec![],
            third_out,
        },
    };
    (before, event)
}

// A valid legacy control supplies no new mechanics and no GIDP assertion.
fn reference(before: &GameState, event: &Event) -> EventTransition {
    let mut event = event.clone();
    event.two_out_contact = None;
    event.delta.credits.retain(|c| {
        matches!(
            c.kind,
            CreditKind::PlateAppearance | CreditKind::Run | CreditKind::Error
        )
    });
    event.delta.credits.push(credit(
        event.participants.batter.as_ref().unwrap(),
        CreditKind::AtBat,
    ));
    apply(before, event).unwrap()
}

fn assert_error(before: &GameState, event: Event, error: TransitionError) {
    let original = before.clone();
    let after = reference(before, &event).after().clone();
    assert_eq!(apply(before, event.clone()), Err(error.clone()));
    assert_eq!(before, &original);
    assert_eq!(
        EventTransition::try_new(before.clone(), event, after),
        Err(error)
    );
}

#[test]
fn force_and_reverse_force_count_one_gidp_without_inferring_vector_order() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..2 {
            for reverse_force in [false, true] {
                for reverse_vectors in [false, true] {
                    let (before, mut event) = fixture(half, outs, reverse_force, false);
                    if reverse_vectors {
                        event.delta.moves.reverse();
                        event.delta.credits.reverse();
                    }
                    let transition = apply(&before, event.clone()).unwrap();
                    assert_eq!(
                        EventTransition::try_new(before.clone(), event, transition.after().clone())
                            .unwrap(),
                        transition
                    );
                    assert_eq!(
                        EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
                        transition
                    );
                    let lines = derive_accounting(std::slice::from_ref(&transition)).unwrap();
                    let side = half.offense();
                    let batter = transition.event().participants.batter.as_ref().unwrap();
                    let pitcher = &transition.event().participants.pitcher;
                    assert_eq!(
                        lines.batters[side][batter].credited_grounded_into_double_plays,
                        1
                    );
                    assert_eq!(lines.batters[side][batter].plate_appearances, 1);
                    assert_eq!(lines.batters[side][batter].at_bats, Some(1));
                    assert_eq!(lines.pitchers[1 - side][pitcher].batters_faced, 1);
                    assert_eq!(lines.pitchers[1 - side][pitcher].at_bats_against, Some(1));
                    assert_eq!(lines.pitchers[1 - side][pitcher].outs_recorded, 2);
                    assert!(!lines.complete_game);
                }
            }
        }
    }
}

#[test]
fn definite_gidp_credit_and_context_contradictions_are_illegal() {
    let (before, event) = fixture(Half::Top, 0, false, false);
    let mut duplicate = event.clone();
    duplicate
        .delta
        .credits
        .push(credit("away0", CreditKind::GroundedIntoDoublePlay));
    assert_error(&before, duplicate, TransitionError::IllegalCredit);
    let mut wrong_actor = event.clone();
    wrong_actor
        .delta
        .credits
        .iter_mut()
        .find(|c| c.kind == CreditKind::GroundedIntoDoublePlay)
        .unwrap()
        .player_id = "away1".into();
    assert_error(&before, wrong_actor, TransitionError::IllegalCredit);
    let mut not_grounded = event.clone();
    not_grounded.two_out_contact.as_mut().unwrap().contact = ContactClass::NotGrounded;
    assert_error(&before, not_grounded, TransitionError::IllegalCredit);
    let mut discontinuous = event.clone();
    discontinuous.two_out_contact.as_mut().unwrap().continuity = ContactContinuity::Discontinuous;
    assert_error(&before, discontinuous, TransitionError::IllegalCredit);
    for incompatible in [
        CreditKind::Hit,
        CreditKind::Strikeout,
        CreditKind::Walk,
        CreditKind::HitByPitch,
        CreditKind::SacrificeBunt,
        CreditKind::SacrificeFly,
        CreditKind::InterferenceAward,
    ] {
        let mut contradictory = event.clone();
        contradictory
            .delta
            .credits
            .push(credit("away0", incompatible));
        assert_error(&before, contradictory, TransitionError::IllegalCredit);
    }
}

#[test]
fn missing_contact_interval_or_continuity_is_unsupported_not_inferred() {
    let (before, event) = fixture(Half::Top, 0, false, false);
    let mut missing = event.clone();
    missing.two_out_contact = None;
    assert_error(&before, missing, TransitionError::UnsupportedRule);
    let mut missing = event.clone();
    missing.two_out_contact.as_mut().unwrap().error_placement = None;
    assert_error(&before, missing, TransitionError::UnsupportedRule);
    let mut missing = event;
    missing.two_out_contact.as_mut().unwrap().continuity = ContactContinuity::Unknown;
    assert_error(&before, missing, TransitionError::UnsupportedRule);
}

#[test]
fn qualified_context_forbids_rbi_even_when_gidp_credit_is_omitted() {
    for omit in [false, true] {
        let (before, mut event) = fixture(Half::Top, 0, false, true);
        if omit {
            event
                .delta
                .credits
                .retain(|c| c.kind != CreditKind::GroundedIntoDoublePlay);
            event.delta.credits.push(credit("away0", CreditKind::AtBat));
        }
        event.delta.credits.push(credit("away0", CreditKind::Rbi));
        assert_error(&before, event, TransitionError::IllegalCredit);
    }
}

#[test]
fn outside_error_preserves_gidp_but_not_earned_run_certification() {
    let (before, mut event) = fixture(Half::Top, 0, false, true);
    event.delta.credits.push(credit("home2", CreditKind::Error));
    event
        .two_out_contact
        .as_mut()
        .unwrap()
        .error_placement
        .as_mut()
        .unwrap()
        .outside_interval
        .push("home2".into());
    let transition = apply(&before, event.clone()).unwrap();
    let lines = derive_accounting(&[transition]).unwrap();
    assert_eq!(
        lines.batters[0]["away0"].credited_grounded_into_double_plays,
        1
    );
    assert_eq!(lines.pitchers[1]["home_pitcher"].earned_runs, None);
    assert_eq!(lines.fielders[1]["home2"].credited_errors, 1);
    let placement = event
        .two_out_contact
        .as_mut()
        .unwrap()
        .error_placement
        .as_mut()
        .unwrap();
    placement.outside_interval.clear();
    placement.between_outs.push("home2".into());
    assert_error(&before, event, TransitionError::IllegalCredit);
}

#[test]
fn preceding_runner_interference_exclusion_is_not_masked_by_unknown_review() {
    let (before, mut event) = fixture(Half::Top, 0, false, false);
    let context = event.two_out_contact.as_mut().unwrap();
    context.first_out.cause = RetirementCause::RunnerInterference;
    context.second_out.cause = RetirementCause::BatterByPrecedingRunnerInterference {
        interferer_id: context.first_out.runner_id.clone(),
    };
    context.continuity = ContactContinuity::Unknown;
    context.error_placement = None;
    assert_error(&before, event, TransitionError::IllegalCredit);
}

fn unique_tag(bytes: &[u8], id: &str, tag: u8) -> usize {
    let mut marker = (id.len() as u64).to_le_bytes().to_vec();
    marker.extend_from_slice(id.as_bytes());
    marker.push(tag);
    let offsets: Vec<_> = bytes
        .windows(marker.len())
        .enumerate()
        .filter_map(|(offset, window)| (window == marker).then_some(offset))
        .collect();
    assert_eq!(offsets.len(), 1);
    offsets[0] + marker.len() - 1
}

#[test]
fn well_formed_gidp_and_contact_forgeries_are_semantically_rejected() {
    let (before, event) = fixture(Half::Top, 0, false, false);
    let valid = apply(&before, event).unwrap();
    let original = valid.to_bytes();
    assert!(original.starts_with(b"BBE25"));
    assert_eq!(EventTransition::from_bytes(&original).unwrap(), valid);
    let mut credit_forgery = original.clone();
    let kind = unique_tag(&credit_forgery, "away0", 22);
    credit_forgery[kind] = 1; // K contradicts the explicitly qualified ground DP.
    assert_eq!(
        EventTransition::from_bytes(&credit_forgery),
        Err(TransitionError::IllegalCredit)
    );
    // Explicit Some/contact/string/cause/location prefix identifies the context
    // independently of state length and movement/credit ordering.
    let id = "away5";
    let mut marker = vec![1, 0];
    marker.extend_from_slice(&(id.len() as u64).to_le_bytes());
    marker.extend_from_slice(id.as_bytes());
    marker.extend_from_slice(&[1, 1]); // Force, Second; existing tags unmodified.
    let offsets: Vec<_> = original
        .windows(marker.len())
        .enumerate()
        .filter_map(|(offset, window)| (window == marker).then_some(offset))
        .collect();
    assert_eq!(offsets.len(), 1);
    let mut contact_forgery = original.clone();
    contact_forgery[offsets[0] + 1] = 1; // NotGrounded, still structurally valid.
    assert_eq!(
        EventTransition::from_bytes(&contact_forgery),
        Err(TransitionError::IllegalCredit)
    );
    contact_forgery[offsets[0] + 1] = 255;
    assert_eq!(
        EventTransition::from_bytes(&contact_forgery),
        Err(TransitionError::InvalidEncoding)
    );
    let mut wrong_base = original.clone();
    wrong_base[offsets[0] + marker.len() - 1] = 2; // Third, not the forced Second.
    assert_eq!(
        EventTransition::from_bytes(&wrong_base),
        Err(TransitionError::DeltaMismatch)
    );
    for prefix in [b"BBE16", b"BBE15", b"BBE14", b"BBE13", b"BBE12", b"BBE11"] {
        let mut old = original.clone();
        old[..5].copy_from_slice(prefix);
        assert_eq!(
            EventTransition::from_bytes(&old),
            Err(TransitionError::InvalidEncoding)
        );
    }
}

fn safe_fixture(half: Half, outs: u8, two_forces: bool, crossing: bool) -> (GameState, Event) {
    let (mut before, mut event) = fixture(half, outs, false, crossing);
    let side = half.offense();
    let second = Runner {
        player_id: before.teams[side].batting_order[6].clone(),
        responsible_pitcher: event.participants.pitcher.clone(),
    };
    before.bases[1] = Some(second.clone());
    event.delta.moves[0].to = Destination::First;
    event.delta.moves.push(RunnerMove {
        runner: second.clone(),
        from: Base::Second,
        to: Destination::Out,
    });
    let first = before.bases[0].as_ref().unwrap().player_id.clone();
    let context = event.two_out_contact.as_mut().unwrap();
    if two_forces {
        context.first_out = RetirementFact {
            runner_id: second.player_id.clone(),
            cause: RetirementCause::Force {
                at: Destination::Third,
            },
        };
        context.second_out = RetirementFact {
            runner_id: first,
            cause: RetirementCause::Force {
                at: Destination::Second,
            },
        };
    } else {
        context.first_out = RetirementFact {
            runner_id: first,
            cause: RetirementCause::Force {
                at: Destination::Second,
            },
        };
        context.second_out = RetirementFact {
            runner_id: second.player_id,
            cause: RetirementCause::Tag {
                at: Destination::Third,
                force_loss: ForceLoss::RemovedByFirstOut,
            },
        };
    }
    event.delta.runs = u8::from(crossing && (outs == 0 || !two_forces));
    event.delta.credits.retain(|c| c.kind != CreditKind::Run);
    if event.delta.runs == 1 {
        event.delta.credits.push(credit(
            &before.bases[2].as_ref().unwrap().player_id,
            CreditKind::Run,
        ));
    }
    event.delta.third_out = (outs == 1).then(|| ThirdOut {
        runner_id: context.second_out.runner_id.clone(),
        kind: if two_forces {
            ThirdOutKind::Force
        } else {
            ThirdOutKind::Timing
        },
        home_before_out: if crossing {
            vec![before.bases[2].as_ref().unwrap().player_id.clone()]
        } else {
            vec![]
        },
    });
    (before, event)
}

fn assert_against(before: &GameState, event: Event, after: GameState, error: TransitionError) {
    let saved = before.clone();
    assert_eq!(apply(before, event.clone()), Err(error.clone()));
    assert_eq!(before, &saved);
    assert_eq!(
        EventTransition::try_new(before.clone(), event, after),
        Err(error)
    );
}

#[test]
fn batter_safe_force_and_reverse_force_preserve_literal_participation_and_runs() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..2 {
            for two_forces in [false, true] {
                for crossing in [false, true] {
                    let (before, mut event) = safe_fixture(half, outs, two_forces, crossing);
                    // Explicit AtBat supplies no second AB. Vector order is unrelated to out order.
                    let batter = event.participants.batter.clone().unwrap();
                    event.delta.credits.push(credit(&batter, CreditKind::AtBat));
                    event.delta.moves.reverse();
                    let t = apply(&before, event.clone()).unwrap();
                    assert_eq!(
                        EventTransition::try_new(before.clone(), event, t.after().clone()).unwrap(),
                        t
                    );
                    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
                    let lines = derive_accounting(std::slice::from_ref(&t)).unwrap();
                    let side = half.offense();
                    let p = &t.event().participants.pitcher;
                    assert_eq!(
                        lines.batters[side][&batter].credited_grounded_into_double_plays,
                        1
                    );
                    assert_eq!(lines.batters[side][&batter].plate_appearances, 1);
                    assert_eq!(lines.batters[side][&batter].at_bats, Some(1));
                    assert_eq!(lines.pitchers[1 - side][p].batters_faced, 1);
                    assert_eq!(lines.pitchers[1 - side][p].at_bats_against, Some(1));
                    assert_eq!(lines.pitchers[1 - side][p].outs_recorded, 2);
                    assert_eq!(
                        t.event().delta.runs,
                        u8::from(crossing && (outs == 0 || !two_forces))
                    );
                    if outs == 0 {
                        assert_eq!(t.after().bases[0].as_ref().unwrap().player_id, batter);
                    }
                }
            }
        }
    }
}

#[test]
fn unrelated_inherited_survivor_is_not_a_replacement_exchange() {
    for safe in [false, true] {
        let (mut before, mut event) = if safe {
            safe_fixture(Half::Top, 0, false, false)
        } else {
            fixture(Half::Top, 0, false, false)
        };
        before.bases[2] = Some(Runner {
            player_id: "away7".into(),
            responsible_pitcher: "home_relief".into(),
        });
        let t = apply(&before, event.clone()).unwrap();
        assert_eq!(t.after().bases[2], before.bases[2]);
        assert_eq!(
            EventTransition::try_new(before.clone(), event.clone(), t.after().clone()).unwrap(),
            t
        );
        let lines = derive_accounting(std::slice::from_ref(&t)).unwrap();
        assert_eq!(
            lines.batters[0]["away0"].credited_grounded_into_double_plays,
            1
        );
        // A relevant retired inherited charge is unsupported only when a new batter charge survives.
        let original = reference(&before, &event).after().clone();
        let retired = before.bases[0].as_mut().unwrap();
        retired.responsible_pitcher = "home_relief".into();
        event
            .delta
            .moves
            .iter_mut()
            .find(|m| m.from == Base::First)
            .unwrap()
            .runner = retired.clone();
        if safe {
            assert_against(&before, event, original, TransitionError::UnsupportedRule);
        } else {
            let t = apply(&before, event).unwrap();
            assert_eq!(t.after().bases[2], before.bases[2]);
        }
    }
}

#[test]
fn third_out_discards_batter_charge_even_when_another_runner_scores() {
    for crossing in [false, true] {
        let (mut before, mut event) = safe_fixture(Half::Top, 1, false, crossing);
        let after = reference(&before, &event).after().clone();
        before.bases[0].as_mut().unwrap().responsible_pitcher = "home_relief".into();
        event
            .delta
            .moves
            .iter_mut()
            .find(|m| m.from == Base::First)
            .unwrap()
            .runner = before.bases[0].clone().unwrap();
        // A third out with another runner scoring still leaves no carried/scored BATTER charge.
        let t = apply(&before, event.clone()).unwrap();
        assert!(t.after().bases.iter().all(Option::is_none));
        assert_eq!(
            derive_accounting(&[t]).unwrap().batters[0]["away0"]
                .credited_grounded_into_double_plays,
            1
        );
        assert_eq!(
            EventTransition::try_new(before.clone(), event, after)
                .unwrap()
                .event()
                .delta
                .runs,
            u8::from(crossing)
        );
    }
}

#[test]
fn final_retirement_and_run_timing_are_explicit_not_vector_order() {
    for reverse in [false, true] {
        let (before, event) = fixture(Half::Top, 1, reverse, true);
        let good = reference(&before, &event);
        let mut wrong = event.clone();
        wrong.delta.third_out.as_mut().unwrap().runner_id = event
            .two_out_contact
            .as_ref()
            .unwrap()
            .first_out
            .runner_id
            .clone();
        assert_against(
            &before,
            wrong,
            good.after().clone(),
            TransitionError::DeltaMismatch,
        );
        let mut wrong = event.clone();
        wrong.delta.third_out.as_mut().unwrap().kind = if reverse {
            ThirdOutKind::Force
        } else {
            ThirdOutKind::BatterAfterFirstTiming
        };
        assert_against(
            &before,
            wrong,
            good.after().clone(),
            TransitionError::DeltaMismatch,
        );
        let mut missing = event.clone();
        missing.delta.third_out = None;
        assert_against(
            &before,
            missing,
            good.after().clone(),
            TransitionError::UnsupportedRule,
        );
        let t = apply(&before, event).unwrap();
        assert_eq!(t.event().delta.runs, u8::from(reverse));
        if reverse {
            let mut late = t.event().clone();
            late.delta
                .third_out
                .as_mut()
                .unwrap()
                .home_before_out
                .clear();
            late.delta.runs = 0;
            late.delta.credits.retain(|c| c.kind != CreditKind::Run);
            assert_eq!(apply(&before, late).unwrap().event().delta.runs, 0);
        }
    }
}

#[test]
fn definite_force_actor_location_and_order_errors_precede_missing_review() {
    for variant in 0..6 {
        let (before, mut event) = fixture(Half::Top, 0, false, false);
        let context = event.two_out_contact.as_mut().unwrap();
        context.error_placement = None;
        match variant {
            0 => context.first_out.runner_id = context.second_out.runner_id.clone(),
            1 => context.first_out.runner_id = "absent".into(),
            2 => {
                context.first_out.cause = RetirementCause::Force {
                    at: Destination::Third,
                }
            }
            3 => {
                context.first_out.cause = RetirementCause::Force {
                    at: Destination::Out,
                }
            }
            4 => context.first_out.cause = RetirementCause::BatterPutoutBeforeFirstTouch,
            5 => {
                std::mem::swap(&mut context.first_out, &mut context.second_out);
            }
            _ => unreachable!(),
        }
        assert_error(&before, event, TransitionError::DeltaMismatch);
    }
    let (before, mut unknown) = fixture(Half::Top, 0, true, false);
    unknown.two_out_contact.as_mut().unwrap().second_out.cause = RetirementCause::Tag {
        at: Destination::Second,
        force_loss: ForceLoss::Unknown,
    };
    assert_error(&before, unknown, TransitionError::UnsupportedRule);
}

#[test]
fn error_unit_multiplicity_and_whole_play_review_do_not_replace_interval_facts() {
    let (before, base) = fixture(Half::Top, 0, false, false);
    for copies in 1..=3 {
        let mut event = base.clone();
        for _ in 0..copies {
            event.delta.credits.push(credit("home2", CreditKind::Error));
        }
        event
            .two_out_contact
            .as_mut()
            .unwrap()
            .error_placement
            .as_mut()
            .unwrap()
            .outside_interval = vec!["home2".into(); copies];
        assert!(apply(&before, event.clone()).is_ok());
        event
            .two_out_contact
            .as_mut()
            .unwrap()
            .error_placement
            .as_mut()
            .unwrap()
            .outside_interval
            .pop();
        assert_error(&before, event, TransitionError::DeltaMismatch);
    }
    let mut unknown = base.clone();
    unknown.fielding_review = FieldingReview::NoMisplay;
    unknown.two_out_contact.as_mut().unwrap().error_placement = None;
    assert_error(&before, unknown, TransitionError::UnsupportedRule);
    let after = reference(&before, &base).after().clone();
    let mut contradictory = base;
    contradictory.fielding_review = FieldingReview::NoMisplay;
    contradictory
        .delta
        .credits
        .push(credit("home2", CreditKind::Error));
    contradictory
        .two_out_contact
        .as_mut()
        .unwrap()
        .error_placement
        .as_mut()
        .unwrap()
        .outside_interval
        .push("home2".into());
    assert_against(
        &before,
        contradictory,
        after,
        TransitionError::IllegalCredit,
    );
}

#[test]
fn omission_nonground_previously_removed_and_k_cs_do_not_infer_gidp() {
    let (before, event) = fixture(Half::Top, 0, false, false);
    for context in 0..3 {
        let mut control = event.clone();
        control
            .delta
            .credits
            .retain(|c| c.kind != CreditKind::GroundedIntoDoublePlay);
        control
            .delta
            .credits
            .push(credit("away0", CreditKind::AtBat));
        match context {
            0 => {}
            1 => control.two_out_contact.as_mut().unwrap().contact = ContactClass::NotGrounded,
            2 => control.two_out_contact = None,
            _ => unreachable!(),
        }
        let t = apply(&before, control).unwrap();
        let lines = derive_accounting(&[t]).unwrap();
        assert_eq!(
            lines.batters[0]["away0"].credited_grounded_into_double_plays,
            0
        );
        assert_eq!(lines.batters[0]["away0"].at_bats, Some(1));
    }
    let (before, mut non_dp) = fixture(Half::Top, 0, true, false);
    non_dp.two_out_contact.as_mut().unwrap().second_out.cause = RetirementCause::Tag {
        at: Destination::Second,
        force_loss: ForceLoss::PreviouslyRemoved,
    };
    assert_error(&before, non_dp.clone(), TransitionError::IllegalCredit);
    non_dp
        .delta
        .credits
        .retain(|c| c.kind != CreditKind::GroundedIntoDoublePlay);
    assert_eq!(
        derive_accounting(&[apply(&before, non_dp).unwrap()])
            .unwrap()
            .batters[0]["away0"]
            .credited_grounded_into_double_plays,
        0
    );
    let (before, mut k_cs) = fixture(Half::Top, 0, false, false);
    k_cs.two_out_contact = None;
    k_cs.delta
        .credits
        .retain(|c| c.kind != CreditKind::GroundedIntoDoublePlay);
    k_cs.delta
        .credits
        .push(credit("away0", CreditKind::Strikeout));
    k_cs.delta
        .credits
        .push(credit("away5", CreditKind::CaughtStealing));
    let lines = derive_accounting(&[apply(&before, k_cs).unwrap()]).unwrap();
    assert_eq!(
        lines.batters[0]["away0"].credited_grounded_into_double_plays,
        0
    );
    assert_eq!(lines.batters[0]["away0"].credited_strikeouts, 1);
    assert_eq!(lines.batters[0]["away5"].credited_caught_stealing, 1);
}

#[test]
fn after_first_batter_tag_and_unasserted_unknown_history_roundtrip_without_gidp() {
    let (before, mut event) = fixture(Half::Top, 1, false, true);
    event
        .delta
        .credits
        .retain(|c| c.kind != CreditKind::GroundedIntoDoublePlay);
    event.delta.credits.push(credit("away0", CreditKind::AtBat));
    event.delta.credits.push(credit("away7", CreditKind::Run));
    event.delta.runs = 1;
    event.two_out_contact.as_mut().unwrap().second_out.cause =
        RetirementCause::BatterAfterFirstTag {
            at: Destination::Second,
        };
    let third = event.delta.third_out.as_mut().unwrap();
    third.kind = ThirdOutKind::BatterAfterFirstTiming;
    third.home_before_out.push("away7".into());
    let t = apply(&before, event.clone()).unwrap();
    assert_eq!(
        EventTransition::try_new(before.clone(), event.clone(), t.after().clone()).unwrap(),
        t
    );
    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
    let lines = derive_accounting(std::slice::from_ref(&t)).unwrap();
    assert_eq!(
        lines.batters[0]["away0"].credited_grounded_into_double_plays,
        0
    );
    assert_eq!(lines.batters[0]["away7"].credited_runs, 1);
    event
        .delta
        .credits
        .push(credit("away0", CreditKind::GroundedIntoDoublePlay));
    assert_against(
        &before,
        event,
        t.after().clone(),
        TransitionError::IllegalCredit,
    );

    let (before, mut partial) = fixture(Half::Top, 0, true, false);
    partial
        .delta
        .credits
        .retain(|c| c.kind != CreditKind::GroundedIntoDoublePlay);
    let context = partial.two_out_contact.as_mut().unwrap();
    context.second_out.cause = RetirementCause::Tag {
        at: Destination::Second,
        force_loss: ForceLoss::Unknown,
    };
    context.continuity = ContactContinuity::Unknown;
    context.error_placement = None;
    let t = apply(&before, partial).unwrap();
    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
    let lines = derive_accounting(&[t]).unwrap();
    assert_eq!(
        lines.batters[0]["away0"].credited_grounded_into_double_plays,
        0
    );
    assert_eq!(lines.batters[0]["away0"].at_bats, None);
    assert!(!lines.complete_game);
}

#[test]
fn impossible_initial_force_chain_is_not_rescued_by_unknown_history() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..2 {
            let (mut before, mut event) = fixture(half, outs, true, false);
            before.bases[2] = before.bases[0].take();
            event
                .delta
                .moves
                .iter_mut()
                .find(|m| m.from == Base::First)
                .unwrap()
                .from = Base::Third;
            event.two_out_contact.as_mut().unwrap().second_out.cause = RetirementCause::Tag {
                at: Destination::Home,
                force_loss: ForceLoss::Unknown,
            };
            assert_error(&before, event.clone(), TransitionError::IllegalCredit);
            // Valid partial control parses Unknown; forging only AB into GDP
            // must fail semantically, not as malformed encoding.
            event
                .delta
                .credits
                .retain(|c| c.kind != CreditKind::GroundedIntoDoublePlay);
            let batter = event.participants.batter.clone().unwrap();
            event.delta.credits.push(credit(&batter, CreditKind::AtBat));
            let t = apply(&before, event).unwrap();
            let mut bytes = t.to_bytes();
            assert_eq!(EventTransition::from_bytes(&bytes).unwrap(), t);
            // Tag12 also equals a fixture pitcher's string length. Identify
            // the complete two-credit vector, not an ambiguous ID/tag suffix.
            let mut marker = 2u64.to_le_bytes().to_vec();
            for tag in [0, 12] {
                marker.extend_from_slice(&(batter.len() as u64).to_le_bytes());
                marker.extend_from_slice(batter.as_bytes());
                marker.push(tag);
            }
            let offsets: Vec<_> = bytes
                .windows(marker.len())
                .enumerate()
                .filter_map(|(i, window)| (window == marker).then_some(i))
                .collect();
            assert_eq!(offsets.len(), 1);
            let offset = offsets[0] + marker.len() - 1;
            bytes[offset] = 22;
            assert_eq!(
                EventTransition::from_bytes(&bytes),
                Err(TransitionError::IllegalCredit)
            );
        }
        // A leading first retirement cannot remove a trailing runner's force.
        let (before, mut wrong_cause) = safe_fixture(half, 0, true, false);
        wrong_cause
            .two_out_contact
            .as_mut()
            .unwrap()
            .second_out
            .cause = RetirementCause::Tag {
            at: Destination::Second,
            force_loss: ForceLoss::Unknown,
        };
        assert_error(&before, wrong_cause, TransitionError::IllegalCredit);
        let (before, mut possible) = fixture(half, 0, true, false);
        possible.two_out_contact.as_mut().unwrap().second_out.cause = RetirementCause::Tag {
            at: Destination::Second,
            force_loss: ForceLoss::Unknown,
        };
        assert_error(&before, possible, TransitionError::UnsupportedRule);
    }
}

#[test]
fn missing_context_or_review_does_not_mask_duplicate_at_bat_units() {
    for half in [Half::Top, Half::Bottom] {
        for missing_context in [false, true] {
            let (before, mut event) = fixture(half, 0, false, false);
            let batter = event.participants.batter.clone().unwrap();
            if missing_context {
                event.two_out_contact = None;
            } else {
                event.two_out_contact.as_mut().unwrap().error_placement = None;
            }
            event.delta.credits.extend([
                credit(&batter, CreditKind::AtBat),
                credit(&batter, CreditKind::AtBat),
            ]);
            assert_error(&before, event, TransitionError::IllegalCredit);
        }
    }
}

#[test]
fn penalty_pair_holds_survivors_without_inventing_action_chronology() {
    for half in [Half::Top, Half::Bottom] {
        let (mut before, mut held) = fixture(half, 0, false, false);
        let side = half.offense();
        let survivor = Runner {
            player_id: before.teams[side].batting_order[6].clone(),
            responsible_pitcher: before.teams[1 - side].bench[0].clone(),
        };
        before.bases[1] = Some(survivor.clone());
        let batter = held.participants.batter.clone().unwrap();
        held.delta
            .credits
            .retain(|c| c.kind != CreditKind::GroundedIntoDoublePlay);
        held.delta.credits.push(credit(&batter, CreditKind::AtBat));
        let context = held.two_out_contact.as_mut().unwrap();
        context.first_out.cause = RetirementCause::RunnerInterference;
        context.second_out.cause = RetirementCause::BatterByPrecedingRunnerInterference {
            interferer_id: context.first_out.runner_id.clone(),
        };
        context.continuity = ContactContinuity::Unknown;
        context.error_placement = None;
        held.delta.moves.push(RunnerMove {
            runner: survivor.clone(),
            from: Base::Second,
            to: Destination::Second,
        });
        let valid = apply(&before, held.clone()).unwrap();
        assert_eq!(valid.after().bases[1], Some(survivor));
        assert_eq!(
            EventTransition::from_bytes(&valid.to_bytes()).unwrap(),
            valid
        );
        let mut advanced = held.clone();
        advanced.delta.moves.last_mut().unwrap().to = Destination::Third;
        // Legacy control proves physical movement/after-state are otherwise valid.
        let mut legacy = advanced.clone();
        legacy.two_out_contact = None;
        let physical = apply(&before, legacy).unwrap();
        assert_against(
            &before,
            advanced,
            physical.after().clone(),
            TransitionError::UnsupportedRule,
        );
        // Derive only the optional context's bytes from paired valid records;
        // splice them into the valid advancing legacy record, including its
        // correct advanced after-state. Both controls parse before forgery.
        let some = valid.to_bytes();
        let mut no_context = held;
        no_context.two_out_contact = None;
        let none = apply(&before, no_context).unwrap().to_bytes();
        let offset = some.iter().zip(&none).position(|(a, b)| a != b).unwrap();
        let context_len = some.len() - none.len() + 1;
        assert_eq!(some[offset], 1);
        assert_eq!(none[offset], 0);
        assert_eq!(&some[offset + context_len..], &none[offset + 1..]);
        let mut forged = physical.to_bytes();
        assert_eq!(EventTransition::from_bytes(&forged).unwrap(), physical);
        assert_eq!(forged[offset], 0);
        forged.splice(
            offset..offset + 1,
            some[offset..offset + context_len].iter().copied(),
        );
        assert_eq!(
            EventTransition::from_bytes(&forged),
            Err(TransitionError::UnsupportedRule)
        );
    }
}

#[test]
fn typed_two_out_winning_run_needs_action_end_order_not_an_assumption() {
    let (mut before, event) = fixture(Half::Bottom, 0, false, true);
    before.inning = 9;
    assert_error(&before, event, TransitionError::UnsupportedRule);
}

#[test]
fn missing_context_cannot_mask_definite_pa_or_whole_play_credit_contradictions() {
    let (before, base) = fixture(Half::Top, 0, false, false);
    let after = reference(&before, &base).after().clone();
    for contradiction in [
        CreditKind::PlateAppearance,
        CreditKind::Error,
        CreditKind::PassedBall,
    ] {
        let mut event = base.clone();
        event.two_out_contact = None;
        if contradiction == CreditKind::PlateAppearance {
            event
                .delta
                .credits
                .retain(|c| c.kind != CreditKind::PlateAppearance);
        } else {
            event.fielding_review = FieldingReview::NoMisplay;
            event.delta.credits.push(credit("home2", contradiction));
        }
        assert_against(
            &before,
            event,
            after.clone(),
            TransitionError::IllegalCredit,
        );
    }
}

#[test]
fn explicit_penalty_interference_contradicts_no_misplay_without_a_gidp_credit() {
    let (before, mut event) = fixture(Half::Top, 0, false, false);
    event
        .delta
        .credits
        .retain(|c| c.kind != CreditKind::GroundedIntoDoublePlay);
    event.delta.credits.push(credit("away0", CreditKind::AtBat));
    let context = event.two_out_contact.as_mut().unwrap();
    context.first_out.cause = RetirementCause::RunnerInterference;
    context.second_out.cause = RetirementCause::BatterByPrecedingRunnerInterference {
        interferer_id: context.first_out.runner_id.clone(),
    };
    context.continuity = ContactContinuity::Unknown;
    context.error_placement = None;
    let valid = apply(&before, event.clone()).unwrap();
    assert_eq!(
        EventTransition::from_bytes(&valid.to_bytes()).unwrap(),
        valid
    );
    event.fielding_review = FieldingReview::NoMisplay;
    assert_against(
        &before,
        event,
        valid.after().clone(),
        TransitionError::IllegalCredit,
    );
}

#[test]
fn two_nonforced_tags_are_legal_without_a_gidp_classification() {
    let (mut before, mut event) = safe_fixture(Half::Top, 0, false, false);
    let first = before.bases[0].take().unwrap();
    before.bases[2] = Some(first.clone());
    event
        .delta
        .moves
        .iter_mut()
        .find(|m| m.from == Base::First)
        .unwrap()
        .from = Base::Third;
    let context = event.two_out_contact.as_mut().unwrap();
    context.first_out = RetirementFact {
        runner_id: before.bases[1].as_ref().unwrap().player_id.clone(),
        cause: RetirementCause::Tag {
            at: Destination::Third,
            force_loss: ForceLoss::NeverForced,
        },
    };
    context.second_out = RetirementFact {
        runner_id: first.player_id,
        cause: RetirementCause::Tag {
            at: Destination::Home,
            force_loss: ForceLoss::NeverForced,
        },
    };
    assert_error(&before, event.clone(), TransitionError::IllegalCredit);
    event
        .delta
        .credits
        .retain(|c| c.kind != CreditKind::GroundedIntoDoublePlay);
    event.delta.credits.push(credit("away0", CreditKind::AtBat));
    let t = apply(&before, event).unwrap();
    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
    let lines = derive_accounting(&[t]).unwrap();
    assert_eq!(
        lines.batters[0]["away0"].credited_grounded_into_double_plays,
        0
    );
    assert_eq!(lines.pitchers[1]["home_pitcher"].outs_recorded, 2);
}

#[test]
fn ground_contact_is_not_a_blanket_hit_or_sacrifice_ban_on_nonforce_two_outs() {
    // Ground single, both preceding runners tagged at nonforced destinations.
    let (mut before, mut event) = safe_fixture(Half::Top, 0, false, false);
    let first = before.bases[0].take().unwrap();
    before.bases[2] = Some(first.clone());
    event
        .delta
        .moves
        .iter_mut()
        .find(|m| m.from == Base::First)
        .unwrap()
        .from = Base::Third;
    let context = event.two_out_contact.as_mut().unwrap();
    context.first_out = RetirementFact {
        runner_id: before.bases[1].as_ref().unwrap().player_id.clone(),
        cause: RetirementCause::Tag {
            at: Destination::Third,
            force_loss: ForceLoss::NeverForced,
        },
    };
    context.second_out = RetirementFact {
        runner_id: first.player_id,
        cause: RetirementCause::Tag {
            at: Destination::Home,
            force_loss: ForceLoss::NeverForced,
        },
    };
    event
        .delta
        .credits
        .retain(|c| c.kind != CreditKind::GroundedIntoDoublePlay);
    event.delta.credits.push(credit("away0", CreditKind::Hit));
    let t = apply(&before, event.clone()).unwrap();
    assert_eq!(
        EventTransition::try_new(before.clone(), event, t.after().clone()).unwrap(),
        t
    );
    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
    let lines = derive_accounting(&[t]).unwrap();
    assert_eq!(lines.batters[0]["away0"].credited_hits, 1);
    assert_eq!(
        lines.batters[0]["away0"].credited_grounded_into_double_plays,
        0
    );
    assert_eq!(lines.batters[0]["away0"].at_bats, Some(1));

    // Rule 9.08(a),(c): bunt advances the runner past his first forced base;
    // he touches Second before the batter putout, then is tagged trying Third.
    // This is not a failed one-base advance or a qualifying reverse-force DP.
    let (before, mut sacrifice) = fixture(Half::Top, 0, true, false);
    sacrifice
        .delta
        .credits
        .retain(|c| c.kind != CreditKind::GroundedIntoDoublePlay);
    sacrifice
        .delta
        .credits
        .push(credit("away0", CreditKind::SacrificeBunt));
    sacrifice.two_out_contact.as_mut().unwrap().second_out.cause = RetirementCause::Tag {
        at: Destination::Third,
        force_loss: ForceLoss::PreviouslyRemoved,
    };
    let t = apply(&before, sacrifice.clone()).unwrap();
    assert_eq!(
        EventTransition::try_new(before.clone(), sacrifice, t.after().clone()).unwrap(),
        t
    );
    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
    let lines = derive_accounting(&[t]).unwrap();
    assert_eq!(lines.batters[0]["away0"].credited_sacrifice_bunts, 1);
    assert_eq!(
        lines.batters[0]["away0"].credited_grounded_into_double_plays,
        0
    );
    assert_eq!(lines.batters[0]["away0"].at_bats, Some(0));
}
