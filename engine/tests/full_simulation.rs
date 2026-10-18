//! Connected ordinary generation: no scripted success sequence or renderer-created events.
#[allow(dead_code)]
#[path = "../examples/support/full_game.rs"]
mod full_game;
#[allow(dead_code)]
#[path = "../examples/support/full_manager.rs"]
mod full_manager;
#[path = "../examples/support/full_simulation.rs"]
mod full_simulation;
#[allow(dead_code)]
#[path = "../examples/support/preview.rs"]
mod preview;
use bbsim_engine::{accounting, CreditKind, EventTransition, GameState, Half, TeamState};

fn initial() -> GameState {
    GameState {
        inning: 1,
        half: Half::Top,
        outs: 0,
        score: [0, 0],
        teams: ["a", "h"].map(|p| TeamState {
            defense: bbsim_engine::DefenseState::LegacyUnassigned,
            batting_order: std::array::from_fn(|i| format!("{p}{i}")),
            next_batter: 0,
            pitcher: (format!("{p}p")).into(),
            dh_slot: Some(8),
            dh_must_bat: false,
            bench: vec![],
            removed: vec![],
        }),
        bases: [None, None, None],
        over: false,
    }
}
fn defense() -> [[String; 9]; 2] {
    ["a", "h"].map(|p| {
        std::array::from_fn(|i| {
            if i == 0 {
                format!("{p}p")
            } else {
                format!("{p}{}", i - 1)
            }
        })
    })
}
fn model() -> full_simulation::Model {
    full_simulation::Model::new(
        [42823, 15142, 1984, 25947, 8531, 785, 6776, 1470, 82245, 61],
        [17188, 19528, 5379, 536, 39614],
    )
    .unwrap()
}
#[test]
fn pending_defense_is_refused_before_any_rng_word() {
    for vacancy in [false, true] {
        let mut before = initial();
        let mut positions = defense()[1].clone().map(Some);
        positions[1] = None;
        before.teams[1].defense = bbsim_engine::DefenseState::Assigned(Box::new(positions));
        if vacancy {
            before.teams[1].dh_slot = None;
            before.teams[1].pitcher = bbsim_engine::PitcherRole::Vacant { batting_slot: 8 };
            if let bbsim_engine::DefenseState::Assigned(positions) = &mut before.teams[1].defense {
                positions[0] = None;
            }
        }
        before.bases[0] = Some(bbsim_engine::Runner {
            player_id: "a1".into(),
            responsible_pitcher: "hp".into(),
        });
        let mut rng = preview::Rng::new(67);
        let expected = preview::Rng::new(67).next_u64();
        assert!(model().next(&before, &defense()[1], &mut rng).is_err());
        assert_eq!(
            rng.next_u64(),
            expected,
            "vacancy={vacancy}: refusal consumed randomness"
        );
        let mut calls = 0;
        assert!(preview::game_from_transitions(67, before.clone(), |_, _| {
            calls += 1;
            Err("producer called before readiness".into())
        })
        .is_err());
        assert_eq!(calls, 0);
        assert!(preview::event_for(&before, preview::Outcome::Walk).is_err());
    }
}
#[test]
fn stale_or_pending_positions_refuse_before_rng_or_probability_callback() {
    let mut before = initial();
    before.teams[1].defense =
        bbsim_engine::DefenseState::Assigned(Box::new(defense()[1].clone().map(Some)));
    let mut stale = defense()[1].clone();
    stale.swap(1, 2);
    let mut rng = preview::Rng::new(67);
    let model = model();
    assert!(model.next(&before, &stale, &mut rng).is_err());
    assert_eq!(rng.next_u64(), preview::Rng::new(67).next_u64());
    if let bbsim_engine::DefenseState::Assigned(positions) = &mut before.teams[1].defense {
        positions[1] = None;
    }
    let mut calls = 0;
    assert!(full_simulation::simulate_with_probabilities(
        &before,
        &defense(),
        &model,
        None,
        67,
        |_| {
            calls += 1;
            Ok(None)
        }
    )
    .is_err());
    assert_eq!(calls, 0);
}
#[test]
fn preview_event_construction_uses_explicit_position_assignments() {
    let mut before = initial();
    let mut positions = defense()[1].clone();
    positions.swap(1, 5);
    before.teams[1].defense =
        bbsim_engine::DefenseState::Assigned(Box::new(positions.clone().map(Some)));
    let strikeout = preview::event_for(&before, preview::Outcome::Strikeout).unwrap();
    assert_eq!(
        strikeout.caught_strikeout.as_ref().unwrap().catcher_id,
        positions[1]
    );
    let out = preview::event_for(&before, preview::Outcome::ContactOut).unwrap();
    assert_eq!(
        out.ordinary_ground_throw.as_ref().unwrap().thrower_id,
        positions[5]
    );
    assert_eq!(
        out.ordinary_ground_throw.as_ref().unwrap().receiver_id,
        positions[2]
    );
    for event in [strikeout, out] {
        let t = bbsim_engine::apply(&before, event).unwrap();
        let bytes = t.to_bytes();
        assert!(bytes.starts_with(b"BBE33"));
        assert_eq!(EventTransition::from_bytes(&bytes).unwrap(), t);
    }
}
#[test]
fn simulation_refreshes_all_canonical_positions_not_only_the_pitcher() {
    let mut before = initial();
    let stale = defense();
    let mut assigned = stale.clone();
    assigned[1].swap(1, 2);
    for (side, positions) in assigned.iter().enumerate() {
        before.teams[side].defense =
            bbsim_engine::DefenseState::Assigned(Box::new(positions.clone().map(Some)));
    }
    let model = model();
    let first = full_simulation::simulate(&before, &stale, &model, None, 67).unwrap();
    let correct = full_simulation::simulate(&before, &assigned, &model, None, 67).unwrap();
    assert_eq!(first, correct);
    assert!(first.last().unwrap().transition().after().over);
    for step in first {
        assert!(step.transition().to_bytes().starts_with(b"BBE33"));
    }
}
#[test]
fn ordinary_seeded_games_complete_with_canonical_continuity_and_independent_totals() {
    let model = model();
    assert!(full_simulation::MODEL_VERSION.starts_with("text-game-recovery-v1"));
    let mut secondary_contexts = [false; 3];
    let mut joint_contexts = [false; 7];
    for seed in 1..=10_000 {
        let steps = full_simulation::simulate(&initial(), &defense(), &model, None, seed)
            .unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        assert!(!steps.is_empty());
        let log: Vec<_> = steps.iter().map(|step| step.transition().clone()).collect();
        assert_eq!(log.first().unwrap().before(), &initial());
        assert!(log.last().unwrap().after().over, "seed {seed}");
        for pair in log.windows(2) {
            assert_eq!(pair[0].after(), pair[1].before(), "seed {seed}");
        }
        for t in &log {
            assert_eq!(
                EventTransition::from_bytes(&t.to_bytes()).unwrap(),
                *t,
                "seed {seed}"
            );
        }
        for step in &steps {
            if let full_simulation::Step::Play(play) = step {
                let before = play.transition().before();
                match play.mechanism() {
                    full_game::Mechanism::Contact {
                        decision: full_game::ContactDecision::GroundForceDoublePlay(_),
                        ..
                    } if before.bases[1].is_some() => secondary_contexts[0] = true,
                    full_game::Mechanism::ForceChoice {
                        failed_bunt: false, ..
                    } if before.bases[1..].iter().any(Option::is_some) => {
                        secondary_contexts[1] = true
                    }
                    full_game::Mechanism::ForceChoice {
                        failed_bunt: true, ..
                    } if before.bases[2].is_some() => secondary_contexts[2] = true,
                    full_game::Mechanism::Hit { decision, .. } => {
                        let event = play.transition().event();
                        assert!(
                            !event
                                .delta
                                .moves
                                .iter()
                                .any(|m| m.from == bbsim_engine::Base::Plate
                                    && m.to == bbsim_engine::Destination::Out),
                            "seed {seed}: selected safe hitter converted to out"
                        );
                        assert_eq!(
                            event
                                .delta
                                .credits
                                .iter()
                                .filter(|c| c.kind == CreditKind::Hit)
                                .count(),
                            1
                        );
                        if decision.tag.is_some() {
                            joint_contexts[2] = true;
                            assert_eq!(event.delta.outs, 1);
                        } else if let Some(first) = &before.bases[0] {
                            joint_contexts[0] |= decision.value == 1
                                && event.delta.moves.iter().any(|m| {
                                    m.runner.player_id == first.player_id
                                        && m.to == bbsim_engine::Destination::Third
                                });
                            joint_contexts[1] |= decision.value == 2
                                && event.delta.moves.iter().any(|m| {
                                    m.runner.player_id == first.player_id
                                        && m.to == bbsim_engine::Destination::Home
                                });
                        }
                    }
                    full_game::Mechanism::UncaughtStrikeout { decision, .. } => {
                        use full_game::StrikeoutDecision::*;
                        joint_contexts[match decision {
                            Automatic => 3,
                            SafeReach { .. } => 4,
                            RecoveredBeforeFirst { .. } => 5,
                            RecoveredBeforeFirstAfterAdvances { .. } => 6,
                        }] = true;
                    }
                    _ => {}
                }
            }
        }
        let stats = accounting::derive_accounting(&log).unwrap();
        let mut runs = [0; 2];
        let mut outs = [0; 2];
        let mut pa = [0; 2];
        for t in &log {
            let offense = t.before().half.offense();
            runs[offense] += u32::from(t.event().delta.runs);
            outs[1 - offense] += u32::from(t.event().delta.outs);
            let is_pa = matches!(
                t.event().kind,
                bbsim_engine::EventKind::PlateAppearance
                    | bbsim_engine::EventKind::OutOfParkHomeRun
                    | bbsim_engine::EventKind::FieldersChoice
            );
            pa[offense] += u32::from(is_pa);
            assert_eq!(
                t.event()
                    .delta
                    .credits
                    .iter()
                    .filter(|c| c.kind == CreditKind::Putout)
                    .count(),
                usize::from(t.event().delta.outs),
                "seed {seed}: raw named putouts must account for every generated retirement"
            );
            for side in 0..2 {
                let expected = if is_pa && side == offense {
                    (t.before().teams[side].next_batter + 1) % 9
                } else {
                    t.before().teams[side].next_batter
                };
                assert_eq!(
                    t.after().teams[side].next_batter,
                    expected,
                    "seed {seed}: batting cursor drift"
                );
            }
            assert_eq!(
                t.event()
                    .delta
                    .credits
                    .iter()
                    .filter(|c| c.kind == CreditKind::PlateAppearance)
                    .count(),
                usize::from(is_pa)
            );
        }
        assert_eq!(stats.runs, runs);
        assert_eq!(runs, log.last().unwrap().after().score.map(u32::from));
        for side in 0..2 {
            assert_eq!(
                stats.batters[side]
                    .values()
                    .map(|b| b.plate_appearances)
                    .sum::<u32>(),
                pa[side]
            );
            assert_eq!(
                stats.pitchers[side]
                    .values()
                    .map(|p| p.outs_recorded)
                    .sum::<u32>(),
                outs[side]
            );
            assert_eq!(
                stats.pitchers[side]
                    .values()
                    .map(|p| p.batters_faced)
                    .sum::<u32>(),
                pa[1 - side]
            );
        }
        let again = full_simulation::simulate(&initial(), &defense(), &model, None, seed).unwrap();
        assert_eq!(again, steps);
    }
    assert_eq!(joint_contexts,[true;7],"ordinary cohort must exercise extra-base single/double, runner-tag hit and all four uncaught-K leaves");
    assert_eq!(
        secondary_contexts, [true; 3],
        "ordinary cohort must exercise secondary-runner DP, FC and failed-bunt contexts"
    );
}
#[test]
fn constructed_ground_opportunities_generate_fc_dp_and_failed_bunt_with_held_secondaries() {
    use bbsim_engine::{Base, Runner};
    let ground = full_simulation::Model::new(
        [0, 0, 0, 0, 0, 0, 0, 0, 1_000_000, 0],
        [1_000_000, 0, 0, 0, 0],
    )
    .unwrap();
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..2 {
            for mask in [1, 3, 5, 7] {
                let mut before = initial();
                before.inning = 2;
                before.half = half;
                before.outs = outs;
                before.bases = std::array::from_fn(|i| {
                    (mask & (1 << i) != 0).then(|| Runner {
                        player_id: before.teams[half.offense()].batting_order[i + 3].clone(),
                        responsible_pitcher: before.teams[1 - half.offense()]
                            .pitcher
                            .active_id()
                            .expect("legacy fixture pitcher")
                            .to_owned(),
                    })
                });
                let mut seen = [false; 4];
                // Constructed repeated opportunities, not filtered ordinary full games.
                for seed in 1..=3_000 {
                    let play=ground.next(&before,&defense()[1-half.offense()],&mut preview::Rng::new(seed)).unwrap_or_else(|e|panic!("constructed mask={mask} outs={outs} half={half:?} seed={seed}: {e}"));
                    match play.mechanism() {
                        full_game::Mechanism::Contact {
                            decision: full_game::ContactDecision::GroundForceDoublePlay(_),
                            ..
                        } => seen[0] = true,
                        full_game::Mechanism::ForceChoice {
                            failed_bunt: false, ..
                        } => seen[1] = true,
                        full_game::Mechanism::ForceChoice {
                            failed_bunt: true, ..
                        } => seen[2] = true,
                        full_game::Mechanism::Contact {
                            decision: full_game::ContactDecision::SacrificeBunt(_),
                            ..
                        } => seen[3] = true,
                        _ => {}
                    }
                    if let full_game::Mechanism::ForceChoice { .. } = play.mechanism() {
                        for origin in [Base::Second, Base::Third] {
                            assert!(!play
                                .transition()
                                .event()
                                .delta
                                .moves
                                .iter()
                                .any(|m| m.from == origin));
                        }
                    }
                }
                assert_eq!(
                    seen,
                    [true, true, true, mask & 2 == 0],
                    "constructed mask={mask} outs={outs} half={half:?}"
                );
            }
        }
    }
}

