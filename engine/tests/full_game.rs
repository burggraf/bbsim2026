//! Integrated generation mechanisms must construct concrete, canonically legal plays.
#[allow(dead_code)]
#[path = "../examples/support/full_game.rs"]
mod full_game;
#[allow(dead_code)]
#[path = "../examples/support/preview.rs"]
mod preview;
use bbsim_engine::{apply, CreditKind, EventTransition, GameState, Half, Runner, TeamState};

fn state(mask: u8, outs: u8) -> GameState {
    GameState {
        inning: 3,
        half: Half::Top,
        outs,
        score: [0, 0],
        teams: ["a", "h"].map(|p| TeamState {
            batting_order: std::array::from_fn(|i| format!("{p}{i}")),
            next_batter: 0,
            pitcher: format!("{p}p"),
            dh_slot: Some(8),
            dh_must_bat: false,
            bench: vec![],
            removed: vec![],
        }),
        bases: std::array::from_fn(|i| {
            (mask & (1 << i) != 0).then(|| Runner {
                player_id: format!("a{}", i + 5),
                responsible_pitcher: "hp".into(),
            })
        }),
        over: false,
    }
}
fn fielders() -> [String; 9] {
    std::array::from_fn(|i| {
        if i == 0 {
            "hp".into()
        } else {
            format!("h{}", i - 1)
        }
    })
}

#[test]
fn supplied_joint_hit_decision_and_defense_stay_bound_to_the_native_transition() {
    let before = state(1, 1);
    let mut decision = full_game::HitDecision {
        value: 1,
        position: 9,
        runners: [Some(bbsim_engine::Destination::Third), None, None],
        tag: None,
    };
    let play = full_game::hit(&before, &fielders(), decision.clone()).unwrap();
    let saved = decision.clone();
    decision.position = 7;
    let full_game::Mechanism::Hit {
        decision: retained,
        fielders: defense,
    } = play.mechanism()
    else {
        panic!("missing bound hit mechanism")
    };
    assert_eq!(retained, &saved);
    assert_eq!(defense, &fielders());
    assert_eq!(play.transition().before(), &before);
    assert_eq!(
        play.transition().after().bases[2].as_ref().unwrap(),
        before.bases[0].as_ref().unwrap()
    );
    assert_eq!(
        EventTransition::from_bytes(&play.transition().to_bytes()).unwrap(),
        *play.transition()
    );
}

#[test]
fn supplied_uncaught_k_decision_is_bound_without_hiding_the_prior_runner_advance() {
    let before = state(2, 1);
    let decision = full_game::StrikeoutDecision::RecoveredBeforeFirstAfterAdvances {
        handling: bbsim_engine::HandlingAssessment::PitcherWild,
    };
    let play = full_game::uncaught_strikeout(&before, &fielders(), decision).unwrap();
    assert!(
        matches!(play.mechanism(), full_game::Mechanism::UncaughtStrikeout { decision: retained, fielders: defense } if retained == &decision && defense == &fielders())
    );
    assert_eq!(play.transition().after().bases[2], before.bases[1]);
    assert_eq!(play.transition().after().outs, 2);
    assert_eq!(
        EventTransition::from_bytes(&play.transition().to_bytes()).unwrap(),
        *play.transition()
    );
}

#[test]
fn throwing_errors_are_not_muffs_and_name_the_selected_actor() {
    for mask in 0..8 {
        for outs in 0..3 {
            for p in 1..=6 {
                let before = state(mask, outs);
                let play = full_game::ordinary(
                    &before,
                    &fielders(),
                    full_game::Ordinary::ThrowingError(p),
                )
                .unwrap();
                let t = play.transition();
                assert!(t.event().muffed_ground_ball.is_none());
                assert_eq!(
                    t.event().fielding_review,
                    bbsim_engine::FieldingReview::Unknown
                );
                assert!(t
                    .event()
                    .delta
                    .credits
                    .iter()
                    .any(|c| c.kind == CreditKind::Error
                        && c.player_id == fielders()[usize::from(p) - 1]));
                assert_eq!(t.event().delta.outs, 0);
                assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
            }
        }
    }
}

