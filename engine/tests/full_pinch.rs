#[path = "../examples/support/full_pinch.rs"]
mod full_pinch;

use bbsim_engine::{
    accounting::derive_accounting, DefenseState, EventTransition, GameState, Half,
    PitcherObligation, PitcherRole, TeamState,
};
use full_pinch::{decide, Candidate};

fn state(dh: bool, half: Half, slot: u8) -> GameState {
    let teams = ["a", "h"].map(|p| {
        let pitcher = format!("{p}p");
        let order = std::array::from_fn(|i| {
            if i == 8 && !dh {
                pitcher.clone()
            } else {
                format!("{p}{i}")
            }
        });
        let positions = std::array::from_fn(|i| {
            Some(if i == 0 {
                pitcher.clone()
            } else {
                order[i - 1].clone()
            })
        });
        TeamState {
            batting_order: order,
            next_batter: slot,
            pitcher: PitcherRole::Active {
                player_id: pitcher,
                obligation: PitcherObligation::Satisfied,
            },
            defense: DefenseState::Assigned(Box::new(positions)),
            dh_slot: dh.then_some(8),
            dh_must_bat: false,
            bench: vec![format!("{p}ph"), format!("{p}fresh"), format!("{p}relief")],
            removed: vec![],
        }
    });
    GameState {
        inning: 7,
        half,
        outs: 2,
        score: [0, 0],
        teams,
        bases: [None, None, None],
        over: false,
    }
}
fn pool(s: &GameState) -> Vec<Candidate> {
    let t = &s.teams[s.half.offense()];
    t.bench[..2]
        .iter()
        .map(|id| Candidate {
            id: id.clone(),
            positions: vec![2],
        })
        .collect()
}
fn q(gain: f64) -> [f64; 9] {
    let mut q = [0.0625; 9];
    q[8] = 0.5 - gain;
    q[6] += gain;
    q
}
fn choose(s: &GameState, pool: &[Candidate]) -> Option<EventTransition> {
    let reserve = s.teams[s.half.offense()].bench[2].clone();
    decide(s, pool, Some(&reserve), |b, p| {
        assert_eq!(Some(p), s.teams[1 - s.half.offense()].pitcher.active_id());
        Ok(q(if b.ends_with("fresh") {
            0.10
        } else if b.ends_with("ph") {
            0.05
        } else {
            0.0
        }))
    })
    .unwrap()
}

#[test]
fn picks_best_legal_hitter_in_both_halves_and_all_slot_roles() {
    assert_eq!(
        full_pinch::POLICY_VERSION,
        "late-close-expected-bases-ph-v1"
    );
    for half in [Half::Top, Half::Bottom] {
        for dh in [false, true] {
            for slot in [0, 8] {
                let s = state(dh, half, slot);
                let frozen = s.clone();
                let t = choose(&s, &pool(&s)).unwrap();
                let side = half.offense();
                let after = t.after();
                let incoming = &s.teams[side].bench[1];
                assert_eq!(&after.teams[side].batting_order[slot as usize], incoming);
                assert_eq!(after.teams[side].next_batter, slot);
                assert_eq!(after.teams[side].dh_slot, s.teams[side].dh_slot);
                assert_eq!(after.bases, s.bases);
                assert_eq!(after.score, s.score);
                assert_eq!(after.outs, s.outs);
                if slot == 8 && !dh {
                    assert!(matches!(
                        after.teams[side].pitcher,
                        PitcherRole::Vacant { batting_slot: 8 }
                    ));
                } else {
                    assert_eq!(after.teams[side].pitcher, s.teams[side].pitcher);
                    if slot == 0 {
                        let DefenseState::Assigned(p) = &after.teams[side].defense else {
                            panic!()
                        };
                        assert_eq!(p[1].as_ref(), Some(incoming));
                    }
                }
                let bytes = t.to_bytes();
                assert!(bytes.starts_with(b"BBE33"));
                assert_eq!(EventTransition::from_bytes(&bytes).unwrap(), t);
                let stats = derive_accounting(&[t]).unwrap();
                assert_eq!(stats.runs, [0, 0]);
                assert!(stats.pitchers.iter().all(|p| p.is_empty()));
                assert!(stats.batters.iter().all(|b| b.is_empty()));
                assert_eq!(s, frozen);
            }
        }
    }
}

