//! Supplied modeled choices, not source reconstruction or an attempt/success policy.
#[path = "../examples/support/runner_only.rs"]
mod runner_only;
use bbsim_engine::{
    accounting::derive_accounting, apply, Base, CreditKind, Destination, EventKind,
    EventTransition, GameState, Half, Runner, RunnerEvent, TeamState, ThirdOutKind,
};
use runner_only::{project, StealCause, StealDecision, StealResult};

fn state(mask: u8, outs: u8, half: Half, dh: bool) -> GameState {
    let teams = ["a", "h"].map(|prefix| {
        let pitcher = format!("{prefix}p");
        let mut batting_order = std::array::from_fn(|i| format!("{prefix}{i}"));
        if !dh {
            batting_order[7] = pitcher.clone();
        }
        TeamState {
            defense: bbsim_engine::DefenseState::LegacyUnassigned,
            batting_order,
            next_batter: 0,
            pitcher: (pitcher).into(),
            dh_slot: dh.then_some(8),
            dh_must_bat: dh,
            bench: vec![format!("{prefix}bench")],
            removed: vec![format!("{prefix}removed")],
        }
    });
    let bases = std::array::from_fn(|i| {
        (mask & (1 << i) != 0).then(|| Runner {
            player_id: teams[half.offense()].batting_order[i + 3].clone(),
            responsible_pitcher: format!("inherited-{i}"),
        })
    });
    GameState {
        inning: 2,
        half,
        outs,
        score: [0, 0],
        teams,
        bases,
        over: false,
    }
}
fn decision(before: &GameState, origin: usize, caught: bool) -> StealDecision {
    let defense = &before.teams[1 - before.half.offense()];
    StealDecision {
        runner_id: before.teams[before.half.offense()].batting_order[origin + 3].clone(),
        target: [Destination::Second, Destination::Third, Destination::Home][origin],
        cause: StealCause::OrdinaryReviewed,
        result: if caught {
            StealResult::Caught {
                putout: defense.batting_order[2].clone(),
                assists: vec![defense
                    .pitcher
                    .active_id()
                    .expect("legacy fixture pitcher")
                    .to_owned()],
            }
        } else {
            StealResult::Safe
        },
    }
}
fn refused(before: &GameState, value: &StealDecision) {
    let saved = before.clone();
    assert!(project(before, value).is_err(), "{before:?} / {value:?}");
    assert_eq!(before, &saved);
}
fn check(before: &GameState, value: &StealDecision, origin: usize, caught: bool) {
    let saved = before.clone();
    let transition = project(before, value).unwrap();
    assert_eq!(before, &saved);
    assert_eq!(transition.before(), before);
    assert_eq!(
        apply(before, transition.event().clone()).unwrap(),
        transition
    );
    assert_eq!(
        EventTransition::try_new(
            before.clone(),
            transition.event().clone(),
            transition.after().clone()
        )
        .unwrap(),
        transition
    );
    assert_eq!(
        EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
        transition
    );
    let event = transition.event();
    assert_eq!(event.kind, EventKind::RunnerOnly(RunnerEvent::StealAttempt));
    assert_eq!(
        event.participants.batter.as_deref(),
        Some(before.teams[before.half.offense()].batting_order[0].as_str())
    );
    assert_eq!(
        event.participants.pitcher.as_deref(),
        before.teams[1 - before.half.offense()].pitcher.active_id()
    );
    assert_eq!(transition.after().teams, before.teams);
    assert_eq!(transition.after().score, before.score);
    assert_eq!(event.delta.outs, u8::from(caught));
    assert_eq!(event.delta.runs, 0);
    assert!(event.delta.changes.is_empty());
    let count = |kind| {
        event
            .delta
            .credits
            .iter()
            .filter(|c| c.kind == kind)
            .count()
    };
    assert_eq!(count(CreditKind::PlateAppearance), 0);
    assert_eq!(count(CreditKind::AtBat), 0);
    assert_eq!(count(CreditKind::StolenBase), usize::from(!caught));
    assert_eq!(count(CreditKind::CaughtStealing), usize::from(caught));
    assert_eq!(
        event.delta.moves.len(),
        before.bases.iter().flatten().count()
    );
    for (movement, (i, runner)) in event.delta.moves.iter().zip(
        before
            .bases
            .iter()
            .enumerate()
            .filter_map(|(i, r)| r.as_ref().map(|r| (i, r))),
    ) {
        assert_eq!(&movement.runner, runner);
        assert_eq!(movement.from, [Base::First, Base::Second, Base::Third][i]);
        let expected = if i == origin {
            if caught {
                Destination::Out
            } else {
                value.target
            }
        } else {
            [Destination::First, Destination::Second, Destination::Third][i]
        };
        assert_eq!(movement.to, expected);
    }
    if caught && before.outs == 2 {
        assert!(transition.after().bases.iter().all(Option::is_none));
        assert_eq!(transition.after().outs, 0);
        assert_eq!(
            transition.after().half,
            if before.half == Half::Top {
                Half::Bottom
            } else {
                Half::Top
            }
        );
        assert_eq!(
            transition.after().inning,
            before.inning + u16::from(before.half == Half::Bottom)
        );
        let third = event.delta.third_out.as_ref().unwrap();
        assert_eq!(third.runner_id, value.runner_id);
        assert_eq!(third.kind, ThirdOutKind::Timing);
        assert!(third.home_before_out.is_empty());
    } else {
        assert!(event.delta.third_out.is_none());
        assert_eq!(transition.after().outs, before.outs + u8::from(caught));
        for i in 0..3 {
            let mut expected = before.bases[i].clone();
            if i == origin {
                expected = None;
            }
            if !caught && i == origin + 1 {
                expected = before.bases[origin].clone();
            }
            assert_eq!(transition.after().bases[i], expected);
        }
    }
    let stats = derive_accounting(std::slice::from_ref(&transition)).unwrap();
    assert_eq!(stats.runs, [0, 0]);
    assert_eq!(
        stats
            .batters
            .iter()
            .flat_map(|team| team.values())
            .map(|line| line.plate_appearances)
            .sum::<u32>(),
        0
    );
    assert_eq!(
        stats
            .pitchers
            .iter()
            .flat_map(|team| team.values())
            .map(|line| line.batters_faced)
            .sum::<u32>(),
        0
    );
    assert_eq!(
        stats
            .pitchers
            .iter()
            .flat_map(|team| team.values())
            .map(|line| line.outs_recorded)
            .sum::<u32>(),
        u32::from(caught)
    );
    let actor = &stats.batters[before.half.offense()][&value.runner_id];
    assert_eq!(actor.credited_steals, u32::from(!caught));
    assert_eq!(actor.credited_caught_stealing, u32::from(caught));
    assert!(
        transition.fielding_contribution().is_none(),
        "no new official PO/A proof"
    );
    if let StealResult::Caught { putout, assists } = &value.result {
        assert_eq!(count(CreditKind::Putout), 1);
        assert_eq!(count(CreditKind::Assist), assists.len());
        assert_eq!(
            stats.fielders[1 - before.half.offense()][putout].credited_putouts,
            1
        );
        for fielder in assists {
            assert_eq!(
                stats.fielders[1 - before.half.offense()][fielder].credited_assists,
                1
            );
        }
    } else {
        assert_eq!(count(CreditKind::Putout) + count(CreditKind::Assist), 0);
    }
}
#[test]
fn vacant_defense_returns_an_error_not_a_panicking_string_assumption() {
    let mut before = state(1, 0, Half::Top, true);
    let value = decision(&before, 0, false);
    before.teams[1].pitcher = bbsim_engine::PitcherRole::Vacant { batting_slot: 8 };
    refused(&before, &value);
}
#[test]
fn standalone_projector_version_is_explicit() {
    assert_eq!(
        runner_only::POLICY_VERSION,
        "ordinary-runner-only-projection-v1"
    );
}

