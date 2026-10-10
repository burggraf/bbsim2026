use bbsim_engine::{
    accounting::{derive_accounting, AccountingError},
    apply, Base, Credit, CreditKind, Delta, Destination, Event, EventKind, GameState, Half,
    Participants, PersonnelChange, Runner, RunnerEvent, RunnerMove, TeamSide, TeamState, ThirdOut,
    ThirdOutKind,
};

fn team(prefix: &str, pitcher: &str) -> TeamState {
    TeamState {
        batting_order: std::array::from_fn(|i| format!("{prefix}{i}")),
        next_batter: 0,
        pitcher: pitcher.into(),
        dh_slot: Some(8),
        dh_must_bat: false,
        bench: vec![format!("{prefix}_reliever")],
        removed: vec![],
    }
}
fn state() -> GameState {
    let mut away = team("away", "away_pitcher");
    away.batting_order[7] = "runner".into();
    GameState {
        inning: 2,
        half: Half::Top,
        outs: 1,
        score: [0, 0],
        teams: [away, team("home", "home_starter")],
        bases: [
            Some(Runner {
                player_id: "runner".into(),
                responsible_pitcher: "home_starter".into(),
            }),
            None,
            None,
        ],
        over: false,
    }
}
fn event(
    before: &GameState,
    kind: EventKind,
    moves: Vec<RunnerMove>,
    outs: u8,
    runs: u8,
    credits: Vec<Credit>,
    changes: Vec<PersonnelChange>,
) -> Event {
    let offense = before.half.offense();
    let mut credits = credits;
    if kind == EventKind::PlateAppearance {
        credits.push(Credit {
            player_id: before.teams[offense].batting_order
                [before.teams[offense].next_batter as usize]
                .clone(),
            kind: CreditKind::PlateAppearance,
        });
    }
    Event {
        kind,
        participants: Participants {
            batter: Some(
                before.teams[offense].batting_order[before.teams[offense].next_batter as usize]
                    .clone(),
            ),
            pitcher: before.teams[1 - offense].pitcher.clone(),
        },
        delta: Delta {
            moves,
            outs,
            runs,
            credits,
            changes,
            third_out: None,
        },
    }
}
fn credit(id: &str, kind: CreditKind) -> Credit {
    Credit {
        player_id: id.into(),
        kind,
    }
}
fn batter_move(before: &GameState, to: Destination) -> RunnerMove {
    let offense = before.half.offense();
    RunnerMove {
        runner: Runner {
            player_id: before.teams[offense].batting_order
                [before.teams[offense].next_batter as usize]
                .clone(),
            responsible_pitcher: before.teams[1 - offense].pitcher.clone(),
        },
        from: Base::Plate,
        to,
    }
}

#[test]
fn golden_partial_log_derives_lines_and_charges_inherited_run_to_prior_pitcher() {
    let before = state();
    let relief = event(
        &before,
        EventKind::Substitution,
        vec![],
        0,
        0,
        vec![],
        vec![PersonnelChange::Pitcher {
            team: TeamSide::Home,
            outgoing: "home_starter".into(),
            incoming: "home_reliever".into(),
        }],
    );
    let first = apply(&before, relief).unwrap();
    let s = first.after();
    let hit = event(
        s,
        EventKind::PlateAppearance,
        vec![
            batter_move(s, Destination::First),
            RunnerMove {
                runner: s.bases[0].clone().unwrap(),
                from: Base::First,
                to: Destination::Third,
            },
        ],
        0,
        0,
        vec![credit("away0", CreditKind::Hit)],
        vec![],
    );
    let second = apply(s, hit).unwrap();
    let s = second.after();
    let run = event(
        s,
        EventKind::RunnerOnly(RunnerEvent::WildPitch),
        vec![RunnerMove {
            runner: s.bases[2].clone().unwrap(),
            from: Base::Third,
            to: Destination::Home,
        }],
        0,
        1,
        vec![credit("runner", CreditKind::Run)],
        vec![],
    );
    let third = apply(s, run).unwrap();
    let s = third.after();
    let out = event(
        s,
        EventKind::PlateAppearance,
        vec![batter_move(s, Destination::Out)],
        1,
        0,
        vec![credit("home5", CreditKind::Putout)],
        vec![],
    );
    let fourth = apply(s, out).unwrap();
    let s = fourth.after();
    let out = event(
        s,
        EventKind::PlateAppearance,
        vec![batter_move(s, Destination::Out)],
        1,
        0,
        vec![credit("home5", CreditKind::Putout)],
        vec![],
    );
    let fifth = apply(s, out).unwrap();
    let log = [first, second, third, fourth, fifth];
    let box_score = derive_accounting(&log).unwrap();
    assert_eq!(box_score.runs, [1, 0]);
    assert_eq!(box_score.innings[&2], [Some(1), None]);
    assert!(!box_score.complete_game);
    assert_eq!(box_score.batters[0]["away0"].plate_appearances, 1);
    assert_eq!(box_score.batters[0]["away0"].credited_hits, 1);
    assert_eq!(box_score.batters[0]["runner"].credited_runs, 1);
    assert_eq!(box_score.batters[0]["away0"].at_bats, Some(1));
    assert_eq!(box_score.batters[0]["away1"].at_bats, None);
    assert_eq!(box_score.pitchers[1]["home_reliever"].at_bats_against, None);
    assert_eq!(box_score.pitchers[1]["home_starter"].runs_allowed, 1);
    assert_eq!(box_score.pitchers[1]["home_starter"].outs_recorded, 0);
    assert_eq!(box_score.pitchers[1]["home_reliever"].outs_recorded, 2);
    assert_eq!(
        box_score.pitchers[1]["home_reliever"].credited_hits_allowed,
        1
    );
    assert_eq!(box_score.pitchers[1]["home_reliever"].earned_runs, None);
    assert_eq!(box_score.fielders[1]["home5"].credited_putouts, 2);
    assert!(!box_score.batters[0].contains_key("home5"));
    assert_eq!(
        box_score.pitchers[1]
            .values()
            .map(|p| p.runs_allowed)
            .sum::<u32>(),
        box_score.runs[0]
    );
    assert_eq!(
        derive_accounting(&[log[0].clone(), log[0].clone()]),
        Err(AccountingError::Discontinuous)
    );
}