#[test]
fn early_lopsided_ended_or_pending_offense_never_calls_ratings() {
    let base = state(true, Half::Top, 8);
    for (inning, score, over) in [
        (6, [0, 0], false),
        (7, [4, 0], false),
        (7, [0, 4], false),
        (9, [0, 0], true),
    ] {
        let mut s = base.clone();
        s.inning = inning;
        s.score = score;
        s.over = over;
        assert!(decide(&s, &pool(&s), None, |_, _| panic!(
            "inapplicable rating callback"
        ))
        .unwrap()
        .is_none());
    }
    let mut pending = base;
    let DefenseState::Assigned(p) = &mut pending.teams[0].defense else {
        panic!()
    };
    p[1] = None;
    assert!(decide(&pending, &pool(&pending), None, |_, _| panic!(
        "pending offense callback"
    ))
    .unwrap()
    .is_none());
}

#[test]
fn unknown_positions_and_missing_reserve_are_not_invented() {
    let mut s = state(false, Half::Top, 8);
    assert!(
        decide(&s, &pool(&s), None, |_, _| panic!("no reserve callback"))
            .unwrap()
            .is_none()
    );
    s.teams[0].next_batter = 0;
    let mut candidates = pool(&s);
    for c in &mut candidates {
        c.positions.clear();
    }
    assert!(decide(&s, &candidates, None, |_, _| panic!(
        "unknown position callback"
    ))
    .unwrap()
    .is_none());
    s.teams[0].dh_slot = Some(8);
    s.teams[0].batting_order[8] = "a8".into();
    s.teams[0].next_batter = 8;
    assert!(choose(&s, &candidates).is_some());
}

#[test]
fn canonical_starting_batter_and_dh_obligations_prevent_selection() {
    for obligation in [
        PitcherObligation::StartingBatter,
        PitcherObligation::ReliefBatter,
        PitcherObligation::UntrackedLegacy,
    ] {
        let mut s = state(false, Half::Top, 8);
        s.teams[0].pitcher = PitcherRole::Active {
            player_id: "ap".into(),
            obligation,
        };
        assert!(decide(&s, &pool(&s), Some("arelief"), |_, _| panic!(
            "obligated callback"
        ))
        .unwrap()
        .is_none());
    }
    let mut s = state(true, Half::Top, 8);
    s.teams[0].dh_must_bat = true;
    assert!(
        decide(&s, &pool(&s), None, |_, _| panic!("mandatory DH callback"))
            .unwrap()
            .is_none()
    );
}

#[test]
fn metadata_and_defensive_errors_precede_all_probability_callbacks() {
    let s = state(true, Half::Top, 8);
    let base = pool(&s);
    let mut cases = vec![vec![base[0].clone(), base[0].clone()]];
    for (id, positions) in [
        ("ghost", vec![]),
        ("hph", vec![]),
        ("aph", vec![1]),
        ("aph", vec![2, 2]),
        ("aph", vec![10]),
        ("bad id", vec![]),
    ] {
        cases.push(vec![Candidate {
            id: id.into(),
            positions,
        }]);
    }
    for bad in cases {
        assert!(decide(&s, &bad, None, |_, _| panic!("invalid metadata callback")).is_err());
    }
    for reserve in ["ghost", "hp", "ap", "aph", "hrelief"] {
        assert!(decide(&s, &base, Some(reserve), |_, _| panic!(
            "invalid reserve callback"
        ))
        .is_err());
    }
    for legacy in [false, true] {
        let mut s = s.clone();
        let DefenseState::Assigned(p) = &mut s.teams[1].defense else {
            panic!()
        };
        p[1] = None;
        if legacy {
            s.teams[1].defense = DefenseState::LegacyUnassigned;
        }
        assert!(decide(&s, &base, None, |_, _| panic!("invalid defense callback")).is_err());
    }
}

