use bbsim_engine::{
    apply, Base, Credit, CreditKind, Delta, Destination, Event, EventKind, EventTransition,
    GameState, Half, Participants, Runner, RunnerMove, TeamState, ThirdOut, ThirdOutKind,
    TransitionError,
};

fn state(inning: u16, half: Half, outs: u8, score: [u16; 2]) -> GameState {
    GameState {
        inning,
        half,
        outs,
        score,
        over: false,
        teams: [
            TeamState {
                batting_order: std::array::from_fn(|i| match i {
                    5 => "r1".into(),
                    6 => "r2".into(),
                    7 => "r3".into(),
                    _ => format!("away{i}"),
                }),
                next_batter: 0,
                pitcher: "away_pitcher".into(),
                dh_slot: Some(8),
                dh_must_bat: false,
                bench: vec!["reliever".into(), "pinch".into()],
                removed: vec![],
            },
            TeamState {
                batting_order: std::array::from_fn(|i| match i {
                    5 => "r1".into(),
                    6 => "r2".into(),
                    7 => "r3".into(),
                    _ => format!("home{i}"),
                }),
                next_batter: 0,
                pitcher: "home_pitcher".into(),
                dh_slot: Some(8),
                dh_must_bat: false,
                bench: vec!["reliever".into(), "pinch".into()],
                removed: vec![],
            },
        ],
        bases: [None, None, None],
    }
}

fn batter_event(
    before: &GameState,
    runner_moves: Vec<RunnerMove>,
    outs: u8,
    runs: u8,
    scoring: &[&str],
) -> Event {
    let offense = before.half.offense();
    let batter =
        before.teams[offense].batting_order[before.teams[offense].next_batter as usize].clone();
    let pitcher = before.teams[1 - offense].pitcher.clone();
    let mut credits = vec![Credit {
        player_id: batter.clone(),
        kind: CreditKind::PlateAppearance,
    }];
    for &id in scoring {
        credits.push(Credit {
            player_id: id.into(),
            kind: CreditKind::Run,
        });
    }
    Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(batter),
            pitcher,
        },
        delta: Delta {
            moves: runner_moves,
            outs,
            runs,
            credits,
            changes: vec![],
            third_out: None,
        },
        fielding_review: bbsim_engine::FieldingReview::Unknown,
    }
}

fn batter_out(before: &GameState) -> RunnerMove {
    let offense = before.half.offense();
    RunnerMove {
        runner: Runner {
            player_id: before.teams[offense].batting_order
                [before.teams[offense].next_batter as usize]
                .clone(),
            responsible_pitcher: before.teams[1 - offense].pitcher.clone(),
        },
        from: Base::Plate,
        to: Destination::Out,
    }
}

#[test]
fn home_lead_after_top_ninth_ends_without_bottom_ninth() {
    let before = state(9, Half::Top, 2, [3, 4]);
    let next = apply(
        &before,
        batter_event(&before, vec![batter_out(&before)], 1, 0, &[]),
    )
    .unwrap();
    assert!(next.after().over);
    assert_eq!(
        (next.after().inning, next.after().half, next.after().outs),
        (9, Half::Top, 0)
    );
    assert_eq!(next.event().delta.outs, 1);
    assert_eq!(next.after().bases, [None, None, None]);
    assert_eq!(EventTransition::from_bytes(&next.to_bytes()).unwrap(), next);
    let mut forged = next.after().clone();
    forged.over = false;
    forged.half = Half::Bottom;
    assert!(EventTransition::try_new(before.clone(), next.event().clone(), forged).is_err());
    let continued = batter_event(next.after(), vec![batter_out(next.after())], 1, 0, &[]);
    assert!(apply(next.after(), continued).is_err());
}

#[test]
fn away_lead_after_completed_ninth_ends_game() {
    let before = state(9, Half::Bottom, 2, [5, 4]);
    let next = apply(
        &before,
        batter_event(&before, vec![batter_out(&before)], 1, 0, &[]),
    )
    .unwrap();
    assert!(next.after().over);
    assert_eq!(
        (next.after().inning, next.after().half, next.after().outs),
        (9, Half::Bottom, 0)
    );
}

