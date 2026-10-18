use bbsim_engine::{
    Base, Credit, CreditKind, Delta, Destination, Event, EventKind, EventTransition, GameState,
    Half, Participants, PersonnelChange, Runner, RunnerEvent, RunnerMove, TeamSide, TeamState,
    TransitionError,
};

fn team(prefix: &str, pitcher: &str) -> TeamState {
    TeamState {
        defense: bbsim_engine::DefenseState::LegacyUnassigned,
        batting_order: std::array::from_fn(|i| format!("{prefix}{i}")),
        next_batter: 0,
        pitcher: pitcher.into(),
        dh_slot: Some(8),
        dh_must_bat: false,
        bench: vec![
            "reliever".into(),
            "bench_batter".into(),
            "pinch_runner".into(),
        ],
        removed: vec![],
    }
}

fn state() -> GameState {
    let mut home = team("home", "home_pitcher");
    home.batting_order[7] = "runner_a".into();
    home.batting_order[6] = "runner_b".into();
    let mut away = team("away", "away_pitcher");
    away.batting_order[5] = "catcher".into();
    GameState {
        inning: 3,
        half: Half::Bottom,
        outs: 2,
        score: [11, 2],
        teams: [away, home],
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
    }
}

fn wild_pitch_event(before: &GameState) -> Event {
    Event {
        kind: EventKind::RunnerOnly(RunnerEvent::WildPitch),
        participants: Participants {
            batter: Some("home0".into()),
            pitcher: Some("away_pitcher".into()),
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
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        ordinary_air_catch: None,
        muffed_ground_ball: None,
        ordinary_ground_throw: None,
    }
}

#[test]
fn wild_pitch_preserves_batter_order_and_runner_identity() {
    let before = state();
    let mut after = before.clone();
    after.bases = [None, before.bases[0].clone(), before.bases[1].clone()];
    let transition =
        EventTransition::try_new(before.clone(), wild_pitch_event(&before), after.clone()).unwrap();
    assert_eq!(transition.before(), &before);
    assert_eq!(transition.after(), &after);
    assert_eq!(transition.event().delta.outs, 0);
}

#[test]
fn rejects_unchanged_bases_when_moves_claim_advances() {
    let before = state();
    let result = EventTransition::try_new(before.clone(), wild_pitch_event(&before), before);
    assert!(result.is_err());
}

#[test]
fn non_pa_rejects_pa_credit_and_lineup_advance() {
    let before = state();
    let mut after = before.clone();
    after.bases = [None, before.bases[0].clone(), before.bases[1].clone()];
    let mut event = wild_pitch_event(&before);
    event.delta.credits.push(Credit {
        player_id: "home0".into(),
        kind: CreditKind::PlateAppearance,
    });
    assert!(EventTransition::try_new(before.clone(), event, after.clone()).is_err());
    after.teams[1].next_batter = 1;
    assert!(EventTransition::try_new(before.clone(), wild_pitch_event(&before), after).is_err());
}

#[test]
fn strikeout_and_steal_share_a_single_pa_transition() {
    let mut before = state();
    before.outs = 1;
    before.bases[1] = None;
    let mut after = before.clone();
    after.outs = 2;
    after.bases = [None, before.bases[0].clone(), None];
    after.teams[1].next_batter = 1;
    let event = Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some("home0".into()),
            pitcher: Some("away_pitcher".into()),
        },
        delta: Delta {
            moves: vec![
                RunnerMove {
                    runner: Runner {
                        player_id: "home0".into(),
                        responsible_pitcher: "away_pitcher".into(),
                    },
                    from: Base::Plate,
                    to: Destination::Out,
                },
                RunnerMove {
                    runner: before.bases[0].clone().unwrap(),
                    from: Base::First,
                    to: Destination::Second,
                },
            ],
            outs: 1,
            runs: 0,
            credits: vec![
                Credit {
                    player_id: "home0".into(),
                    kind: CreditKind::PlateAppearance,
                },
                Credit {
                    player_id: "home0".into(),
                    kind: CreditKind::Strikeout,
                },
                Credit {
                    player_id: "runner_a".into(),
                    kind: CreditKind::StolenBase,
                },
            ],
            changes: vec![],
            third_out: None,
        },
        fielding_review: bbsim_engine::FieldingReview::Unknown,
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        ordinary_air_catch: None,
        muffed_ground_ball: None,
        ordinary_ground_throw: None,
    };
    assert!(EventTransition::try_new(before, event, after).is_ok());
}

