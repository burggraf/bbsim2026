//! Synthetic modeled support, not reconstructed source causes or calibrated policy.
use bbsim_engine::{
    accounting::derive_accounting, apply, Base, Credit, CreditKind, Delta, Destination, Event,
    EventKind, EventTransition, FieldingReview, GameState, Half, Participants, Runner, RunnerMove,
    SoleSafeHitFacts, TeamState, TransitionError,
};

fn state(mask: u8, outs: u8, half: Half) -> GameState {
    let team = |prefix: &str| TeamState {
        defense: bbsim_engine::DefenseState::LegacyUnassigned,
        batting_order: std::array::from_fn(|i| format!("{prefix}{i}")),
        next_batter: 0,
        pitcher: (format!("{prefix}p")).into(),
        dh_slot: Some(8),
        dh_must_bat: false,
        bench: vec![],
        removed: vec![],
    };
    let teams = [team("a"), team("h")];
    let bases = std::array::from_fn(|i| {
        (mask & (1 << i) != 0).then(|| Runner {
            player_id: teams[half.offense()].batting_order[i + 3].clone(),
            responsible_pitcher: "inherited-pitcher".into(),
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

fn credit(id: &str, kind: CreditKind) -> Credit {
    Credit {
        player_id: id.into(),
        kind,
    }
}

fn projection(before: &GameState, value: usize, destinations: [usize; 3]) -> Event {
    let batter = before.teams[before.half.offense()].batting_order[0].clone();
    let pitcher = before.teams[1 - before.half.offense()]
        .pitcher
        .active_id()
        .expect("legacy fixture pitcher")
        .to_owned();
    let targets = [
        Destination::First,
        Destination::Second,
        Destination::Third,
        Destination::Home,
    ];
    let mut moves = vec![RunnerMove {
        runner: Runner {
            player_id: batter.clone(),
            responsible_pitcher: pitcher.clone(),
        },
        from: Base::Plate,
        to: targets[value - 1],
    }];
    let mut credits = vec![
        credit(&batter, CreditKind::PlateAppearance),
        credit(&batter, CreditKind::Hit),
        credit(
            &batter,
            if value == 1 {
                CreditKind::Single
            } else {
                CreditKind::Double
            },
        ),
    ];
    let mut runs = 0;
    for (i, base) in [Base::First, Base::Second, Base::Third]
        .into_iter()
        .enumerate()
    {
        if let Some(runner) = &before.bases[i] {
            let to = targets[destinations[i] - 1];
            moves.push(RunnerMove {
                runner: runner.clone(),
                from: base,
                to,
            });
            if to == Destination::Home {
                runs += 1;
                credits.extend([
                    credit(&runner.player_id, CreditKind::Run),
                    credit(&batter, CreditKind::Rbi),
                ]);
            }
        }
    }
    Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(batter.clone()),
            pitcher: Some(pitcher),
        },
        delta: Delta {
            moves,
            outs: 0,
            runs,
            credits,
            changes: vec![],
            third_out: None,
        },
        // These are EXPLICIT MODEL ASSUMPTIONS, never inferred from source endpoints.
        fielding_review: FieldingReview::NoMisplay,
        sole_safe_hit: Some(SoleSafeHitFacts { batter_id: batter }),
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        ordinary_ground_throw: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        muffed_ground_ball: None,
        ordinary_air_catch: None,
    }
}

#[test]
fn every_ordered_safe_projection_keeps_canonical_state_codec_and_independent_accounting() {
    let mut frames = 0;
    let mut held = 0;
    let mut nonstation = 0;
    for half in [Half::Top, Half::Bottom] {
        for mask in 0..8 {
            for outs in 0..3 {
                let before = state(mask, outs, half);
                for value in [1, 2] {
                    for code in 0..64 {
                        let targets = [code % 4 + 1, (code / 4) % 4 + 1, code / 16 + 1];
                        if (0..3).any(|i| mask & (1 << i) == 0 && targets[i] != 1) {
                            continue;
                        }
                        let mut ordered = vec![value];
                        for (i, target) in targets.iter().enumerate() {
                            if mask & (1 << i) != 0 {
                                ordered.push(*target);
                            }
                        }
                        if (0..3).any(|i| mask & (1 << i) != 0 && targets[i] < i + 1)
                            || ordered.windows(2).any(|p| p[0] >= p[1] && p != [4, 4])
                        {
                            continue;
                        }
                        let event = projection(&before, value, targets);
                        let transition = apply(&before, event.clone()).unwrap();
                        assert_eq!(transition.before(), &before);
                        assert_eq!(transition.after().outs, outs);
                        assert_eq!(transition.after().teams[half.offense()].next_batter, 1);
                        assert_eq!(
                            transition.after().score[half.offense()],
                            u16::from(event.delta.runs)
                        );
                        assert!(!transition.after().over);
                        assert_eq!(
                            EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
                            transition
                        );
                        let lines = derive_accounting(std::slice::from_ref(&transition)).unwrap();
                        let batter = &before.teams[half.offense()].batting_order[0];
                        assert_eq!(lines.batters[half.offense()][batter].plate_appearances, 1);
                        assert_eq!(
                            lines.batters[half.offense()][batter].classified_rbi,
                            Some(u32::from(event.delta.runs))
                        );
                        let current = before.teams[1 - half.offense()]
                            .pitcher
                            .active_id()
                            .unwrap();
                        assert_eq!(lines.pitchers[1 - half.offense()][current].batters_faced, 1);
                        assert_eq!(lines.pitchers[1 - half.offense()][current].runs_allowed, 0);
                        if event.delta.runs > 0 {
                            assert_eq!(
                                lines.pitchers[1 - half.offense()]["inherited-pitcher"]
                                    .runs_allowed,
                                u32::from(event.delta.runs)
                            );
                        }
                        for (i, target) in targets.iter().enumerate() {
                            if let Some(runner) = &before.bases[i] {
                                if *target == i + 1 {
                                    held += 1;
                                }
                                if *target != (i + value + 1).min(4) {
                                    nonstation += 1;
                                }
                                if *target < 4 {
                                    assert_eq!(
                                        transition.after().bases[*target - 1].as_ref(),
                                        Some(runner)
                                    );
                                }
                            }
                        }
                        frames += 1;
                    }
                }
            }
        }
    }
    assert!(frames > 0 && held > 0 && nonstation > 0);
    println!("Modeled ordered safe projections: {frames}; explicit held effects: {held}; nonstation effects: {nonstation}");
}

#[test]
fn source_endpoint_shape_cannot_supply_missing_cause_or_double_review() {
    let before = state(2, 0, Half::Top);
    let mut single = projection(&before, 1, [1, 4, 1]);
    single.sole_safe_hit = None;
    single.fielding_review = FieldingReview::Unknown;
    let transition = apply(&before, single).unwrap();
    assert!(transition.sole_safe_hit_rbi().is_none());
    assert_eq!(
        derive_accounting(&[transition]).unwrap().batters[0]["a0"].classified_rbi,
        None
    );
    let mut double = projection(&before, 2, [1, 4, 1]);
    double.fielding_review = FieldingReview::Unknown;
    assert_eq!(
        apply(&before, double),
        Err(TransitionError::UnsupportedRule)
    );
}

#[test]
fn regular_inning_projection_does_not_waive_terminal_hit_value_ceiling() {
    let mut before = state(4, 0, Half::Bottom);
    before.inning = 9;
    assert_eq!(
        apply(&before, projection(&before, 2, [1, 1, 4])),
        Err(TransitionError::IllegalCredit)
    );
    let single = apply(&before, projection(&before, 1, [1, 1, 4])).unwrap();
    assert!(single.after().over);
    assert_eq!(single.after().score, [0, 1]);
}
