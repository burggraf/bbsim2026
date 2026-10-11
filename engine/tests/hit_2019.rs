use bbsim_engine::accounting::derive_accounting;
use bbsim_engine::{
    apply, Base, Credit, CreditKind, Delta, Destination, Event, EventKind, EventTransition,
    FieldingReview, GameState, Half, Participants, Runner, RunnerMove, TeamState, ThirdOut,
    ThirdOutKind, TransitionError,
};

fn state(mask: u8, half: Half) -> GameState {
    let teams = std::array::from_fn(|side| {
        let prefix = ["away", "home"][side];
        TeamState {
            batting_order: std::array::from_fn(|slot| format!("{prefix}{slot}")),
            next_batter: 0,
            pitcher: format!("{prefix}_pitcher"),
            dh_slot: Some(8),
            dh_must_bat: false,
            bench: vec![],
            removed: vec![],
        }
    });
    let offense = half.offense();
    let bases = std::array::from_fn(|base| {
        (mask & (1 << base) != 0).then(|| Runner {
            player_id: teams[offense].batting_order[5 + base].clone(),
            responsible_pitcher: teams[1 - offense].pitcher.clone(),
        })
    });
    GameState {
        inning: 2,
        half,
        outs: 2,
        score: [0, 0],
        over: false,
        teams,
        bases,
    }
}

fn force_event(before: &GameState, out_base: usize, reverse: bool) -> Event {
    let offense = before.half.offense();
    let batter = before.teams[offense].batting_order[0].clone();
    let pitcher = before.teams[1 - offense].pitcher.clone();
    let runner_id = before.bases[out_base].as_ref().unwrap().player_id.clone();
    let mut moves = vec![RunnerMove {
        runner: Runner {
            player_id: batter.clone(),
            responsible_pitcher: pitcher.clone(),
        },
        from: Base::Plate,
        to: Destination::First,
    }];
    for base in 0..=out_base {
        moves.push(RunnerMove {
            runner: before.bases[base].clone().unwrap(),
            from: [Base::First, Base::Second, Base::Third][base],
            to: if base == out_base {
                Destination::Out
            } else {
                [Destination::Second, Destination::Third][base]
            },
        });
    }
    if reverse {
        moves.reverse();
    }
    Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(batter.clone()),
            pitcher,
        },
        delta: Delta {
            moves,
            outs: 1,
            runs: 0,
            credits: vec![
                Credit {
                    player_id: batter.clone(),
                    kind: CreditKind::PlateAppearance,
                },
                Credit {
                    player_id: batter,
                    kind: CreditKind::AtBat,
                },
            ],
            changes: vec![],
            third_out: Some(ThirdOut {
                runner_id,
                kind: ThirdOutKind::Force,
                home_before_out: vec![],
            }),
        },
        fielding_review: FieldingReview::Unknown,
        two_out_contact: None,
    }
}

fn cases() -> Vec<(GameState, Event)> {
    let mut cases = vec![];
    for half in [Half::Top, Half::Bottom] {
        for mask in 1..8 {
            for out_base in 0..3 {
                let chain = (1 << (out_base + 1)) - 1;
                if mask & chain != chain {
                    continue;
                }
                for reverse in [false, true] {
                    let before = state(mask, half);
                    let event = force_event(&before, out_base, reverse);
                    cases.push((before, event));
                }
            }
        }
    }
    assert_eq!(cases.len(), 28);
    cases
}

fn with_hit(mut event: Event) -> Event {
    assert_eq!(event.delta.credits[1].kind, CreditKind::AtBat);
    event.delta.credits[1].kind = CreditKind::Hit;
    event
}

fn assert_line(transition: &EventTransition, hits: u32) {
    let offense = transition.before().half.offense();
    let batter = transition.event().participants.batter.as_ref().unwrap();
    let pitcher = &transition.event().participants.pitcher;
    let line = derive_accounting(std::slice::from_ref(transition)).unwrap();
    assert_eq!(line.batters[offense][batter].plate_appearances, 1);
    assert_eq!(line.batters[offense][batter].at_bats, Some(1));
    assert_eq!(line.batters[offense][batter].credited_hits, hits);
    assert_eq!(line.pitchers[1 - offense][pitcher].batters_faced, 1);
    assert_eq!(line.pitchers[1 - offense][pitcher].at_bats_against, Some(1));
    assert_eq!(line.pitchers[1 - offense][pitcher].outs_recorded, 1);
    assert_eq!(
        line.pitchers[1 - offense][pitcher].credited_hits_allowed,
        hits
    );
    assert_eq!(line.runs, [0, 0]);
    assert_eq!(transition.after().bases, [None, None, None]);
    assert_eq!(
        EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
        *transition
    );
}

#[test]
fn reducer_rejects_hit_on_explicit_force_third_out_for_every_chain() {
    for (before, event) in cases() {
        let valid = apply(&before, event.clone()).unwrap();
        assert_line(&valid, 0);
        assert!(matches!(
            apply(&before, with_hit(event)),
            Err(TransitionError::IllegalCredit)
        ));
    }
}

