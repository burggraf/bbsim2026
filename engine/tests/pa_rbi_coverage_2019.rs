use bbsim_engine::{
    accounting::{derive_accounting, RbiContribution},
    apply, Base, Credit, CreditKind, Delta, Destination, Event, EventKind, EventTransition,
    FieldingReview, GameState, Half, Participants, ProtectedAwardFacts, Runner, RunnerEvent,
    RunnerMove, TeamState, ThirdOut, ThirdOutKind,
};

fn state(mask: u8, outs: u8, half: Half) -> GameState {
    let team = |prefix: &str| TeamState {
        batting_order: std::array::from_fn(|i| format!("{prefix}{i}")),
        next_batter: 0,
        pitcher: format!("{prefix}p"),
        dh_slot: Some(8),
        dh_must_bat: false,
        bench: vec![],
        removed: vec![],
    };
    let teams = [team("a"), team("h")];
    GameState {
        inning: 2,
        half,
        outs,
        score: [0, 0],
        bases: std::array::from_fn(|i| {
            (mask & (1 << i) != 0).then(|| Runner {
                player_id: teams[half.offense()].batting_order[i + 3].clone(),
                responsible_pitcher: "oldp".into(),
            })
        }),
        teams,
        over: false,
    }
}
fn credit(id: &str, kind: CreditKind) -> Credit {
    Credit {
        player_id: id.into(),
        kind,
    }
}
fn blank(before: &GameState) -> Event {
    let offense = &before.teams[before.half.offense()];
    Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(offense.batting_order[offense.next_batter as usize].clone()),
            pitcher: before.teams[1 - before.half.offense()].pitcher.clone(),
        },
        fielding_review: FieldingReview::Unknown,
        delta: Delta {
            moves: vec![],
            outs: 0,
            runs: 0,
            credits: vec![],
            changes: vec![],
            third_out: None,
        },
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        ordinary_ground_throw: None,
        caught_strikeout: None,
        protected_award: None,
    }
}
fn batter(event: &Event) -> String {
    event.participants.batter.clone().unwrap()
}
fn batter_move(event: &Event, to: Destination) -> RunnerMove {
    RunnerMove {
        runner: Runner {
            player_id: batter(event),
            responsible_pitcher: event.participants.pitcher.clone(),
        },
        from: Base::Plate,
        to,
    }
}
fn out(before: &GameState, classified: bool) -> Event {
    let mut event = blank(before);
    let id = batter(&event);
    event
        .delta
        .moves
        .push(batter_move(&event, Destination::Out));
    event.delta.outs = 1;
    event
        .delta
        .credits
        .push(credit(&id, CreditKind::PlateAppearance));
    if classified {
        event.delta.credits.push(credit(&id, CreditKind::Strikeout));
    }
    if before.outs == 2 {
        event.delta.third_out = Some(ThirdOut {
            runner_id: id,
            kind: ThirdOutKind::BatterBeforeFirst,
            home_before_out: vec![],
        });
    }
    event
}
fn award(before: &GameState, cause: bool, hbp: bool) -> Event {
    let mut event = blank(before);
    let id = batter(&event);
    event
        .delta
        .moves
        .push(batter_move(&event, Destination::First));
    event.delta.credits.extend([
        credit(&id, CreditKind::PlateAppearance),
        credit(
            &id,
            if hbp {
                CreditKind::HitByPitch
            } else {
                CreditKind::Walk
            },
        ),
    ]);
    if !hbp {
        event
            .delta
            .credits
            .push(credit(&id, CreditKind::OrdinaryWalk));
    }
    for (i, base) in [Base::First, Base::Second, Base::Third]
        .into_iter()
        .enumerate()
    {
        let Some(runner) = &before.bases[i] else {
            break;
        };
        let to = [Destination::Second, Destination::Third, Destination::Home][i];
        event.delta.moves.push(RunnerMove {
            runner: runner.clone(),
            from: base,
            to,
        });
        if to == Destination::Home {
            event.delta.runs += 1;
            event.delta.credits.extend([
                credit(&runner.player_id, CreditKind::Run),
                credit(&id, CreditKind::Rbi),
            ]);
        }
    }
    if cause {
        event.protected_award = Some(ProtectedAwardFacts { batter_id: id });
    }
    event
}
fn hr(before: &GameState) -> Event {
    let mut event = blank(before);
    event.kind = EventKind::OutOfParkHomeRun;
    let id = batter(&event);
    event
        .delta
        .moves
        .push(batter_move(&event, Destination::Home));
    event.delta.credits.extend([
        credit(&id, CreditKind::PlateAppearance),
        credit(&id, CreditKind::Hit),
        credit(&id, CreditKind::Run),
        credit(&id, CreditKind::Rbi),
    ]);
    for (i, base) in [Base::First, Base::Second, Base::Third]
        .into_iter()
        .enumerate()
    {
        if let Some(runner) = &before.bases[i] {
            event.delta.moves.push(RunnerMove {
                runner: runner.clone(),
                from: base,
                to: Destination::Home,
            });
            event.delta.credits.extend([
                credit(&runner.player_id, CreditKind::Run),
                credit(&id, CreditKind::Rbi),
            ]);
        }
    }
    event.delta.runs = event.delta.moves.len() as u8;
    event
}
fn accepted(before: &GameState, event: Event) -> EventTransition {
    let snapshot = before.clone();
    let t = apply(before, event.clone()).unwrap();
    assert_eq!(before, &snapshot);
    assert_eq!(
        EventTransition::try_new(before.clone(), event, t.after().clone()).unwrap(),
        t
    );
    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
    t
}
fn named(id: &str, n: u32) -> Option<RbiContribution> {
    Some(RbiContribution {
        runs_batted_in: [(id.into(), n)].into(),
    })
}

