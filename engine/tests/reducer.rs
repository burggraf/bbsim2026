use bbsim_engine::{
    apply, Base, Credit, CreditKind, Delta, Destination, Event, EventKind, GameState, Half,
    Participants, PersonnelChange, Runner, RunnerEvent, RunnerMove, TeamSide, TeamState,
};

fn state() -> GameState {
    let mut state = GameState {
        inning: 3,
        half: Half::Bottom,
        outs: 2,
        score: [11, 2],
        teams: [
            TeamState {
                batting_order: std::array::from_fn(|i| format!("away{i}")),
                next_batter: 0,
                pitcher: "away_pitcher".into(),
                dh_slot: Some(8),
                dh_must_bat: false,
                bench: vec!["reliever".into(), "pinch".into()],
                removed: vec![],
            },
            TeamState {
                batting_order: std::array::from_fn(|i| format!("home{i}")),
                next_batter: 0,
                pitcher: "home_pitcher".into(),
                dh_slot: Some(8),
                dh_must_bat: false,
                bench: vec!["reliever".into(), "pinch".into()],
                removed: vec![],
            },
        ],
        bases: [
            Some(Runner {
                player_id: "runner_a".into(),
                responsible_pitcher: "away_pitcher".into(),
            }),
            Some(Runner {
                player_id: "runner_b".into(),
                responsible_pitcher: "away_pitcher".into(),
            }),
            None,
        ],
        over: false,
    };
    state.teams[1].batting_order[7] = "runner_a".into();
    state.teams[1].batting_order[6] = "runner_b".into();
    state
}

#[test]
fn derives_f02_runner_only_wild_pitch_without_advancing_batter() {
    let before = state();
    let event = Event {
        kind: EventKind::RunnerOnly(RunnerEvent::WildPitch),
        participants: Participants {
            batter: Some("home0".into()),
            pitcher: "away_pitcher".into(),
        },
        delta: Delta {
            moves: vec![
                RunnerMove {
                    runner: before.bases[0].clone().unwrap(),
                    from: Base::First,
                    to: Destination::Second,
                },
                RunnerMove {
                    runner: before.bases[1].clone().unwrap(),
                    from: Base::Second,
                    to: Destination::Third,
                },
            ],
            outs: 0,
            runs: 0,
            credits: vec![],
            changes: vec![],
            third_out: None,
        },
        fielding_review: bbsim_engine::FieldingReview::Unknown,
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        ordinary_ground_throw: None,
    };
    let transition = apply(&before, event).unwrap();
    let mut expected = before.clone();
    expected.bases = [None, before.bases[0].clone(), before.bases[1].clone()];
    assert_eq!(transition.before(), &before);
    assert_eq!(transition.after(), &expected);
    assert_eq!(before.bases[0].as_ref().unwrap().player_id, "runner_a");
}

fn batter(before: &GameState) -> Runner {
    Runner {
        player_id: "home0".into(),
        responsible_pitcher: before.teams[0].pitcher.clone(),
    }
}

fn pa(
    before: &GameState,
    moves: Vec<RunnerMove>,
    outs: u8,
    runs: u8,
    mut credits: Vec<Credit>,
) -> Event {
    credits.push(Credit {
        player_id: "home0".into(),
        kind: CreditKind::PlateAppearance,
    });
    Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some("home0".into()),
            pitcher: before.teams[0].pitcher.clone(),
        },
        delta: Delta {
            moves,
            outs,
            runs,
            credits,
            changes: vec![],
            third_out: None,
        },
        fielding_review: bbsim_engine::FieldingReview::Unknown,
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        ordinary_ground_throw: None,
    }
}

#[test]
fn strikeout_with_steal_is_one_atomic_plate_appearance() {
    let mut before = state();
    before.outs = 1;
    before.bases[1] = None;
    let event = pa(
        &before,
        vec![
            RunnerMove {
                runner: batter(&before),
                from: Base::Plate,
                to: Destination::Out,
            },
            RunnerMove {
                runner: before.bases[0].clone().unwrap(),
                from: Base::First,
                to: Destination::Second,
            },
        ],
        1,
        0,
        vec![
            Credit {
                player_id: "home0".into(),
                kind: CreditKind::Strikeout,
            },
            Credit {
                player_id: "runner_a".into(),
                kind: CreditKind::StolenBase,
            },
        ],
    );
    let next = apply(&before, event).unwrap();
    assert_eq!(next.after().outs, 2);
    assert_eq!(next.after().teams[1].next_batter, 1);
    assert_eq!(next.after().bases[1], before.bases[0]);
}

