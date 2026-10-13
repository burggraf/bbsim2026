use bbsim_engine::{
    accounting::derive_accounting, apply, Base, Credit, CreditKind, Delta, Destination, Event,
    EventKind, EventTransition, FieldingReview, GameState, Half, Participants, Runner, RunnerMove,
    TeamState, ThirdOut, ThirdOutKind, TransitionError,
};

fn state(half: Half, mask: u8, outs: u8) -> GameState {
    let teams = ["away", "home"].map(|prefix| TeamState {
        batting_order: std::array::from_fn(|i| format!("{prefix}{i}")),
        next_batter: 0,
        pitcher: format!("{prefix}_pitcher"),
        dh_slot: Some(8),
        dh_must_bat: false,
        bench: vec![],
        removed: vec![],
    });
    let offense = half.offense();
    let bases = std::array::from_fn(|i| {
        (mask & (1 << i) != 0).then(|| Runner {
            player_id: teams[offense].batting_order[6 + i].clone(),
            responsible_pitcher: format!("prior_pitcher_{i}"),
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

fn safe_reach(before: &GameState, destination: Destination) -> (Event, GameState) {
    let offense = before.half.offense();
    let batter = before.teams[offense].batting_order[0].clone();
    let pitcher = before.teams[1 - offense].pitcher.clone();
    let batter_runner = Runner {
        player_id: batter.clone(),
        responsible_pitcher: pitcher.clone(),
    };
    let mut moves = vec![RunnerMove {
        runner: batter_runner.clone(),
        from: Base::Plate,
        to: destination,
    }];
    let mut credits = vec![
        Credit {
            player_id: batter.clone(),
            kind: CreditKind::PlateAppearance,
        },
        Credit {
            player_id: batter.clone(),
            kind: CreditKind::Strikeout,
        },
        Credit {
            player_id: before.teams[1 - offense].batting_order[1].clone(),
            kind: CreditKind::PassedBall,
        },
    ];
    // On this same play the existing runners advance home. This frees first
    // in the after-state, but cannot change third-strike eligibility before it.
    for (i, runner) in before.bases.iter().enumerate() {
        if let Some(runner) = runner {
            moves.push(RunnerMove {
                runner: runner.clone(),
                from: [Base::First, Base::Second, Base::Third][i],
                to: Destination::Home,
            });
            credits.push(Credit {
                player_id: runner.player_id.clone(),
                kind: CreditKind::Run,
            });
        }
    }
    if destination == Destination::Home {
        credits.push(Credit {
            player_id: batter.clone(),
            kind: CreditKind::Run,
        });
    }
    let runs = credits.iter().filter(|c| c.kind == CreditKind::Run).count() as u8;
    let event = Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(batter),
            pitcher,
        },
        fielding_review: FieldingReview::Unknown,
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
        delta: Delta {
            moves,
            outs: 0,
            runs,
            credits,
            changes: vec![],
            third_out: None,
        },
    };
    let mut after = before.clone();
    after.teams[offense].next_batter = 1;
    after.score[offense] = u16::from(runs);
    after.bases = [None, None, None];
    match destination {
        Destination::First => after.bases[0] = Some(batter_runner),
        Destination::Second => after.bases[1] = Some(batter_runner),
        Destination::Third => after.bases[2] = Some(batter_runner),
        Destination::Home => {}
        Destination::Out => unreachable!(),
    }
    (event, after)
}

fn check_safe_reach(constructor: bool) {
    for half in [Half::Top, Half::Bottom] {
        for mask in 0..8 {
            for outs in 0..3 {
                for destination in [
                    Destination::First,
                    Destination::Second,
                    Destination::Third,
                    Destination::Home,
                ] {
                    let before = state(half, mask, outs);
                    let (event, after) = safe_reach(&before, destination);
                    let result = if constructor {
                        EventTransition::try_new(before.clone(), event, after.clone())
                    } else {
                        apply(&before, event)
                    };
                    if mask & 1 != 0 && outs < 2 {
                        assert_eq!(
                            result,
                            Err(TransitionError::IllegalStateChange),
                            "{half:?} mask={mask} outs={outs} to={destination:?}"
                        );
                    } else {
                        let transition = result.unwrap();
                        assert_eq!(transition.after(), &after);
                        assert_eq!(
                            EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
                            transition
                        );
                        let accounting =
                            derive_accounting(std::slice::from_ref(&transition)).unwrap();
                        let offense = half.offense();
                        let batter = &before.teams[offense].batting_order[0];
                        let pitcher = &before.teams[1 - offense].pitcher;
                        assert_eq!(accounting.batters[offense][batter].plate_appearances, 1);
                        assert_eq!(accounting.batters[offense][batter].credited_strikeouts, 1);
                        assert_eq!(accounting.batters[offense][batter].at_bats, Some(1));
                        assert_eq!(accounting.pitchers[1 - offense][pitcher].batters_faced, 1);
                        assert_eq!(accounting.pitchers[1 - offense][pitcher].outs_recorded, 0);
                        assert_eq!(accounting.runs[offense], u32::from(after.score[offense]));
                    }
                }
            }
        }
    }
}

#[test]
fn reducer_checks_uncaught_third_strike_eligibility_for_all_masks_outs_and_halves() {
    check_safe_reach(false);
}

#[test]
fn constructor_checks_uncaught_third_strike_eligibility_for_all_masks_outs_and_halves() {
    check_safe_reach(true);
}

#[test]
fn retired_strikeouts_remain_legal_and_keep_pre_boundary_fielding_attribution() {
    for half in [Half::Top, Half::Bottom] {
        for mask in 0..8 {
            for outs in 0..3 {
                let before = state(half, mask, outs);
                let (mut event, _) = safe_reach(&before, Destination::First);
                event.delta.moves.truncate(1);
                event.delta.moves[0].to = Destination::Out;
                event.delta.outs = 1;
                event.delta.runs = 0;
                event.delta.credits.retain(|c| {
                    matches!(c.kind, CreditKind::PlateAppearance | CreditKind::Strikeout)
                });
                let catcher = before.teams[1 - half.offense()].batting_order[1].clone();
                event.delta.credits.push(Credit {
                    player_id: catcher.clone(),
                    kind: CreditKind::Putout,
                });
                if outs == 2 {
                    event.delta.third_out = Some(ThirdOut {
                        runner_id: event.participants.batter.clone().unwrap(),
                        kind: ThirdOutKind::BatterBeforeFirst,
                        home_before_out: vec![],
                    });
                }
                // A strikeout with a simultaneous steal remains a legal out,
                // including when first was occupied below two outs.
                if mask == 1 && outs < 2 {
                    let runner = before.bases[0].clone().unwrap();
                    event.delta.credits.push(Credit {
                        player_id: runner.player_id.clone(),
                        kind: CreditKind::StolenBase,
                    });
                    event.delta.moves.push(RunnerMove {
                        runner,
                        from: Base::First,
                        to: Destination::Second,
                    });
                }
                let transition = apply(&before, event.clone()).unwrap();
                assert_eq!(
                    EventTransition::try_new(before.clone(), event, transition.after().clone())
                        .unwrap(),
                    transition
                );
                assert_eq!(
                    EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
                    transition
                );
                let accounting = derive_accounting(std::slice::from_ref(&transition)).unwrap();
                let offense = half.offense();
                assert_eq!(
                    accounting.pitchers[1 - offense][&before.teams[1 - offense].pitcher]
                        .outs_recorded,
                    1
                );
                assert_eq!(
                    accounting.pitchers[1 - offense][&before.teams[1 - offense].pitcher]
                        .batters_faced,
                    1
                );
                assert_eq!(
                    accounting.fielders[1 - offense][&catcher].credited_putouts,
                    1
                );
                assert!(accounting.fielders[offense].is_empty());
                if outs == 2 {
                    assert_ne!(transition.after().half, half);
                    assert_eq!(transition.after().outs, 0);
                    assert!(transition.after().bases.iter().all(Option::is_none));
                } else if mask == 1 {
                    assert!(transition.after().bases[0].is_none());
                    assert_eq!(transition.after().bases[1], before.bases[0]);
                    assert_eq!(
                        accounting.batters[offense][&before.bases[0].as_ref().unwrap().player_id]
                            .credited_steals,
                        1
                    );
                }
            }
        }
    }
}

#[test]
fn automatic_strikeout_cannot_be_relabelled_as_a_later_after_first_third_out() {
    for half in [Half::Top, Half::Bottom] {
        let before = state(half, 1, 1);
        let (mut event, _) = safe_reach(&before, Destination::First);
        let batter = event.participants.batter.clone().unwrap();
        event.delta.moves[0].to = Destination::Out;
        event.delta.moves[1].to = Destination::Out;
        event.delta.outs = 2;
        event.delta.runs = 0;
        event
            .delta
            .credits
            .retain(|c| matches!(c.kind, CreditKind::PlateAppearance | CreditKind::Strikeout));
        event.delta.third_out = Some(ThirdOut {
            runner_id: batter,
            kind: ThirdOutKind::BatterAfterFirstTiming,
            home_before_out: vec![],
        });
        let mut after = before.clone();
        after.teams[half.offense()].next_batter = 1;
        after.bases = [None, None, None];
        after.outs = 0;
        after.half = if half == Half::Top {
            Half::Bottom
        } else {
            Half::Top
        };
        after.inning += u16::from(half == Half::Bottom);
        assert_eq!(
            apply(&before, event.clone()),
            Err(TransitionError::IllegalStateChange)
        );
        assert_eq!(
            EventTransition::try_new(before, event, after),
            Err(TransitionError::IllegalStateChange)
        );
    }
}

// Identify exact BBE11 state prefixes instead of assuming variable-length
// before/event/after lengths. Change only both snapshots' outs; effects remain
// structurally consistent, so rejection must be semantic, not malformed bytes.
fn state_prefix(state: &GameState) -> Vec<u8> {
    let mut prefix = state.inning.to_le_bytes().to_vec();
    prefix.push(u8::from(state.half == Half::Bottom));
    prefix.push(state.outs);
    for score in state.score {
        prefix.extend_from_slice(&score.to_le_bytes());
    }
    let first = &state.teams[0].batting_order[0];
    prefix.extend_from_slice(&(first.len() as u64).to_le_bytes());
    prefix.extend_from_slice(first.as_bytes());
    prefix
}

#[test]
fn codec_rejects_strikeout_reach_when_first_occupied_below_two_outs() {
    for half in [Half::Top, Half::Bottom] {
        let before = state(half, 1, 2);
        let (event, _) = safe_reach(&before, Destination::First);
        let valid = apply(&before, event).unwrap();
        let mut offsets = vec![];
        for snapshot in [valid.before(), valid.after()] {
            let prefix = state_prefix(snapshot);
            let matches: Vec<_> = valid
                .to_bytes()
                .windows(prefix.len())
                .enumerate()
                .filter_map(|(i, bytes)| (bytes == prefix).then_some(i))
                .collect();
            assert_eq!(matches.len(), 1);
            offsets.push(matches[0] + 3);
        }
        for outs in [0, 1] {
            let mut bytes = valid.to_bytes();
            for offset in &offsets {
                assert_eq!(bytes[*offset], 2);
                bytes[*offset] = outs;
            }
            assert_eq!(
                EventTransition::from_bytes(&bytes),
                Err(TransitionError::IllegalStateChange)
            );
        }
    }
}