#[test]
fn tied_ninth_continues_to_empty_extra_inning() {
    let top = state(9, Half::Top, 2, [4, 4]);
    let bottom = apply(&top, batter_event(&top, vec![batter_out(&top)], 1, 0, &[])).unwrap();
    assert!(!bottom.after().over);
    assert_eq!(
        (
            bottom.after().inning,
            bottom.after().half,
            bottom.after().outs
        ),
        (9, Half::Bottom, 0)
    );
    let mut last = bottom.after().clone();
    last.outs = 2;
    let extra = apply(
        &last,
        batter_event(&last, vec![batter_out(&last)], 1, 0, &[]),
    )
    .unwrap();
    assert!(!extra.after().over);
    assert_eq!(
        (extra.after().inning, extra.after().half, extra.after().outs),
        (10, Half::Top, 0)
    );
    assert_eq!(extra.after().bases, [None, None, None]);
}

#[test]
fn winning_run_in_bottom_ninth_ends_immediately() {
    let mut before = state(9, Half::Bottom, 1, [4, 4]);
    before.teams[1].batting_order[7] = "runner_on_third".into();
    before.bases[2] = Some(Runner {
        player_id: "runner_on_third".into(),
        responsible_pitcher: "prior_pitcher".into(),
    });
    let moves = vec![
        batter_out(&before),
        RunnerMove {
            runner: before.bases[2].clone().unwrap(),
            from: Base::Third,
            to: Destination::Home,
        },
    ];
    let next = apply(
        &before,
        batter_event(&before, moves, 1, 1, &["runner_on_third"]),
    )
    .unwrap();
    assert!(next.after().over);
    assert_eq!(
        (next.after().inning, next.after().half, next.after().outs),
        (9, Half::Bottom, 2)
    );
    assert_eq!(next.after().score, [4, 5]);
}

#[test]
fn ambiguous_extra_walkoff_runs_need_an_explicit_home_run_or_timing() {
    let mut before = state(9, Half::Bottom, 0, [4, 4]);
    before.bases[1] = Some(Runner {
        player_id: "r2".into(),
        responsible_pitcher: "prior_pitcher".into(),
    });
    before.bases[2] = Some(Runner {
        player_id: "r3".into(),
        responsible_pitcher: "prior_pitcher".into(),
    });
    let batter = Runner {
        player_id: "home0".into(),
        responsible_pitcher: "away_pitcher".into(),
    };
    let moves = vec![
        RunnerMove {
            runner: batter,
            from: Base::Plate,
            to: Destination::First,
        },
        RunnerMove {
            runner: before.bases[1].clone().unwrap(),
            from: Base::Second,
            to: Destination::Home,
        },
        RunnerMove {
            runner: before.bases[2].clone().unwrap(),
            from: Base::Third,
            to: Destination::Home,
        },
    ];
    assert!(matches!(
        apply(&before, batter_event(&before, moves, 0, 2, &["r2", "r3"])),
        Err(TransitionError::UnsupportedRule)
    ));
}

#[test]
fn third_out_run_without_out_reason_and_timing_is_not_credited() {
    let mut before = state(8, Half::Bottom, 2, [4, 4]);
    before.bases[2] = Some(Runner {
        player_id: "r3".into(),
        responsible_pitcher: "prior_pitcher".into(),
    });
    let moves = vec![
        batter_out(&before),
        RunnerMove {
            runner: before.bases[2].clone().unwrap(),
            from: Base::Third,
            to: Destination::Home,
        },
    ];
    assert!(matches!(
        apply(&before, batter_event(&before, moves, 1, 1, &["r3"])),
        Err(TransitionError::UnsupportedRule)
    ));
}

#[test]
fn run_credit_must_identify_the_runner_who_reached_home() {
    let mut before = state(8, Half::Bottom, 1, [4, 4]);
    before.bases[2] = Some(Runner {
        player_id: "r3".into(),
        responsible_pitcher: "prior_pitcher".into(),
    });
    let moves = vec![
        batter_out(&before),
        RunnerMove {
            runner: before.bases[2].clone().unwrap(),
            from: Base::Third,
            to: Destination::Home,
        },
    ];
    assert!(apply(&before, batter_event(&before, moves, 1, 1, &["not_r3"])).is_err());
}