#[test]
fn third_out_retains_play_outs_but_normalizes_next_half() {
    let before = state();
    let mut after = before.clone();
    after.inning = 4;
    after.half = Half::Top;
    after.outs = 0;
    after.bases = [None, None, None];
    let event = Event {
        kind: EventKind::RunnerOnly(RunnerEvent::StealAttempt),
        participants: Participants {
            batter: Some("home0".into()),
            pitcher: Some("away_pitcher".into()),
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
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        ordinary_air_catch: None,
        muffed_ground_ball: None,
        ordinary_ground_throw: None,
    };
    let transition = EventTransition::try_new(before, event, after).unwrap();
    assert_eq!(transition.event().delta.outs, 1);
    assert_eq!(transition.after().outs, 0);
}

#[test]
fn rejects_score_change_without_a_scoring_move() {
    let before = state();
    let mut after = before.clone();
    after.score[1] += 1;
    let mut event = wild_pitch_event(&before);
    event.delta.runs = 1;
    after.bases = [None, before.bases[0].clone(), before.bases[1].clone()];
    assert!(EventTransition::try_new(before, event, after).is_err());
}

#[test]
fn continuity_uses_complete_snapshots() {
    let before = state();
    let mut after = before.clone();
    after.bases = [None, before.bases[0].clone(), before.bases[1].clone()];
    let first = EventTransition::try_new(before.clone(), wild_pitch_event(&before), after).unwrap();
    let different = EventTransition::try_new(
        before.clone(),
        wild_pitch_event(&before),
        first.after().clone(),
    )
    .unwrap();
    assert!(different.follows(&first).is_err());
    let next_before = first.after().clone();
    let mut next_after = next_before.clone();
    next_after.teams[0].pitcher = "reliever".into();
    next_after.teams[0].bench.retain(|id| id != "reliever");
    next_after.teams[0].removed.push("away_pitcher".into());
    let next = EventTransition::try_new(
        next_before,
        Event {
            kind: EventKind::Substitution,
            participants: Participants {
                batter: Some("home0".into()),
                pitcher: Some("away_pitcher".into()),
            },
            delta: Delta {
                moves: vec![],
                outs: 0,
                runs: 0,
                credits: vec![],
                changes: vec![PersonnelChange::Pitcher {
                    team: TeamSide::Away,
                    outgoing: "away_pitcher".into(),
                    incoming: "reliever".into(),
                }],
                third_out: None,
            },
            fielding_review: bbsim_engine::FieldingReview::Unknown,
            two_out_contact: None,
            single_delivery: None,
            balk: None,
            caught_strikeout: None,
            protected_award: None,
            sole_home_run: None,
            sole_safe_hit: None,
            ordinary_air_catch: None,
            muffed_ground_ball: None,
            ordinary_ground_throw: None,
        },
        next_after,
    )
    .unwrap();
    assert!(next.follows(&first).is_ok());
}

#[test]
fn versioned_record_roundtrips_and_rejects_corruption() {
    let before = state();
    let mut after = before.clone();
    after.bases = [None, before.bases[0].clone(), before.bases[1].clone()];
    let transition =
        EventTransition::try_new(before.clone(), wild_pitch_event(&before), after).unwrap();
    let bytes = transition.to_bytes();
    assert!(bytes.starts_with(b"BBE25"));
    assert_eq!(EventTransition::from_bytes(&bytes).unwrap(), transition);
    for earlier_prefix in [
        b"BBE19", b"BBE18", b"BBE17", b"BBE16", b"BBE15", b"BBE14", b"BBE13", b"BBE12", b"BBE11",
    ] {
        let mut earlier = bytes.clone();
        earlier[..5].copy_from_slice(earlier_prefix);
        assert_eq!(
            EventTransition::from_bytes(&earlier),
            Err(TransitionError::InvalidEncoding)
        );
    }
    assert!(EventTransition::from_bytes(&bytes[..bytes.len() - 1]).is_err());
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(EventTransition::from_bytes(&trailing).is_err());
    let mut wrong_version = bytes;
    wrong_version[0] = b'X';
    assert!(EventTransition::from_bytes(&wrong_version).is_err());
}

#[test]
fn rejects_early_or_inconsistent_terminal_state() {
    let before = state();
    let mut after = before.clone();
    after.over = true;
    after.bases = [None, before.bases[0].clone(), before.bases[1].clone()];
    assert!(EventTransition::try_new(before.clone(), wild_pitch_event(&before), after).is_err());
}

#[test]
fn rejects_duplicate_runner_identity_on_two_bases() {
    let mut before = state();
    before.bases[1] = before.bases[0].clone();
    let mut after = before.clone();
    after.bases = [None, before.bases[0].clone(), before.bases[1].clone()];
    assert!(EventTransition::try_new(before.clone(), wild_pitch_event(&before), after).is_err());
}

#[test]
fn substitution_preserves_batting_cursor_and_roundtrips() {
    let before = state();
    let mut after = before.clone();
    after.teams[0].pitcher = "reliever".into();
    after.teams[0].bench.retain(|id| id != "reliever");
    after.teams[0].removed.push("away_pitcher".into());
    let event = Event {
        kind: EventKind::Substitution,
        participants: Participants {
            batter: Some("home0".into()),
            pitcher: Some("away_pitcher".into()),
        },
        delta: Delta {
            moves: vec![],
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
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        ordinary_air_catch: None,
        muffed_ground_ball: None,
        ordinary_ground_throw: None,
    };
    // A personnel change must be named in the delta, not hidden in `after`.
    assert!(EventTransition::try_new(before.clone(), event.clone(), after.clone()).is_err());
    let mut event = event;
    event.delta.changes.push(PersonnelChange::Pitcher {
        team: TeamSide::Away,
        outgoing: "away_pitcher".into(),
        incoming: "reliever".into(),
    });
    let transition = EventTransition::try_new(before, event, after).unwrap();
    assert_eq!(
        EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
        transition
    );
}

#[test]
fn strikeout_passed_ball_can_reach_without_an_out() {
    // F-02 ANA201904040 #96's simultaneous outcome shape.
    let mut before = state();
    before.inning = 9;
    before.outs = 0;
    before.bases = [None, None, None];
    let mut after = before.clone();
    after.teams[1].next_batter = 1;
    after.bases[0] = Some(Runner {
        player_id: "home0".into(),
        responsible_pitcher: "away_pitcher".into(),
    });
    let event = Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some("home0".into()),
            pitcher: Some("away_pitcher".into()),
        },
        delta: Delta {
            moves: vec![RunnerMove {
                runner: after.bases[0].clone().unwrap(),
                from: Base::Plate,
                to: Destination::First,
            }],
            outs: 0,
            runs: 0,
            credits: vec![
                Credit {
                    player_id: "home0".into(),
                    kind: CreditKind::PlateAppearance,
                },
                Credit {
                    player_id: "home0".into(),
                    kind: CreditKind::Strikeout,
                },
                Credit {
                    player_id: "catcher".into(),
                    kind: CreditKind::PassedBall,
                },
            ],
            changes: vec![],
            third_out: None,
        },
        fielding_review: bbsim_engine::FieldingReview::Unknown,
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        ordinary_air_catch: None,
        muffed_ground_ball: None,
        ordinary_ground_throw: None,
    };
    let transition = EventTransition::try_new(before, event, after).unwrap();
    assert_eq!(
        EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
        transition
    );
    assert_eq!(transition.after().outs, 0);
}