#[test]
fn every_clear_next_target_projects_full_vectors_and_independent_credits() {
    let mut frames = 0;
    for half in [Half::Top, Half::Bottom] {
        for inning in [2, 9, 10] {
            if half == Half::Bottom && inning >= 9 {
                continue;
            }
            for dh in [false, true] {
                for outs in 0..3 {
                    for mask in 0..8 {
                        for origin in 0..2 {
                            if mask & (1 << origin) == 0 || mask & (1 << (origin + 1)) != 0 {
                                continue;
                            }
                            let mut before = state(mask, outs, half, dh);
                            before.inning = inning;
                            for caught in [false, true] {
                                check(&before, &decision(&before, origin, caught), origin, caught);
                                frames += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(frames, 192);
}
#[test]
fn independent_geometry_oracle_covers_empty_occupied_nonnext_and_home_targets() {
    let mut attempts = 0;
    let mut accepted = 0;
    for half in [Half::Top, Half::Bottom] {
        for dh in [false, true] {
            for outs in 0..3 {
                for mask in 0..8 {
                    for origin in 0..3 {
                        let before = state(mask, outs, half, dh);
                        for target in [
                            Destination::First,
                            Destination::Second,
                            Destination::Third,
                            Destination::Home,
                            Destination::Out,
                        ] {
                            for caught in [false, true] {
                                let mut value = decision(&before, origin, caught);
                                value.target = target;
                                let legal = origin < 2
                                    && mask & (1 << origin) != 0
                                    && mask & (1 << (origin + 1)) == 0
                                    && target
                                        == [
                                            Destination::Second,
                                            Destination::Third,
                                            Destination::Home,
                                        ][origin];
                                let saved = before.clone();
                                assert_eq!(
                                    project(&before, &value).is_ok(),
                                    legal,
                                    "{mask}/{outs}/{origin}/{target:?}/{caught}"
                                );
                                assert_eq!(before, saved);
                                attempts += 1;
                                accepted += usize::from(legal);
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!((attempts, accepted), (2880, 96));
}
#[test]
fn unknown_cause_bad_actor_and_malformed_or_ended_state_refuse_without_mutation() {
    let before = state(1, 0, Half::Top, true);
    let good = decision(&before, 0, false);
    let mut unknown = good.clone();
    unknown.cause = StealCause::Unknown;
    refused(&before, &unknown);
    for actor in ["", "foreign", "a0", "abench", "aremoved"] {
        let mut bad = good.clone();
        bad.runner_id = actor.into();
        refused(&before, &bad);
    }
    for defect in 0..8 {
        let mut bad = before.clone();
        match defect {
            0 => bad.over = true,
            1 => bad.inning = 0,
            2 => bad.outs = 3,
            3 => bad.teams[0].next_batter = 9,
            4 => bad.bases[1] = bad.bases[0].clone(),
            5 => bad.bases[0].as_mut().unwrap().responsible_pitcher.clear(),
            6 => bad.teams[1].batting_order[1] = bad.teams[1].batting_order[0].clone(),
            _ => bad.bases[0].as_mut().unwrap().player_id = "a0".into(),
        }
        refused(&bad, &good);
    }
}
#[test]
fn all_terminal_bottom_innings_refuse_but_top_ninth_and_extra_controls_work() {
    let mut before = state(1, 2, Half::Bottom, true);
    let value = decision(&before, 0, true);
    for inning in 9..=u16::MAX {
        before.inning = inning;
        refused(&before, &value);
    }
    before = state(1, 2, Half::Top, true);
    before.inning = 9;
    before.score = [0, 1];
    let value = decision(&before, 0, true);
    let ended = project(&before, &value).unwrap();
    assert!(ended.after().over);
    assert_eq!(ended.after().half, Half::Top);
    before.inning = 10;
    before.score = [1, 1];
    check(&before, &value, 0, true);
    assert!(!project(&before, &value).unwrap().after().over);
}
#[test]
fn named_fielding_actors_must_be_eligible_and_assist_units_unique() {
    for half in [Half::Top, Half::Bottom] {
        for dh in [false, true] {
            let before = state(1, 0, half, dh);
            let defense = &before.teams[1 - half.offense()];
            for actor in defense
                .batting_order
                .iter()
                .map(String::as_str)
                .chain(defense.pitcher.active_id())
            {
                let mut value = decision(&before, 0, true);
                value.result = StealResult::Caught {
                    putout: actor.to_owned(),
                    assists: vec![actor.to_owned()],
                };
                if dh && actor == defense.batting_order[8] {
                    refused(&before, &value);
                } else {
                    check(&before, &value, 0, true);
                }
            }
            let mut bad_actors = vec![
                String::new(),
                "foreign".into(),
                defense.bench[0].clone(),
                defense.removed[0].clone(),
                before.teams[half.offense()]
                    .pitcher
                    .active_id()
                    .expect("legacy fixture pitcher")
                    .to_owned(),
            ];
            if dh {
                bad_actors.push(defense.batting_order[8].clone());
            }
            for actor in bad_actors {
                for bad_putout in [false, true] {
                    let mut value = decision(&before, 0, true);
                    value.result = if bad_putout {
                        StealResult::Caught {
                            putout: actor.clone(),
                            assists: vec![],
                        }
                    } else {
                        StealResult::Caught {
                            putout: defense
                                .pitcher
                                .active_id()
                                .expect("legacy fixture pitcher")
                                .to_owned(),
                            assists: vec![actor.clone()],
                        }
                    };
                    refused(&before, &value);
                }
            }
            let mut repeated = decision(&before, 0, true);
            repeated.result = StealResult::Caught {
                putout: defense
                    .pitcher
                    .active_id()
                    .expect("legacy fixture pitcher")
                    .to_owned(),
                assists: vec![defense.batting_order[1].clone(); 2],
            };
            refused(&before, &repeated);
            let mut multiple = decision(&before, 0, true);
            multiple.result = StealResult::Caught {
                putout: defense
                    .pitcher
                    .active_id()
                    .expect("legacy fixture pitcher")
                    .to_owned(),
                assists: vec![
                    defense.batting_order[1].clone(),
                    defense.batting_order[2].clone(),
                ],
            };
            check(&before, &multiple, 0, true);
            let mut unassisted = decision(&before, 0, true);
            unassisted.result = StealResult::Caught {
                putout: defense
                    .pitcher
                    .active_id()
                    .expect("legacy fixture pitcher")
                    .to_owned(),
                assists: vec![],
            };
            check(&before, &unassisted, 0, true);
        }
    }
}