#[test]
fn constructor_rejects_hit_on_explicit_force_third_out_for_every_chain() {
    for (before, event) in cases() {
        let valid = apply(&before, event.clone()).unwrap();
        assert!(matches!(
            EventTransition::try_new(before, with_hit(event), valid.after().clone()),
            Err(TransitionError::IllegalCredit)
        ));
    }
}

fn unique_offset(bytes: &[u8], marker: &[u8]) -> usize {
    let matches: Vec<_> = bytes
        .windows(marker.len())
        .enumerate()
        .filter_map(|(offset, window)| (window == marker).then_some(offset))
        .collect();
    assert_eq!(matches.len(), 1, "wire marker must be unique");
    matches[0]
}

#[test]
fn codec_rejects_hit_credit_forgery_on_force_but_parses_nonforce_control() {
    for half in [Half::Top, Half::Bottom] {
        for reverse in [false, true] {
            let before = state(1, half);
            let valid = apply(&before, force_event(&before, 0, reverse)).unwrap();
            let mut bytes = valid.to_bytes();
            assert_eq!(EventTransition::from_bytes(&bytes).unwrap(), valid);
            // BBE11: final AtBat (12), empty changes, then Some(third_out).
            let batter = valid.event().participants.batter.as_ref().unwrap();
            let mut marker = (batter.len() as u64).to_le_bytes().to_vec();
            marker.extend_from_slice(batter.as_bytes());
            marker.push(12);
            marker.extend_from_slice(&[0; 8]);
            marker.push(1);
            let offset = unique_offset(&bytes, &marker) + 8 + batter.len();
            bytes[offset] = 3; // Hit: structurally valid, only credit changed.
            assert_eq!(
                EventTransition::from_bytes(&bytes),
                Err(TransitionError::IllegalCredit)
            );

            // Prove valid parsing, not corruption detection: a supplied
            // nonforce tag beyond second can coexist with a genuine single.
            let runner = &valid.event().delta.third_out.as_ref().unwrap().runner_id;
            let mut marker = vec![1];
            marker.extend_from_slice(&(runner.len() as u64).to_le_bytes());
            marker.extend_from_slice(runner.as_bytes());
            marker.push(1); // Force.
            let offset = unique_offset(&bytes, &marker) + 1 + 8 + runner.len();
            bytes[offset] = 2; // Timing.
            let nonforce = EventTransition::from_bytes(&bytes).unwrap();
            assert_line(&nonforce, 1);
        }
    }
}

#[test]
fn uncaught_third_strike_can_retire_a_forced_runner_without_a_hit() {
    for (before, mut event) in cases() {
        // Two outs makes the batter eligible despite occupied first; the
        // forced runner, not the struck-out batter, supplies the play out.
        event.delta.credits[1].kind = CreditKind::Strikeout;
        let valid = apply(&before, event).unwrap();
        assert_line(&valid, 0);
        let line = derive_accounting(std::slice::from_ref(&valid)).unwrap();
        let offense = before.half.offense();
        assert_eq!(
            line.batters[offense][&before.teams[offense].batting_order[0]].credited_strikeouts,
            1
        );
        assert_eq!(
            line.pitchers[1 - offense][&before.teams[1 - offense].pitcher].credited_strikeouts,
            1
        );
    }
}

#[test]
fn supplied_hits_with_other_retirements_are_not_inferred_as_force_outs() {
    for half in [Half::Top, Half::Bottom] {
        for reverse in [false, true] {
            let before = state(1, half);
            let mut event = with_hit(force_event(&before, 0, reverse));
            event.delta.third_out.as_mut().unwrap().kind = ThirdOutKind::Timing;
            let valid = apply(&before, event).unwrap();
            assert_line(&valid, 1);
        }
        let before = state(0, half);
        let batter = before.teams[half.offense()].batting_order[0].clone();
        for kind in [
            Some(ThirdOutKind::BatterAfterFirstTiming),
            Some(ThirdOutKind::BatterBeforeFirst),
            None,
        ] {
            // A supplied Hit may remain legal after first, or under the
            // scorer's batter-interference exception (9.05(b)(5)). A generic
            // before-first label is not an explicit missed-first appeal.
            let event = Event {
                kind: EventKind::PlateAppearance,
                participants: Participants {
                    batter: Some(batter.clone()),
                    pitcher: before.teams[1 - half.offense()].pitcher.clone(),
                },
                delta: Delta {
                    moves: vec![RunnerMove {
                        runner: Runner {
                            player_id: batter.clone(),
                            responsible_pitcher: before.teams[1 - half.offense()].pitcher.clone(),
                        },
                        from: Base::Plate,
                        to: Destination::Out,
                    }],
                    outs: 1,
                    runs: 0,
                    credits: vec![
                        Credit {
                            player_id: batter.clone(),
                            kind: CreditKind::PlateAppearance,
                        },
                        Credit {
                            player_id: batter.clone(),
                            kind: CreditKind::Hit,
                        },
                    ],
                    changes: vec![],
                    third_out: kind.map(|kind| ThirdOut {
                        runner_id: batter.clone(),
                        kind,
                        home_before_out: vec![],
                    }),
                },
                fielding_review: FieldingReview::Unknown,
                two_out_contact: None,
            };
            let valid = apply(&before, event).unwrap();
            assert_line(&valid, 1);
        }
    }
}