#[test]
fn third_out_home_crossing_does_not_appear_in_line_or_pitcher_runs() {
    let mut before = state();
    before.inning = 8;
    before.outs = 2;
    before.bases[2] = Some(Runner {
        player_id: "away6".into(),
        responsible_pitcher: "home_starter".into(),
    });
    let moves = vec![
        batter_move(&before, Destination::First),
        RunnerMove {
            runner: before.bases[0].clone().unwrap(),
            from: Base::First,
            to: Destination::Out,
        },
        RunnerMove {
            runner: before.bases[2].clone().unwrap(),
            from: Base::Third,
            to: Destination::Home,
        },
    ];
    let mut action = event(
        &before,
        EventKind::PlateAppearance,
        moves,
        1,
        0,
        vec![],
        vec![],
    );
    action.delta.third_out = Some(ThirdOut {
        runner_id: "runner".into(),
        kind: ThirdOutKind::Force,
        home_before_out: vec!["away6".into()],
    });
    let transition = apply(&before, action).unwrap();
    let accounting = derive_accounting(&[transition]).unwrap();
    assert_eq!(accounting.innings[&8], [Some(0), None]);
    assert_eq!(accounting.runs, [0, 0]);
    assert_eq!(accounting.pitchers[1]["home_starter"].runs_allowed, 0);
}

#[test]
fn statistical_credit_must_belong_to_an_eligible_participant() {
    let before = state();
    let action = event(
        &before,
        EventKind::PlateAppearance,
        vec![batter_move(&before, Destination::Out)],
        1,
        0,
        vec![credit("ghost", CreditKind::Putout)],
        vec![],
    );
    assert!(apply(&before, action).is_err());
    let action = event(
        &before,
        EventKind::PlateAppearance,
        vec![batter_move(&before, Destination::Out)],
        1,
        0,
        vec![credit("home5", CreditKind::Hit)],
        vec![],
    );
    assert!(apply(&before, action).is_err());
}

#[test]
fn stolen_base_credit_requires_an_advance_not_a_return_to_the_same_base() {
    let mut before = state();
    before.bases[1] = before.bases[0].take();
    let runner = before.bases[1].clone().unwrap();
    let action = event(
        &before,
        EventKind::RunnerOnly(RunnerEvent::StealAttempt),
        vec![RunnerMove {
            runner,
            from: Base::Second,
            to: Destination::Second,
        }],
        0,
        0,
        vec![credit("runner", CreditKind::StolenBase)],
        vec![],
    );
    assert!(apply(&before, action).is_err());
}