#[test]
fn substitution_records_lineup_and_pinch_runner_responsibility() {
    let before = state();
    let mut after = before.clone();
    after.teams[1].batting_order[4] = "bench_batter".into();
    after.teams[1].batting_order[7] = "pinch_runner".into();
    after.teams[1]
        .bench
        .retain(|id| id != "bench_batter" && id != "pinch_runner");
    after.teams[1]
        .removed
        .extend(["home4".into(), "runner_a".into()]);
    after.bases[0] = Some(Runner {
        player_id: "pinch_runner".into(),
        responsible_pitcher: "away_pitcher".into(),
    });
    let event = Event {
        kind: EventKind::Substitution,
        participants: Participants {
            batter: Some("home0".into()),
            pitcher: Some("away_pitcher".into()),
        },
        delta: Delta {
            moves: vec![],
            outs: 0,
            runs: 0,
            credits: vec![],
            changes: vec![
                PersonnelChange::BattingSlot {
                    team: TeamSide::Home,
                    slot: 4,
                    outgoing: "home4".into(),
                    incoming: "bench_batter".into(),
                },
                PersonnelChange::PinchRunner {
                    base: Base::First,
                    outgoing: before.bases[0].clone().unwrap(),
                    incoming: after.bases[0].clone().unwrap(),
                },
            ],
            third_out: None,
        },
        fielding_review: bbsim_engine::FieldingReview::Unknown,
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        ordinary_air_catch: None,
        muffed_ground_ball: None,
        ordinary_ground_throw: None,
    };
    let transition =
        EventTransition::try_new(before.clone(), event.clone(), after.clone()).unwrap();
    assert_eq!(
        EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
        transition
    );
    let mut uncharged = event;
    if let PersonnelChange::PinchRunner { incoming, .. } = &mut uncharged.delta.changes[1] {
        incoming.responsible_pitcher = "wrong_pitcher".into();
    }
    assert!(EventTransition::try_new(before, uncharged, after).is_err());
}