#[test]
fn force_choice_retires_lead_runner_preserves_carried_charge_and_failed_bunt_is_not_a_sacrifice() {
    for outs in 0..3 {
        for failed_bunt in [false, true] {
            let mut before = state(1, outs);
            before.bases[0].as_mut().unwrap().responsible_pitcher = "prior-pitcher".into();
            let play = full_game::force_choice(&before, &fielders(), 6, failed_bunt).unwrap();
            let t = play.transition();
            assert_eq!(t.event().kind, bbsim_engine::EventKind::FieldersChoice);
            assert_eq!(t.event().delta.outs, 1);
            assert!(!t
                .event()
                .delta
                .credits
                .iter()
                .any(|c| c.kind == CreditKind::SacrificeBunt));
            if outs < 2 {
                assert_eq!(
                    t.after().bases[0].as_ref().unwrap().responsible_pitcher,
                    "prior-pitcher"
                );
            }
            assert_eq!(apply(&before, t.event().clone()).unwrap(), *t);
            assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
        }
    }
    assert!(full_game::force_choice(&state(2, 0), &fielders(), 6, false).is_err());
}

// Secondary runners hold after the original First force is removed; they do not
// create another retirement, replace a carried charge or invalidate the direct relay.
#[test]
fn occupied_first_force_choices_cover_secondary_runners_without_changing_their_charge() {
    for half in [Half::Top, Half::Bottom] {
        for mask in [1, 3, 5, 7] {
            for outs in 0..3 {
                for failed_bunt in [false, true] {
                    for position in 1..=6 {
                        let mut before = state(mask, outs);
                        before.half = half;
                        if half == Half::Bottom {
                            before.teams.swap(0, 1);
                        }
                        before.bases[0].as_mut().unwrap().responsible_pitcher =
                            "inherited-first".into();
                        let saved = before.clone();
                        let play = full_game::force_choice(&before, &fielders(), position, failed_bunt)
                            .unwrap_or_else(|e| panic!("{half:?} mask={mask} outs={outs} position={position} failed_bunt={failed_bunt}: {e}"));
                        let t = play.transition();
                        assert_eq!(before, saved);
                        assert_eq!(t.event().delta.outs, 1);
                        assert_eq!(t.event().delta.runs, 0);
                        assert_eq!(t.after().teams[half.offense()].next_batter, 1);
                        if outs < 2 {
                            assert_eq!(
                                t.after().bases[0].as_ref().unwrap().responsible_pitcher,
                                "inherited-first"
                            );
                            assert_eq!(t.after().bases[1..], before.bases[1..]);
                        } else {
                            assert!(t.after().bases.iter().all(Option::is_none));
                        }
                        assert!(!t
                            .event()
                            .delta
                            .credits
                            .iter()
                            .any(|c| c.kind == CreditKind::SacrificeBunt));
                        assert_eq!(apply(&before, t.event().clone()).unwrap(), *t);
                        assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
                    }
                }
            }
        }
    }
}

#[test]
fn direct_force_relays_cover_all_occupied_first_masks_with_secondary_runners_held() {
    for half in [Half::Top, Half::Bottom] {
        for mask in [1, 3, 5, 7] {
            for outs in 0..2 {
                for position in 1u8..=6 {
                    let mut before = state(mask, outs);
                    before.half = half;
                    if half == Half::Bottom {
                        before.teams.swap(0, 1);
                    }
                    let defense = fielders();
                    let facts = bbsim_engine::GroundForceDoublePlayFacts {
                        batter_id: "a0".into(),
                        forced_runner_id: "a5".into(),
                        initial_fielder_id: defense[usize::from(position) - 1].clone(),
                        pivot_id: defense[if position == 4 { 5 } else { 3 }].clone(),
                        first_receiver_id: defense[2].clone(),
                    };
                    let play = full_game::contact(
                        &before,
                        &defense,
                        full_game::ContactDecision::GroundForceDoublePlay(facts.clone()),
                    )
                    .unwrap_or_else(|e| {
                        panic!("{half:?} mask={mask} outs={outs} position={position}: {e}")
                    });
                    let t = play.transition();
                    assert_eq!(t.event().delta.outs, 2);
                    assert_eq!(t.event().delta.runs, 0);
                    if outs == 0 {
                        assert_eq!(t.after().bases[1..], before.bases[1..]);
                    } else {
                        assert!(t.after().bases.iter().all(Option::is_none));
                    }
                    assert_eq!(
                        t.event()
                            .delta
                            .credits
                            .iter()
                            .filter(|c| c.kind == CreditKind::Putout)
                            .count(),
                        2
                    );
                    assert_eq!(
                        t.event()
                            .delta
                            .credits
                            .iter()
                            .filter(|c| c.kind == CreditKind::Assist)
                            .count(),
                        2
                    );
                    assert_eq!(
                        bbsim_engine::apply_with_ground_force_double_play_facts(
                            &before,
                            t.event().clone(),
                            facts
                        )
                        .unwrap(),
                        *t
                    );
                    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
                }
            }
        }
    }
}

