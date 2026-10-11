use bbsim_engine::{
    accounting::derive_accounting, apply, Base, Credit, CreditKind, Delta, Destination, Event,
    EventKind, EventTransition, FieldingReview, GameState, Half, Participants, Runner, RunnerEvent,
    RunnerMove, TeamState, ThirdOut, ThirdOutKind, TransitionError,
};

fn state(half: Half, mask: u8, outs: u8) -> GameState {
    let teams = ["a", "h"].map(|prefix| TeamState {
        batting_order: std::array::from_fn(|i| format!("{prefix}{i}")),
        next_batter: 0,
        pitcher: format!("{prefix}p"),
        dh_slot: Some(8),
        dh_must_bat: false,
        bench: vec![],
        removed: vec![],
    });
    let offense = half.offense();
    let bases = std::array::from_fn(|i| {
        (mask & (1 << i) != 0).then(|| Runner {
            player_id: teams[offense].batting_order[3 + i].clone(),
            responsible_pitcher: teams[1 - offense].pitcher.clone(),
        })
    });
    GameState {
        inning: 3,
        half,
        outs,
        score: [0, 0],
        teams,
        bases,
        over: false,
    }
}
fn credit(id: &str, kind: CreditKind) -> Credit {
    Credit {
        player_id: id.into(),
        kind,
    }
}
fn event(before: &GameState, kind: EventKind, review: FieldingReview) -> Event {
    let offense = before.half.offense();
    Event {
        kind,
        participants: Participants {
            batter: Some(
                before.teams[offense].batting_order[before.teams[offense].next_batter as usize]
                    .clone(),
            ),
            pitcher: before.teams[1 - offense].pitcher.clone(),
        },
        fielding_review: review,
        delta: Delta {
            moves: vec![],
            outs: 0,
            runs: 0,
            credits: vec![],
            changes: vec![],
            third_out: None,
        },
    }
}
fn steal(before: &GameState, bases: &[usize], review: FieldingReview, error: bool) -> Event {
    let mut action = event(
        before,
        EventKind::RunnerOnly(RunnerEvent::StealAttempt),
        review,
    );
    let fielder = &before.teams[1 - before.half.offense()].batting_order[0];
    for &base in bases {
        let runner = before.bases[base].clone().unwrap();
        action
            .delta
            .credits
            .push(credit(&runner.player_id, CreditKind::CaughtStealing));
        action
            .delta
            .credits
            .push(credit(fielder, CreditKind::Putout));
        action.delta.moves.push(RunnerMove {
            runner,
            from: [Base::First, Base::Second, Base::Third][base],
            to: Destination::Out,
        });
        action.delta.outs += 1;
    }
    if error {
        action
            .delta
            .credits
            .push(credit(fielder, CreditKind::Error));
    }
    action
}
fn reviews() -> [(FieldingReview, bool, TransitionError); 3] {
    [
        (
            FieldingReview::NoMisplay,
            false,
            TransitionError::IllegalCredit,
        ),
        (
            FieldingReview::Unknown,
            false,
            TransitionError::UnsupportedRule,
        ),
        (
            FieldingReview::Unknown,
            true,
            TransitionError::UnsupportedRule,
        ),
    ]
}
fn duplicate_cases() -> Vec<(GameState, Event, TransitionError)> {
    let mut cases = vec![];
    for half in [Half::Top, Half::Bottom] {
        for mask in 1..8 {
            for base in 0..3 {
                if mask & (1 << base) == 0 {
                    continue;
                }
                for outs in 0..3 {
                    let before = state(half, mask, outs);
                    for (review, error, expected) in reviews() {
                        for extra in 1..=2 {
                            for at_front in [false, true] {
                                let mut action = steal(&before, &[base], review, error);
                                let cs = action.delta.credits[0].clone();
                                for _ in 0..extra {
                                    if at_front {
                                        action.delta.credits.insert(0, cs.clone());
                                    } else {
                                        action.delta.credits.push(cs.clone());
                                    }
                                }
                                cases.push((before.clone(), action, expected.clone()));
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(cases.len(), 864);
    cases
}

#[test]
fn reducer_rejects_same_actor_cs_multiplicity_without_mutating_caller() {
    for (before, action, expected) in duplicate_cases() {
        let unchanged = before.clone();
        assert_eq!(apply(&before, action), Err(expected));
        assert_eq!(before, unchanged);
    }
}
#[test]
fn unknown_cs_multiplicity_is_unsupported_not_proven_illegal_from_absent_errors() {
    for (before, action, expected) in duplicate_cases()
        .into_iter()
        .filter(|(_, action, _)| action.fielding_review == FieldingReview::Unknown)
    {
        assert_eq!(expected, TransitionError::UnsupportedRule);
        assert_eq!(apply(&before, action), Err(expected));
    }
}

#[test]
fn constructor_distinguishes_reviewed_contradiction_from_unknown_occurrences() {
    for (before, action, expected) in duplicate_cases() {
        let mut singleton = action.clone();
        let mut retained = false;
        singleton.delta.credits.retain(|c| {
            if c.kind != CreditKind::CaughtStealing {
                return true;
            }
            if retained {
                return false;
            }
            retained = true;
            true
        });
        let valid = apply(&before, singleton).unwrap();
        assert_eq!(
            EventTransition::try_new(before, action, valid.after().clone()),
            Err(expected)
        );
    }
}

#[test]
fn codec_rejects_same_actor_cs_but_parses_distinct_actor_control() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..2 {
            let before = state(half, 3, outs);
            for (review, error, expected) in reviews() {
                for reversed in [false, true] {
                    let mut action = steal(&before, &[0, 1], review, error);
                    if reversed {
                        action.delta.credits.reverse();
                    }
                    let valid = apply(&before, action).unwrap();
                    let mut bytes = valid.to_bytes();
                    assert_eq!(EventTransition::from_bytes(&bytes), Ok(valid));
                    let first = &before.bases[0].as_ref().unwrap().player_id;
                    let second = &before.bases[1].as_ref().unwrap().player_id;
                    assert_eq!(first.len(), second.len());
                    // BBE11 encodes Credit as UTF-8 length/id, then tag 7 (CS).
                    // Short fixture IDs/pitcher strings cannot alias this whole unit.
                    let mut marker = (second.len() as u64).to_le_bytes().to_vec();
                    marker.extend_from_slice(second.as_bytes());
                    marker.push(7);
                    let positions: Vec<_> = bytes
                        .windows(marker.len())
                        .enumerate()
                        .filter_map(|(i, window)| (window == marker).then_some(i))
                        .collect();
                    assert_eq!(positions.len(), 1);
                    let start = positions[0] + 8;
                    let original = bytes.clone();
                    bytes[start..start + second.len()].copy_from_slice(first.as_bytes());
                    assert_eq!(&bytes[..start], &original[..start]);
                    assert_eq!(
                        &bytes[start + second.len()..],
                        &original[start + second.len()..]
                    );
                    assert_eq!(EventTransition::from_bytes(&bytes), Err(expected.clone()));
                }
            }
        }
    }
}

fn check_steal(before: &GameState, action: Event) {
    let offense = before.half.offense();
    let ids: Vec<_> = action
        .delta
        .moves
        .iter()
        .map(|m| m.runner.player_id.clone())
        .collect();
    let count = ids.len() as u32;
    let next = apply(before, action).unwrap();
    assert_eq!(next.after().score, before.score);
    assert_eq!(
        next.after().teams[offense].next_batter,
        before.teams[offense].next_batter
    );
    assert_eq!(
        EventTransition::from_bytes(&next.to_bytes()),
        Ok(next.clone())
    );
    let folded = derive_accounting(&[next]).unwrap();
    for id in ids {
        assert_eq!(folded.batters[offense][&id].credited_caught_stealing, 1);
    }
    assert_eq!(
        folded.batters[offense]
            .values()
            .map(|b| b.plate_appearances)
            .sum::<u32>(),
        0
    );
    let pitcher = &folded.pitchers[1 - offense][&before.teams[1 - offense].pitcher];
    assert_eq!(pitcher.outs_recorded, count);
    assert_eq!(pitcher.batters_faced, 0);
    // Multiple putouts by one fielder are units, not duplicate data to discard.
    assert_eq!(
        folded.fielders[1 - offense][&before.teams[1 - offense].batting_order[0]].credited_putouts,
        count
    );
}
#[test]
fn singletons_and_distinct_retired_actors_preserve_cs_and_repeated_putouts() {
    for half in [Half::Top, Half::Bottom] {
        for mask in 1..8 {
            let occupied: Vec<_> = (0..3).filter(|i| mask & (1 << i) != 0).collect();
            for (review, error, _) in reviews() {
                for outs in 0..3 {
                    let before = state(half, mask, outs);
                    for &base in &occupied {
                        check_steal(&before, steal(&before, &[base], review, error));
                    }
                    if occupied.len() == 2 && outs < 2 {
                        for reverse_moves in [false, true] {
                            for reverse_credits in [false, true] {
                                let mut action = steal(&before, &occupied, review, error);
                                if reverse_moves {
                                    action.delta.moves.reverse();
                                }
                                if reverse_credits {
                                    action.delta.credits.reverse();
                                }
                                check_steal(&before, action);
                            }
                        }
                    }
                }
            }
        }
    }
}
fn pa(before: &GameState, walk: bool) -> Event {
    let mut action = event(
        before,
        EventKind::PlateAppearance,
        FieldingReview::NoMisplay,
    );
    let batter = action.participants.batter.clone().unwrap();
    action.delta.moves.push(RunnerMove {
        runner: Runner {
            player_id: batter.clone(),
            responsible_pitcher: action.participants.pitcher.clone(),
        },
        from: Base::Plate,
        to: if walk {
            Destination::First
        } else {
            Destination::Out
        },
    });
    action.delta.outs = u8::from(!walk);
    action
        .delta
        .credits
        .push(credit(&batter, CreditKind::PlateAppearance));
    action.delta.credits.push(credit(
        &batter,
        if walk {
            CreditKind::Walk
        } else {
            CreditKind::Strikeout
        },
    ));
    if !walk {
        action
            .delta
            .credits
            .push(credit(&batter, CreditKind::AtBat));
    }
    action
}
#[test]
fn strikeout_cs_keeps_participation_and_boundary_attribution_but_rejects_multiplicity() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..2 {
            let before = state(half, 1, outs);
            let offense = half.offense();
            let runner = &before.bases[0].as_ref().unwrap().player_id;
            for (review, error, expected) in reviews() {
                let mut action = pa(&before, false);
                let attempt = steal(&before, &[0], review, error);
                action.fielding_review = review;
                action.delta.moves.extend(attempt.delta.moves);
                action.delta.credits.extend(attempt.delta.credits);
                action.delta.outs = 2;
                if outs == 1 {
                    action.delta.third_out = Some(ThirdOut {
                        runner_id: runner.clone(),
                        kind: ThirdOutKind::Timing,
                        home_before_out: vec![],
                    });
                }
                let valid = apply(&before, action.clone()).unwrap();
                assert_eq!(
                    EventTransition::from_bytes(&valid.to_bytes()),
                    Ok(valid.clone())
                );
                let folded = derive_accounting(std::slice::from_ref(&valid)).unwrap();
                let batter = &folded.batters[offense][action.participants.batter.as_ref().unwrap()];
                assert_eq!(
                    (
                        batter.plate_appearances,
                        batter.at_bats,
                        batter.credited_strikeouts
                    ),
                    (1, Some(1), 1)
                );
                assert_eq!(folded.batters[offense][runner].credited_caught_stealing, 1);
                let pitcher = &folded.pitchers[1 - offense][&action.participants.pitcher];
                assert_eq!((pitcher.batters_faced, pitcher.outs_recorded), (1, 2));
                assert_eq!(valid.after().teams[offense].next_batter, 1);
                if outs == 1 {
                    assert_ne!(valid.after().half, before.half);
                }
                action
                    .delta
                    .credits
                    .push(credit(runner, CreditKind::CaughtStealing));
                assert_eq!(apply(&before, action.clone()), Err(expected.clone()));
                assert_eq!(
                    EventTransition::try_new(before.clone(), action, valid.after().clone()),
                    Err(expected)
                );
            }
        }
    }
}
#[test]
fn same_active_runner_can_reach_again_and_receive_cs_in_a_later_event() {
    for half in [Half::Top, Half::Bottom] {
        let before = state(half, 1, 0);
        let offense = half.offense();
        let id = before.bases[0].as_ref().unwrap().player_id.clone();
        let first = apply(
            &before,
            steal(&before, &[0], FieldingReview::NoMisplay, false),
        )
        .unwrap();
        let mut current = first.after().clone();
        let mut log = vec![first];
        for _ in 0..6 {
            let next = apply(&current, pa(&current, false)).unwrap();
            current = next.after().clone();
            log.push(next);
        }
        assert_eq!(current.half, half);
        assert_eq!(
            current.teams[offense].batting_order[current.teams[offense].next_batter as usize],
            id
        );
        let walk = apply(&current, pa(&current, true)).unwrap();
        current = walk.after().clone();
        log.push(walk);
        let second = apply(
            &current,
            steal(&current, &[0], FieldingReview::Unknown, false),
        )
        .unwrap();
        log.push(second);
        for next in &log {
            assert_eq!(
                EventTransition::from_bytes(&next.to_bytes()),
                Ok(next.clone())
            );
        }
        let folded = derive_accounting(&log).unwrap();
        assert_eq!(folded.batters[offense][&id].credited_caught_stealing, 2);
        assert_eq!(folded.batters[offense][&id].credited_walks, 1);
        assert_eq!(
            folded
                .pitchers
                .iter()
                .flat_map(|p| p.values())
                .map(|p| p.outs_recorded)
                .sum::<u32>(),
            8
        );
        assert_eq!(
            folded
                .pitchers
                .iter()
                .flat_map(|p| p.values())
                .map(|p| p.batters_faced)
                .sum::<u32>(),
            7
        );
    }
}
#[test]
fn two_run_home_run_preserves_two_rbi_units_for_one_batter() {
    for half in [Half::Top, Half::Bottom] {
        let before = state(half, 1, 0);
        let offense = half.offense();
        let mut action = event(
            &before,
            EventKind::OutOfParkHomeRun,
            FieldingReview::NoMisplay,
        );
        let batter = action.participants.batter.clone().unwrap();
        let runner = before.bases[0].clone().unwrap();
        action.delta.moves = vec![
            RunnerMove {
                runner: Runner {
                    player_id: batter.clone(),
                    responsible_pitcher: action.participants.pitcher.clone(),
                },
                from: Base::Plate,
                to: Destination::Home,
            },
            RunnerMove {
                runner: runner.clone(),
                from: Base::First,
                to: Destination::Home,
            },
        ];
        action.delta.runs = 2;
        action.delta.credits = vec![
            credit(&batter, CreditKind::PlateAppearance),
            credit(&batter, CreditKind::Hit),
            credit(&batter, CreditKind::Run),
            credit(&runner.player_id, CreditKind::Run),
            credit(&batter, CreditKind::Rbi),
            credit(&batter, CreditKind::Rbi),
        ];
        let next = apply(&before, action).unwrap();
        assert_eq!(
            EventTransition::from_bytes(&next.to_bytes()),
            Ok(next.clone())
        );
        let folded = derive_accounting(&[next]).unwrap();
        assert_eq!(folded.batters[offense][&batter].credited_rbi, 2);
        assert_eq!(
            folded.pitchers[1 - offense][&before.teams[1 - offense].pitcher].runs_allowed,
            2
        );
    }
}
