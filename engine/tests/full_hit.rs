//! Observable supplied-projector contract, fixed before implementation:
//! - No RNG/source inference: value, actual field positions, joint endpoints and tag clock are supplied.
//! - Exact occupancy, hitter safe S/D/T, identity/charge conservation; reject malformed/backward,
//!   collision/passing and invalid defense before any winning-stop normalization.
//! - Safe winning crossings are generated Third, Second, First; stop at exactly needed run.
//!   Official value=min(selected value, actual winner's advance). Retain safe trailing endpoints
//!   unless stopping requires clipping: from lead to trail, cap uncounted Home at Third and
//!   each following endpoint below its leader. Batter attains official base in natural course
//!   AfterWinningRun (Rule9.06(f)); never use a closest-scorer ceiling or change supplied tags.
//! - Tag has exactly one Out, hitter attained before tag, actual selected OF assist;
//!   Third=3B, Home=C, Second=SS for LF throws / 2B for CF/RF. Named pre-tag origins are a
//!   set, not Vec chronology, but must be a leading prefix among Home endpoints to avoid passing.
//!   At third out only those crossings score; otherwise all Home score.
//! - Safe native apply; walkoff/tag supplemental apply + constructor + codec agree, immutably.
//! - Conservative forced-next/target<=hitter/no-passing refusals remain; no fallback or second roll.
#[path = "../examples/support/full_hit.rs"]
mod full_hit;
use bbsim_engine::{accounting::derive_accounting, *};
use full_hit::{project, HitDecision, TagAttempt};