#[test]
fn ordinary_ground_outs_bind_selected_position_including_first_base_unassisted() {
    for mask in 0..8 {
        for outs in 0..3 {
            for position in 1..=6 {
                let before = state(mask, outs);
                let play = full_game::ground_out(&before, &fielders(), position).unwrap();
                let t = play.transition();
                assert_eq!(t.before(), &before);
                assert_eq!(t.event().delta.outs, 1);
                assert_eq!(t.after().score, before.score);
                assert_eq!(apply(&before, t.event().clone()).unwrap(), *t);
                assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
                let full_game::Mechanism::GroundOut {
                    fielder,
                    receiver,
                    position: recorded,
                } = play.mechanism()
                else {
                    panic!("wrong selected ground mechanism")
                };
                assert_eq!(*recorded, position);
                assert_eq!(fielder, &fielders()[position as usize - 1]);
                assert_eq!(receiver, &fielders()[2]);
                let putouts: Vec<_> = t
                    .event()
                    .delta
                    .credits
                    .iter()
                    .filter(|c| c.kind == CreditKind::Putout)
                    .collect();
                assert_eq!(putouts.len(), 1);
                assert_eq!(putouts[0].player_id, fielders()[2]);
                let assists: Vec<_> = t
                    .event()
                    .delta
                    .credits
                    .iter()
                    .filter(|c| c.kind == CreditKind::Assist)
                    .collect();
                assert_eq!(assists.len(), usize::from(position != 3));
                if position == 3 {
                    assert!(t.event().ordinary_ground_throw.is_none());
                } else {
                    assert_eq!(assists[0].player_id, *fielder);
                }
            }
        }
    }
}

#[test]
fn ended_games_and_unassigned_positions_never_become_completed_plays() {
    for position in [0, 7, 8, 9, 10, 255] {
        assert!(full_game::ground_out(&state(0, 0), &fielders(), position).is_err());
    }
    let mut before = state(0, 0);
    before.over = true;
    assert!(full_game::ground_out(&before, &fielders(), 6).is_err());
    let mut defenders = fielders();
    defenders[0] = "spectator".into();
    assert!(full_game::ground_out(&state(0, 0), &defenders, 1).is_err());
}