#[test]
fn simultaneous_out_and_score_reconciles_both_deltas() {
    let mut before = state();
    before.outs = 1;
    before.bases[1] = None;
    before.teams[1].batting_order[5] = "scorer".into();
    before.bases[2] = Some(Runner {
        player_id: "scorer".into(),
        responsible_pitcher: "away_pitcher".into(),
    });
    let mut after = before.clone();
    after.outs = 2;
    after.score[1] += 1;
    after.bases[2] = None;
    after.teams[1].next_batter = 1;
    let event = Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some("home0".into()),
            pitcher: Some("away_pitcher".into()),
        },
        delta: Delta {
            moves: vec![
                RunnerMove {
                    runner: Runner {
                        player_id: "home0".into(),
                        responsible_pitcher: "away_pitcher".into(),
                    },
                    from: Base::Plate,
                    to: Destination::Out,
                },
                RunnerMove {
                    runner: before.bases[2].clone().unwrap(),
                    from: Base::Third,
                    to: Destination::Home,
                },
            ],
            outs: 1,
            runs: 1,
            credits: vec![
                Credit {
                    player_id: "home0".into(),
                    kind: CreditKind::PlateAppearance,
                },
                Credit {
                    player_id: "scorer".into(),
                    kind: CreditKind::Run,
                },
                Credit {
                    player_id: "home0".into(),
                    kind: CreditKind::Rbi,
                },
            ],
            changes: vec![],
            third_out: None,
        },
        fielding_review: bbsim_engine::FieldingReview::Unknown,
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        ordinary_air_catch: None,
        muffed_ground_ball: None,
        ordinary_ground_throw: None,
    };
    assert!(EventTransition::try_new(before, event, after).is_ok());
}
