//! Supplied runner-only decisions projected through the canonical reducer.
#[path = "../examples/support/full_runner.rs"]
mod full_runner;

use bbsim_engine::accounting::derive_accounting;
use bbsim_engine::{
    apply, Base, CreditKind, Destination, EventKind, EventTransition, GameState, Half, Runner,
    RunnerEvent, TeamState, ThirdOutKind,
};
use full_runner::{project, RunnerDecision};

fn state(mask: u8, outs: u8, half: Half, inning: u16, score: [u16; 2]) -> GameState {
    let teams = ["away", "home"].map(|prefix| TeamState {
        defense: bbsim_engine::DefenseState::LegacyUnassigned,
        batting_order: std::array::from_fn(|slot| format!("{prefix}{slot}")),
        next_batter: 0,
        pitcher: (format!("{prefix}_p")).into(),
        dh_slot: Some(8),
        dh_must_bat: false,
        bench: vec![format!("{prefix}_bench")],
        removed: vec![format!("{prefix}_removed")],
    });
    let offense = half.offense();
    let bases = std::array::from_fn(|base| {
        (mask & (1 << base) != 0).then(|| Runner {
            player_id: teams[offense].batting_order[base + 3].clone(),
            responsible_pitcher: format!("inherited_{base}"),
        })
    });
    GameState {
        inning,
        half,
        outs,
        score,
        teams,
        bases,
        over: false,
    }
}

// P/C/1B/2B/3B/SS/LF/CF/RF; only selected actors are checked canonically.
fn fielders(before: &GameState) -> [String; 9] {
    let defense = &before.teams[1 - before.half.offense()];
    [
        defense
            .pitcher
            .active_id()
            .expect("legacy fixture pitcher")
            .to_owned(),
        defense.batting_order[1].clone(),
        defense.batting_order[2].clone(),
        defense.batting_order[3].clone(),
        defense.batting_order[4].clone(),
        defense.batting_order[5].clone(),
        defense.batting_order[6].clone(),
        defense.batting_order[7].clone(),
        defense.batting_order[0].clone(),
    ]
}

fn destination(base: Base) -> Destination {
    match base {
        Base::First => Destination::First,
        Base::Second => Destination::Second,
        Base::Third => Destination::Third,
        Base::Plate => panic!("plate is not a runner base"),
    }
}

fn has_credit(transition: &EventTransition, player_id: &str, kind: CreditKind) -> bool {
    transition
        .event()
        .delta
        .credits
        .iter()
        .any(|credit| credit.player_id == player_id && credit.kind == kind)
}

fn checked(before: &GameState, decision: &RunnerDecision) -> EventTransition {
    let original = before.clone();
    let transition = project(before, &fielders(before), decision).unwrap();
    assert_eq!(before, &original);
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
    assert_eq!(transition.after().teams, before.teams);
    assert_eq!(
        transition.event().participants.batter.as_deref(),
        Some(before.teams[before.half.offense()].batting_order[0].as_str())
    );
    assert_eq!(
        transition.event().participants.pitcher.as_deref(),
        before.teams[1 - before.half.offense()].pitcher.active_id()
    );
    assert_eq!(transition.event().delta.changes, []);
    assert!(!transition
        .event()
        .delta
        .credits
        .iter()
        .any(|credit| { matches!(credit.kind, CreditKind::PlateAppearance | CreditKind::AtBat) }));
    let stats = derive_accounting(std::slice::from_ref(&transition)).unwrap();
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
    transition
}

fn assert_runner_moves(
    transition: &EventTransition,
    before: &GameState,
    changed: usize,
    to: Destination,
) {
    assert_eq!(
        transition.event().delta.moves.len(),
        before.bases.iter().flatten().count()
    );
    for (index, movement, runner) in transition
        .event()
        .delta
        .moves
        .iter()
        .zip(
            before
                .bases
                .iter()
                .enumerate()
                .filter_map(|(index, runner)| runner.as_ref().map(|runner| (index, runner))),
        )
        .map(|(movement, (index, runner))| (index, movement, runner))
    {
        assert_eq!(&movement.runner, runner);
        assert_eq!(
            movement.from,
            [Base::First, Base::Second, Base::Third][index]
        );
        assert_eq!(
            movement.to,
            if index == changed {
                to
            } else {
                destination(movement.from)
            }
        );
    }
}