#[test]
fn all_base_masks_credit_hr_runs_to_the_right_pitchers() {
    for mask in 0u8..8 {
        let mut before = state();
        before.outs = 0;
        before.bases = [None, None, None];
        let mut moves = vec![batter_move(&before, Destination::Home)];
        let mut credits = vec![
            credit("away0", CreditKind::PlateAppearance),
            credit("away0", CreditKind::Hit),
            credit("away0", CreditKind::Run),
        ];
        for base in 0..3 {
            if mask & (1 << base) == 0 {
                continue;
            }
            let id = match base {
                0 => "away5",
                1 => "away6",
                _ => "runner",
            };
            let from = match base {
                0 => Base::First,
                1 => Base::Second,
                _ => Base::Third,
            };
            let responsibility = if base == 0 {
                "old_pitcher"
            } else {
                "home_starter"
            };
            let runner = Runner {
                player_id: id.into(),
                responsible_pitcher: responsibility.into(),
            };
            before.bases[base] = Some(runner.clone());
            moves.push(RunnerMove {
                runner,
                from,
                to: Destination::Home,
            });
            credits.push(credit(id, CreditKind::Run));
        }
        let action = event(
            &before,
            EventKind::OutOfParkHomeRun,
            moves,
            0,
            1 + mask.count_ones() as u8,
            credits,
            vec![],
        );
        let transition = apply(&before, action).unwrap();
        let result = derive_accounting(&[transition]).unwrap();
        let expected = 1 + mask.count_ones();
        assert_eq!(result.runs, [expected, 0], "mask {mask}");
        assert_eq!(
            result.innings[&2],
            [Some(expected as u16), None],
            "mask {mask}"
        );
        assert_eq!(
            result.pitchers[1]
                .values()
                .map(|p| p.runs_allowed)
                .sum::<u32>(),
            expected,
            "mask {mask}"
        );
        assert_eq!(
            result.pitchers[1]
                .get("old_pitcher")
                .map(|p| p.runs_allowed),
            (mask & 1 != 0).then_some(1),
            "mask {mask}"
        );
    }
}

#[test]
fn complete_nine_inning_log_distinguishes_unplayed_bottom_ninth_from_zero() {
    let mut before = state();
    before.inning = 1;
    before.outs = 0;
    before.bases = [None, None, None];
    let mut log = Vec::new();
    while !before.over {
        assert!(log.len() < 60, "game failed to terminate");
        let action = if before.inning == 1
            && before.half == Half::Bottom
            && before.outs == 0
            && before.score[1] == 0
        {
            let batter =
                before.teams[1].batting_order[before.teams[1].next_batter as usize].clone();
            event(
                &before,
                EventKind::OutOfParkHomeRun,
                vec![batter_move(&before, Destination::Home)],
                0,
                1,
                vec![
                    credit(&batter, CreditKind::PlateAppearance),
                    credit(&batter, CreditKind::Hit),
                    credit(&batter, CreditKind::Run),
                ],
                vec![],
            )
        } else {
            event(
                &before,
                EventKind::PlateAppearance,
                vec![batter_move(&before, Destination::Out)],
                1,
                0,
                vec![],
                vec![],
            )
        };
        let transition = apply(&before, action).unwrap();
        before = transition.after().clone();
        log.push(transition);
    }
    let accounting = derive_accounting(&log).unwrap();
    assert_eq!(log.len(), 52);
    assert!(accounting.complete_game);
    assert_eq!(accounting.runs, [0, 1]);
    assert_eq!(accounting.innings.len(), 9);
    assert_eq!(accounting.innings[&1], [Some(0), Some(1)]);
    assert_eq!(accounting.innings[&9], [Some(0), None]);
    assert_eq!(
        accounting.batters[0]
            .values()
            .map(|b| b.plate_appearances)
            .sum::<u32>(),
        27
    );
    assert_eq!(
        accounting.batters[1]
            .values()
            .map(|b| b.plate_appearances)
            .sum::<u32>(),
        25
    );
    assert_eq!(accounting.pitchers[1]["home_starter"].outs_recorded, 27);
    assert_eq!(accounting.pitchers[0]["away_pitcher"].outs_recorded, 24);
}

