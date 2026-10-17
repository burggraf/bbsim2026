use bbsim_engine::{accounting::derive_accounting, *};

fn state(mask: u8, outs: u8, half: Half) -> GameState {
    let teams = ["a", "h"].map(|p| TeamState {
        batting_order: std::array::from_fn(|i| format!("{p}{i}")),
        next_batter: 0,
        pitcher: format!("{p}p"),
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
fn credit(id: &str, kind: CreditKind) -> Credit {
    Credit {
        player_id: id.into(),
        kind,
    }
}
fn fixture(
    s: &GameState,
    value: usize,
    retired: usize,
    at: Destination,
) -> (Event, HitRunnerTagFacts) {
    let offense = &s.teams[s.half.offense()];
    let defense = &s.teams[1 - s.half.offense()];
    let batter = &offense.batting_order[0];
    let runner = s.bases[retired].as_ref().unwrap();
    let facts = HitRunnerTagFacts {
        batter_id: batter.clone(),
        runner_id: runner.player_id.clone(),
        attempted_base: at,
        thrower_id: defense.batting_order[1].clone(),
        tagger_id: defense.batting_order[2].clone(),
        order: HitRunnerTagOrder::BatterAttainedCreditedBaseBeforeTag,
        home_before_tag: vec![],
    };
    let mut moves = vec![RunnerMove {
        runner: Runner {
            player_id: batter.clone(),
            responsible_pitcher: defense.pitcher.clone(),
        },
        from: Base::Plate,
        to: [Destination::First, Destination::Second, Destination::Third][value - 1],
    }];
    let mut credits = vec![
        credit(batter, CreditKind::PlateAppearance),
        credit(batter, CreditKind::AtBat),
        credit(batter, CreditKind::Hit),
        credit(
            batter,
            [CreditKind::Single, CreditKind::Double, CreditKind::Triple][value - 1],
        ),
        credit(&facts.thrower_id, CreditKind::Assist),
        credit(&facts.tagger_id, CreditKind::Putout),
    ];
    let mut facts = facts;
    for (i, r) in s
        .bases
        .iter()
        .enumerate()
        .filter_map(|(i, r)| r.as_ref().map(|r| (i, r)))
    {
        let to = if i == retired {
            Destination::Out
        } else if i + 1 > value {
            [Destination::First, Destination::Second, Destination::Third][i]
        } else {
            Destination::Home
        };
        if to == Destination::Home {
            facts.home_before_tag.push(r.player_id.clone());
            credits.push(credit(&r.player_id, CreditKind::Run));
        }
        moves.push(RunnerMove {
            runner: r.clone(),
            from: [Base::First, Base::Second, Base::Third][i],
            to,
        });
    }
    let event = Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(batter.clone()),
            pitcher: defense.pitcher.clone(),
        },
        fielding_review: FieldingReview::NoMisplay,
        delta: Delta {
            moves,
            outs: 1,
            runs: facts.home_before_tag.len() as u8,
            credits,
            changes: vec![],
            third_out: (s.outs == 2).then(|| ThirdOut {
                runner_id: facts.runner_id.clone(),
                kind: ThirdOutKind::Timing,
                home_before_out: facts.home_before_tag.clone(),
            }),
        },
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        ordinary_ground_throw: None,
        ordinary_air_catch: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        muffed_ground_ball: None,
    };
    (event, facts)
}

#[test]
fn all_masks_outs_halfs_values_and_forward_tag_targets() {
    let mut cells = 0;
    let mut accepted = 0;
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            for mask in 1..8 {
                for value in 1..=3 {
                    if mask == 1 {
                        let empty = state(0, outs, half);
                        let (e, f) = fixture(&state(1, outs, half), value, 0, Destination::Home);
                        assert!(apply_with_hit_runner_tag_facts(&empty, e, f).is_err());
                    }
                    for retired in 0..3 {
                        if mask & (1 << retired) == 0 {
                            continue;
                        }
                        for at in [Destination::Second, Destination::Third, Destination::Home]
                            .into_iter()
                            .skip(retired)
                        {
                            let s = state(mask, outs, half);
                            let (event, facts) = fixture(&s, value, retired, at);
                            assert_eq!(
                                apply(&s, event.clone()),
                                Err(TransitionError::UnsupportedRule)
                            );
                            // The bounded proof excludes live forces and missing no-passing chronology.
                            let attempted = match at {
                                Destination::Second => 2,
                                Destination::Third => 3,
                                Destination::Home => 4,
                                _ => unreachable!(),
                            };
                            let live_force =
                                mask & ((1 << (retired + 1)) - 1) == (1 << (retired + 1)) - 1;
                            let ambiguous_endpoint = event.delta.moves.iter().any(|m| {
                                let from = match m.from {
                                    Base::Plate => 0,
                                    Base::First => 1,
                                    Base::Second => 2,
                                    Base::Third => 3,
                                };
                                let to = match m.to {
                                    Destination::First => 1,
                                    Destination::Second => 2,
                                    Destination::Third => 3,
                                    Destination::Home => 4,
                                    Destination::Out => 0,
                                };
                                from != 0
                                    && to != 0
                                    && (to <= value
                                        || (from < retired + 1 && to >= attempted)
                                        || (from > retired + 1 && to < attempted))
                            });
                            // Independent fixture oracle: origins through `value` go Home;
                            // higher surviving origins hold, so their ranks must stay ordered.
                            let safe_endpoints: Vec<_> = (1..=3)
                                .filter(|origin| {
                                    mask & (1 << (origin - 1)) != 0 && *origin != retired + 1
                                })
                                .map(|origin| if origin <= value { 4 } else { origin })
                                .collect();
                            let survivor_passing =
                                safe_endpoints.windows(2).any(|pair| pair[0] > pair[1]);
                            cells += 1;
                            if (live_force && attempted == retired + 2)
                                || value >= attempted
                                || ambiguous_endpoint
                                || survivor_passing
                            {
                                assert_eq!(apply_with_hit_runner_tag_facts(&s, event, facts), Err(TransitionError::UnsupportedRule), "mask={mask} outs={outs} half={half:?} hit={value} runner={retired} at={at:?}");
                                continue;
                            }
                            let t =
                                apply_with_hit_runner_tag_facts(&s, event.clone(), facts.clone())
                                    .unwrap();
                            accepted += 1;
                            assert_eq!(t.hit_runner_tag_facts(), Some(&facts));
                            assert!(t.fielding_contribution().is_none());
                            if event.delta.runs > 0 {
                                assert!(t.rbi_contribution().is_none());
                            }
                            assert_eq!(t.event(), &event);
                            assert_eq!(t.event().delta.outs, 1);
                            assert_eq!(t.after().teams[half.offense()].next_batter, 1);
                            assert_eq!(t.after().outs, (outs + 1) % 3);
                            assert!(t.event().sole_safe_hit.is_none());
                            assert!(t
                                .event()
                                .delta
                                .moves
                                .iter()
                                .all(|m| m.from != Base::Plate || m.to != Destination::Out));
                            for m in &t.event().delta.moves {
                                if m.from != Base::Plate {
                                    assert!(m.runner.responsible_pitcher.starts_with("inherited-"));
                                }
                                if outs < 2 && !matches!(m.to, Destination::Home | Destination::Out)
                                {
                                    assert!(t.after().bases.contains(&Some(m.runner.clone())));
                                }
                            }
                            let a = derive_accounting(std::slice::from_ref(&t)).unwrap();
                            let line = &a.batters[half.offense()][&facts.batter_id];
                            assert_eq!(line.plate_appearances, 1);
                            assert_eq!(line.at_bats, Some(1));
                            assert_eq!(line.credited_hits, 1);
                            assert_eq!(
                                line.credited_hit_breakdown.as_ref().unwrap().total_bases(),
                                value as u32
                            );
                            assert_eq!(
                                a.pitchers[1 - half.offense()][&event.participants.pitcher]
                                    .batters_faced,
                                1
                            );
                            let pitcher =
                                &a.pitchers[1 - half.offense()][&event.participants.pitcher];
                            assert_eq!(pitcher.outs_recorded, 1);
                            assert_eq!(pitcher.at_bats_against, Some(1));
                            assert_eq!(pitcher.credited_hits_allowed, 1);
                            assert_eq!(
                                a.fielders[1 - half.offense()][&facts.thrower_id].credited_assists,
                                1
                            );
                            assert_eq!(
                                a.fielders[1 - half.offense()][&facts.tagger_id].credited_putouts,
                                1
                            );
                            assert_eq!(&t.to_bytes()[..5], b"BBE31");
                            assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
                        }
                    }
                }
            }
        }
    }
    assert_eq!(cells, 432);
    assert_eq!(accepted, 144); // 24 nonpassing joint shapes × 3 outs × 2 halfs.
}