#[test]
fn used_pool_members_cannot_reenter_and_ties_ignore_input_order() {
    let mut s = state(true, Half::Top, 8);
    let mut candidates = pool(&s);
    let choose_tie = |c: &[Candidate]| {
        decide(&s, c, None, |b, _| {
            Ok(q(if b == "a8" { 0.0 } else { 0.10 }))
        })
        .unwrap()
    };
    let first = choose_tie(&candidates).unwrap();
    candidates.reverse();
    assert_eq!(choose_tie(&candidates).unwrap(), first);
    assert_eq!(first.after().teams[0].batting_order[8], "afresh");
    s.teams[0].bench.remove(1);
    s.teams[0].removed.push("afresh".into());
    let selected = decide(&s, &candidates, None, |b, _| {
        assert_ne!(b, "afresh");
        Ok(q(if b == "aph" { 0.10 } else { 0.0 }))
    })
    .unwrap()
    .unwrap();
    assert_eq!(selected.after().teams[0].batting_order[8], "aph");
}

#[test]
fn a_malformed_declared_reserve_is_rejected_before_ratings() {
    let mut s = state(false, Half::Top, 8);
    s.teams[0].bench[2] = "bad reserve".into();
    let mut calls = 0;
    let result = decide(&s, &pool(&s), Some("bad reserve"), |_, _| {
        calls += 1;
        Ok(q(0.0))
    });
    assert!(result.is_err());
    assert_eq!(calls, 0);
}

#[test]
fn base_occupancy_outs_and_carried_pitcher_charges_survive_selection() {
    for half in [Half::Top, Half::Bottom] {
        for dh in [false, true] {
            for outs in 0..3 {
                for mask in 0..8 {
                    let mut s = state(dh, half, 8);
                    s.outs = outs;
                    let side = half.offense();
                    for i in 0..3 {
                        if mask & (1 << i) != 0 {
                            s.bases[i] = Some(bbsim_engine::Runner {
                                player_id: s.teams[side].batting_order[i].clone(),
                                responsible_pitcher: format!("carried-{i}"),
                            });
                        }
                    }
                    let frozen = s.clone();
                    let t = choose(&s, &pool(&s)).unwrap();
                    assert_eq!(t.after().bases, s.bases);
                    assert_eq!(t.after().outs, outs);
                    assert_eq!(t.before(), &frozen);
                    assert_eq!(s, frozen);
                    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
                }
            }
        }
    }
}

#[test]
fn improvement_bound_and_probability_validation_are_explicit() {
    let mut s = state(true, Half::Top, 8);
    for score in [[3, 0], [0, 3]] {
        s.score = score;
        for (gain, yes) in [(0.0, false), (0.0249, false), (0.0251, true)] {
            let result = decide(&s, &pool(&s), None, |b, _| {
                Ok(q(if b == "a8" { 0.0 } else { gain }))
            })
            .unwrap();
            assert_eq!(result.is_some(), yes);
        }
    }
    for bad in [
        [f64::NAN; 9],
        [f64::INFINITY; 9],
        [0.0; 9],
        [-1.0; 9],
        [0.2; 9],
    ] {
        assert!(decide(&s, &pool(&s), None, |_, _| Ok(bad)).is_err());
        assert!(decide(&s, &pool(&s), None, |b, _| Ok(if b == "a8" {
            q(0.0)
        } else {
            bad
        }))
        .is_err());
    }
    assert!(decide(&s, &pool(&s), None, |_, _| Err("rating unavailable".into())).is_err());
}