#[test]
fn fielding_credits_are_separate_from_batting_and_pitching_counts() {
    let mut before = state();
    before.bases = [None, None, None];
    let action = event(
        &before,
        EventKind::PlateAppearance,
        vec![batter_move(&before, Destination::First)],
        0,
        0,
        vec![
            credit("away0", CreditKind::Strikeout),
            credit("home5", CreditKind::PassedBall),
        ],
        vec![],
    );
    let first = apply(&before, action).unwrap();
    let s = first.after();
    let action = event(
        s,
        EventKind::PlateAppearance,
        vec![
            batter_move(s, Destination::First),
            RunnerMove {
                runner: s.bases[0].clone().unwrap(),
                from: Base::First,
                to: Destination::Second,
            },
        ],
        0,
        0,
        vec![credit("home5", CreditKind::Error)],
        vec![],
    );
    let second = apply(s, action).unwrap();
    let s = second.after();
    let action = event(
        s,
        EventKind::PlateAppearance,
        vec![batter_move(s, Destination::Out)],
        1,
        0,
        vec![
            credit("home5", CreditKind::Assist),
            credit("home4", CreditKind::Putout),
        ],
        vec![],
    );
    let third = apply(s, action).unwrap();
    let box_score = derive_accounting(&[first, second, third]).unwrap();
    assert_eq!(box_score.fielders[1]["home5"].credited_passed_balls, 1);
    assert_eq!(box_score.fielders[1]["home5"].credited_errors, 1);
    assert_eq!(box_score.fielders[1]["home5"].credited_assists, 1);
    assert_eq!(box_score.fielders[1]["home4"].credited_putouts, 1);
    assert_eq!(box_score.batters[0]["away0"].credited_strikeouts, 1);
    assert_eq!(box_score.pitchers[1]["home_starter"].credited_strikeouts, 1);
    assert!(!box_score.batters[0].contains_key("home5"));
}

#[test]
fn pinch_runner_scores_against_his_predecessors_responsible_pitcher() {
    let mut before = state();
    before.teams[0].bench.push("pinch".into());
    let outgoing = before.bases[0].clone().unwrap();
    let incoming = Runner {
        player_id: "pinch".into(),
        responsible_pitcher: "home_starter".into(),
    };
    let changes = vec![
        PersonnelChange::Pitcher {
            team: TeamSide::Home,
            outgoing: "home_starter".into(),
            incoming: "home_reliever".into(),
        },
        PersonnelChange::PinchRunner {
            base: Base::First,
            outgoing,
            incoming,
        },
    ];
    let action = event(
        &before,
        EventKind::Substitution,
        vec![],
        0,
        0,
        vec![],
        changes,
    );
    let first = apply(&before, action).unwrap();
    let s = first.after();
    let action = event(
        s,
        EventKind::RunnerOnly(RunnerEvent::WildPitch),
        vec![RunnerMove {
            runner: s.bases[0].clone().unwrap(),
            from: Base::First,
            to: Destination::Home,
        }],
        0,
        1,
        vec![credit("pinch", CreditKind::Run)],
        vec![],
    );
    let second = apply(s, action).unwrap();
    let lines = derive_accounting(&[first, second]).unwrap();
    assert_eq!(lines.batters[0]["pinch"].credited_runs, 1);
    assert_eq!(lines.batters[0]["pinch"].plate_appearances, 0);
    assert_eq!(lines.pitchers[1]["home_starter"].runs_allowed, 1);
    assert_eq!(lines.pitchers[1]["home_reliever"].runs_allowed, 0);
}

#[test]
fn a_single_plate_appearance_cannot_credit_two_hits() {
    let mut before = state();
    before.bases = [None, None, None];
    let action = event(
        &before,
        EventKind::PlateAppearance,
        vec![batter_move(&before, Destination::First)],
        0,
        0,
        vec![
            credit("away0", CreditKind::Hit),
            credit("away0", CreditKind::Hit),
        ],
        vec![],
    );
    assert!(apply(&before, action).is_err());
}