#[test]
fn explicit_third_out_clock_scores_only_named_pre_tag_crossings() {
    let s = state(7, 2, Half::Top);
    let (mut e, mut f) = fixture(&s, 3, 0, Destination::Home);
    let scorer = s.bases[2].as_ref().unwrap().player_id.clone();
    f.home_before_tag = vec![scorer.clone()];
    e.delta.third_out.as_mut().unwrap().home_before_out = vec![scorer.clone()];
    e.delta.runs = 1;
    e.delta
        .credits
        .retain(|c| c.kind != CreditKind::Run || c.player_id == scorer);
    let t = apply_with_hit_runner_tag_facts(&s, e.clone(), f.clone()).unwrap();
    assert_eq!(t.after().score, [1, 0]);
    assert!(t.after().bases.iter().all(Option::is_none));
    let a = derive_accounting(&[t]).unwrap();
    assert_eq!(a.pitchers[1]["inherited-2"].runs_allowed, 1);
    assert_eq!(a.pitchers[1]["hp"].runs_allowed, 0);
    assert_eq!(a.pitchers[1]["inherited-2"].earned_runs, None);
    f.home_before_tag.clear();
    e.delta.third_out.as_mut().unwrap().home_before_out.clear();
    e.delta.runs = 0;
    e.delta.credits.retain(|c| c.kind != CreditKind::Run);
    assert_eq!(
        apply_with_hit_runner_tag_facts(&s, e.clone(), f.clone())
            .unwrap()
            .after()
            .score,
        [0, 0]
    );
    e.delta.third_out = None;
    assert_eq!(
        apply_with_hit_runner_tag_facts(&s, e, f),
        Err(TransitionError::UnsupportedRule)
    );
}