#[test]
fn constructed_uncaught_k_sampler_distinguishes_countable_win_from_before_first_third_out() {
    use bbsim_engine::{Destination, Runner};
    let strikeouts =
        full_simulation::Model::new([1_000_000, 0, 0, 0, 0, 0, 0, 0, 1, 0], [1, 0, 0, 0, 0])
            .unwrap();
    let mut seen = [false; 3];
    for outs in 0..3 {
        let mut before = initial();
        before.inning = 9;
        before.half = Half::Bottom;
        before.outs = outs;
        before.bases = std::array::from_fn(|i| {
            (i == 2 || outs == 2).then(|| Runner {
                player_id: format!("h{}", i + 3),
                responsible_pitcher: "ap".into(),
            })
        });
        for seed in 1..=3_000 {
            let play = strikeouts
                .next(&before, &defense()[0], &mut preview::Rng::new(seed))
                .unwrap_or_else(|e| panic!("constructed K outs={outs} seed={seed}: {e}"));
            if let full_game::Mechanism::UncaughtStrikeout { decision, .. } = play.mechanism() {
                let t = play.transition();
                if outs < 2
                    && t.event()
                        .delta
                        .moves
                        .iter()
                        .any(|m| m.from == bbsim_engine::Base::Third && m.to == Destination::Home)
                {
                    assert!(matches!(
                        decision,
                        full_game::StrikeoutDecision::SafeReach { .. }
                    ));
                    assert!(t.after().over);
                    assert_eq!(t.event().delta.runs, 1);
                    seen[usize::from(outs)] = true;
                } else if outs == 2
                    && matches!(
                        decision,
                        full_game::StrikeoutDecision::RecoveredBeforeFirstAfterAdvances { .. }
                    )
                {
                    assert!(t
                        .event()
                        .delta
                        .moves
                        .iter()
                        .any(|m| m.from == bbsim_engine::Base::Third && m.to == Destination::Home));
                    assert_eq!(t.event().delta.runs, 0);
                    assert!(!t.after().over);
                    assert_eq!(t.after().inning, 10);
                    assert_eq!(t.after().half, Half::Top);
                    seen[2] = true;
                }
                assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
            }
        }
    }
    assert_eq!(seen,[true;3],"constructed countable winning advances and suppressed before-First third-out crossing must all generate");
}