#[test]
fn official_at_bats_are_derived_only_from_classified_plate_appearances() {
    let mut before = state();
    before.bases = [None, None, None];
    let action = event(
        &before,
        EventKind::PlateAppearance,
        vec![batter_move(&before, Destination::Out)],
        1,
        0,
        vec![credit("away0", CreditKind::AtBat)],
        vec![],
    );
    let first = apply(&before, action).unwrap();
    let s = first.after();
    let action = event(
        s,
        EventKind::PlateAppearance,
        vec![batter_move(s, Destination::First)],
        0,
        0,
        vec![credit("away1", CreditKind::Walk)],
        vec![],
    );
    let second = apply(s, action).unwrap();
    let s = second.after();
    let action = event(
        s,
        EventKind::PlateAppearance,
        vec![
            batter_move(s, Destination::First),
            RunnerMove {
                runner: s.bases[0].clone().unwrap(),
                from: Base::First,
                to: Destination::Second,
            },
        ],
        0,
        0,
        vec![credit("away2", CreditKind::HitByPitch)],
        vec![],
    );
    let third = apply(s, action).unwrap();
    let result = derive_accounting(&[first, second, third]).unwrap();
    assert_eq!(result.batters[0]["away0"].at_bats, Some(1));
    assert_eq!(result.batters[0]["away1"].at_bats, Some(0));
    assert_eq!(result.batters[0]["away2"].at_bats, Some(0));
    assert_eq!(result.batters[0]["away2"].credited_hit_by_pitch, 1);
    assert_eq!(result.pitchers[1]["home_starter"].at_bats_against, Some(1));
    assert_eq!(result.pitchers[1]["home_starter"].batters_faced, 3);

    let mut before = state();
    before.bases = [None, None, None];
    let unknown = apply(
        &before,
        event(
            &before,
            EventKind::PlateAppearance,
            vec![batter_move(&before, Destination::Out)],
            1,
            0,
            vec![],
            vec![],
        ),
    )
    .unwrap();
    let result = derive_accounting(&[unknown]).unwrap();
    assert_eq!(result.batters[0]["away0"].at_bats, None);
    assert_eq!(result.pitchers[1]["home_starter"].at_bats_against, None);
}

#[test]
fn sacrifice_fly_and_interference_award_do_not_create_at_bats() {
    let mut before = state();
    before.bases = [
        None,
        None,
        Some(Runner {
            player_id: "away6".into(),
            responsible_pitcher: "home_starter".into(),
        }),
    ];
    let action = event(
        &before,
        EventKind::PlateAppearance,
        vec![
            batter_move(&before, Destination::Out),
            RunnerMove {
                runner: before.bases[2].clone().unwrap(),
                from: Base::Third,
                to: Destination::Home,
            },
        ],
        1,
        1,
        vec![
            credit("away0", CreditKind::SacrificeFly),
            credit("away6", CreditKind::Run),
            credit("away0", CreditKind::Rbi),
        ],
        vec![],
    );
    let first = apply(&before, action).unwrap();
    let s = first.after();
    let action = event(
        s,
        EventKind::PlateAppearance,
        vec![batter_move(s, Destination::First)],
        0,
        0,
        vec![credit("away1", CreditKind::InterferenceAward)],
        vec![],
    );
    let second = apply(s, action).unwrap();
    let s = second.after();
    let action = event(
        s,
        EventKind::PlateAppearance,
        vec![
            batter_move(s, Destination::Out),
            RunnerMove {
                runner: s.bases[0].clone().unwrap(),
                from: Base::First,
                to: Destination::Second,
            },
        ],
        1,
        0,
        vec![credit("away2", CreditKind::SacrificeBunt)],
        vec![],
    );
    let third = apply(s, action).unwrap();
    let result = derive_accounting(&[first, second, third]).unwrap();
    assert_eq!(result.batters[0]["away0"].credited_sacrifice_flies, 1);
    assert_eq!(result.batters[0]["away0"].at_bats, Some(0));
    assert_eq!(result.batters[0]["away1"].credited_interference_awards, 1);
    assert_eq!(result.batters[0]["away1"].at_bats, Some(0));
    assert_eq!(result.batters[0]["away2"].credited_sacrifice_bunts, 1);
    assert_eq!(result.batters[0]["away2"].at_bats, Some(0));
    assert_eq!(
        result.pitchers[1]["home_starter"].credited_sacrifice_bunts,
        1
    );
    assert_eq!(result.runs, [1, 0]);
}

#[test]
fn conflicting_pa_classification_cannot_claim_a_hit_and_hit_by_pitch() {
    let mut before = state();
    before.bases = [None, None, None];
    let action = event(
        &before,
        EventKind::PlateAppearance,
        vec![batter_move(&before, Destination::First)],
        0,
        0,
        vec![
            credit("away0", CreditKind::Hit),
            credit("away0", CreditKind::HitByPitch),
        ],
        vec![],
    );
    assert!(apply(&before, action).is_err());
}

#[test]
fn awarded_first_base_cannot_end_with_batter_out_without_touch_metadata() {
    let mut before = state();
    before.bases = [None, None, None];
    for kind in [
        CreditKind::Walk,
        CreditKind::HitByPitch,
        CreditKind::InterferenceAward,
    ] {
        let action = event(
            &before,
            EventKind::PlateAppearance,
            vec![batter_move(&before, Destination::Out)],
            1,
            0,
            vec![credit("away0", kind)],
            vec![],
        );
        assert!(apply(&before, action).is_err(), "{kind:?}");
    }
}