#[test]
fn tag_clock_cannot_certify_a_trailing_home_crossing_before_its_leader() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            let before = state(7, outs, half);
            let (mut event, mut facts) = fixture(&before, 3, 0, Destination::Home);
            let baseline =
                apply_with_hit_runner_tag_facts(&before, event.clone(), facts.clone()).unwrap();
            let scorer = before.bases[1].as_ref().unwrap().player_id.clone();
            facts.home_before_tag = vec![scorer.clone()];
            let mut after = baseline.after().clone();
            if outs == 2 {
                event.delta.third_out.as_mut().unwrap().home_before_out = vec![scorer.clone()];
                event.delta.runs = 1;
                event
                    .delta
                    .credits
                    .retain(|c| c.kind != CreditKind::Run || c.player_id == scorer);
                after.score[half.offense()] = 1;
            }
            if outs < 2 {
                let mut valid = facts.clone();
                let leader = before.bases[2].as_ref().unwrap().player_id.clone();
                valid.home_before_tag = vec![leader.clone()];
                let t = apply_with_hit_runner_tag_facts(&before, event.clone(), valid).unwrap();
                let mut forged = t.to_bytes();
                assert!(forged.ends_with(leader.as_bytes()));
                let start = forged.len() - scorer.len();
                forged[start..].copy_from_slice(scorer.as_bytes());
                assert!(EventTransition::from_bytes(&forged).is_err());
            }
            let saved = before.clone();
            assert_eq!(
                apply_with_hit_runner_tag_facts(&before, event.clone(), facts.clone()),
                Err(TransitionError::UnsupportedRule)
            );
            assert_eq!(
                EventTransition::try_new_with_hit_runner_tag_facts(
                    before.clone(),
                    event,
                    after,
                    facts
                ),
                Err(TransitionError::UnsupportedRule)
            );
            assert_eq!(before, saved);
        }
    }
}