fn on_base(id: &str) -> Runner {
    Runner {
        player_id: id.into(),
        responsible_pitcher: "prior_pitcher".into(),
    }
}
fn batter_reaches_first(before: &GameState) -> RunnerMove {
    let mut movement = batter_out(before);
    movement.to = Destination::First;
    movement
}
fn to_home(before: &GameState, index: usize, from: Base) -> RunnerMove {
    RunnerMove {
        runner: before.bases[index].clone().unwrap(),
        from,
        to: Destination::Home,
    }
}

#[test]
fn force_third_out_voids_runner_who_physically_crossed_home() {
    let mut before = state(8, Half::Bottom, 2, [4, 4]);
    before.bases[0] = Some(on_base("r1"));
    before.bases[2] = Some(on_base("r3"));
    let moves = vec![
        batter_reaches_first(&before),
        RunnerMove {
            runner: before.bases[0].clone().unwrap(),
            from: Base::First,
            to: Destination::Out,
        },
        to_home(&before, 2, Base::Third),
    ];
    let mut event = batter_event(&before, moves, 1, 0, &[]);
    event.delta.third_out = Some(ThirdOut {
        runner_id: "r1".into(),
        kind: ThirdOutKind::Force,
        home_before_out: vec!["r3".into()],
    });
    let next = apply(&before, event).unwrap();
    assert_eq!(next.after().score, [4, 4]);
    assert_eq!(
        (next.after().inning, next.after().half, next.after().outs),
        (9, Half::Top, 0)
    );
    assert_eq!(next.event().delta.moves[2].to, Destination::Home);
    assert_eq!(EventTransition::from_bytes(&next.to_bytes()).unwrap(), next);
}

#[test]
fn batter_before_first_third_out_voids_home_crossing() {
    let mut before = state(8, Half::Bottom, 2, [4, 4]);
    before.bases[2] = Some(on_base("r3"));
    let moves = vec![batter_out(&before), to_home(&before, 2, Base::Third)];
    let mut event = batter_event(&before, moves, 1, 0, &[]);
    event.delta.third_out = Some(ThirdOut {
        runner_id: "home0".into(),
        kind: ThirdOutKind::BatterBeforeFirst,
        home_before_out: vec!["r3".into()],
    });
    let next = apply(&before, event).unwrap();
    assert_eq!(next.after().score, [4, 4]);
}

#[test]
fn nonforce_third_out_counts_only_runs_before_the_tag() {
    let mut before = state(8, Half::Bottom, 2, [4, 4]);
    before.bases[1] = Some(on_base("r2"));
    before.bases[2] = Some(on_base("r3"));
    let moves = vec![
        batter_reaches_first(&before),
        RunnerMove {
            runner: before.bases[1].clone().unwrap(),
            from: Base::Second,
            to: Destination::Out,
        },
        to_home(&before, 2, Base::Third),
    ];
    let mut event = batter_event(&before, moves, 1, 1, &["r3"]);
    event.delta.third_out = Some(ThirdOut {
        runner_id: "r2".into(),
        kind: ThirdOutKind::Timing,
        home_before_out: vec!["r3".into()],
    });
    let mut forged = event.clone();
    forged.delta.third_out.as_mut().unwrap().kind = ThirdOutKind::BatterAfterFirstTiming;
    assert!(matches!(
        apply(&before, forged),
        Err(TransitionError::DeltaMismatch)
    ));
    let next = apply(&before, event).unwrap();
    assert_eq!(next.after().score, [4, 5]);
    assert_eq!(next.event().delta.runs, 1);
}