#[test]
fn managed_ph_usage_is_bounded_by_gain_and_best_score_and_does_not_reenter() {
    use std::collections::BTreeMap;
    let mut s = state(true, Half::Top, 8);
    let candidates = pool(&s);
    let pace = BTreeMap::from([("aph".into(), 1), ("afresh".into(), -1)]);
    let incoming =
        |t: Option<EventTransition>| t.map(|t| t.after().teams[0].batting_order[8].clone());
    // q(gain) changes expected bases by 4*gain.
    for (ph, fresh, expected) in [
        (0.05, 0.06, "aph"),
        (0.05, 0.0625, "aph"), // inclusive 0.05 expected-bases window
        (0.05, 0.0624, "aph"),
        (0.05, 0.0626, "afresh"),
        (0.05, 0.07, "afresh"),
        (0.02, 0.03, "afresh"),
        (0.025, 0.025, ""),
    ] {
        let t =
            full_pinch::decide_managed_states(&s, &candidates, Some("arelief"), &pace, |state| {
                Ok(q(match state.teams[0].batting_order[8].as_str() {
                    "aph" => ph,
                    "afresh" => fresh,
                    _ => 0.0,
                }))
            })
            .unwrap();
        assert_eq!(
            incoming(t),
            if expected.is_empty() {
                None
            } else {
                Some(expected.into())
            }
        );
    }
    let tied = BTreeMap::from([("aph".into(), 0), ("afresh".into(), 0)]);
    let choose = |s: &GameState, pace: &BTreeMap<String, i8>, gains: [f64; 2]| {
        full_pinch::decide_managed_states(s, &candidates, Some("arelief"), pace, |state| {
            Ok(q(match state.teams[0].batting_order[8].as_str() {
                "aph" => gains[0],
                "afresh" => gains[1],
                _ => 0.0,
            }))
        })
        .unwrap()
    };
    assert_eq!(
        incoming(choose(&s, &tied, [0.05, 0.06])),
        Some("afresh".into())
    );
    assert_eq!(
        incoming(choose(&s, &tied, [0.05, 0.05])),
        Some("afresh".into())
    ); // stable ID, not pool order
    s.teams[0].bench.retain(|id| id != "aph");
    s.teams[0].removed.push("aph".into());
    assert_eq!(
        incoming(choose(&s, &pace, [0.06, 0.05])),
        Some("afresh".into())
    );
    // Standalone policy still chooses greatest expected bases, irrespective of usage hints.
    let original = state(true, Half::Top, 8);
    let old = full_pinch::decide_states(&original, &candidates, Some("arelief"), |state| {
        Ok(q(match state.teams[0].batting_order[8].as_str() {
            "aph" => 0.05,
            "afresh" => 0.06,
            _ => 0.0,
        }))
    })
    .unwrap();
    assert_eq!(incoming(old), Some("afresh".into()));
}

#[test]
fn managed_ph_obligations_and_pace_validation_precede_rating_queries() {
    use std::collections::BTreeMap;
    let mut s = state(false, Half::Top, 8);
    let candidates = pool(&s);
    let pace = BTreeMap::from([("aph".into(), 1), ("afresh".into(), -1)]);
    for obligation in [
        PitcherObligation::StartingBatter,
        PitcherObligation::ReliefBatter,
    ] {
        s.teams[0].pitcher = PitcherRole::Active {
            player_id: "ap".into(),
            obligation,
        };
        assert!(full_pinch::decide_managed_states(
            &s,
            &candidates,
            Some("arelief"),
            &pace,
            |_| panic!("obligated managed ratings")
        )
        .unwrap()
        .is_none());
    }
    let s = state(true, Half::Top, 8);
    for bad in [
        BTreeMap::new(),
        BTreeMap::from([("aph".into(), 2), ("afresh".into(), 0)]),
    ] {
        assert!(full_pinch::decide_managed_states(
            &s,
            &candidates,
            Some("arelief"),
            &bad,
            |_| panic!("invalid managed pace ratings")
        )
        .is_err());
    }
}
