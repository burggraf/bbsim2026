use bbsim_engine::{
    accounting::{derive_accounting, AccountingError},
    apply, Base, Credit, CreditKind, Delta, Destination, Event, EventKind, EventTransition,
    FieldingReview, GameState, Half, Participants, PersonnelChange, Runner, RunnerEvent,
    RunnerMove, TeamSide, TeamState, ThirdOut, ThirdOutKind, TransitionError,
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
        fielding_review: bbsim_engine::FieldingReview::Unknown,
        two_out_contact: None,
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
    assert_eq!(box_score.pitchers[1]["home_reliever"].earned_runs, Some(0));
    assert_eq!(box_score.pitchers[1]["home_starter"].earned_runs, None);
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

fn reviewed(mut event: Event) -> Event {
    event.fielding_review = FieldingReview::NoMisplay;
    event
}

fn finish_with_three_strikeouts(
    mut before: GameState,
    log: &mut Vec<bbsim_engine::EventTransition>,
    review: FieldingReview,
) {
    let next_half = match before.half {
        Half::Top => Half::Bottom,
        Half::Bottom => Half::Top,
    };
    for _ in 0..3 {
        let offense = before.half.offense();
        let batter =
            before.teams[offense].batting_order[before.teams[offense].next_batter as usize].clone();
        let action = event(
            &before,
            EventKind::PlateAppearance,
            vec![batter_move(&before, Destination::Out)],
            1,
            0,
            vec![credit(&batter, CreditKind::Strikeout)],
            vec![],
        );
        let mut action = action;
        action.fielding_review = review;
        let next = apply(&before, action).unwrap();
        before = next.after().clone();
        log.push(next);
    }
    assert_eq!(before.half, next_half);
}

#[test]
fn unreviewed_homer_and_strikeouts_do_not_certify_earned_runs() {
    let mut before = state();
    before.inning = 1;
    before.outs = 0;
    before.bases = [None, None, None];
    let action = event(
        &before,
        EventKind::OutOfParkHomeRun,
        vec![batter_move(&before, Destination::Home)],
        0,
        1,
        vec![
            credit("away0", CreditKind::PlateAppearance),
            credit("away0", CreditKind::Hit),
            credit("away0", CreditKind::Run),
        ],
        vec![],
    );
    let first = apply(&before, action).unwrap();
    let mut log = vec![first.clone()];
    finish_with_three_strikeouts(first.after().clone(), &mut log, FieldingReview::Unknown);
    let result = derive_accounting(&log).unwrap();
    assert_eq!(result.pitchers[1]["home_starter"].runs_allowed, 1);
    assert_eq!(result.pitchers[1]["home_starter"].earned_runs, None);
}

#[test]
fn walked_runner_on_wild_pitches_is_earned_but_passed_ball_needs_reconstruction() {
    for final_kind in [RunnerEvent::WildPitch, RunnerEvent::PassedBall] {
        let mut before = state();
        before.inning = 1;
        before.outs = 0;
        before.bases = [None, None, None];
        let action = event(
            &before,
            EventKind::PlateAppearance,
            vec![batter_move(&before, Destination::First)],
            0,
            0,
            vec![credit("away0", CreditKind::Walk)],
            vec![],
        );
        let first = apply(&before, reviewed(action)).unwrap();
        let mut log = vec![first];
        for (base, to) in [
            (Base::First, Destination::Second),
            (Base::Second, Destination::Third),
            (Base::Third, Destination::Home),
        ] {
            let s = log.last().unwrap().after();
            let index = match base {
                Base::First => 0,
                Base::Second => 1,
                _ => 2,
            };
            let kind = if base == Base::Third {
                final_kind
            } else {
                RunnerEvent::WildPitch
            };
            let runs = u8::from(to == Destination::Home);
            let action = event(
                s,
                EventKind::RunnerOnly(kind),
                vec![RunnerMove {
                    runner: s.bases[index].clone().unwrap(),
                    from: base,
                    to,
                }],
                0,
                runs,
                if runs == 1 {
                    vec![credit("away0", CreditKind::Run)]
                } else {
                    vec![]
                },
                vec![],
            );
            let action = if kind == RunnerEvent::PassedBall {
                action
            } else {
                reviewed(action)
            };
            log.push(apply(s, action).unwrap());
        }
        finish_with_three_strikeouts(
            log.last().unwrap().after().clone(),
            &mut log,
            FieldingReview::NoMisplay,
        );
        let result = derive_accounting(&log).unwrap();
        let expected = if final_kind == RunnerEvent::WildPitch {
            Some(1)
        } else {
            None
        };
        assert_eq!(
            result.pitchers[1]["home_starter"].earned_runs, expected,
            "{final_kind:?}"
        );
    }
}

#[test]
fn error_before_homer_prevents_unjustified_earned_run_certification() {
    let mut before = state();
    before.inning = 1;
    before.outs = 0;
    before.bases = [None, None, None];
    let mut log = Vec::new();
    for _ in 0..2 {
        let batter = before.teams[0].batting_order[before.teams[0].next_batter as usize].clone();
        let action = event(
            &before,
            EventKind::PlateAppearance,
            vec![batter_move(&before, Destination::Out)],
            1,
            0,
            vec![credit(&batter, CreditKind::Strikeout)],
            vec![],
        );
        let next = apply(&before, action).unwrap();
        before = next.after().clone();
        log.push(next);
    }
    let action = event(
        &before,
        EventKind::PlateAppearance,
        vec![batter_move(&before, Destination::First)],
        0,
        0,
        vec![
            credit("home5", CreditKind::Error),
            credit("away2", CreditKind::AtBat),
        ],
        vec![],
    );
    let next = apply(&before, action).unwrap();
    before = next.after().clone();
    log.push(next);
    let action = event(
        &before,
        EventKind::OutOfParkHomeRun,
        vec![
            batter_move(&before, Destination::Home),
            RunnerMove {
                runner: before.bases[0].clone().unwrap(),
                from: Base::First,
                to: Destination::Home,
            },
        ],
        0,
        2,
        vec![
            credit("away3", CreditKind::PlateAppearance),
            credit("away3", CreditKind::Hit),
            credit("away2", CreditKind::Run),
            credit("away3", CreditKind::Run),
        ],
        vec![],
    );
    let next = apply(&before, action).unwrap();
    before = next.after().clone();
    log.push(next);
    let batter = before.teams[0].batting_order[before.teams[0].next_batter as usize].clone();
    let action = event(
        &before,
        EventKind::PlateAppearance,
        vec![batter_move(&before, Destination::Out)],
        1,
        0,
        vec![credit(&batter, CreditKind::Strikeout)],
        vec![],
    );
    log.push(apply(&before, action).unwrap());
    let result = derive_accounting(&log).unwrap();
    assert_eq!(result.pitchers[1]["home_starter"].runs_allowed, 2);
    assert_eq!(result.pitchers[1]["home_starter"].earned_runs, None);
    // Review every other play so this tests the error, not just missing review.
    let reviewed_log: Vec<_> = log
        .iter()
        .map(|t| {
            let action = if t
                .event()
                .delta
                .credits
                .iter()
                .any(|c| c.kind == CreditKind::Error)
            {
                t.event().clone()
            } else {
                reviewed(t.event().clone())
            };
            apply(t.before(), action).unwrap()
        })
        .collect();
    assert_eq!(
        derive_accounting(&reviewed_log).unwrap().pitchers[1]["home_starter"].earned_runs,
        None
    );
}

#[test]
fn clean_inherited_runner_is_earned_to_old_pitcher_not_reliever() {
    let mut before = state();
    before.inning = 1;
    before.outs = 0;
    before.bases = [None, None, None];
    let walk = event(
        &before,
        EventKind::PlateAppearance,
        vec![batter_move(&before, Destination::First)],
        0,
        0,
        vec![credit("away0", CreditKind::Walk)],
        vec![],
    );
    let first = apply(&before, reviewed(walk)).unwrap();
    let mut log = vec![first];
    let s = log.last().unwrap().after();
    let change = event(
        s,
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
    log.push(apply(s, reviewed(change)).unwrap());
    for (from, to) in [
        (Base::First, Destination::Second),
        (Base::Second, Destination::Third),
        (Base::Third, Destination::Home),
    ] {
        let s = log.last().unwrap().after();
        let index = match from {
            Base::First => 0,
            Base::Second => 1,
            _ => 2,
        };
        let runs = u8::from(to == Destination::Home);
        let action = event(
            s,
            EventKind::RunnerOnly(RunnerEvent::WildPitch),
            vec![RunnerMove {
                runner: s.bases[index].clone().unwrap(),
                from,
                to,
            }],
            0,
            runs,
            if runs == 1 {
                vec![credit("away0", CreditKind::Run)]
            } else {
                vec![]
            },
            vec![],
        );
        log.push(apply(s, reviewed(action)).unwrap());
    }
    finish_with_three_strikeouts(
        log.last().unwrap().after().clone(),
        &mut log,
        FieldingReview::NoMisplay,
    );
    let result = derive_accounting(&log).unwrap();
    assert_eq!(result.pitchers[1]["home_starter"].runs_allowed, 1);
    assert_eq!(result.pitchers[1]["home_starter"].earned_runs, Some(1));
    assert_eq!(result.pitchers[1]["home_reliever"].runs_allowed, 0);
    assert_eq!(result.pitchers[1]["home_reliever"].earned_runs, Some(0));
}

fn reviewed_homer_half(inning: u16) -> Vec<bbsim_engine::EventTransition> {
    let mut before = state();
    before.inning = inning;
    before.outs = 0;
    before.bases = [None, None, None];
    reviewed_homer_from(before)
}

fn reviewed_homer_from(before: GameState) -> Vec<bbsim_engine::EventTransition> {
    let offense = before.half.offense();
    let batter = &before.teams[offense].batting_order[before.teams[offense].next_batter as usize];
    let action = reviewed(event(
        &before,
        EventKind::OutOfParkHomeRun,
        vec![batter_move(&before, Destination::Home)],
        0,
        1,
        vec![
            credit(batter, CreditKind::PlateAppearance),
            credit(batter, CreditKind::Hit),
            credit(batter, CreditKind::Run),
        ],
        vec![],
    ));
    let first = apply(&before, action).unwrap();
    let mut log = vec![first];
    finish_with_three_strikeouts(
        log.last().unwrap().after().clone(),
        &mut log,
        FieldingReview::NoMisplay,
    );
    log
}

#[test]
fn every_play_in_the_half_requires_review_for_earned_run_proof() {
    let clean = reviewed_homer_half(1);
    assert_eq!(
        derive_accounting(&clean).unwrap().pitchers[1]["home_starter"].earned_runs,
        Some(1)
    );
    for unknown_index in 0..clean.len() {
        let mut log = Vec::new();
        for (index, transition) in clean.iter().enumerate() {
            let mut action = transition.event().clone();
            if index == unknown_index {
                action.fielding_review = FieldingReview::Unknown;
            }
            log.push(apply(transition.before(), action).unwrap());
        }
        assert_eq!(
            derive_accounting(&log).unwrap().pitchers[1]["home_starter"].earned_runs,
            None,
            "unknown play {unknown_index}"
        );
    }
}

#[test]
fn later_segment_start_cannot_prove_that_earlier_chances_were_observed() {
    // Zero actual outs/empty bases can also follow an earlier solo HR. The
    // preceding batters may have had omitted errors, so this is not a boundary.
    let log = reviewed_homer_half(2);
    let result = derive_accounting(&log).unwrap();
    assert!(!result.complete_game);
    assert_eq!(result.pitchers[1]["home_starter"].earned_runs, None);
}

#[test]
fn reviewed_but_unfinished_half_does_not_certify_scored_runs() {
    let log = reviewed_homer_half(1);
    for end in 1..log.len() {
        assert_eq!(
            derive_accounting(&log[..end]).unwrap().pitchers[1]["home_starter"].earned_runs,
            None
        );
    }
}

#[test]
fn preexisting_out_or_runner_prevents_clean_half_proof() {
    for occupied in [false, true] {
        let mut before = state();
        before.inning = 1;
        before.outs = u8::from(!occupied);
        if !occupied {
            before.bases = [None, None, None];
        }
        let mut moves = vec![batter_move(&before, Destination::Home)];
        let mut credits = vec![
            credit("away0", CreditKind::PlateAppearance),
            credit("away0", CreditKind::Hit),
            credit("away0", CreditKind::Run),
        ];
        if occupied {
            moves.push(RunnerMove {
                runner: before.bases[0].clone().unwrap(),
                from: Base::First,
                to: Destination::Home,
            });
            credits.push(credit("runner", CreditKind::Run));
        }
        let runs = 1 + u8::from(occupied);
        let first = apply(
            &before,
            reviewed(event(
                &before,
                EventKind::OutOfParkHomeRun,
                moves,
                0,
                runs,
                credits,
                vec![],
            )),
        )
        .unwrap();
        let mut log = vec![first];
        while log.last().unwrap().after().half == Half::Top {
            let s = log.last().unwrap().after();
            let batter = &s.teams[0].batting_order[s.teams[0].next_batter as usize];
            let action = reviewed(event(
                s,
                EventKind::PlateAppearance,
                vec![batter_move(s, Destination::Out)],
                1,
                0,
                vec![credit(batter, CreditKind::Strikeout)],
                vec![],
            ));
            log.push(apply(s, action).unwrap());
        }
        assert_eq!(
            derive_accounting(&log).unwrap().pitchers[1]["home_starter"].earned_runs,
            None
        );
    }
}

#[test]
fn fielding_review_roundtrips_and_rejects_conflicting_credits() {
    let log = reviewed_homer_half(1);
    let record = &log[0];
    let decoded = bbsim_engine::EventTransition::from_bytes(&record.to_bytes()).unwrap();
    assert_eq!(decoded, *record);
    assert_eq!(decoded.event().fielding_review, FieldingReview::NoMisplay);
    for (id, kind) in [
        ("home5", CreditKind::Error),
        ("home5", CreditKind::PassedBall),
        ("away0", CreditKind::InterferenceAward),
    ] {
        let mut before = state();
        before.bases = [None, None, None];
        let action = reviewed(event(
            &before,
            EventKind::PlateAppearance,
            vec![batter_move(&before, Destination::First)],
            0,
            0,
            vec![credit(id, kind)],
            vec![],
        ));
        assert_eq!(
            apply(&before, action),
            Err(bbsim_engine::TransitionError::IllegalCredit),
            "{kind:?}"
        );
    }
}

#[test]
fn half_boundaries_accumulate_known_runs_and_never_restore_unknown_totals() {
    let mut clean = reviewed_homer_half(1);
    clean.extend(reviewed_homer_from(clean.last().unwrap().after().clone()));
    clean.extend(reviewed_homer_from(clean.last().unwrap().after().clone()));
    let result = derive_accounting(&clean).unwrap();
    assert_eq!(result.pitchers[1]["home_starter"].earned_runs, Some(2));
    assert_eq!(result.pitchers[0]["away_pitcher"].earned_runs, Some(1));
    assert_eq!(result.runs, [2, 1]);
    for unknown_index in [0, 8] {
        let log: Vec<_> = clean
            .iter()
            .enumerate()
            .map(|(index, t)| {
                let mut action = t.event().clone();
                if index == unknown_index {
                    action.fielding_review = FieldingReview::Unknown;
                }
                apply(t.before(), action).unwrap()
            })
            .collect();
        let result = derive_accounting(&log).unwrap();
        assert_eq!(result.pitchers[1]["home_starter"].earned_runs, None);
        assert_eq!(result.pitchers[0]["away_pitcher"].earned_runs, Some(1));
    }
}

#[test]
fn reviewed_generic_contact_is_not_yet_an_errorless_play_proof() {
    let clean = reviewed_homer_half(1);
    let log: Vec<_> = clean
        .iter()
        .enumerate()
        .map(|(index, t)| {
            let mut action = t.event().clone();
            if index == 1 {
                action.delta.credits.iter_mut().for_each(|credit| {
                    if credit.kind == CreditKind::Strikeout {
                        credit.kind = CreditKind::AtBat;
                    }
                });
            }
            apply(t.before(), action).unwrap()
        })
        .collect();
    assert_eq!(
        derive_accounting(&log).unwrap().pitchers[1]["home_starter"].earned_runs,
        None
    );
}

#[test]
fn observed_boundary_and_reviewed_walkoff_homer_can_certify_earned_run() {
    let mut before = state();
    before.inning = 9;
    before.outs = 2;
    before.score = [1, 1];
    before.bases = [None, None, None];
    let out = reviewed(event(
        &before,
        EventKind::PlateAppearance,
        vec![batter_move(&before, Destination::Out)],
        1,
        0,
        vec![credit("away0", CreditKind::Strikeout)],
        vec![],
    ));
    let first = apply(&before, out).unwrap();
    let s = first.after();
    let homer = reviewed(event(
        s,
        EventKind::OutOfParkHomeRun,
        vec![batter_move(s, Destination::Home)],
        0,
        1,
        vec![
            credit("home0", CreditKind::PlateAppearance),
            credit("home0", CreditKind::Hit),
            credit("home0", CreditKind::Run),
        ],
        vec![],
    ));
    let last = apply(s, homer).unwrap();
    assert!(last.after().over);
    let result = derive_accounting(&[first, last]).unwrap();
    assert_eq!(result.pitchers[0]["away_pitcher"].earned_runs, Some(1));
    assert_eq!(result.pitchers[1]["home_starter"].earned_runs, Some(0));
}

// Walk -> optional clean WP advances -> relief -> bounded FC -> HR -> two K.
// The FC replacement carries the walker's charge, not the reliever's BF/outs.
fn reviewed_fc_half(
    base: Base,
    half: Half,
    walk_review: FieldingReview,
    fc_review: FieldingReview,
) -> (Vec<bbsim_engine::EventTransition>, usize) {
    let mut before = state();
    before.inning = 1;
    before.outs = 0;
    before.bases = [None, None, None];
    let mut log = vec![];
    if half == Half::Bottom {
        finish_with_three_strikeouts(before, &mut log, FieldingReview::NoMisplay);
        before = log.last().unwrap().after().clone();
    }
    let offense = half.offense();
    let defense = 1 - offense;
    let walker = before.teams[offense].batting_order[0].clone();
    let mut walk = event(
        &before,
        EventKind::PlateAppearance,
        vec![batter_move(&before, Destination::First)],
        0,
        0,
        vec![credit(&walker, CreditKind::Walk)],
        vec![],
    );
    walk.fielding_review = walk_review;
    let next = apply(&before, walk).unwrap();
    before = next.after().clone();
    log.push(next);
    let advances = match base {
        Base::First => 0,
        Base::Second => 1,
        Base::Third => 2,
        Base::Plate => unreachable!(),
    };
    for i in 0..advances {
        let advance = reviewed(event(
            &before,
            EventKind::RunnerOnly(RunnerEvent::WildPitch),
            vec![RunnerMove {
                runner: before.bases[i].clone().unwrap(),
                from: [Base::First, Base::Second][i],
                to: [Destination::Second, Destination::Third][i],
            }],
            0,
            0,
            vec![],
            vec![],
        ));
        let next = apply(&before, advance).unwrap();
        before = next.after().clone();
        log.push(next);
    }
    let relief = reviewed(event(
        &before,
        EventKind::Substitution,
        vec![],
        0,
        0,
        vec![],
        vec![PersonnelChange::Pitcher {
            team: if defense == 0 {
                TeamSide::Away
            } else {
                TeamSide::Home
            },
            outgoing: before.teams[defense].pitcher.clone(),
            incoming: before.teams[defense].bench[0].clone(),
        }],
    ));
    let next = apply(&before, relief).unwrap();
    before = next.after().clone();
    log.push(next);
    let runner = before.bases[advances].clone().unwrap();
    let mut replacement = batter_move(&before, Destination::First);
    replacement.runner.responsible_pitcher = runner.responsible_pitcher.clone();
    let batter = replacement.runner.player_id.clone();
    let mut fc = event(
        &before,
        EventKind::FieldersChoice,
        vec![
            RunnerMove {
                runner,
                from: base,
                to: Destination::Out,
            },
            replacement,
        ],
        1,
        0,
        vec![credit(&batter, CreditKind::PlateAppearance)],
        vec![],
    );
    fc.fielding_review = fc_review;
    let fc_index = log.len();
    let next = apply(&before, fc).unwrap();
    before = next.after().clone();
    log.push(next);
    let batter = before.teams[offense].batting_order[2].clone();
    let homer = reviewed(event(
        &before,
        EventKind::OutOfParkHomeRun,
        vec![
            batter_move(&before, Destination::Home),
            RunnerMove {
                runner: before.bases[0].clone().unwrap(),
                from: Base::First,
                to: Destination::Home,
            },
        ],
        0,
        2,
        vec![
            credit(&batter, CreditKind::PlateAppearance),
            credit(&batter, CreditKind::Hit),
            credit(&batter, CreditKind::Run),
            credit(
                &before.bases[0].as_ref().unwrap().player_id,
                CreditKind::Run,
            ),
        ],
        vec![],
    ));
    let next = apply(&before, homer).unwrap();
    before = next.after().clone();
    log.push(next);
    for _ in 0..2 {
        let batter =
            before.teams[offense].batting_order[before.teams[offense].next_batter as usize].clone();
        let strikeout = reviewed(event(
            &before,
            EventKind::PlateAppearance,
            vec![batter_move(&before, Destination::Out)],
            1,
            0,
            vec![credit(&batter, CreditKind::Strikeout)],
            vec![],
        ));
        let next = apply(&before, strikeout).unwrap();
        before = next.after().clone();
        log.push(next);
    }
    assert_ne!(before.half, half);
    (log, fc_index)
}

#[test]
fn reviewed_clean_fc_replacement_preserves_earned_charge_not_actual_participation() {
    for half in [Half::Top, Half::Bottom] {
        for base in [Base::First, Base::Second, Base::Third] {
            let (log, fc_index) = reviewed_fc_half(
                base,
                half,
                FieldingReview::NoMisplay,
                FieldingReview::NoMisplay,
            );
            for transition in &log {
                assert_eq!(
                    bbsim_engine::EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
                    *transition
                );
            }
            let fc = &log[fc_index];
            let offense = half.offense();
            let defense = 1 - offense;
            let charge = &fc.after().bases[0].as_ref().unwrap().responsible_pitcher;
            let reliever = &fc.before().teams[defense].pitcher;
            assert_ne!(charge, reliever);
            let result = derive_accounting(&log).unwrap();
            assert_eq!(result.runs[offense], 2);
            assert_eq!(result.innings[&1][offense], Some(2));
            assert_eq!(result.pitchers[defense][charge].runs_allowed, 1);
            assert_eq!(result.pitchers[defense][reliever].runs_allowed, 1);
            assert_eq!(
                result.pitchers[defense][charge].earned_runs,
                Some(1),
                "{half:?} {base:?}"
            );
            assert_eq!(result.pitchers[defense][reliever].earned_runs, Some(1));
            assert_eq!(result.pitchers[defense][charge].batters_faced, 1);
            assert_eq!(result.pitchers[defense][charge].outs_recorded, 0);
            assert_eq!(result.pitchers[defense][reliever].batters_faced, 4);
            assert_eq!(result.pitchers[defense][reliever].outs_recorded, 3);
            let batter = fc.event().participants.batter.as_ref().unwrap();
            assert_eq!(result.batters[offense][batter].plate_appearances, 1);
            assert_eq!(result.batters[offense][batter].at_bats, Some(1));
            assert_eq!(result.batters[offense][batter].credited_runs, 1);
        }
    }
}

#[test]
fn clean_fc_cannot_certify_unknown_origin_review_or_unfinished_half() {
    for half in [Half::Top, Half::Bottom] {
        for base in [Base::First, Base::Second, Base::Third] {
            for (walk_review, fc_review) in [
                (FieldingReview::Unknown, FieldingReview::NoMisplay),
                (FieldingReview::NoMisplay, FieldingReview::Unknown),
                (FieldingReview::NoMisplay, FieldingReview::NoMisplay),
            ] {
                let (log, fc_index) = reviewed_fc_half(base, half, walk_review, fc_review);
                let defense = 1 - half.offense();
                let charge = &log[fc_index].after().bases[0]
                    .as_ref()
                    .unwrap()
                    .responsible_pitcher;
                let reliever = &log[fc_index].before().teams[defense].pitcher;
                let mut uncertain = vec![&log[fc_index..], &log[..log.len() - 1]];
                if walk_review == FieldingReview::Unknown || fc_review == FieldingReview::Unknown {
                    uncertain.push(&log);
                }
                for partial in uncertain {
                    let result = derive_accounting(partial).unwrap();
                    assert_eq!(result.runs[half.offense()], 2);
                    assert_eq!(
                        result.pitchers[defense][charge].earned_runs, None,
                        "{half:?} {base:?} {walk_review:?} {fc_review:?}"
                    );
                    assert_eq!(result.pitchers[defense][reliever].earned_runs, None);
                }
            }
        }
    }
}

#[test]
fn codec_rejects_corrupted_review_and_well_formed_review_error_contradiction() {
    for half in [Half::Top, Half::Bottom] {
        let mut before = state();
        before.inning = 1;
        before.half = half;
        before.outs = 0;
        before.bases = [None, None, None];
        let offense = half.offense();
        let batter = &before.teams[offense].batting_order[0];
        let action = event(
            &before,
            EventKind::PlateAppearance,
            vec![batter_move(&before, Destination::Out)],
            1,
            0,
            vec![credit(batter, CreditKind::Strikeout)],
            vec![],
        );
        let unknown = apply(&before, action.clone()).unwrap();
        let clean = apply(&before, reviewed(action.clone())).unwrap();
        assert_eq!(unknown.after(), clean.after());
        let unknown_bytes = unknown.to_bytes();
        let clean_bytes = clean.to_bytes();
        assert_eq!(unknown_bytes.len(), clean_bytes.len());
        let differences: Vec<_> = unknown_bytes
            .iter()
            .zip(&clean_bytes)
            .enumerate()
            .filter_map(|(index, (left, right))| (left != right).then_some(index))
            .collect();
        assert_eq!(differences.len(), 1);
        let review_offset = differences[0];
        assert_eq!(unknown_bytes[review_offset], 0);
        assert_eq!(clean_bytes[review_offset], 1);
        assert_eq!(
            EventTransition::from_bytes(&unknown_bytes).unwrap(),
            unknown
        );
        assert_eq!(EventTransition::from_bytes(&clean_bytes).unwrap(), clean);
        let mut corrupted = clean_bytes;
        corrupted[review_offset] = 255;
        assert_eq!(
            EventTransition::from_bytes(&corrupted),
            Err(TransitionError::InvalidEncoding)
        );

        // A missed foul chance can credit Error even if the PA ends in K.
        // Adding that credit changes no state, only the event's wire length.
        let fielder = &before.teams[1 - offense].batting_order[5];
        let mut with_error = action;
        with_error
            .delta
            .credits
            .push(credit(fielder, CreditKind::Error));
        let error = apply(&before, with_error).unwrap();
        assert_eq!(error.after(), unknown.after());
        let mut forged = error.to_bytes();
        assert_eq!(EventTransition::from_bytes(&forged).unwrap(), error);
        // BBE11 writes review after participants and before Delta. Only
        // Delta changed; the paired-record position has an identical prefix.
        assert_eq!(&forged[..review_offset], &unknown_bytes[..review_offset]);
        assert_eq!(forged[review_offset], 0);
        forged[review_offset] = 1; // Valid NoMisplay tag, conflicting Error retained.
        assert_eq!(
            EventTransition::from_bytes(&forged),
            Err(TransitionError::IllegalCredit)
        );
        let line = derive_accounting(std::slice::from_ref(&error)).unwrap();
        assert_eq!(line.fielders[1 - offense][fielder].credited_errors, 1);
    }
}

#[test]
fn reviewed_clean_terminal_third_out_certifies_only_observed_completed_segment() {
    for half in [Half::Top, Half::Bottom] {
        let mut before = state();
        before.outs = 0;
        before.bases = [None, None, None];
        // Observe the preceding boundary, not an invented complete-game
        // origin. The earlier three runs are outside this segment's lines.
        match half {
            Half::Top => {
                before.inning = 8;
                before.half = Half::Bottom;
                before.score = [0, 3];
            }
            Half::Bottom => {
                before.inning = 9;
                before.half = Half::Top;
                before.score = [3, 0];
            }
        }
        let mut log = vec![];
        finish_with_three_strikeouts(before, &mut log, FieldingReview::NoMisplay);
        let s = log.last().unwrap().after();
        assert_eq!((s.inning, s.half, s.outs), (9, half, 0));
        let offense = half.offense();
        let pitcher = s.teams[1 - offense].pitcher.clone();
        let batter = &s.teams[offense].batting_order[s.teams[offense].next_batter as usize];
        let homer = reviewed(event(
            s,
            EventKind::OutOfParkHomeRun,
            vec![batter_move(s, Destination::Home)],
            0,
            1,
            vec![
                credit(batter, CreditKind::PlateAppearance),
                credit(batter, CreditKind::Hit),
                credit(batter, CreditKind::Run),
            ],
            vec![],
        ));
        let next = apply(s, homer).unwrap();
        log.push(next);
        for _ in 0..3 {
            let s = log.last().unwrap().after();
            let batter = &s.teams[offense].batting_order[s.teams[offense].next_batter as usize];
            let out = reviewed(event(
                s,
                EventKind::PlateAppearance,
                vec![batter_move(s, Destination::Out)],
                1,
                0,
                vec![credit(batter, CreditKind::Strikeout)],
                vec![],
            ));
            let next = apply(s, out).unwrap();
            log.push(next);
        }
        let final_state = log.last().unwrap().after();
        assert!(final_state.over);
        assert_eq!(
            (final_state.inning, final_state.half, final_state.outs),
            (9, half, 0)
        );
        let decoded: Vec<_> = log
            .iter()
            .map(|t| EventTransition::from_bytes(&t.to_bytes()).unwrap())
            .collect();
        assert_eq!(decoded, log);
        let line = derive_accounting(&decoded).unwrap();
        let mut runs = [0, 0];
        runs[offense] = 1;
        assert_eq!(line.runs, runs);
        assert!(!line.complete_game);
        assert_eq!(line.pitchers[1 - offense][&pitcher].runs_allowed, 1);
        assert_eq!(line.pitchers[1 - offense][&pitcher].earned_runs, Some(1));
        assert_eq!(line.pitchers[1 - offense][&pitcher].outs_recorded, 3);
        assert_eq!(line.pitchers[1 - offense][&pitcher].batters_faced, 4);
        assert_eq!(
            line.innings[&9],
            if half == Half::Top {
                [Some(1), None]
            } else {
                [Some(0), Some(1)]
            }
        );
        let unfinished = derive_accounting(&log[..log.len() - 1]).unwrap();
        assert_eq!(unfinished.pitchers[1 - offense][&pitcher].runs_allowed, 1);
        assert_eq!(unfinished.pitchers[1 - offense][&pitcher].earned_runs, None);
    }
}

#[test]
fn passed_ball_kind_cannot_claim_no_misplay_even_without_fielding_credit() {
    let before = state();
    let action = reviewed(event(
        &before,
        EventKind::RunnerOnly(RunnerEvent::PassedBall),
        vec![RunnerMove {
            runner: before.bases[0].clone().unwrap(),
            from: Base::First,
            to: Destination::Second,
        }],
        0,
        0,
        vec![],
        vec![],
    ));
    assert_eq!(
        apply(&before, action),
        Err(bbsim_engine::TransitionError::IllegalCredit)
    );
}