#[test]
fn preceding_runner_appeal_voids_following_runners_only() {
    let mut before = state(8, Half::Bottom, 2, [4, 4]);
    before.bases = [
        Some(on_base("r1")),
        Some(on_base("r2")),
        Some(on_base("r3")),
    ];
    let moves = vec![
        batter_reaches_first(&before),
        to_home(&before, 0, Base::First),
        RunnerMove {
            runner: before.bases[1].clone().unwrap(),
            from: Base::Second,
            to: Destination::Out,
        },
        to_home(&before, 2, Base::Third),
    ];
    let mut event = batter_event(&before, moves, 1, 1, &["r3"]);
    event.delta.third_out = Some(ThirdOut {
        runner_id: "r2".into(),
        kind: ThirdOutKind::PrecedingRunnerAppeal,
        home_before_out: vec!["r1".into(), "r3".into()],
    });
    let next = apply(&before, event).unwrap();
    assert_eq!(next.after().score, [4, 5]);
    assert_eq!(
        next.event()
            .delta
            .moves
            .iter()
            .filter(|m| m.to == Destination::Home)
            .count(),
        2
    );
}

#[test]
fn force_label_cannot_override_batter_retired_before_first() {
    let mut before = state(8, Half::Bottom, 1, [4, 4]);
    before.bases[0] = Some(on_base("r1"));
    before.bases[2] = Some(on_base("r3"));
    let moves = vec![
        batter_out(&before),
        RunnerMove {
            runner: before.bases[0].clone().unwrap(),
            from: Base::First,
            to: Destination::Out,
        },
        to_home(&before, 2, Base::Third),
    ];
    let mut event = batter_event(&before, moves, 2, 0, &[]);
    event.delta.third_out = Some(ThirdOut {
        runner_id: "r1".into(),
        kind: ThirdOutKind::Force,
        home_before_out: vec!["r3".into()],
    });
    assert!(apply(&before, event).is_err());
}

#[test]
fn runner_on_second_is_not_forced_without_runner_on_first() {
    let mut before = state(8, Half::Bottom, 2, [4, 4]);
    before.bases[1] = Some(on_base("r2"));
    before.bases[2] = Some(on_base("r3"));
    let moves = vec![
        batter_reaches_first(&before),
        RunnerMove {
            runner: before.bases[1].clone().unwrap(),
            from: Base::Second,
            to: Destination::Out,
        },
        to_home(&before, 2, Base::Third),
    ];
    let mut event = batter_event(&before, moves, 1, 0, &[]);
    event.delta.third_out = Some(ThirdOut {
        runner_id: "r2".into(),
        kind: ThirdOutKind::Force,
        home_before_out: vec!["r3".into()],
    });
    assert!(apply(&before, event).is_err());
}

#[test]
fn timing_out_of_batter_requires_proof_he_reached_first() {
    let mut before = state(8, Half::Bottom, 2, [4, 4]);
    before.bases[2] = Some(on_base("r3"));
    let moves = vec![batter_out(&before), to_home(&before, 2, Base::Third)];
    let mut event = batter_event(&before, moves, 1, 1, &["r3"]);
    event.delta.third_out = Some(ThirdOut {
        runner_id: "home0".into(),
        kind: ThirdOutKind::Timing,
        home_before_out: vec!["r3".into()],
    });
    assert!(matches!(
        apply(&before, event),
        Err(TransitionError::UnsupportedRule)
    ));
}

#[test]
fn multi_out_force_sequence_needs_ordering_before_voiding_runs() {
    let mut before = state(8, Half::Bottom, 1, [4, 4]);
    before.bases = [
        Some(on_base("r1")),
        Some(on_base("r2")),
        Some(on_base("r3")),
    ];
    let moves = vec![
        batter_reaches_first(&before),
        RunnerMove {
            runner: before.bases[0].clone().unwrap(),
            from: Base::First,
            to: Destination::Out,
        },
        RunnerMove {
            runner: before.bases[1].clone().unwrap(),
            from: Base::Second,
            to: Destination::Out,
        },
        to_home(&before, 2, Base::Third),
    ];
    let mut event = batter_event(&before, moves, 2, 0, &[]);
    event.delta.third_out = Some(ThirdOut {
        runner_id: "r2".into(),
        kind: ThirdOutKind::Force,
        home_before_out: vec!["r3".into()],
    });
    assert!(matches!(
        apply(&before, event),
        Err(TransitionError::UnsupportedRule)
    ));
}