#[test]
fn native_tag_rejects_passing_between_surviving_runners_not_only_passing_the_tagged_runner() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            for scoring in [false, true] {
                for reverse in [false, true] {
                    let before = state(7, outs, half);
                    let (mut event, mut facts) = fixture(
                        &before,
                        1,
                        if scoring { 0 } else { 2 },
                        if scoring {
                            Destination::Third
                        } else {
                            Destination::Home
                        },
                    );
                    for m in &mut event.delta.moves {
                        if !scoring && m.from == Base::First {
                            m.to = Destination::Third;
                        }
                        if !scoring && m.from == Base::Second {
                            m.to = Destination::Second;
                        }
                        if scoring && m.from == Base::Second {
                            m.to = Destination::Home;
                        }
                        if scoring && m.from == Base::Third {
                            m.to = Destination::Third;
                        }
                    }
                    if scoring {
                        event.delta.credits.push(credit(
                            &before.bases[1].as_ref().unwrap().player_id,
                            CreditKind::Run,
                        ));
                    }
                    facts.home_before_tag = if scoring {
                        vec![before.bases[1].as_ref().unwrap().player_id.clone()]
                    } else {
                        vec![]
                    };
                    event.delta.runs = u8::from(scoring);
                    event.delta.credits.retain(|c| {
                        c.kind != CreditKind::Run || facts.home_before_tag.contains(&c.player_id)
                    });
                    if let Some(third) = &mut event.delta.third_out {
                        third.home_before_out = facts.home_before_tag.clone();
                    }
                    if reverse {
                        event.delta.moves.reverse();
                        event.delta.credits.reverse();
                    }
                    let mut after = before.clone();
                    after.teams[half.offense()].next_batter = 1;
                    after.score[half.offense()] += u16::from(scoring);
                    after.outs = (outs + 1) % 3;
                    after.bases = if outs == 2 {
                        [None, None, None]
                    } else {
                        let batter = Runner {
                            player_id: facts.batter_id.clone(),
                            responsible_pitcher: before.teams[1 - half.offense()].pitcher.clone(),
                        };
                        if scoring {
                            [Some(batter), None, before.bases[2].clone()]
                        } else {
                            [
                                Some(batter),
                                before.bases[1].clone(),
                                before.bases[0].clone(),
                            ]
                        }
                    };
                    if outs == 2 {
                        if half == Half::Top {
                            after.half = Half::Bottom;
                        } else {
                            after.half = Half::Top;
                            after.inning += 1;
                        }
                    }
                    assert_eq!(
                        apply_with_hit_runner_tag_facts(&before, event.clone(), facts.clone()),
                        Err(TransitionError::UnsupportedRule)
                    );
                    assert_eq!(
                        EventTransition::try_new_with_hit_runner_tag_facts(
                            before.clone(),
                            event,
                            after,
                            facts
                        ),
                        Err(TransitionError::UnsupportedRule)
                    );
                }
            }
        }
    }
}