#[test]
fn caught_stealing_third_out_clears_stranded_runners() {
    let before = state();
    let event = Event {
        kind: EventKind::RunnerOnly(RunnerEvent::StealAttempt),
        participants: Participants {
            batter: Some("home0".into()),
            pitcher: "away_pitcher".into(),
        },
        delta: Delta {
            moves: vec![RunnerMove {
                runner: before.bases[1].clone().unwrap(),
                from: Base::Second,
                to: Destination::Out,
            }],
            outs: 1,
            runs: 0,
            credits: vec![Credit {
                player_id: "runner_b".into(),
                kind: CreditKind::CaughtStealing,
            }],
            changes: vec![],
            third_out: None,
        },
        fielding_review: bbsim_engine::FieldingReview::Unknown,
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        ordinary_ground_throw: None,
    };
    let next = apply(&before, event).unwrap();
    assert_eq!(
        (next.after().inning, next.after().half, next.after().outs),
        (4, Half::Top, 0)
    );
    assert_eq!(next.after().bases, [None, None, None]);
    assert_eq!(next.event().delta.outs, 1);
    assert_eq!(next.after().teams[1].next_batter, 0);
}

#[test]
fn bases_loaded_walk_forces_a_run_without_loss_of_identity() {
    let mut before = state();
    before.outs = 1;
    before.teams[1].batting_order[5] = "runner_c".into();
    before.bases[2] = Some(Runner {
        player_id: "runner_c".into(),
        responsible_pitcher: "older_pitcher".into(),
    });
    let event = pa(
        &before,
        vec![
            RunnerMove {
                runner: before.bases[2].clone().unwrap(),
                from: Base::Third,
                to: Destination::Home,
            },
            RunnerMove {
                runner: before.bases[1].clone().unwrap(),
                from: Base::Second,
                to: Destination::Third,
            },
            RunnerMove {
                runner: before.bases[0].clone().unwrap(),
                from: Base::First,
                to: Destination::Second,
            },
            RunnerMove {
                runner: batter(&before),
                from: Base::Plate,
                to: Destination::First,
            },
        ],
        0,
        1,
        vec![
            Credit {
                player_id: "home0".into(),
                kind: CreditKind::Walk,
            },
            Credit {
                player_id: "runner_c".into(),
                kind: CreditKind::Run,
            },
        ],
    );
    let next = apply(&before, event).unwrap();
    assert_eq!(next.after().score, [11, 3]);
    assert_eq!(next.after().bases[0].as_ref().unwrap().player_id, "home0");
    assert_eq!(next.after().bases[1], before.bases[0]);
    assert_eq!(next.after().bases[2], before.bases[1]);
    assert_eq!(
        next.event().delta.moves[0].runner.responsible_pitcher,
        "older_pitcher"
    );
}

#[test]
fn multi_out_play_counts_batter_and_runner_outs_once_each() {
    let mut before = state();
    before.outs = 0;
    before.bases[1] = None;
    let event = pa(
        &before,
        vec![
            RunnerMove {
                runner: batter(&before),
                from: Base::Plate,
                to: Destination::Out,
            },
            RunnerMove {
                runner: before.bases[0].clone().unwrap(),
                from: Base::First,
                to: Destination::Out,
            },
        ],
        2,
        0,
        vec![],
    );
    let next = apply(&before, event).unwrap();
    assert_eq!(next.after().outs, 2);
    assert_eq!(next.after().bases, [None, None, None]);
}

#[test]
fn substitution_changes_pitcher_and_inherits_pinch_runner_responsibility() {
    let before = state();
    let event = Event {
        kind: EventKind::Substitution,
        participants: Participants {
            batter: Some("home0".into()),
            pitcher: "away_pitcher".into(),
        },
        delta: Delta {
            moves: vec![],
            outs: 0,
            runs: 0,
            credits: vec![],
            changes: vec![
                PersonnelChange::Pitcher {
                    team: TeamSide::Away,
                    outgoing: "away_pitcher".into(),
                    incoming: "reliever".into(),
                },
                PersonnelChange::PinchRunner {
                    base: Base::First,
                    outgoing: before.bases[0].clone().unwrap(),
                    incoming: Runner {
                        player_id: "pinch".into(),
                        responsible_pitcher: "away_pitcher".into(),
                    },
                },
            ],
            third_out: None,
        },
        fielding_review: bbsim_engine::FieldingReview::Unknown,
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        ordinary_ground_throw: None,
    };
    let next = apply(&before, event).unwrap();
    assert_eq!(next.after().teams[0].pitcher, "reliever");
    assert_eq!(
        next.after().bases[0].as_ref().unwrap().responsible_pitcher,
        "away_pitcher"
    );
    assert_eq!(next.after().teams[1].next_batter, 0);
}

#[test]
fn illegal_collision_does_not_mutate_caller_state() {
    let before = state();
    let original = before.clone();
    let event = Event {
        kind: EventKind::RunnerOnly(RunnerEvent::WildPitch),
        participants: Participants {
            batter: Some("home0".into()),
            pitcher: "away_pitcher".into(),
        },
        delta: Delta {
            moves: vec![RunnerMove {
                runner: before.bases[0].clone().unwrap(),
                from: Base::First,
                to: Destination::Second,
            }],
            outs: 0,
            runs: 0,
            credits: vec![],
            changes: vec![],
            third_out: None,
        },
        fielding_review: bbsim_engine::FieldingReview::Unknown,
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        ordinary_ground_throw: None,
    };
    assert!(apply(&before, event).is_err());
    assert_eq!(before, original);
}