#[test]
fn ordinary_leaves_preserve_selected_result_and_actual_named_defense_in_every_base_out_cell() {
    use full_game::Ordinary::*;
    let choices = [
        CalledStrikeout,
        SwingingStrikeout,
        Walk,
        IntentionalWalk,
        Hbp,
        Single(7),
        Double(8),
        Triple(9),
        HomeRun(8),
        FlyOut(7),
        LineOut(4),
        PopOut(2),
        FieldingError(5),
        CatcherInterference,
    ];
    for mask in 0..8 {
        for outs in 0..3 {
            for choice in choices {
                let before = state(mask, outs);
                let play = full_game::ordinary(&before, &fielders(), choice).unwrap();
                let t = play.transition();
                assert_eq!(t.before(), &before);
                assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
                assert_eq!(apply(&before, t.event().clone()).unwrap(), *t);
                assert_eq!(
                    t.event()
                        .delta
                        .credits
                        .iter()
                        .filter(|c| c.kind == CreditKind::PlateAppearance)
                        .count(),
                    1
                );
                let full_game::Mechanism::Ordinary {
                    choice: recorded,
                    fielders: recorded_fielders,
                } = play.mechanism()
                else {
                    panic!("ordinary selected result changed")
                };
                assert_eq!(*recorded, choice);
                assert_eq!(*recorded_fielders, fielders());
                let count = |kind| {
                    t.event()
                        .delta
                        .credits
                        .iter()
                        .filter(|c| c.kind == kind)
                        .count()
                };
                match choice {
                    Single(_) | Double(_) | Triple(_) | HomeRun(_) => {
                        assert_eq!(count(CreditKind::Hit), 1);
                        assert_eq!(
                            t.event().delta.outs,
                            0,
                            "selected hit must not face another out roll"
                        );
                    }
                    FlyOut(p) | LineOut(p) | PopOut(p) => {
                        assert!(t.event().ordinary_air_catch.is_some());
                        assert_eq!(t.event().delta.outs, 1);
                        assert_eq!(count(CreditKind::Assist), 0);
                        assert!(t
                            .event()
                            .delta
                            .credits
                            .iter()
                            .any(|c| c.kind == CreditKind::Putout
                                && c.player_id == fielders()[p as usize - 1]));
                    }
                    CalledStrikeout | SwingingStrikeout => {
                        assert_eq!(count(CreditKind::Strikeout), 1);
                    }
                    Walk => {
                        assert_eq!(count(CreditKind::OrdinaryWalk), 1);
                    }
                    IntentionalWalk => {
                        assert_eq!(count(CreditKind::IntentionalWalk), 1);
                        assert_eq!(count(CreditKind::OrdinaryWalk), 0);
                    }
                    Hbp => {
                        assert_eq!(count(CreditKind::HitByPitch), 1);
                    }
                    FieldingError(p) | ThrowingError(p) => {
                        assert_eq!(count(CreditKind::Error), 1);
                        assert!(t
                            .event()
                            .delta
                            .credits
                            .iter()
                            .any(|c| c.kind == CreditKind::Error
                                && c.player_id == fielders()[p as usize - 1]));
                    }
                    CatcherInterference => {
                        assert_eq!(count(CreditKind::InterferenceAward), 1);
                        assert_eq!(count(CreditKind::Error), 1);
                        assert_eq!(count(CreditKind::Walk), 0);
                    }
                }
            }
        }
    }
}

#[test]
fn supplied_contact_and_runner_decisions_remain_bound_to_their_transitions() {
    let before = state(1, 0);
    let defense = fielders();
    let choice = full_game::ContactDecision::GroundForceDoublePlay(
        bbsim_engine::GroundForceDoublePlayFacts {
            batter_id: "a0".into(),
            forced_runner_id: "a5".into(),
            initial_fielder_id: defense[5].clone(),
            pivot_id: defense[3].clone(),
            first_receiver_id: defense[2].clone(),
        },
    );
    let play = full_game::contact(&before, &defense, choice.clone()).unwrap();
    assert_eq!(play.transition().event().delta.outs, 2);
    assert!(
        matches!(play.mechanism(), full_game::Mechanism::Contact { decision, .. } if decision == &choice)
    );
}

#[test]
fn supplied_runner_decision_remains_bound_without_advancing_pa_cursor() {
    let before = state(1, 0);
    let defense = fielders();
    let decision = full_game::RunnerDecision::Steal {
        origin: bbsim_engine::Base::First,
        target: bbsim_engine::Destination::Second,
        caught: false,
    };
    let play = full_game::runner(&before, &defense, decision).unwrap();
    assert!(
        matches!(play.mechanism(), full_game::Mechanism::Runner { decision: recorded, fielders } if recorded == &decision && fielders == &defense)
    );
    assert_eq!(play.transition().after().teams[0].next_batter, 0);
    assert_eq!(
        EventTransition::from_bytes(&play.transition().to_bytes()).unwrap(),
        *play.transition()
    );
}