#[test]
fn bbe31_revalidates_survivor_order_even_when_third_out_clears_both_destinations() {
    fn string_bytes(text: &str) -> Vec<u8> {
        let mut b = (text.len() as u64).to_le_bytes().to_vec();
        b.extend_from_slice(text.as_bytes());
        b
    }
    fn replace(bytes: &mut [u8], old: &[u8], new: &[u8], count: usize) {
        assert_eq!(old.len(), new.len());
        let offsets: Vec<_> = bytes
            .windows(old.len())
            .enumerate()
            .filter_map(|(i, b)| (b == old).then_some(i))
            .collect();
        assert_eq!(offsets.len(), count);
        for i in offsets {
            bytes[i..i + old.len()].copy_from_slice(new);
        }
    }
    for half in [Half::Top, Half::Bottom] {
        let before = state(7, 2, half);
        let second = before.bases[1].as_ref().unwrap();
        let third = before.bases[2].as_ref().unwrap();
        let (mut event, mut facts) = fixture(&before, 1, 0, Destination::Third);
        for m in &mut event.delta.moves {
            if m.from == Base::Second {
                m.to = Destination::Third;
            }
            if m.from == Base::Third {
                m.to = Destination::Home;
            }
        }
        facts.home_before_tag = vec![third.player_id.clone()];
        event.delta.third_out.as_mut().unwrap().home_before_out = facts.home_before_tag.clone();
        event.delta.runs = 1;
        event
            .delta
            .credits
            .push(credit(&third.player_id, CreditKind::Run));
        let legal = apply_with_hit_runner_tag_facts(&before, event.clone(), facts).unwrap();
        let mut bytes = legal.to_bytes();
        for (runner, from, old_to, new_to) in [(second, 2, 2, 3), (third, 3, 3, 2)] {
            let mut old = string_bytes(&runner.player_id);
            old.extend(string_bytes(&runner.responsible_pitcher));
            old.extend([from, old_to]);
            let mut new = old.clone();
            *new.last_mut().unwrap() = new_to;
            replace(&mut bytes, &old, &new, 1);
        }
        let mut old = string_bytes(&third.player_id);
        old.push(4);
        let mut new = string_bytes(&second.player_id);
        new.push(4);
        replace(&mut bytes, &old, &new, 1);
        let mut old = 1u64.to_le_bytes().to_vec();
        old.extend(string_bytes(&third.player_id));
        let mut new = 1u64.to_le_bytes().to_vec();
        new.extend(string_bytes(&second.player_id));
        replace(&mut bytes, &old, &new, 2);
        // One counted run and cleared after-bases stay exact; only runner order is false.
        assert_eq!(
            EventTransition::from_bytes(&bytes),
            Err(TransitionError::UnsupportedRule)
        );
    }
}

#[test]
fn refuses_winning_tag_even_without_third_out_but_keeps_nonwinning_endings() {
    for outs in 0..3 {
        for inning in [9, 10] {
            let mut s = state(3, outs, Half::Bottom);
            s.inning = inning;
            let (e, f) = fixture(&s, 2, 0, Destination::Home);
            assert_eq!(
                apply_with_hit_runner_tag_facts(&s, e.clone(), f.clone()),
                Err(TransitionError::UnsupportedRule)
            );
            s.score = [2, 0];
            let t = apply_with_hit_runner_tag_facts(&s, e, f).unwrap();
            assert_eq!(t.after().over, outs == 2);
        }
    }
    let mut s = state(1, 2, Half::Top);
    s.inning = 9;
    s.score = [0, 1];
    let (e, f) = fixture(&s, 1, 0, Destination::Third);
    assert!(
        apply_with_hit_runner_tag_facts(&s, e, f)
            .unwrap()
            .after()
            .over
    );
}

#[test]
fn movement_credit_and_clock_vector_order_is_not_chronology() {
    let s = state(7, 2, Half::Top);
    let (e, f) = fixture(&s, 3, 0, Destination::Home);
    let expected = apply_with_hit_runner_tag_facts(&s, e.clone(), f.clone()).unwrap();
    for rotation in 0..4 {
        let mut e = e.clone();
        let mut f = f.clone();
        e.delta.moves.rotate_left(rotation);
        e.delta.credits.reverse();
        e.delta
            .third_out
            .as_mut()
            .unwrap()
            .home_before_out
            .reverse();
        f.home_before_tag.reverse();
        let t = apply_with_hit_runner_tag_facts(&s, e, f).unwrap();
        assert_eq!(t.after(), expected.after());
        assert_eq!(
            derive_accounting(&[t]).unwrap(),
            derive_accounting(std::slice::from_ref(&expected)).unwrap()
        );
    }
}