fn state(mask: u8, outs: u8, half: Half) -> GameState {
    let teams = ["a", "h"].map(|p| TeamState {
        defense: bbsim_engine::DefenseState::LegacyUnassigned,
        batting_order: std::array::from_fn(|i| format!("{p}{i}")),
        next_batter: 0,
        pitcher: (format!("{p}p")).into(),
        dh_slot: Some(8),
        dh_must_bat: false,
        bench: vec![format!("{p}bench")],
        removed: vec![format!("{p}removed")],
    });
    GameState {
        inning: 2,
        half,
        outs,
        score: [0, 0],
        over: false,
        bases: std::array::from_fn(|i| {
            (mask & (1 << i) != 0).then(|| Runner {
                player_id: teams[half.offense()].batting_order[i + 3].clone(),
                responsible_pitcher: format!("inherited-{i}"),
            })
        }),
        teams,
    }
}
fn fielders(s: &GameState) -> [String; 9] {
    let team = &s.teams[1 - s.half.offense()];
    std::array::from_fn(|i| {
        if i == 0 {
            team.pitcher
                .active_id()
                .expect("legacy fixture pitcher")
                .to_owned()
        } else {
            team.batting_order[i - 1].clone()
        }
    })
}
#[test]
fn supplied_positions_must_match_canonical_defense_even_on_a_safe_hit() {
    let mut before = state(0, 0, Half::Top);
    let stale = fielders(&before);
    let mut current = stale.clone();
    current.swap(1, 5);
    before.teams[1].defense = DefenseState::Assigned(Box::new(current.clone().map(Some)));
    before.teams[1].pitcher = PitcherRole::Active {
        player_id: current[0].clone(),
        obligation: PitcherObligation::Satisfied,
    };
    let choice = decision(1, [None; 3]);
    assert!(project(&before, &stale, &choice).is_err());
    let t = project(&before, &current, &choice).unwrap();
    assert!(t.to_bytes().starts_with(b"BBE33"));
    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
}
fn dest(n: u8) -> Destination {
    [
        Destination::Out,
        Destination::First,
        Destination::Second,
        Destination::Third,
        Destination::Home,
    ][usize::from(n)]
}
fn decision(value: u8, ends: [Option<u8>; 3]) -> HitDecision {
    HitDecision {
        value,
        position: 7,
        runners: ends.map(|r| r.map(dest)),
        tag: None,
    }
}
fn checked(s: &GameState, d: &HitDecision) -> EventTransition {
    let original = s.clone();
    let f = fielders(s);
    let d0 = d.clone();
    let t = project(s, &f, d).unwrap_or_else(|e| panic!("{s:?} {d:?}: {e}"));
    assert_eq!(s, &original);
    assert_eq!(d, &d0);
    assert_eq!(fielders(s), f);
    assert_eq!(t.before(), s);
    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
    let reapply = |e: Event| {
        if let Some(f) = t.hit_runner_tag_facts() {
            apply_with_hit_runner_tag_facts(s, e, f.clone())
        } else if let Some(f) = t.walkoff_hit_facts() {
            apply_with_walkoff_hit_facts(s, e, f.clone())
        } else {
            apply(s, e)
        }
    };
    assert_eq!(reapply(t.event().clone()).unwrap(), t);
    let constructed = if let Some(f) = t.hit_runner_tag_facts() {
        assert!(apply(s, t.event().clone()).is_err());
        EventTransition::try_new_with_hit_runner_tag_facts(
            s.clone(),
            t.event().clone(),
            t.after().clone(),
            f.clone(),
        )
    } else if let Some(f) = t.walkoff_hit_facts() {
        EventTransition::try_new_with_walkoff_hit_facts(
            s.clone(),
            t.event().clone(),
            t.after().clone(),
            f.clone(),
        )
    } else {
        EventTransition::try_new(s.clone(), t.event().clone(), t.after().clone())
    };
    assert_eq!(constructed.unwrap(), t);
    let mut e = t.event().clone();
    e.delta.moves.reverse();
    e.delta.credits.reverse();
    let permuted = reapply(e).unwrap();
    assert_eq!(t.after(), permuted.after());
    assert_eq!(
        derive_accounting(std::slice::from_ref(&t)).unwrap(),
        derive_accounting(&[permuted]).unwrap()
    );
    t
}
fn accounting(s: &GameState, t: &EventTransition, value: u8, runs: u8, outs: u8) {
    let a = derive_accounting(std::slice::from_ref(t)).unwrap();
    let o = s.half.offense();
    let b = &a.batters[o][&s.teams[o].batting_order[0]];
    assert_eq!(
        (b.plate_appearances, b.at_bats, b.credited_hits),
        (1, Some(1), 1)
    );
    assert_eq!(
        b.credited_hit_breakdown.as_ref().unwrap().total_bases(),
        u32::from(value)
    );
    let p = &a.pitchers[1 - o][s.teams[1 - o].pitcher.active_id().unwrap()];
    assert_eq!(
        (
            p.batters_faced,
            p.at_bats_against,
            p.credited_hits_allowed,
            p.outs_recorded
        ),
        (1, Some(1), 1, u32::from(outs))
    );
    assert_eq!(a.runs[o], u32::from(runs));
    assert_eq!(p.runs_allowed, 0);
    for m in &t.event().delta.moves {
        if m.from != Base::Plate {
            assert!(m.runner.responsible_pitcher.starts_with("inherited-"));
        }
        if m.from == Base::Plate {
            assert_eq!(m.to, dest(value));
        }
        if t.event()
            .delta
            .credits
            .iter()
            .any(|c| c.kind == CreditKind::Run && c.player_id == m.runner.player_id)
        {
            assert_eq!(
                a.pitchers[1 - o][&m.runner.responsible_pitcher].runs_allowed,
                1
            );
        }
    }
}
#[test]
fn ordinary_cartesian_safe_joint_cells() {
    let mut accepted = 0;
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            for mask in 0..8 {
                for value in 1..=3 {
                    let s = state(mask, outs, half);
                    for a in 1..=4 {
                        for b in 1..=4 {
                            for c in 1..=4 {
                                let ends = [a, b, c];
                                // Avoid repeated unoccupied-axis cells; oracle independent of projector.
                                if (0..3).any(|i| mask & (1 << i) == 0 && ends[i] != 1) {
                                    continue;
                                }
                                let d = decision(
                                    value,
                                    std::array::from_fn(|i| {
                                        (mask & (1 << i) != 0).then_some(ends[i])
                                    }),
                                );
                                let mut ordered = vec![(0, value)];
                                ordered.extend(
                                    (0..3)
                                        .filter(|i| mask & (1 << i) != 0)
                                        .map(|i| (i + 1, ends[i])),
                                );
                                let legal = ordered.iter().all(|(from, to)| *to as usize >= *from)
                                    && ordered
                                        .windows(2)
                                        .all(|p| p[0].1 < p[1].1 || p[0].1 == 4 && p[1].1 == 4);
                                if !legal {
                                    assert!(project(&s, &fielders(&s), &d).is_err(), "{s:?} {d:?}");
                                    continue;
                                }
                                let t = checked(&s, &d);
                                accepted += 1;
                                let runs = ends
                                    .iter()
                                    .enumerate()
                                    .filter(|(i, n)| mask & (1 << i) != 0 && **n == 4)
                                    .count() as u8;
                                accounting(&s, &t, value, runs, 0);
                                assert_eq!(t.after().outs, outs);
                                assert!(!t.after().over);
                                assert!(t.event().sole_safe_hit.is_some());
                                assert!(t.hit_runner_tag_facts().is_none());
                                assert_eq!(t.after().score[half.offense()], u16::from(runs));
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(accepted, 288); // 48 legal joint S/D/T vectors × 3 outs × 2 halfs.
}
#[test]
fn winning_stop_uses_actual_winner_and_official_single_double_triple() {
    for outs in 0..3 {
        for needed in 1..=3 {
            for value in 1..=3 {
                let mut s = state(7, outs, Half::Bottom);
                s.inning = 10;
                s.score = [10 + needed - 1, 10];
                let d = decision(value, [Some(4); 3]);
                let t = checked(&s, &d);
                let official = value.min(needed as u8);
                accounting(&s, &t, official, needed as u8, 0);
                assert_eq!(t.after().score, [10 + needed - 1, 10 + needed]);
                assert!(t.after().over);
                let f = t.walkoff_hit_facts().unwrap();
                assert_eq!(
                    f.winning_runner_id,
                    s.bases[3 - needed as usize].as_ref().unwrap().player_id
                );
                assert_eq!(f.timing, NaturalCourseTouchTiming::AfterWinningRun);
                for i in 0..3 {
                    let m = t
                        .event()
                        .delta
                        .moves
                        .iter()
                        .find(|m| m.runner == *s.bases[i].as_ref().unwrap())
                        .unwrap();
                    assert_eq!(
                        m.to,
                        if i >= 3 - needed as usize {
                            Destination::Home
                        } else {
                            dest((needed + 1 + i as u16) as u8)
                        }
                    );
                }
            }
        }
    }
}
#[test]
fn tags_are_separate_timed_outs_with_actual_named_defense() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            for position in 7..=9 {
                for pre in [false, true] {
                    let s = state(5, outs, half);
                    let mut d = decision(1, [Some(0), None, Some(4)]);
                    d.position = position;
                    d.tag = Some(TagAttempt {
                        origin: Base::First,
                        attempted_base: Destination::Third,
                        home_before_tag: if pre { vec![Base::Third] } else { vec![] },
                    });
                    let t = checked(&s, &d);
                    let runs = u8::from(outs < 2 || pre);
                    accounting(&s, &t, 1, runs, 1);
                    let f = t.hit_runner_tag_facts().unwrap();
                    let fs = fielders(&s);
                    assert_eq!(f.thrower_id, fs[position as usize - 1]);
                    assert_eq!(f.tagger_id, fs[4]);
                    assert_eq!(
                        f.order,
                        HitRunnerTagOrder::BatterAttainedCreditedBaseBeforeTag
                    );
                    assert!(t.event().sole_safe_hit.is_none());
                    let a = derive_accounting(std::slice::from_ref(&t)).unwrap();
                    assert_eq!(
                        a.fielders[1 - half.offense()][&f.thrower_id].credited_assists,
                        1
                    );
                    assert_eq!(
                        a.fielders[1 - half.offense()][&f.tagger_id].credited_putouts,
                        1
                    );
                    if outs == 2 {
                        let third = t.event().delta.third_out.as_ref().unwrap();
                        assert_eq!(third.kind, ThirdOutKind::Timing);
                        assert_eq!(third.home_before_out, f.home_before_tag);
                        assert!(t.after().bases.iter().all(Option::is_none));
                    }
                }
            }
        }
    }
    for (origin, mask, at, value) in [
        (Base::Second, 2, Destination::Second, 1),
        (Base::First, 1, Destination::Home, 2),
        (Base::First, 1, Destination::Home, 3),
    ] {
        for position in 7..=9 {
            let s = state(mask, 0, Half::Top);
            let index = if origin == Base::First { 0 } else { 1 };
            let mut d = decision(value, [None; 3]);
            d.runners[index] = Some(Destination::Out);
            d.position = position;
            d.tag = Some(TagAttempt {
                origin,
                attempted_base: at,
                home_before_tag: vec![],
            });
            if at == Destination::Second {
                assert!(project(&s, &fielders(&s), &d).is_err());
                continue;
            }
            let t = checked(&s, &d);
            assert_eq!(t.hit_runner_tag_facts().unwrap().tagger_id, fielders(&s)[1]);
        }
    }
    // A non-forced First->Second tag needs First not forced: impossible in this family;
    // Second can be reached by no supported forward origin except First, so conservative guard refuses.
}
#[test]
fn malformed_vectors_fielders_and_tag_clocks_refuse_immutably() {
    let s = state(5, 0, Half::Top);
    let good = decision(1, [Some(3), None, Some(4)]);
    let mut bad = vec![];
    for value in [0, 4, 255] {
        let mut d = good.clone();
        d.value = value;
        bad.push(d);
    }
    for position in [0, 1, 6, 10, 255] {
        let mut d = good.clone();
        d.position = position;
        bad.push(d);
    }
    for runners in [
        [None, None, Some(dest(4))],
        [Some(dest(3)), Some(dest(4)), Some(dest(4))],
        [Some(dest(0)), None, Some(dest(4))],
        [Some(dest(3)), None, Some(dest(3))],
        [Some(dest(4)), None, Some(dest(3))],
    ] {
        let mut d = good.clone();
        d.runners = runners;
        bad.push(d);
    }
    for d in bad {
        let old = s.clone();
        assert!(project(&s, &fielders(&s), &d).is_err());
        assert_eq!(s, old);
    }
    for i in 0..9 {
        for actor in ["", "h8", "hbench", "hremoved", "a2", "missing", "h0"] {
            let mut fs = fielders(&s);
            fs[i] = actor.into();
            if fs == fielders(&s) {
                continue;
            }
            assert!(project(&s, &fs, &good).is_err(), "{fs:?}");
        }
    }
    let mut fs = fielders(&s);
    fs.swap(0, 1);
    assert!(project(&s, &fs, &good).is_err());
    let mut d = decision(1, [Some(0), None, Some(4)]);
    for origins in [
        vec![Base::Plate],
        vec![Base::Second],
        vec![Base::First],
        vec![Base::Third, Base::Third],
    ] {
        d.tag = Some(TagAttempt {
            origin: Base::First,
            attempted_base: Destination::Third,
            home_before_tag: origins,
        });
        assert!(project(&s, &fielders(&s), &d).is_err());
    }
    for (origin, at) in [
        (Base::Plate, Destination::Third),
        (Base::Second, Destination::Third),
        (Base::First, Destination::Second),
        (Base::First, Destination::First),
        (Base::First, Destination::Out),
    ] {
        d.tag = Some(TagAttempt {
            origin,
            attempted_base: at,
            home_before_tag: vec![],
        });
        assert!(project(&s, &fielders(&s), &d).is_err());
    }
    let mut s = state(2, 0, Half::Top);
    let d = decision(1, [None, Some(1), None]);
    assert!(project(&s, &fielders(&s), &d).is_err());
    s.over = true;
    assert!(project(&s, &fielders(&s), &decision(1, [None, Some(4), None])).is_err());
}
#[test]
fn late_nonwinning_tags_stay_tags_and_winning_tags_refuse() {
    for inning in [8, 9, 10] {
        for outs in 0..3 {
            for deficit in 0..=2 {
                let mut s = state(5, outs, Half::Bottom);
                s.inning = inning;
                s.score = [10 + deficit, 10];
                let mut d = decision(1, [Some(0), None, Some(4)]);
                d.tag = Some(TagAttempt {
                    origin: Base::First,
                    attempted_base: Destination::Third,
                    home_before_tag: vec![Base::Third],
                });
                if inning >= 9 && deficit == 0 {
                    assert!(project(&s, &fielders(&s), &d).is_err());
                } else {
                    let t = checked(&s, &d);
                    assert_eq!(t.event().delta.outs, 1);
                    assert!(t.hit_runner_tag_facts().is_some());
                }
            }
        }
    }
}

#[test]
fn cartesian_tag_vectors_preserve_bounded_refusals_and_exact_crossing_subsets() {
    let mut accepted = 0;
    let mut refused = 0;
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            for mask in 1..8 {
                for value in 1..=3 {
                    let s = state(mask, outs, half);
                    for retired in 0..3 {
                        if mask & (1 << retired) == 0 {
                            continue;
                        }
                        for attempted in 2..=4 {
                            for a in 1..=4 {
                                for b in 1..=4 {
                                    for c in 1..=4 {
                                        let mut ends = [a, b, c];
                                        if ends[retired] != 1
                                            || (0..3).any(|i| mask & (1 << i) == 0 && ends[i] != 1)
                                        {
                                            continue;
                                        }
                                        ends[retired] = 0;
                                        for crossing_mask in 0u8..8 {
                                            if (0..3).any(|i| {
                                                crossing_mask & (1 << i) != 0
                                                    && (mask & (1 << i) == 0 || ends[i] != 4)
                                            }) {
                                                continue;
                                            }
                                            let mut d = decision(
                                                value,
                                                std::array::from_fn(|i| {
                                                    (mask & (1 << i) != 0).then_some(ends[i])
                                                }),
                                            );
                                            d.tag = Some(TagAttempt {
                                                origin: [Base::First, Base::Second, Base::Third]
                                                    [retired],
                                                attempted_base: dest(attempted),
                                                home_before_tag: (0..3)
                                                    .filter(|i| crossing_mask & (1 << i) != 0)
                                                    .map(|i| {
                                                        [Base::First, Base::Second, Base::Third][i]
                                                    })
                                                    .collect(),
                                            });
                                            let survivors: Vec<_> = (0..3)
                                                .filter(|i| mask & (1 << i) != 0 && *i != retired)
                                                .collect();
                                            let forced = mask & ((1 << (retired + 1)) - 1)
                                                == (1 << (retired + 1)) - 1;
                                            let clock_is_prefix = (0..3).all(|i| {
                                                crossing_mask & (1 << i) == 0
                                                    || (i + 1..3).all(|j| {
                                                        mask & (1 << j) == 0
                                                            || ends[j] != 4
                                                            || crossing_mask & (1 << j) != 0
                                                    })
                                            });
                                            let legal = clock_is_prefix
                                                && usize::from(attempted) > retired + 1
                                                && attempted > value
                                                && !(forced
                                                    && usize::from(attempted) == retired + 2)
                                                && survivors.iter().all(|i| {
                                                    usize::from(ends[*i]) > *i
                                                        && ends[*i] > value
                                                        && if *i < retired {
                                                            ends[*i] < attempted
                                                        } else {
                                                            ends[*i] >= attempted
                                                        }
                                                })
                                                && survivors.windows(2).all(|p| {
                                                    ends[p[0]] < ends[p[1]]
                                                        || ends[p[0]] == 4 && ends[p[1]] == 4
                                                });
                                            if !legal {
                                                refused += 1;
                                                assert!(
                                                    project(&s, &fielders(&s), &d).is_err(),
                                                    "{s:?} {d:?}"
                                                );
                                                continue;
                                            }
                                            accepted += 1;
                                            let t = checked(&s, &d);
                                            let runs = if outs == 2 {
                                                crossing_mask.count_ones() as u8
                                            } else {
                                                (0..3)
                                                    .filter(|i| {
                                                        mask & (1 << i) != 0 && ends[*i] == 4
                                                    })
                                                    .count()
                                                    as u8
                                            };
                                            accounting(&s, &t, value, runs, 1);
                                            assert_eq!(
                                                t.event()
                                                    .delta
                                                    .credits
                                                    .iter()
                                                    .filter(|c| c.kind == CreditKind::Rbi)
                                                    .count(),
                                                usize::from(runs)
                                            );
                                            let mut e = t.event().clone();
                                            let mut f = t.hit_runner_tag_facts().unwrap().clone();
                                            f.home_before_tag.reverse();
                                            if let Some(third) = &mut e.delta.third_out {
                                                third.home_before_out.reverse();
                                            }
                                            let reordered =
                                                apply_with_hit_runner_tag_facts(&s, e, f).unwrap();
                                            assert_eq!(t.after(), reordered.after());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(
        accepted > 300 && refused > 1000,
        "accepted={accepted}, refused={refused}"
    );
    eprintln!("tag joint cells accepted={accepted}, refused={refused}");
}

#[test]
fn every_legal_safe_vector_in_late_bottom_stops_only_after_validation() {
    let mut cells = 0;
    for mask in 0..8 {
        for value in 1..=3 {
            for a in 1..=4 {
                for b in 1..=4 {
                    for c in 1..=4 {
                        let ends = [a, b, c];
                        if (0..3).any(|i| mask & (1 << i) == 0 && ends[i] != 1) {
                            continue;
                        }
                        let mut v = vec![value];
                        v.extend((0..3).filter(|i| mask & (1 << i) != 0).map(|i| ends[i]));
                        let legal = (0..3)
                            .all(|i| mask & (1 << i) == 0 || usize::from(ends[i]) > i)
                            && v.windows(2).all(|p| p[0] < p[1] || p[0] == 4 && p[1] == 4);
                        let scorers: Vec<_> = (0..3)
                            .rev()
                            .filter(|i| mask & (1 << i) != 0 && ends[*i] == 4)
                            .collect();
                        for inning in [9, 10] {
                            for outs in 0..3 {
                                for needed in 1..=4 {
                                    let mut s = state(mask, outs, Half::Bottom);
                                    s.inning = inning;
                                    s.score = [10 + needed - 1, 10];
                                    let d = decision(
                                        value,
                                        std::array::from_fn(|i| {
                                            (mask & (1 << i) != 0).then_some(ends[i])
                                        }),
                                    );
                                    if !legal {
                                        assert!(project(&s, &fielders(&s), &d).is_err());
                                        continue;
                                    }
                                    cells += 1;
                                    let t = checked(&s, &d);
                                    let winning = scorers.len() >= needed as usize;
                                    assert_eq!(t.after().over, winning);
                                    let official = if winning {
                                        value.min(3 - scorers[needed as usize - 1] as u8)
                                    } else {
                                        value
                                    };
                                    let runs = (scorers.len() as u8).min(needed as u8);
                                    accounting(&s, &t, official, runs, 0);
                                    for m in t.event().delta.moves.iter().skip(1) {
                                        let i = [Base::First, Base::Second, Base::Third]
                                            .iter()
                                            .position(|b| *b == m.from)
                                            .unwrap();
                                        assert_eq!(
                                            m.to == Destination::Home,
                                            scorers.iter().take(needed as usize).any(|j| *j == i)
                                        );
                                        if !winning {
                                            assert_eq!(m.to, dest(ends[i]));
                                        }
                                    }
                                    if winning {
                                        assert_eq!(
                                            t.walkoff_hit_facts().unwrap().winning_runner_id,
                                            s.bases[scorers[needed as usize - 1]]
                                                .as_ref()
                                                .unwrap()
                                                .player_id
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(cells, 1152);
}

#[test]
fn invalid_snapshots_and_nondh_lineups_do_not_bypass_native_validation() {
    let mut s = state(0, 0, Half::Top);
    let d = decision(3, [None; 3]);
    s.teams[1].dh_slot = None;
    s.teams[1].batting_order[8] = s.teams[1]
        .pitcher
        .active_id()
        .expect("legacy fixture pitcher")
        .to_owned();
    checked(&s, &d);
    for mutate in 0..5 {
        let mut bad = s.clone();
        match mutate {
            0 => bad.outs = 3,
            1 => bad.inning = 0,
            2 => bad.teams[0].next_batter = 9,
            3 => bad.teams[1].batting_order[0] = bad.teams[1].batting_order[1].clone(),
            _ => {
                bad.bases[0] = Some(Runner {
                    player_id: "a0".into(),
                    responsible_pitcher: "old".into(),
                })
            }
        }
        let old = bad.clone();
        assert!(project(&bad, &fielders(&bad), &d).is_err());
        assert_eq!(old, bad);
    }
    let mut s = state(1, 0, Half::Top);
    let mut d = decision(1, [Some(0), None, None]);
    d.tag = Some(TagAttempt {
        origin: Base::First,
        attempted_base: Destination::Third,
        home_before_tag: vec![],
    });
    s.teams[0].next_batter = 8;
    let t = project(&s, &fielders(&s), &d).unwrap();
    assert_eq!(t.after().teams[0].next_batter, 0);
    let mut e = t.event().clone();
    let f = t.hit_runner_tag_facts().unwrap().clone();
    e.delta.moves[0].to = Destination::Out;
    assert!(apply_with_hit_runner_tag_facts(&s, e, f).is_err());
}

#[test]
fn tag_clock_cannot_place_a_trailing_home_crossing_before_its_leader() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            let s = state(7, outs, half);
            let mut d = decision(3, [Some(0), Some(4), Some(4)]);
            d.tag = Some(TagAttempt {
                origin: Base::First,
                attempted_base: Destination::Home,
                home_before_tag: vec![Base::Second],
            });
            assert!(project(&s, &fielders(&s), &d).is_err());
            d.tag.as_mut().unwrap().home_before_tag = vec![Base::Second, Base::Third];
            checked(&s, &d);
        }
    }
}