#[test]
fn generated_accounting_check_reconciles_a_log_and_refuses_missing_retirement_evidence() {
    let before = initial();
    let play = full_game::ordinary(&before, &defense()[1], full_game::Ordinary::FlyOut(8)).unwrap();
    let ground = full_game::ground_out(play.transition().after(), &defense()[1], 6).unwrap();
    let log = vec![play.transition().clone(), ground.transition().clone()];
    full_simulation::verify_accounting(&log).unwrap();
    assert_eq!(
        full_simulation::verify_accounting(&[]).unwrap_err(),
        "empty generated log"
    );
    assert_eq!(
        full_simulation::verify_accounting(&[log[0].clone(), log[0].clone()]).unwrap_err(),
        "generated log is discontinuous"
    );
    let mut unobserved = log[0].event().clone();
    unobserved.ordinary_air_catch = None;
    unobserved.fielding_review = bbsim_engine::FieldingReview::Unknown;
    unobserved
        .delta
        .credits
        .retain(|c| !matches!(c.kind, CreditKind::Putout | CreditKind::Assist));
    let legal_unobserved = bbsim_engine::apply(&before, unobserved).unwrap();
    assert_eq!(
        full_simulation::verify_accounting(&[legal_unobserved]).unwrap_err(),
        "generated retirement lacks exact named putout evidence"
    );
}

#[test]
fn invalid_batting_cursor_returns_a_diagnostic_not_a_panic() {
    let mut before = initial();
    before.teams[0].next_batter = 9;
    assert!(
        full_simulation::simulate(&before, &defense(), &model(), None, 1)
            .unwrap_err()
            .contains("invalid batting cursor")
    );
}

#[test]
fn invalid_or_unreconciled_model_mass_refuses_before_drawing_games() {
    assert!(full_simulation::Model::new([0; 10], [0; 5]).is_err());
    assert!(full_simulation::Model::new([1, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0; 5]).is_err());
    assert!(full_simulation::Model::new([1; 10], [1; 5]).is_err());
    assert!(full_simulation::Model::new([u64::MAX; 10], [1; 5]).is_err());
}