#[test]
fn malformed_facts_actors_and_destinations_are_rejected() {
    let s = state(7, 2, Half::Top);
    let (e, f) = fixture(&s, 3, 0, Destination::Home);
    for id in ["", "a0", "hbench", "hremoved", "h8", "absent"] {
        let mut bad = f.clone();
        bad.thrower_id = id.into();
        assert!(
            apply_with_hit_runner_tag_facts(&s, e.clone(), bad).is_err(),
            "thrower {id}"
        );
        let mut bad = f.clone();
        bad.tagger_id = id.into();
        assert!(
            apply_with_hit_runner_tag_facts(&s, e.clone(), bad).is_err(),
            "tagger {id}"
        );
    }
    let mut bad = f.clone();
    bad.tagger_id = bad.thrower_id.clone();
    assert!(apply_with_hit_runner_tag_facts(&s, e.clone(), bad).is_err());
    for id in ["", "wrong", "a0", "a4"] {
        let mut bad = f.clone();
        bad.runner_id = id.into();
        assert!(apply_with_hit_runner_tag_facts(&s, e.clone(), bad).is_err());
    }
    let mut bad = f.clone();
    bad.batter_id = "a3".into();
    assert!(apply_with_hit_runner_tag_facts(&s, e.clone(), bad).is_err());
    for at in [Destination::First, Destination::Out] {
        let mut bad = f.clone();
        bad.attempted_base = at;
        assert!(apply_with_hit_runner_tag_facts(&s, e.clone(), bad).is_err());
    }
    for retired in 1..3 {
        let (e, mut f) = fixture(&s, 1, retired, Destination::Home);
        f.attempted_base = [Destination::Second, Destination::Third][retired - 1];
        assert!(apply_with_hit_runner_tag_facts(&s, e, f).is_err());
    }
    let empty = state(0, 0, Half::Top);
    assert!(apply_with_hit_runner_tag_facts(&empty, e.clone(), f.clone()).is_err());
    for kind in [
        ThirdOutKind::Force,
        ThirdOutKind::BatterBeforeFirst,
        ThirdOutKind::BatterAfterFirstTiming,
        ThirdOutKind::PrecedingRunnerAppeal,
    ] {
        let mut bad = e.clone();
        bad.delta.third_out.as_mut().unwrap().kind = kind;
        assert!(apply_with_hit_runner_tag_facts(&s, bad, f.clone()).is_err());
    }
    let mut bad = f.clone();
    bad.home_before_tag.clear();
    assert!(apply_with_hit_runner_tag_facts(&s, e.clone(), bad).is_err());
    let mut bad = f.clone();
    bad.home_before_tag.push(bad.home_before_tag[0].clone());
    assert!(apply_with_hit_runner_tag_facts(&s, e.clone(), bad).is_err());
    let mut bad = e.clone();
    bad.delta.third_out.as_mut().unwrap().runner_id = "a4".into();
    assert!(apply_with_hit_runner_tag_facts(&s, bad, f).is_err());
}