#[test]
fn out_of_park_home_run_scores_all_on_walkoff() {
    let mut before = state(9, Half::Bottom, 1, [4, 4]);
    before.bases[2] = Some(on_base("r3"));
    let moves = vec![
        RunnerMove {
            runner: Runner {
                player_id: "home0".into(),
                responsible_pitcher: "away_pitcher".into(),
            },
            from: Base::Plate,
            to: Destination::Home,
        },
        to_home(&before, 2, Base::Third),
    ];
    let mut event = batter_event(&before, moves, 0, 2, &["r3", "home0"]);
    event.delta.credits.push(Credit {
        player_id: "home0".into(),
        kind: CreditKind::Hit,
    });
    event.kind = EventKind::OutOfParkHomeRun;
    let next = apply(&before, event).unwrap();
    assert!(next.after().over);
    assert_eq!(next.after().score, [4, 6]);
    assert_eq!(EventTransition::from_bytes(&next.to_bytes()).unwrap(), next);
}

#[test]
fn out_of_park_hr_scores_batter_and_every_occupied_base_for_all_masks() {
    for mask in 0u8..8 {
        let mut before = state(9, Half::Bottom, 0, [5, 5]);
        let mut moves = vec![RunnerMove {
            runner: Runner {
                player_id: "home0".into(),
                responsible_pitcher: "away_pitcher".into(),
            },
            from: Base::Plate,
            to: Destination::Home,
        }];
        let mut scorers = vec!["home0"];
        for (index, from) in [(0, Base::First), (1, Base::Second), (2, Base::Third)] {
            if mask & (1 << index) != 0 {
                let id = match index {
                    0 => "r1",
                    1 => "r2",
                    _ => "r3",
                };
                before.bases[index] = Some(on_base(id));
                moves.push(to_home(&before, index, from));
                scorers.push(id);
            }
        }
        let mut event = batter_event(&before, moves, 0, scorers.len() as u8, &scorers);
        event.kind = EventKind::OutOfParkHomeRun;
        event.delta.credits.push(Credit {
            player_id: "home0".into(),
            kind: CreditKind::Hit,
        });
        let next = apply(&before, event).unwrap();
        assert!(next.after().over, "mask {mask}");
        assert_eq!(
            next.after().score,
            [5, 5 + scorers.len() as u16],
            "mask {mask}"
        );
    }
}

#[test]
fn hr_label_does_not_make_a_single_walkoff_run_an_out_of_park_homer() {
    let mut before = state(9, Half::Bottom, 0, [5, 5]);
    before.bases[2] = Some(on_base("r3"));
    let moves = vec![
        batter_reaches_first(&before),
        to_home(&before, 2, Base::Third),
    ];
    let mut event = batter_event(&before, moves, 0, 1, &["r3"]);
    event.kind = EventKind::OutOfParkHomeRun;
    event.delta.credits.push(Credit {
        player_id: "home0".into(),
        kind: CreditKind::Hit,
    });
    assert!(matches!(
        apply(&before, event),
        Err(TransitionError::DeltaMismatch)
    ));
}

#[test]
fn no_dh_requires_the_pitcher_in_the_batting_order() {
    let mut before = state(7, Half::Bottom, 0, [2, 3]);
    before.teams[1].dh_slot = None;
    let event = batter_event(&before, vec![batter_out(&before)], 1, 0, &[]);
    assert!(matches!(
        apply(&before, event),
        Err(TransitionError::InvalidState)
    ));
}

#[test]
fn batter_tagged_after_first_is_a_timing_out_not_before_first() {
    let mut before = state(8, Half::Bottom, 2, [4, 4]);
    before.bases[2] = Some(on_base("r3"));
    let moves = vec![batter_out(&before), to_home(&before, 2, Base::Third)];
    let mut event = batter_event(&before, moves, 1, 1, &["r3"]);
    event.delta.third_out = Some(ThirdOut {
        runner_id: "home0".into(),
        kind: ThirdOutKind::BatterAfterFirstTiming,
        home_before_out: vec!["r3".into()],
    });
    let next = apply(&before, event).unwrap();
    assert_eq!(next.after().score, [4, 5]);
    assert_eq!((next.after().inning, next.after().half), (9, Half::Top));
    assert_eq!(EventTransition::from_bytes(&next.to_bytes()).unwrap(), next);
}