#[test]
fn steals_cover_every_mask_out_count_half_and_nonterminal_or_terminal_inning() {
    let mut frames = 0;
    for half in [Half::Top, Half::Bottom] {
        for inning in [2, 9, 10] {
            for outs in 0..=2 {
                for mask in 0..8 {
                    let score = if half == Half::Bottom && inning >= 9 {
                        [1, 1]
                    } else {
                        [0, 0]
                    };
                    let before = state(mask, outs, half, inning, score);
                    for origin in 0..3 {
                        if mask & (1 << origin) == 0
                            || (origin < 2 && mask & (1 << (origin + 1)) != 0)
                        {
                            continue;
                        }
                        let from = [Base::First, Base::Second, Base::Third][origin];
                        let target =
                            [Destination::Second, Destination::Third, Destination::Home][origin];
                        let runner = before.bases[origin].as_ref().unwrap();
                        for caught in [false, true] {
                            let decision = RunnerDecision::Steal {
                                origin: from,
                                target,
                                caught,
                            };
                            let transition = checked(&before, &decision);
                            let event = transition.event();
                            assert_eq!(
                                event.kind,
                                EventKind::RunnerOnly(RunnerEvent::StealAttempt)
                            );
                            let to = if caught { Destination::Out } else { target };
                            assert_runner_moves(&transition, &before, origin, to);
                            assert_eq!(event.delta.outs, u8::from(caught));
                            assert_eq!(
                                event.delta.runs,
                                u8::from(!caught && target == Destination::Home)
                            );
                            let credit_count = |kind| {
                                event
                                    .delta
                                    .credits
                                    .iter()
                                    .filter(|c| c.kind == kind)
                                    .count()
                            };
                            assert_eq!(credit_count(CreditKind::StolenBase), usize::from(!caught));
                            assert_eq!(
                                credit_count(CreditKind::CaughtStealing),
                                usize::from(caught)
                            );
                            assert_eq!(
                                credit_count(CreditKind::Run),
                                usize::from(!caught && target == Destination::Home)
                            );
                            assert_eq!(credit_count(CreditKind::Putout), usize::from(caught));
                            assert_eq!(
                                credit_count(CreditKind::Assist),
                                usize::from(caught && target != Destination::Home)
                            );
                            assert!(has_credit(
                                &transition,
                                &runner.player_id,
                                if caught {
                                    CreditKind::CaughtStealing
                                } else {
                                    CreditKind::StolenBase
                                }
                            ));
                            if caught {
                                let assignments = fielders(&before);
                                let putout = match target {
                                    Destination::Second => &assignments[3],
                                    Destination::Third => &assignments[4],
                                    Destination::Home => &assignments[1],
                                    Destination::First | Destination::Out => unreachable!(),
                                };
                                assert!(has_credit(&transition, putout, CreditKind::Putout));
                                if target != Destination::Home {
                                    assert!(has_credit(
                                        &transition,
                                        &assignments[1],
                                        CreditKind::Assist
                                    ));
                                }
                            }
                            if !caught && target == Destination::Home {
                                assert!(has_credit(
                                    &transition,
                                    &runner.player_id,
                                    CreditKind::Run
                                ));
                            }
                            assert_eq!(transition.after().teams, before.teams);
                            assert_eq!(
                                transition.after().score[half.offense()],
                                score[half.offense()]
                                    + u16::from(!caught && target == Destination::Home)
                            );
                            if caught && outs == 2 {
                                assert_eq!(
                                    event.delta.third_out.as_ref().unwrap().runner_id,
                                    runner.player_id
                                );
                                assert_eq!(
                                    event.delta.third_out.as_ref().unwrap().kind,
                                    ThirdOutKind::Timing
                                );
                                assert!(event
                                    .delta
                                    .third_out
                                    .as_ref()
                                    .unwrap()
                                    .home_before_out
                                    .is_empty());
                                assert_eq!(transition.after().outs, 0);
                                assert!(transition.after().bases.iter().all(Option::is_none));
                                assert_eq!(
                                    transition.after().half,
                                    if half == Half::Top {
                                        Half::Bottom
                                    } else {
                                        Half::Top
                                    }
                                );
                                assert_eq!(
                                    transition.after().inning,
                                    inning + u16::from(half == Half::Bottom)
                                );
                            }
                            if !caught
                                && target == Destination::Home
                                && half == Half::Bottom
                                && inning >= 9
                            {
                                assert!(
                                    transition.after().over,
                                    "walk-off steal home must end the game"
                                );
                            }
                            if caught && outs < 2 {
                                assert!(event.delta.third_out.is_none());
                            }
                            frames += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(frames, 288);
}

#[test]
fn pickoffs_preserve_held_runners_and_record_a_named_out_only_when_caught() {
    let mut frames = 0;
    for half in [Half::Top, Half::Bottom] {
        for inning in [2, 9] {
            let score = if half == Half::Bottom && inning == 9 {
                [1, 1]
            } else {
                [0, 0]
            };
            for outs in 0..=2 {
                for mask in 1..8 {
                    let before = state(mask, outs, half, inning, score);
                    for origin in 0..3 {
                        if mask & (1 << origin) == 0 {
                            continue;
                        }
                        let runner = before.bases[origin].as_ref().unwrap();
                        for caught in [false, true] {
                            let decision = RunnerDecision::Pickoff {
                                origin: [Base::First, Base::Second, Base::Third][origin],
                                caught,
                            };
                            let transition = checked(&before, &decision);
                            assert_eq!(
                                transition.event().kind,
                                EventKind::RunnerOnly(RunnerEvent::Pickoff)
                            );
                            assert_runner_moves(
                                &transition,
                                &before,
                                origin,
                                if caught {
                                    Destination::Out
                                } else {
                                    destination([Base::First, Base::Second, Base::Third][origin])
                                },
                            );
                            assert_eq!(transition.event().delta.outs, u8::from(caught));
                            assert_eq!(transition.event().delta.runs, 0);
                            let count = |kind| {
                                transition
                                    .event()
                                    .delta
                                    .credits
                                    .iter()
                                    .filter(|c| c.kind == kind)
                                    .count()
                            };
                            assert_eq!(count(CreditKind::Putout), usize::from(caught));
                            assert_eq!(count(CreditKind::Assist), usize::from(caught));
                            assert_eq!(count(CreditKind::CaughtStealing), 0);
                            assert_eq!(count(CreditKind::StolenBase), 0);
                            if caught {
                                let assignments = fielders(&before);
                                assert!(has_credit(
                                    &transition,
                                    &assignments[2 + origin],
                                    CreditKind::Putout
                                ));
                                assert!(has_credit(
                                    &transition,
                                    &assignments[0],
                                    CreditKind::Assist
                                ));
                            }
                            assert_eq!(transition.after().score, before.score);
                            if caught && outs == 2 {
                                assert_eq!(
                                    transition
                                        .event()
                                        .delta
                                        .third_out
                                        .as_ref()
                                        .unwrap()
                                        .runner_id,
                                    runner.player_id
                                );
                                assert_eq!(transition.after().outs, 0);
                                assert!(transition.after().bases.iter().all(Option::is_none));
                            }
                            frames += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(frames, 288);
}

fn check_delivery(kind: RunnerEvent, mask: u8, outs: u8, half: Half, inning: u16) {
    let score = if half == Half::Bottom && inning >= 9 {
        [1, 1]
    } else {
        [0, 0]
    };
    let before = state(mask, outs, half, inning, score);
    let decision = if kind == RunnerEvent::WildPitch {
        RunnerDecision::WildPitch
    } else {
        RunnerDecision::PassedBall
    };
    let transition = checked(&before, &decision);
    assert_eq!(transition.event().kind, EventKind::RunnerOnly(kind));
    assert_eq!(transition.event().delta.outs, 0);
    assert_eq!(transition.event().delta.runs, u8::from(mask & 4 != 0));
    assert_eq!(
        transition.event().delta.moves.len(),
        mask.count_ones() as usize
    );
    for (movement, (index, runner)) in transition.event().delta.moves.iter().zip(
        before
            .bases
            .iter()
            .enumerate()
            .filter_map(|(index, runner)| runner.as_ref().map(|runner| (index, runner))),
    ) {
        assert_eq!(&movement.runner, runner);
        assert_eq!(
            movement.to,
            [Destination::Second, Destination::Third, Destination::Home][index]
        );
    }
    assert_eq!(transition.after().teams, before.teams);
    assert_eq!(
        transition.after().score[half.offense()],
        score[half.offense()] + u16::from(mask & 4 != 0)
    );
    let count = |credit| {
        transition
            .event()
            .delta
            .credits
            .iter()
            .filter(|c| c.kind == credit)
            .count()
    };
    assert_eq!(count(CreditKind::Run), usize::from(mask & 4 != 0));
    assert_eq!(
        count(CreditKind::WildPitch),
        usize::from(kind == RunnerEvent::WildPitch)
    );
    assert_eq!(
        count(CreditKind::PassedBall),
        usize::from(kind == RunnerEvent::PassedBall)
    );
    let assignments = fielders(&before);
    let handling_credit = if kind == RunnerEvent::WildPitch {
        (assignments[0].as_str(), CreditKind::WildPitch)
    } else {
        (assignments[1].as_str(), CreditKind::PassedBall)
    };
    assert!(has_credit(
        &transition,
        handling_credit.0,
        handling_credit.1
    ));
    if kind == RunnerEvent::PassedBall {
        assert_eq!(
            transition.event().fielding_review,
            bbsim_engine::FieldingReview::Unknown
        );
    }
    if mask & 4 != 0 {
        assert!(has_credit(
            &transition,
            &before.bases[2].as_ref().unwrap().player_id,
            CreditKind::Run
        ));
    }
    assert_eq!(
        transition.after().over,
        half == Half::Bottom && inning >= 9 && mask & 4 != 0
    );
}

#[test]
fn wild_pitch_and_passed_ball_advance_all_occupied_runners_together() {
    let mut frames = 0;
    for kind in [RunnerEvent::WildPitch, RunnerEvent::PassedBall] {
        for half in [Half::Top, Half::Bottom] {
            for inning in [2, 9] {
                for outs in 0..=2 {
                    for mask in 1..8 {
                        check_delivery(kind, mask, outs, half, inning);
                        frames += 1;
                    }
                }
            }
        }
    }
    assert_eq!(frames, 168);
}

#[test]
fn balks_apply_minimum_progress_for_every_nonempty_mask_and_walk_off_when_due() {
    let mut frames = 0;
    for half in [Half::Top, Half::Bottom] {
        for inning in [2, 9] {
            for outs in 0..=2 {
                for mask in 1..8 {
                    let score = if half == Half::Bottom && inning == 9 {
                        [1, 1]
                    } else {
                        [0, 0]
                    };
                    let before = state(mask, outs, half, inning, score);
                    let transition = checked(&before, &RunnerDecision::Balk);
                    assert_eq!(
                        transition.event().kind,
                        EventKind::RunnerOnly(RunnerEvent::Balk)
                    );
                    assert_eq!(transition.event().delta.outs, 0);
                    assert_eq!(transition.event().delta.runs, u8::from(mask & 4 != 0));
                    assert_eq!(
                        transition.event().delta.moves.len(),
                        mask.count_ones() as usize
                    );
                    for (movement, (index, runner)) in transition.event().delta.moves.iter().zip(
                        before
                            .bases
                            .iter()
                            .enumerate()
                            .filter_map(|(index, runner)| {
                                runner.as_ref().map(|runner| (index, runner))
                            }),
                    ) {
                        assert_eq!(&movement.runner, runner);
                        assert_eq!(
                            movement.to,
                            [Destination::Second, Destination::Third, Destination::Home][index]
                        );
                    }
                    let count = |kind| {
                        transition
                            .event()
                            .delta
                            .credits
                            .iter()
                            .filter(|c| c.kind == kind)
                            .count()
                    };
                    assert_eq!(count(CreditKind::Balk), 1);
                    assert_eq!(count(CreditKind::Run), usize::from(mask & 4 != 0));
                    let assignments = fielders(&before);
                    assert!(has_credit(&transition, &assignments[0], CreditKind::Balk));
                    if mask & 4 != 0 {
                        assert!(has_credit(
                            &transition,
                            &before.bases[2].as_ref().unwrap().player_id,
                            CreditKind::Run
                        ));
                    }
                    assert_eq!(
                        transition.after().over,
                        half == Half::Bottom && inning == 9 && mask & 4 != 0
                    );
                    frames += 1;
                }
            }
        }
    }
    assert_eq!(frames, 84);
}

#[test]
fn runner_only_transitions_chain_without_touching_the_pa_cursor() {
    let before = state(0b101, 0, Half::Top, 3, [0, 0]);
    let first = checked(
        &before,
        &RunnerDecision::Steal {
            origin: Base::First,
            target: Destination::Second,
            caught: false,
        },
    );
    let second = project(
        first.after(),
        &fielders(first.after()),
        &RunnerDecision::Pickoff {
            origin: Base::Second,
            caught: false,
        },
    )
    .unwrap();
    assert!(second.follows(&first).is_ok());
    assert_eq!(second.after().teams, first.after().teams);
    assert_eq!(
        second.after().teams[0].next_batter,
        before.teams[0].next_batter
    );
}

#[test]
fn caught_runner_events_handle_top_ninth_ends_and_bottom_ninth_extra_innings() {
    let top = state(1, 2, Half::Top, 9, [0, 1]);
    let ended = checked(
        &top,
        &RunnerDecision::Pickoff {
            origin: Base::First,
            caught: true,
        },
    );
    assert!(ended.after().over);

    let tied = state(0b100, 2, Half::Bottom, 9, [1, 1]);
    let extra = checked(
        &tied,
        &RunnerDecision::Steal {
            origin: Base::Third,
            target: Destination::Home,
            caught: true,
        },
    );
    assert!(!extra.after().over);
    assert_eq!(
        (extra.after().half, extra.after().inning, extra.after().outs),
        (Half::Top, 10, 0)
    );

    let losing = state(0b100, 2, Half::Bottom, 9, [1, 0]);
    let ended = checked(
        &losing,
        &RunnerDecision::Pickoff {
            origin: Base::Third,
            caught: true,
        },
    );
    assert!(ended.after().over);
}

#[test]
fn invalid_targets_empty_origins_empty_delivery_contexts_and_selected_actors_refuse() {
    let before = state(0b001, 0, Half::Top, 2, [0, 0]);
    for decision in [
        RunnerDecision::Steal {
            origin: Base::Plate,
            target: Destination::First,
            caught: false,
        },
        RunnerDecision::Steal {
            origin: Base::First,
            target: Destination::Third,
            caught: false,
        },
        RunnerDecision::Steal {
            origin: Base::First,
            target: Destination::Home,
            caught: false,
        },
        RunnerDecision::Pickoff {
            origin: Base::Plate,
            caught: false,
        },
        RunnerDecision::Pickoff {
            origin: Base::Third,
            caught: true,
        },
    ] {
        assert!(
            project(&before, &fielders(&before), &decision).is_err(),
            "{decision:?}"
        );
    }
    let occupied = state(0b011, 0, Half::Top, 2, [0, 0]);
    assert!(project(
        &occupied,
        &fielders(&occupied),
        &RunnerDecision::Steal {
            origin: Base::First,
            target: Destination::Second,
            caught: false
        }
    )
    .is_err());
    let empty = state(0, 0, Half::Top, 2, [0, 0]);
    for decision in [
        RunnerDecision::WildPitch,
        RunnerDecision::PassedBall,
        RunnerDecision::Balk,
    ] {
        assert!(project(&empty, &fielders(&empty), &decision).is_err());
    }

    let mut wrong_pitcher = fielders(&before);
    wrong_pitcher[0] = "not-the-active-pitcher".into();
    assert!(project(&before, &wrong_pitcher, &RunnerDecision::Balk).is_err());
    let defense = &before.teams[1 - before.half.offense()];
    for unavailable in [
        defense.bench[0].clone(),
        defense.removed[0].clone(),
        defense.batting_order[8].clone(),
        defense
            .pitcher
            .active_id()
            .expect("legacy fixture pitcher")
            .to_owned(),
        "not-an-active-fielder".into(),
    ] {
        let mut wrong_catcher = fielders(&before);
        wrong_catcher[1] = unavailable;
        assert!(project(&before, &wrong_catcher, &RunnerDecision::PassedBall).is_err());
    }
    let mut wrong_tag_fielder = fielders(&before);
    wrong_tag_fielder[3] = "not-an-active-fielder".into();
    assert!(project(
        &before,
        &wrong_tag_fielder,
        &RunnerDecision::Steal {
            origin: Base::First,
            target: Destination::Second,
            caught: true
        }
    )
    .is_err());
    let mut wrong_pickoff_receiver = fielders(&before);
    wrong_pickoff_receiver[2] = "not-an-active-fielder".into();
    assert!(project(
        &before,
        &wrong_pickoff_receiver,
        &RunnerDecision::Pickoff {
            origin: Base::First,
            caught: true
        }
    )
    .is_err());
}