#[test]
fn malformed_credits_movements_and_overlapping_causes_are_rejected() {
    let s = state(1, 0, Half::Top);
    let (e, f) = fixture(&s, 1, 0, Destination::Third);
    for i in 0..e.delta.credits.len() {
        let mut bad = e.clone();
        bad.delta.credits.push(bad.delta.credits[i].clone());
        assert!(apply_with_hit_runner_tag_facts(&s, bad, f.clone()).is_err());
        if e.delta.credits[i].kind != CreditKind::AtBat {
            let mut bad = e.clone();
            bad.delta.credits.remove(i);
            assert!(apply_with_hit_runner_tag_facts(&s, bad, f.clone()).is_err());
        }
        let mut bad = e.clone();
        bad.delta.credits[i].player_id = "wrong".into();
        assert!(apply_with_hit_runner_tag_facts(&s, bad, f.clone()).is_err());
    }
    for kind in [
        CreditKind::Error,
        CreditKind::NoHit,
        CreditKind::CaughtStealing,
        CreditKind::SacrificeFly,
        CreditKind::Rbi,
    ] {
        let mut bad = e.clone();
        bad.delta.credits.push(credit("a0", kind));
        assert!(apply_with_hit_runner_tag_facts(&s, bad, f.clone()).is_err());
    }
    for review in [FieldingReview::Unknown, FieldingReview::SoleGroundMuff] {
        let mut bad = e.clone();
        bad.fielding_review = review;
        assert!(apply_with_hit_runner_tag_facts(&s, bad, f.clone()).is_err());
    }
    let mut bad = e.clone();
    bad.sole_safe_hit = Some(SoleSafeHitFacts {
        batter_id: "a0".into(),
    });
    assert!(apply_with_hit_runner_tag_facts(&s, bad, f.clone()).is_err());
    let mut bad = e.clone();
    bad.sole_home_run = Some(SoleHomeRunFacts {
        batter_id: "a0".into(),
    });
    assert!(apply_with_hit_runner_tag_facts(&s, bad, f.clone()).is_err());
    let mut bad = e.clone();
    bad.protected_award = Some(ProtectedAwardFacts {
        batter_id: "a0".into(),
    });
    assert!(apply_with_hit_runner_tag_facts(&s, bad, f.clone()).is_err());
    let mut bad = e.clone();
    bad.ordinary_ground_throw = Some(OrdinaryGroundThrowFacts {
        batter_id: "a0".into(),
        thrower_id: "h1".into(),
        receiver_id: "h2".into(),
    });
    assert!(apply_with_hit_runner_tag_facts(&s, bad, f.clone()).is_err());
    let mut bad = e.clone();
    bad.ordinary_air_catch = Some(OrdinaryAirCatchFacts {
        batter_id: "a0".into(),
        fielder_id: "h1".into(),
        contact: AirContact::Fly,
    });
    assert!(apply_with_hit_runner_tag_facts(&s, bad, f.clone()).is_err());
    let mut bad = e.clone();
    bad.caught_strikeout = Some(CaughtStrikeoutFacts {
        batter_id: "a0".into(),
        catcher_id: "h2".into(),
    });
    assert!(apply_with_hit_runner_tag_facts(&s, bad, f.clone()).is_err());
    let mut bad = e.clone();
    bad.muffed_ground_ball = Some(MuffedGroundBallFacts {
        batter_id: "a0".into(),
        fielder_id: "h1".into(),
    });
    assert!(apply_with_hit_runner_tag_facts(&s, bad, f.clone()).is_err());
    let mut bad = e.clone();
    bad.balk = Some(BalkFacts {
        cause: BalkCause::Unknown,
        disposition: BalkDisposition::Unknown,
        resolution: BalkResolution::Unknown,
    });
    assert!(apply_with_hit_runner_tag_facts(&s, bad, f.clone()).is_err());
    let mut bad = e.clone();
    bad.delta.moves[0].to = Destination::Out;
    bad.delta.outs = 2;
    assert!(apply_with_hit_runner_tag_facts(&s, bad, f.clone()).is_err());
    let mut bad = e.clone();
    bad.delta.moves[0].to = Destination::Second;
    assert!(apply_with_hit_runner_tag_facts(&s, bad, f.clone()).is_err());
    let mut bad = e.clone();
    bad.delta.moves[1].runner.responsible_pitcher = "wrong".into();
    assert!(apply_with_hit_runner_tag_facts(&s, bad, f.clone()).is_err());
    let mut bad = e.clone();
    bad.delta.moves[1].from = Base::Second;
    assert!(apply_with_hit_runner_tag_facts(&s, bad, f.clone()).is_err());
    let mut bad = e.clone();
    bad.delta.moves.pop();
    bad.delta.outs = 0;
    assert!(apply_with_hit_runner_tag_facts(&s, bad, f).is_err());
}