#[test]
fn zero_legal_run_pa_has_independent_zero_without_hit_or_ab_inference() {
    let mut frames = 0;
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            for mask in 0..8 {
                for classified in [false, true] {
                    for reviewed in [false, true] {
                        for reverse in [false, true] {
                            let before = state(mask, outs, half);
                            let mut event = out(&before, classified);
                            let id = batter(&event);
                            if reviewed {
                                event.fielding_review = FieldingReview::NoMisplay;
                            }
                            for (i, base) in [Base::First, Base::Second, Base::Third]
                                .into_iter()
                                .enumerate()
                            {
                                if let Some(runner) = &before.bases[i] {
                                    event.delta.moves.push(RunnerMove {
                                        runner: runner.clone(),
                                        from: base,
                                        to: [
                                            Destination::First,
                                            Destination::Second,
                                            Destination::Third,
                                        ][i],
                                    });
                                }
                            }
                            if reverse {
                                event.delta.moves.reverse();
                                event.delta.credits.reverse();
                            }
                            let t = accepted(&before, event);
                            assert_eq!(t.protected_award_rbi(), None);
                            assert_eq!(t.rbi_contribution(), named(&id, 0));
                            let lines = derive_accounting(&[t]).unwrap();
                            let b = &lines.batters[half.offense()][&id];
                            assert_eq!(
                                (b.classified_rbi, b.credited_rbi, b.plate_appearances),
                                (Some(0), 0, 1)
                            );
                            assert_eq!(b.at_bats, classified.then_some(1));
                            assert_eq!(b.classified_hits, classified.then_some(0));
                            frames += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(frames, 384);
}

#[test]
fn scoring_award_cause_is_distinct_from_supplied_rbi_and_review() {
    for half in [Half::Top, Half::Bottom] {
        for hbp in [false, true] {
            for cause in [false, true] {
                let before = state(7, 1, half);
                let mut event = award(&before, cause, hbp);
                event.fielding_review = FieldingReview::NoMisplay;
                let id = batter(&event);
                let t = accepted(&before, event);
                assert_eq!(t.rbi_contribution(), cause.then(|| named(&id, 1).unwrap()));
                let lines = derive_accounting(&[t]).unwrap();
                assert_eq!(
                    (
                        lines.batters[half.offense()][&id].classified_rbi,
                        lines.batters[half.offense()][&id].credited_rbi
                    ),
                    (cause.then_some(1), 1)
                );
                assert_eq!(lines.pitchers[1 - half.offense()]["oldp"].runs_allowed, 1);
            }
        }
    }
}

#[test]
fn voided_home_crossing_is_zero_rbi_not_one_physical_crossing() {
    for half in [Half::Top, Half::Bottom] {
        let before = state(5, 2, half);
        let mut event = blank(&before);
        let id = batter(&event);
        let first = before.bases[0].as_ref().unwrap();
        let third = before.bases[2].as_ref().unwrap();
        event.delta.moves = vec![
            batter_move(&event, Destination::First),
            RunnerMove {
                runner: first.clone(),
                from: Base::First,
                to: Destination::Out,
            },
            RunnerMove {
                runner: third.clone(),
                from: Base::Third,
                to: Destination::Home,
            },
        ];
        event.delta.outs = 1;
        event.delta.third_out = Some(ThirdOut {
            runner_id: first.player_id.clone(),
            kind: ThirdOutKind::Force,
            home_before_out: vec![third.player_id.clone()],
        });
        event.delta.credits = vec![
            credit(&id, CreditKind::PlateAppearance),
            credit(&id, CreditKind::AtBat),
            credit(&id, CreditKind::NoHit),
        ];
        let t = accepted(&before, event);
        assert_eq!(t.after().score, [0, 0]);
        assert_eq!(t.rbi_contribution(), named(&id, 0));
        assert_eq!(
            derive_accounting(&[t]).unwrap().batters[half.offense()][&id].classified_rbi,
            Some(0)
        );
    }
}

#[test]
fn runner_only_scoring_does_not_create_current_batter_rbi_coverage() {
    let before = state(4, 1, Half::Top);
    let mut event = blank(&before);
    event.kind = EventKind::RunnerOnly(RunnerEvent::WildPitch);
    let id = batter(&event);
    let runner = before.bases[2].as_ref().unwrap();
    event.delta.moves = vec![RunnerMove {
        runner: runner.clone(),
        from: Base::Third,
        to: Destination::Home,
    }];
    event.delta.runs = 1;
    event.delta.credits = vec![credit(&runner.player_id, CreditKind::Run)];
    let t = accepted(&before, event);
    assert_eq!(t.after().teams[0].next_batter, 0);
    assert_eq!(t.rbi_contribution(), None);
    let lines = derive_accounting(&[t]).unwrap();
    assert!(!lines.batters[0].contains_key(&id));
    assert_eq!(
        (
            lines.batters[0][&runner.player_id].plate_appearances,
            lines.batters[0][&runner.player_id].classified_rbi
        ),
        (0, Some(0))
    );
}

#[test]
fn runner_scoring_and_pitcher_substitution_leave_known_batter_coverage_unchanged() {
    let mut before = state(7, 1, Half::Top);
    before.teams[1].bench.push("reliever".into());
    let first = award(&before, true, false);
    let id = batter(&first);
    let mut log = vec![accepted(&before, first)];
    let before = log.last().unwrap().after();
    let mut runner_event = blank(before);
    let current_batter = batter(&runner_event);
    runner_event.kind = EventKind::RunnerOnly(RunnerEvent::WildPitch);
    let runner = before.bases[2].as_ref().unwrap();
    runner_event.delta.moves = vec![RunnerMove {
        runner: runner.clone(),
        from: Base::Third,
        to: Destination::Home,
    }];
    runner_event.delta.runs = 1;
    runner_event.delta.credits = vec![credit(&runner.player_id, CreditKind::Run)];
    log.push(accepted(before, runner_event));
    let before = log.last().unwrap().after();
    let mut substitution = blank(before);
    substitution.kind = EventKind::Substitution;
    substitution.delta.changes = vec![bbsim_engine::PersonnelChange::Pitcher {
        team: bbsim_engine::TeamSide::Home,
        outgoing: before.teams[1].pitcher.clone(),
        incoming: "reliever".into(),
    }];
    let t = accepted(before, substitution);
    assert_eq!(t.rbi_contribution(), None);
    log.push(t);
    let lines = derive_accounting(&log).unwrap();
    assert_eq!(
        (
            lines.batters[0][&id].classified_rbi,
            lines.batters[0][&id].credited_rbi
        ),
        (Some(1), 1)
    );
    assert!(!lines.batters[0].contains_key(&current_batter));
    assert_eq!(lines.pitchers[1]["oldp"].runs_allowed, 2);
}

fn cycle_to(log: &mut Vec<EventTransition>, id: &str) {
    for _ in 0..30 {
        let before = log.last().unwrap().after();
        let next = &before.teams[before.half.offense()];
        if next.batting_order[next.next_batter as usize] == id {
            return;
        }
        let t = accepted(before, out(before, true));
        log.push(t);
    }
    panic!("literal K cycle did not return to actor");
}

#[test]
fn unknown_rbi_is_irreversible_and_actor_local_across_contiguous_pas() {
    for unknown_first in [false, true] {
        let before = state(7, 1, Half::Top);
        let first = if unknown_first {
            hr(&before)
        } else {
            award(&before, true, false)
        };
        let id = batter(&first);
        let mut log = vec![accepted(&before, first)];
        cycle_to(&mut log, &id);
        let before = log.last().unwrap().after();
        let second = if unknown_first {
            out(before, true)
        } else {
            hr(before)
        };
        log.push(accepted(before, second));
        let lines = derive_accounting(&log).unwrap();
        assert_eq!(lines.batters[0][&id].plate_appearances, 2);
        assert_eq!(lines.batters[0][&id].classified_rbi, None);
        assert_eq!(lines.batters[0]["a1"].classified_rbi, Some(0));
        assert_eq!(
            lines.batters[0][&id].credited_rbi,
            if unknown_first { 4 } else { 2 }
        );
    }
}

#[test]
fn covered_positive_then_zero_accumulates_without_requiring_specific_zero_cause() {
    let before = state(7, 1, Half::Top);
    let first = award(&before, true, true);
    let id = batter(&first);
    let mut log = vec![accepted(&before, first)];
    cycle_to(&mut log, &id);
    let before = log.last().unwrap().after();
    let zero = accepted(before, award(before, false, false));
    assert_eq!(zero.protected_award_rbi(), None);
    assert_eq!(zero.rbi_contribution(), named(&id, 0));
    log.push(zero);
    let lines = derive_accounting(&log).unwrap();
    assert_eq!(
        (
            lines.batters[0][&id].classified_rbi,
            lines.batters[0][&id].credited_rbi
        ),
        (Some(1), 1)
    );
}

#[test]
fn positive_hit_or_hr_and_supplied_rbi_do_not_fill_cause_coverage() {
    for review in [FieldingReview::Unknown, FieldingReview::NoMisplay] {
        for kind in [EventKind::PlateAppearance, EventKind::OutOfParkHomeRun] {
            for omitted in [false, true] {
                let before = state(7, 1, Half::Top);
                let mut event = hr(&before);
                event.kind = kind;
                event.fielding_review = review;
                if omitted {
                    event.delta.credits.retain(|c| c.kind != CreditKind::Rbi);
                }
                let id = batter(&event);
                let t = accepted(&before, event);
                assert_eq!(t.rbi_contribution(), None);
                let lines = derive_accounting(&[t]).unwrap();
                assert_eq!(
                    (
                        lines.batters[0][&id].credited_rbi,
                        lines.batters[0][&id].classified_rbi
                    ),
                    (if omitted { 0 } else { 4 }, None)
                );
            }
        }
    }
}