#[test]
fn malformed_lineup_cursor_returns_error_instead_of_panicking() {
    let mut before = state();
    before.teams[1].next_batter = u8::MAX;
    let event = pa(
        &before,
        vec![RunnerMove {
            runner: batter(&before),
            from: Base::Plate,
            to: Destination::Out,
        }],
        1,
        0,
        vec![],
    );
    assert!(apply(&before, event).is_err());
}

#[test]
fn two_runners_score_on_one_plate_appearance() {
    let mut before = state();
    before.outs = 1;
    before.bases[0] = None;
    before.teams[1].batting_order[5] = "runner_c".into();
    before.bases[2] = Some(Runner {
        player_id: "runner_c".into(),
        responsible_pitcher: "prior_pitcher".into(),
    });
    let event = pa(
        &before,
        vec![
            RunnerMove {
                runner: batter(&before),
                from: Base::Plate,
                to: Destination::Second,
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
        ],
        0,
        2,
        vec![
            Credit {
                player_id: "home0".into(),
                kind: CreditKind::Hit,
            },
            Credit {
                player_id: "runner_b".into(),
                kind: CreditKind::Run,
            },
            Credit {
                player_id: "runner_c".into(),
                kind: CreditKind::Run,
            },
        ],
    );
    let next = apply(&before, event).unwrap();
    assert_eq!(next.after().score, [11, 4]);
    assert_eq!(next.after().bases[1].as_ref().unwrap().player_id, "home0");
    assert_eq!(next.event().delta.runs, 2);
    assert_eq!(
        next.event().delta.moves[2].runner.responsible_pitcher,
        "prior_pitcher"
    );
}

#[test]
fn pickoff_follows_prior_event_without_plate_appearance() {
    let mut before = state();
    before.outs = 1;
    before.bases[1] = None;
    let first = pa(
        &before,
        vec![
            RunnerMove {
                runner: batter(&before),
                from: Base::Plate,
                to: Destination::First,
            },
            RunnerMove {
                runner: before.bases[0].clone().unwrap(),
                from: Base::First,
                to: Destination::Second,
            },
        ],
        0,
        0,
        vec![Credit {
            player_id: "home0".into(),
            kind: CreditKind::Walk,
        }],
    );
    let prior = apply(&before, first).unwrap();
    let runner = prior.after().bases[1].clone().unwrap();
    let event = Event {
        kind: EventKind::RunnerOnly(RunnerEvent::Pickoff),
        participants: Participants {
            batter: Some("home1".into()),
            pitcher: "away_pitcher".into(),
        },
        delta: Delta {
            moves: vec![RunnerMove {
                runner,
                from: Base::Second,
                to: Destination::Out,
            }],
            outs: 1,
            runs: 0,
            credits: vec![],
            changes: vec![],
            third_out: None,
        },
        fielding_review: bbsim_engine::FieldingReview::Unknown,
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        ordinary_ground_throw: None,
    };
    let next = apply(prior.after(), event).unwrap();
    assert!(next.follows(&prior).is_ok());
    assert_eq!(next.after().teams[1].next_batter, 1);
    assert_eq!(next.after().outs, 2);
}

#[test]
fn all_base_masks_preserve_or_strand_runners_at_the_third_out() {
    for outs in 0..=2 {
        for mask in 0..8u8 {
            let mut before = state();
            before.outs = outs;
            for base in 0..3 {
                if mask & (1 << base) != 0 {
                    before.teams[1].batting_order[5 + base] = format!("runner_{base}");
                }
                before.bases[base] = (mask & (1 << base) != 0).then(|| Runner {
                    player_id: format!("runner_{base}"),
                    responsible_pitcher: "prior_pitcher".into(),
                });
            }
            let event = pa(
                &before,
                vec![RunnerMove {
                    runner: batter(&before),
                    from: Base::Plate,
                    to: Destination::Out,
                }],
                1,
                0,
                vec![],
            );
            let next = apply(&before, event).unwrap();
            assert_eq!(next.event().delta.outs, 1, "outs={outs}, mask={mask}");
            if outs == 2 {
                assert_eq!(next.after().bases, [None, None, None], "mask={mask}");
                assert_eq!(
                    (next.after().inning, next.after().half, next.after().outs),
                    (4, Half::Top, 0)
                );
            } else {
                assert_eq!(next.after().bases, before.bases, "outs={outs}, mask={mask}");
                assert_eq!(next.after().outs, outs + 1);
            }
        }
    }
}

#[test]
fn maximal_inning_without_boundary_does_not_require_another_inning() {
    let mut before = state();
    before.inning = u16::MAX;
    before.outs = 0;
    let event = pa(
        &before,
        vec![RunnerMove {
            runner: batter(&before),
            from: Base::Plate,
            to: Destination::Out,
        }],
        1,
        0,
        vec![],
    );
    let next = apply(&before, event).unwrap();
    assert_eq!(
        (next.after().inning, next.after().half, next.after().outs),
        (u16::MAX, Half::Bottom, 1)
    );
}