#[test]
fn codec_revalidates_facts_and_supplied_after_and_preserves_legacy_bytes() {
    let s = state(1, 0, Half::Top);
    let (e, f) = fixture(&s, 1, 0, Destination::Third);
    let t = apply_with_hit_runner_tag_facts(&s, e.clone(), f.clone()).unwrap();
    assert_eq!(
        EventTransition::try_new_with_hit_runner_tag_facts(
            s.clone(),
            e.clone(),
            t.after().clone(),
            f.clone()
        )
        .unwrap(),
        t
    );
    let mut after = t.after().clone();
    after.score[0] += 1;
    assert!(EventTransition::try_new_with_hit_runner_tag_facts(
        s.clone(),
        e.clone(),
        after,
        f.clone()
    )
    .is_err());
    assert_eq!(
        EventTransition::try_new(s.clone(), e.clone(), t.after().clone()),
        Err(TransitionError::UnsupportedRule)
    );
    let bytes = t.to_bytes();
    for n in 0..bytes.len() {
        assert!(EventTransition::from_bytes(&bytes[..n]).is_err());
    }
    let mut bad = bytes.clone();
    bad.push(0);
    assert!(EventTransition::from_bytes(&bad).is_err());
    let pos = bytes.windows(2).rposition(|v| v == b"h2").unwrap();
    let mut bad = bytes.clone();
    bad[pos] = b'a';
    assert!(EventTransition::from_bytes(&bad).is_err());
    let mut bad = bytes.clone();
    bad[bytes.len() - 9] = 1; // Unknown typed order; empty home list occupies final 8 bytes.
    assert_eq!(
        EventTransition::from_bytes(&bad),
        Err(TransitionError::InvalidEncoding)
    );
    let mut bad = bytes.clone();
    bad[..5].copy_from_slice(b"BBE25");
    assert!(EventTransition::from_bytes(&bad).is_err());
    // Removing the new facts and changing the tag to a safe advance is a legacy no-facts hit.
    let mut legacy = e;
    legacy.delta.moves[1].to = Destination::Third;
    legacy.delta.outs = 0;
    legacy
        .delta
        .credits
        .retain(|c| !matches!(c.kind, CreditKind::Assist | CreditKind::Putout));
    let old = apply(&s, legacy).unwrap();
    assert_eq!(&old.to_bytes()[..5], b"BBE25");
    assert_eq!(
        EventTransition::from_bytes(&old.to_bytes())
            .unwrap()
            .to_bytes(),
        old.to_bytes()
    );
}

#[test]
fn eligible_pitcher_roles_and_implicit_hit_at_bat_preserve_unknown_coverage() {
    for half in [Half::Top, Half::Bottom] {
        for dh in [false, true] {
            for pitcher_throws in [false, true] {
                let mut s = state(1, 0, half);
                let defense = &mut s.teams[1 - half.offense()];
                if !dh {
                    defense.dh_slot = None;
                    defense.batting_order[8] = defense.pitcher.clone();
                }
                let pitcher = defense.pitcher.clone();
                let (mut e, mut f) = fixture(&s, 2, 0, Destination::Home);
                let (old, kind) = if pitcher_throws {
                    let old = f.thrower_id.clone();
                    f.thrower_id = pitcher.clone();
                    (old, CreditKind::Assist)
                } else {
                    let old = f.tagger_id.clone();
                    f.tagger_id = pitcher.clone();
                    (old, CreditKind::Putout)
                };
                e.delta
                    .credits
                    .iter_mut()
                    .find(|c| c.kind == kind && c.player_id == old)
                    .unwrap()
                    .player_id = pitcher;
                e.delta.credits.retain(|c| c.kind != CreditKind::AtBat);
                let t = apply_with_hit_runner_tag_facts(&s, e, f).unwrap();
                let a = derive_accounting(&[t]).unwrap();
                assert_eq!(
                    a.batters[half.offense()][&s.teams[half.offense()].batting_order[0]].at_bats,
                    Some(1)
                );
            }
        }
    }
}
