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
        single_delivery: None,
        balk: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        muffed_ground_ball: None,
        ordinary_ground_throw: None,
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
fn explicit_no_hit_covers_observed_pa_without_inventing_ab_or_fielding() {
    for half in [Half::Top, Half::Bottom] {
        for supplied_ab in [false, true] {
            let mut before = state();
            before.half = half;
            before.bases = [None, None, None];
            let offense = half.offense();
            let batter = before.teams[offense].batting_order[0].clone();
            let pitcher = before.teams[1 - offense].pitcher.clone();
            let mut credits = vec![credit(&batter, CreditKind::NoHit)];
            if supplied_ab {
                credits.push(credit(&batter, CreditKind::AtBat));
            }
            let action = event(
                &before,
                EventKind::PlateAppearance,
                vec![batter_move(&before, Destination::Out)],
                1,
                0,
                credits,
                vec![],
            );
            let t = apply(&before, action).unwrap();
            assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
            let a = derive_accounting(&[t]).unwrap();
            let b = &a.batters[offense][&batter];
            let p = &a.pitchers[1 - offense][&pitcher];
            assert_eq!(b.classified_hits, Some(0));
            assert_eq!(p.classified_hits_allowed, Some(0));
            assert_eq!(
                (b.plate_appearances, p.batters_faced, p.outs_recorded),
                (1, 1, 1)
            );
            assert_eq!((b.credited_hits, p.credited_hits_allowed), (0, 0));
            assert_eq!(b.at_bats, if supplied_ab { Some(1) } else { None });
            assert_eq!(p.at_bats_against, b.at_bats);
            assert!(a.fielders.iter().all(|f| f.is_empty()));
            assert!(!a.complete_game);
        }
    }
}

#[test]
fn hit_classification_does_not_infer_value_and_bare_ab_is_unknown() {
    for half in [Half::Top, Half::Bottom] {
        for kind in [
            CreditKind::Hit,
            CreditKind::AtBat,
            CreditKind::Strikeout,
            CreditKind::Walk,
            CreditKind::HitByPitch,
            CreditKind::InterferenceAward,
        ] {
            let mut before = state();
            before.half = half;
            before.bases = [None, None, None];
            let offense = half.offense();
            let b = before.teams[offense].batting_order[0].clone();
            let p = before.teams[1 - offense].pitcher.clone();
            let retired = matches!(kind, CreditKind::AtBat | CreditKind::Strikeout);
            let t = apply(
                &before,
                event(
                    &before,
                    EventKind::PlateAppearance,
                    vec![batter_move(
                        &before,
                        if retired {
                            Destination::Out
                        } else {
                            Destination::First
                        },
                    )],
                    u8::from(retired),
                    0,
                    vec![credit(&b, kind)],
                    vec![],
                ),
            )
            .unwrap();
            let a = derive_accounting(&[t]).unwrap();
            let expected = match kind {
                CreditKind::Hit => Some(1),
                CreditKind::AtBat => None,
                _ => Some(0),
            };
            assert_eq!(
                a.batters[offense][&b].classified_hits, expected,
                "{half:?} {kind:?}"
            );
            assert_eq!(
                a.pitchers[1 - offense][&p].classified_hits_allowed,
                expected
            );
            if kind == CreditKind::Hit {
                assert_eq!(a.batters[offense][&b].credited_hit_breakdown, None);
            }
        }
    }
}

#[test]
fn complete_physical_log_preserves_actor_local_missing_hit_coverage() {
    let mut before = state();
    before.inning = 1;
    before.outs = 0;
    before.bases = [None, None, None];
    let mut log = vec![];
    for frame in 0..17 {
        // Home scores twice in the eighth; Top9 has only three K-outs.
        // An away lead after Top9 would NOT end a regulation game.
        if frame == 15 {
            let b = before.teams[1].batting_order[before.teams[1].next_batter as usize].clone();
            let homer = event(
                &before,
                EventKind::OutOfParkHomeRun,
                vec![batter_move(&before, Destination::Home)],
                0,
                1,
                vec![
                    credit(&b, CreditKind::PlateAppearance),
                    credit(&b, CreditKind::Hit),
                    credit(&b, CreditKind::Run),
                ],
                vec![],
            );
            before = record_clean(&mut log, &before, homer);
        }
        for play in 0..4 {
            if frame == 16 && play == 0 {
                continue;
            }
            let b = before.teams[before.half.offense()].batting_order
                [before.teams[before.half.offense()].next_batter as usize]
                .clone();
            let action = if play == 0 {
                event(
                    &before,
                    EventKind::OutOfParkHomeRun,
                    vec![batter_move(&before, Destination::Home)],
                    0,
                    1,
                    vec![
                        credit(&b, CreditKind::PlateAppearance),
                        credit(&b, CreditKind::Hit),
                        credit(&b, CreditKind::Run),
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
                    vec![credit(&b, CreditKind::Strikeout)],
                    vec![],
                )
            };
            before = record_clean(&mut log, &before, action);
        }
    }
    assert!(before.over);
    // Replace only the first K classification, retaining identical physical
    // history. NoHit+AB says nothing about its missing retirement/fielding facts.
    let classified = log
        .iter()
        .enumerate()
        .map(|(index, t)| {
            let mut e = t.event().clone();
            if index == 1 {
                e.delta.credits.retain(|c| c.kind != CreditKind::Strikeout);
                e.delta.credits.extend([
                    credit("away1", CreditKind::NoHit),
                    credit("away1", CreditKind::AtBat),
                ]);
            }
            apply(t.before(), e).unwrap()
        })
        .collect::<Vec<_>>();
    let a = derive_accounting(&classified).unwrap();
    assert!(a.complete_game);
    assert_eq!(a.runs, [8, 9]);
    assert_eq!(
        a.pitchers[1]["home_starter"].classified_hits_allowed,
        Some(8)
    );
    assert_eq!(
        a.pitchers[0]["away_pitcher"].classified_hits_allowed,
        Some(9)
    );
    assert!(a.batters[0]
        .iter()
        .all(|(id, b)| b.classified_hits == Some(u32::from(id != "away5"))));
    assert_eq!(a.pitchers[1]["home_starter"].earned_runs, None);
    assert!(a.fielders.iter().all(|f| f.is_empty()));
    let incomplete = classified
        .iter()
        .enumerate()
        .map(|(index, t)| {
            let mut e = t.event().clone();
            if index == 1 {
                e.delta.credits.retain(|c| c.kind != CreditKind::NoHit);
            }
            apply(t.before(), e).unwrap()
        })
        .collect::<Vec<_>>();
    let partial = derive_accounting(&incomplete).unwrap();
    assert!(partial.complete_game);
    assert_eq!(partial.batters[0]["away1"].classified_hits, None);
    assert_eq!(
        partial.pitchers[1]["home_starter"].classified_hits_allowed,
        None
    );
    assert_eq!(
        partial.pitchers[0]["away_pitcher"].classified_hits_allowed,
        Some(9)
    );
    assert!(partial.batters[0]
        .iter()
        .filter(|(id, _)| id.as_str() != "away1")
        .all(|(id, b)| b.classified_hits == Some(u32::from(id != "away5"))));
    assert_eq!(partial.pitchers[1]["home_starter"].credited_hits_allowed, 8);
}

#[test]
fn no_hit_contradictions_precede_inherited_charge_ambiguity() {
    let mut before = state();
    before.bases[0].as_mut().unwrap().responsible_pitcher = "prior".into();
    let moves = vec![
        batter_move(&before, Destination::First),
        RunnerMove {
            runner: before.bases[0].clone().unwrap(),
            from: Base::First,
            to: Destination::Out,
        },
    ];
    let control = apply(
        &before,
        event(
            &before,
            EventKind::PlateAppearance,
            moves.clone(),
            1,
            0,
            vec![credit("away0", CreditKind::Hit)],
            vec![],
        ),
    )
    .unwrap();
    for credits in [
        vec![credit("runner", CreditKind::NoHit)],
        vec![
            credit("away0", CreditKind::NoHit),
            credit("away0", CreditKind::NoHit),
        ],
        vec![
            credit("away0", CreditKind::Hit),
            credit("away0", CreditKind::NoHit),
        ],
    ] {
        let action = event(
            &before,
            EventKind::PlateAppearance,
            moves.clone(),
            1,
            0,
            credits,
            vec![],
        );
        assert_eq!(
            apply(&before, action.clone()),
            Err(TransitionError::IllegalCredit)
        );
        assert_eq!(
            EventTransition::try_new(before.clone(), action, control.after().clone()),
            Err(TransitionError::IllegalCredit)
        );
    }
    let still_ambiguous = event(
        &before,
        EventKind::PlateAppearance,
        moves,
        1,
        0,
        vec![credit("away0", CreditKind::NoHit)],
        vec![],
    );
    assert_eq!(
        apply(&before, still_ambiguous),
        Err(TransitionError::UnsupportedRule)
    );
}

#[test]
fn runner_only_opportunities_do_not_become_no_hit_plate_appearances() {
    let before = state();
    let action = event(
        &before,
        EventKind::RunnerOnly(RunnerEvent::StealAttempt),
        vec![RunnerMove {
            runner: before.bases[0].clone().unwrap(),
            from: Base::First,
            to: Destination::Second,
        }],
        0,
        0,
        vec![],
        vec![],
    );
    let control = apply(&before, action.clone()).unwrap();
    let a = derive_accounting(std::slice::from_ref(&control)).unwrap();
    assert!(a.batters[0].is_empty());
    assert_eq!(a.pitchers[1]["home_starter"].batters_faced, 0);
    assert_eq!(
        a.pitchers[1]["home_starter"].classified_hits_allowed,
        Some(0)
    );
    assert_eq!(
        control.before().teams[0].next_batter,
        control.after().teams[0].next_batter
    );
    let mut illegal = action;
    illegal
        .delta
        .credits
        .push(credit("away0", CreditKind::NoHit));
    assert_eq!(
        apply(&before, illegal.clone()),
        Err(TransitionError::IllegalCredit)
    );
    assert_eq!(
        EventTransition::try_new(before, illegal, control.after().clone()),
        Err(TransitionError::IllegalCredit)
    );
    let bytes = control.to_bytes();
    assert_eq!(EventTransition::from_bytes(&bytes).unwrap(), control);
    // Locate the entire valid one-movement Delta, not a guessed byte offset.
    let mut delta = (1u64).to_le_bytes().to_vec();
    let runner = &control.event().delta.moves[0].runner;
    for id in [&runner.player_id, &runner.responsible_pitcher] {
        delta.extend_from_slice(&(id.len() as u64).to_le_bytes());
        delta.extend_from_slice(id.as_bytes());
    }
    delta.extend_from_slice(&[1, 1, 0, 0]); // First -> Second, u8 outs/u8 runs.
    let credit_offset = delta.len();
    delta.extend_from_slice(&[0; 16]); // empty credits and personnel changes.
    delta.push(0); // no third-out record.
    let positions = bytes
        .windows(delta.len())
        .enumerate()
        .filter_map(|(i, v)| (v == delta).then_some(i))
        .collect::<Vec<_>>();
    assert_eq!(positions.len(), 1);
    let mut extra = (1u64).to_le_bytes().to_vec();
    extra.extend_from_slice(&(5u64).to_le_bytes());
    extra.extend_from_slice(b"away0");
    extra.push(25);
    let start = positions[0] + credit_offset;
    let mut permitted = (1u64).to_le_bytes().to_vec();
    permitted.extend_from_slice(&(runner.player_id.len() as u64).to_le_bytes());
    permitted.extend_from_slice(runner.player_id.as_bytes());
    permitted.push(6); // actual runner's completed stolen base is valid.
    let mut positive = bytes.clone();
    positive.splice(start..start + 8, permitted);
    let decoded = EventTransition::from_bytes(&positive).unwrap();
    assert_eq!(decoded.after(), control.after());
    assert_eq!(
        decoded.event().delta.credits,
        [credit(&runner.player_id, CreditKind::StolenBase)]
    );
    let mut forged = bytes;
    forged.splice(start..start + 8, extra);
    assert_eq!(
        EventTransition::from_bytes(&forged),
        Err(TransitionError::IllegalCredit)
    );
}

#[test]
fn no_hit_semantic_wire_contradiction_has_valid_hit_control() {
    let mut before = state();
    before.bases = [None, None, None];
    let original = apply(
        &before,
        event(
            &before,
            EventKind::PlateAppearance,
            vec![batter_move(&before, Destination::First)],
            0,
            0,
            vec![
                credit("away0", CreditKind::Hit),
                credit("away0", CreditKind::AtBat),
            ],
            vec![],
        ),
    )
    .unwrap();
    let bytes = original.to_bytes();
    assert_eq!(EventTransition::from_bytes(&bytes).unwrap(), original);
    assert!(bytes.starts_with(b"BBE25"));
    let mut needle = (3u64).to_le_bytes().to_vec();
    for tag in [3u8, 12, 0] {
        needle.extend_from_slice(&(5u64).to_le_bytes());
        needle.extend_from_slice(b"away0");
        needle.push(tag);
    }
    let positions = bytes
        .windows(needle.len())
        .enumerate()
        .filter_map(|(i, v)| (v == needle).then_some(i))
        .collect::<Vec<_>>();
    assert_eq!(positions.len(), 1);
    let mut changed = bytes.clone();
    changed[positions[0] + 8 + 2 * (8 + 5 + 1) - 1] = 25; // only AB -> NoHit, full valid layout.
    assert_eq!(
        EventTransition::from_bytes(&changed),
        Err(TransitionError::IllegalCredit)
    );
    // Same valid physical record with explicit NoHit instead of Hit. It is
    // partial about why the batter reached, not a fielding-completeness claim.
    let mut no_hit = bytes.clone();
    no_hit[positions[0] + 8 + (8 + 5 + 1) - 1] = 25;
    assert!(EventTransition::from_bytes(&no_hit).is_ok());
    let mut duplicate = no_hit.clone();
    duplicate[positions[0] + 8 + 2 * (8 + 5 + 1) - 1] = 25;
    assert_eq!(
        EventTransition::from_bytes(&duplicate),
        Err(TransitionError::IllegalCredit)
    );
    let mut foreign = no_hit;
    foreign[positions[0] + 8 + 8 + 4] = b'1'; // NoHit actor only: away0 -> away1.
    assert_eq!(
        EventTransition::from_bytes(&foreign),
        Err(TransitionError::IllegalCredit)
    );
    let mut prior = bytes;
    prior[..5].copy_from_slice(b"BBE18");
    assert_eq!(
        EventTransition::from_bytes(&prior),
        Err(TransitionError::InvalidEncoding)
    );
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
    assert_eq!(box_score.batters[0]["away0"].classified_hits, Some(1));
    assert_eq!(box_score.batters[0]["away1"].classified_hits, None);
    assert_eq!(
        box_score.pitchers[1]["home_reliever"].classified_hits_allowed,
        None
    );
    // Carried run responsibility is not an observed hit allowed/PA opportunity.
    assert_eq!(
        box_score.pitchers[1]["home_starter"].classified_hits_allowed,
        Some(0)
    );
    assert_eq!(box_score.pitchers[1]["home_starter"].batters_faced, 0);
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

// Complete observed half: one safe PA, clean one-base WP steps, then three K.
// These are partial-credit fixtures, not complete official fielding/WP boxes.
fn clean_safe_pa_half(kind: CreditKind, half: Half) -> (Vec<EventTransition>, usize) {
    let mut before = state();
    before.inning = 1;
    before.outs = 0;
    before.bases = [None, None, None];
    let mut log = vec![];
    if half == Half::Bottom {
        finish_with_three_strikeouts(before.clone(), &mut log, FieldingReview::NoMisplay);
        before = log.last().unwrap().after().clone();
    }
    let value = match kind {
        CreditKind::Single | CreditKind::HitByPitch => 1,
        CreditKind::Double => 2,
        CreditKind::Triple => 3,
        _ => panic!("fixture requires a safe classified hit or HBP"),
    };
    let to = match value {
        1 => Destination::First,
        2 => Destination::Second,
        _ => Destination::Third,
    };
    let batter = before.teams[half.offense()].batting_order[0].clone();
    let mut credits = vec![credit(&batter, kind)];
    if kind != CreditKind::HitByPitch {
        credits.push(credit(&batter, CreditKind::Hit));
    }
    let hit_index = log.len();
    let first = apply(
        &before,
        reviewed(event(
            &before,
            EventKind::PlateAppearance,
            vec![batter_move(&before, to)],
            0,
            0,
            credits,
            vec![],
        )),
    )
    .unwrap();
    log.push(first);
    for position in value..=3 {
        let s = log.last().unwrap().after();
        let from = match position {
            1 => Base::First,
            2 => Base::Second,
            _ => Base::Third,
        };
        let to = match position {
            1 => Destination::Second,
            2 => Destination::Third,
            _ => Destination::Home,
        };
        let action = reviewed(event(
            s,
            EventKind::RunnerOnly(RunnerEvent::WildPitch),
            vec![RunnerMove {
                runner: s.bases[position - 1].clone().unwrap(),
                from,
                to,
            }],
            0,
            u8::from(position == 3),
            if position == 3 {
                vec![credit(&batter, CreditKind::Run)]
            } else {
                vec![]
            },
            vec![],
        ));
        log.push(apply(s, action).unwrap());
    }
    finish_with_three_strikeouts(
        log.last().unwrap().after().clone(),
        &mut log,
        FieldingReview::NoMisplay,
    );
    (log, hit_index)
}

#[test]
fn reviewed_classified_safe_hits_certify_literal_earned_run() {
    for kind in [CreditKind::Triple, CreditKind::Single, CreditKind::Double] {
        for half in [Half::Top, Half::Bottom] {
            let (log, hit_index) = clean_safe_pa_half(kind, half);
            for pair in log.windows(2) {
                pair[1].follows(&pair[0]).unwrap();
            }
            for t in &log {
                assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
            }
            let a = derive_accounting(&log).unwrap();
            let offense = half.offense();
            let pitcher = if half == Half::Top {
                "home_starter"
            } else {
                "away_pitcher"
            };
            let p = &a.pitchers[1 - offense][pitcher];
            assert_eq!(a.runs[offense], 1);
            assert_eq!(a.innings[&1][offense], Some(1));
            assert_eq!(p.earned_runs, Some(1), "{half:?} {kind:?}");
            assert_eq!(
                (p.runs_allowed, p.batters_faced, p.outs_recorded),
                (1, 4, 3)
            );
            assert_eq!(p.at_bats_against, Some(4));
            let batter = log[hit_index].event().participants.batter.as_ref().unwrap();
            let b = &a.batters[offense][batter];
            assert_eq!(
                (b.plate_appearances, b.credited_hits, b.credited_runs),
                (1, 1, 1)
            );
            assert_eq!(
                b.credited_hit_breakdown.as_ref().unwrap().total_bases(),
                match kind {
                    CreditKind::Single => 1,
                    CreditKind::Double => 2,
                    _ => 3,
                }
            );
        }
    }
}

#[test]
fn reviewed_simple_hbp_certifies_literal_earned_run() {
    for half in [Half::Top, Half::Bottom] {
        let (log, _) = clean_safe_pa_half(CreditKind::HitByPitch, half);
        let a = derive_accounting(&log).unwrap();
        let pitcher = if half == Half::Top {
            "home_starter"
        } else {
            "away_pitcher"
        };
        let p = &a.pitchers[1 - half.offense()][pitcher];
        assert_eq!(p.earned_runs, Some(1), "{half:?}");
        assert_eq!(
            (p.runs_allowed, p.batters_faced, p.outs_recorded),
            (1, 4, 3)
        );
        assert_eq!(p.at_bats_against, Some(3));
        assert_eq!((p.credited_hit_batters, p.credited_hits_allowed), (1, 0));
    }
}

#[test]
fn safe_hit_er_keeps_missing_value_review_origin_and_end_unknown() {
    for half in [Half::Top, Half::Bottom] {
        // Existing canonical value>1 guard rejects missing whole-play review;
        // use a legal Single for the fold-level uncertainty controls.
        for kind in [CreditKind::Double, CreditKind::Triple] {
            let (classified, index) = clean_safe_pa_half(kind, half);
            let mut e = classified[index].event().clone();
            e.fielding_review = FieldingReview::Unknown;
            assert_eq!(
                apply(classified[index].before(), e),
                Err(TransitionError::UnsupportedRule)
            );
        }
        let (log, hit_index) = clean_safe_pa_half(CreditKind::Single, half);
        let pitcher = if half == Half::Top {
            "home_starter"
        } else {
            "away_pitcher"
        };
        for missing in [hit_index, log.len() - 1] {
            let changed = log
                .iter()
                .enumerate()
                .map(|(index, t)| {
                    let mut e = t.event().clone();
                    if index == missing {
                        e.fielding_review = FieldingReview::Unknown;
                    }
                    apply(t.before(), e).unwrap()
                })
                .collect::<Vec<_>>();
            assert_eq!(
                derive_accounting(&changed).unwrap().pitchers[1 - half.offense()][pitcher]
                    .earned_runs,
                None
            );
        }
        let unclassified = log
            .iter()
            .enumerate()
            .map(|(index, t)| {
                let mut e = t.event().clone();
                if index == hit_index {
                    e.delta.credits.retain(|c| c.kind != CreditKind::Single);
                }
                apply(t.before(), e).unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            derive_accounting(&unclassified).unwrap().pitchers[1 - half.offense()][pitcher]
                .earned_runs,
            None
        );
        assert_eq!(
            derive_accounting(&log[hit_index + 1..]).unwrap().pitchers[1 - half.offense()][pitcher]
                .earned_runs,
            None
        );
        assert_eq!(
            derive_accounting(&log[..log.len() - 1]).unwrap().pitchers[1 - half.offense()][pitcher]
                .earned_runs,
            None
        );
    }
}

fn record_clean(log: &mut Vec<EventTransition>, before: &GameState, action: Event) -> GameState {
    let next = apply(before, reviewed(action)).unwrap();
    let after = next.after().clone();
    log.push(next);
    after
}

// Establish every base mask through observed simple BB/WP episodes, not an
// invented occupied initial snapshot. Then a hit, HR and remaining K-outs.
fn station_hit_half(
    mask: u8,
    outs: u8,
    half: Half,
    value: usize,
    extra_base: bool,
) -> Vec<EventTransition> {
    let mut before = state();
    before.inning = 1;
    before.outs = 0;
    before.bases = [None, None, None];
    let mut log = vec![];
    if half == Half::Bottom {
        finish_with_three_strikeouts(before.clone(), &mut log, FieldingReview::NoMisplay);
        before = log.last().unwrap().after().clone();
    }
    for base in (0..3).rev() {
        if mask & (1 << base) == 0 {
            continue;
        }
        let batter = before.teams[half.offense()].batting_order
            [before.teams[half.offense()].next_batter as usize]
            .clone();
        let walk = event(
            &before,
            EventKind::PlateAppearance,
            vec![batter_move(&before, Destination::First)],
            0,
            0,
            vec![credit(&batter, CreditKind::Walk)],
            vec![],
        );
        before = record_clean(&mut log, &before, walk);
        for step in 0..base {
            let from = if step == 0 { Base::First } else { Base::Second };
            let to = if step == 0 {
                Destination::Second
            } else {
                Destination::Third
            };
            let advance = event(
                &before,
                EventKind::RunnerOnly(RunnerEvent::WildPitch),
                vec![RunnerMove {
                    runner: before.bases[step].clone().unwrap(),
                    from,
                    to,
                }],
                0,
                0,
                vec![],
                vec![],
            );
            before = record_clean(&mut log, &before, advance);
        }
    }
    for _ in 0..outs {
        let batter = before.teams[half.offense()].batting_order
            [before.teams[half.offense()].next_batter as usize]
            .clone();
        let out = event(
            &before,
            EventKind::PlateAppearance,
            vec![batter_move(&before, Destination::Out)],
            1,
            0,
            vec![credit(&batter, CreditKind::Strikeout)],
            vec![],
        );
        before = record_clean(&mut log, &before, out);
    }
    let destinations = [
        Destination::First,
        Destination::Second,
        Destination::Third,
        Destination::Home,
    ];
    let kinds = [CreditKind::Single, CreditKind::Double, CreditKind::Triple];
    let batter = before.teams[half.offense()].batting_order
        [before.teams[half.offense()].next_batter as usize]
        .clone();
    let mut credits = vec![
        credit(&batter, CreditKind::Hit),
        credit(&batter, kinds[value - 1]),
    ];
    let mut moves = vec![batter_move(&before, destinations[value - 1])];
    let mut runs = 0;
    for (index, from) in [Base::First, Base::Second, Base::Third]
        .into_iter()
        .enumerate()
    {
        if let Some(runner) = &before.bases[index] {
            let target = if extra_base && index == 0 {
                2
            } else {
                (index + value).min(3)
            };
            let to = destinations[target];
            if to == Destination::Home {
                runs += 1;
                credits.push(credit(&runner.player_id, CreditKind::Run));
            }
            moves.push(RunnerMove {
                runner: runner.clone(),
                from,
                to,
            });
        }
    }
    let hit = event(
        &before,
        EventKind::PlateAppearance,
        moves,
        0,
        runs,
        credits,
        vec![],
    );
    before = record_clean(&mut log, &before, hit);
    let batter = before.teams[half.offense()].batting_order
        [before.teams[half.offense()].next_batter as usize]
        .clone();
    let mut credits = vec![
        credit(&batter, CreditKind::PlateAppearance),
        credit(&batter, CreditKind::Hit),
        credit(&batter, CreditKind::Run),
    ];
    let mut moves = vec![batter_move(&before, Destination::Home)];
    for (index, from) in [Base::First, Base::Second, Base::Third]
        .into_iter()
        .enumerate()
    {
        if let Some(runner) = &before.bases[index] {
            credits.push(credit(&runner.player_id, CreditKind::Run));
            moves.push(RunnerMove {
                runner: runner.clone(),
                from,
                to: Destination::Home,
            });
        }
    }
    let homer = event(
        &before,
        EventKind::OutOfParkHomeRun,
        moves.clone(),
        0,
        moves.len() as u8,
        credits,
        vec![],
    );
    before = record_clean(&mut log, &before, homer);
    for _ in outs..3 {
        let batter = before.teams[half.offense()].batting_order
            [before.teams[half.offense()].next_batter as usize]
            .clone();
        let out = event(
            &before,
            EventKind::PlateAppearance,
            vec![batter_move(&before, Destination::Out)],
            1,
            0,
            vec![credit(&batter, CreditKind::Strikeout)],
            vec![],
        );
        before = record_clean(&mut log, &before, out);
    }
    log
}

#[test]
fn station_hits_certify_all_observed_masks_outs_and_halves() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            for mask in 0u8..8 {
                for value in 1..=3 {
                    let log = station_hit_half(mask, outs, half, value, false);
                    for t in &log {
                        assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
                    }
                    let a = derive_accounting(&log).unwrap();
                    let pitcher = if half == Half::Top {
                        "home_starter"
                    } else {
                        "away_pitcher"
                    };
                    let p = &a.pitchers[1 - half.offense()][pitcher];
                    let runs = mask.count_ones() + 2;
                    assert_eq!(
                        p.earned_runs,
                        Some(runs),
                        "{half:?} outs{outs} mask{mask} value{value}"
                    );
                    assert_eq!(
                        (p.runs_allowed, p.outs_recorded, p.batters_faced),
                        (runs, 3, mask.count_ones() + 5)
                    );
                    assert_eq!(p.at_bats_against, Some(5));
                    assert_eq!(
                        p.credited_hit_breakdown_allowed
                            .as_ref()
                            .unwrap()
                            .total_bases(),
                        value as u32 + 4
                    );
                    assert_eq!(a.runs[half.offense()], runs);
                }
            }
        }
    }
}

#[test]
fn legal_nonstation_hit_keeps_earned_runs_unknown() {
    for half in [Half::Top, Half::Bottom] {
        let log = station_hit_half(1, 0, half, 1, true);
        let a = derive_accounting(&log).unwrap();
        let pitcher = if half == Half::Top {
            "home_starter"
        } else {
            "away_pitcher"
        };
        assert_eq!(a.pitchers[1 - half.offense()][pitcher].runs_allowed, 3);
        assert_eq!(a.pitchers[1 - half.offense()][pitcher].earned_runs, None);
    }
}

// Start with a positive charged run, then establish every occupied mask through
// observed BB/WP episodes. The ground third out strands, rather than scores,
// its held runners. Other cases finish with HR and the remaining K-outs.
fn clean_ground_half(
    mask: u8,
    outs: u8,
    half: Half,
    explicit_holds: bool,
    reverse: bool,
) -> (Vec<EventTransition>, usize) {
    let mut before = state();
    before.inning = 1;
    before.outs = 0;
    before.bases = [None, None, None];
    let mut log = vec![];
    if half == Half::Bottom {
        finish_with_three_strikeouts(before, &mut log, FieldingReview::NoMisplay);
        before = log.last().unwrap().after().clone();
    }
    let opening_homer = reviewed_homer_from(before.clone()).remove(0);
    before = opening_homer.after().clone();
    log.push(opening_homer);
    for base in (0..3).rev() {
        if mask & (1 << base) == 0 {
            continue;
        }
        let batter = before.teams[half.offense()].batting_order
            [before.teams[half.offense()].next_batter as usize]
            .clone();
        let walk = event(
            &before,
            EventKind::PlateAppearance,
            vec![batter_move(&before, Destination::First)],
            0,
            0,
            vec![credit(&batter, CreditKind::Walk)],
            vec![],
        );
        before = record_clean(&mut log, &before, walk);
        for step in 0..base {
            let advance = event(
                &before,
                EventKind::RunnerOnly(RunnerEvent::WildPitch),
                vec![RunnerMove {
                    runner: before.bases[step].clone().unwrap(),
                    from: [Base::First, Base::Second][step],
                    to: [Destination::Second, Destination::Third][step],
                }],
                0,
                0,
                vec![],
                vec![],
            );
            before = record_clean(&mut log, &before, advance);
        }
    }
    for _ in 0..outs {
        let batter = before.teams[half.offense()].batting_order
            [before.teams[half.offense()].next_batter as usize]
            .clone();
        let out = event(
            &before,
            EventKind::PlateAppearance,
            vec![batter_move(&before, Destination::Out)],
            1,
            0,
            vec![credit(&batter, CreditKind::Strikeout)],
            vec![],
        );
        before = record_clean(&mut log, &before, out);
    }
    let batter = before.teams[half.offense()].batting_order
        [before.teams[half.offense()].next_batter as usize]
        .clone();
    let defense = &before.teams[1 - half.offense()];
    let thrower = defense.batting_order[3].clone();
    let receiver = defense.batting_order[1].clone();
    let mut moves = vec![batter_move(&before, Destination::Out)];
    if explicit_holds {
        for (index, from) in [Base::First, Base::Second, Base::Third]
            .into_iter()
            .enumerate()
        {
            if let Some(runner) = &before.bases[index] {
                moves.push(RunnerMove {
                    runner: runner.clone(),
                    from,
                    to: [Destination::First, Destination::Second, Destination::Third][index],
                });
            }
        }
    }
    if reverse {
        moves.reverse();
    }
    let mut ground = event(
        &before,
        EventKind::PlateAppearance,
        moves,
        1,
        0,
        vec![
            credit(&batter, CreditKind::AtBat),
            credit(&batter, CreditKind::NoHit),
            credit(&receiver, CreditKind::Putout),
            credit(&thrower, CreditKind::Assist),
        ],
        vec![],
    );
    ground.ordinary_ground_throw = Some(bbsim_engine::OrdinaryGroundThrowFacts {
        batter_id: batter.clone(),
        thrower_id: thrower,
        receiver_id: receiver,
    });
    if outs == 2 {
        ground.delta.third_out = Some(ThirdOut {
            runner_id: batter,
            kind: ThirdOutKind::BatterBeforeFirst,
            home_before_out: vec![],
        });
    }
    let ground_index = log.len();
    before = record_clean(&mut log, &before, ground);
    if outs < 2 {
        let batter = before.teams[half.offense()].batting_order
            [before.teams[half.offense()].next_batter as usize]
            .clone();
        let mut moves = vec![batter_move(&before, Destination::Home)];
        let mut credits = vec![
            credit(&batter, CreditKind::PlateAppearance),
            credit(&batter, CreditKind::Hit),
            credit(&batter, CreditKind::Run),
        ];
        for (index, from) in [Base::First, Base::Second, Base::Third]
            .into_iter()
            .enumerate()
        {
            if let Some(runner) = &before.bases[index] {
                credits.push(credit(&runner.player_id, CreditKind::Run));
                moves.push(RunnerMove {
                    runner: runner.clone(),
                    from,
                    to: Destination::Home,
                });
            }
        }
        let homer = event(
            &before,
            EventKind::OutOfParkHomeRun,
            moves,
            0,
            (mask.count_ones() + 1) as u8,
            credits,
            vec![],
        );
        before = record_clean(&mut log, &before, homer);
        while before.half == half {
            let batter = before.teams[half.offense()].batting_order
                [before.teams[half.offense()].next_batter as usize]
                .clone();
            let out = event(
                &before,
                EventKind::PlateAppearance,
                vec![batter_move(&before, Destination::Out)],
                1,
                0,
                vec![credit(&batter, CreditKind::Strikeout)],
                vec![],
            );
            before = record_clean(&mut log, &before, out);
        }
    }
    (log, ground_index)
}

#[test]
fn bound_ground_er_certifies_observed_masks_outs_halves_and_held_vectors() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            for mask in 0u8..8 {
                for explicit_holds in [false, true] {
                    for reverse in [false, true] {
                        let (log, index) =
                            clean_ground_half(mask, outs, half, explicit_holds, reverse);
                        for t in &log {
                            assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
                        }
                        let a = derive_accounting(&log).unwrap();
                        let side = 1 - half.offense();
                        let pitcher = log[index].event().participants.pitcher.clone();
                        let p = &a.pitchers[side][&pitcher];
                        let runs = if outs == 2 { 1 } else { mask.count_ones() + 2 };
                        let ab = if outs == 2 { 4 } else { 5 };
                        assert_eq!(
                            p.earned_runs,
                            Some(runs),
                            "{half:?} outs{outs} mask{mask} holds{explicit_holds} reverse{reverse}"
                        );
                        assert_eq!(
                            (p.runs_allowed, p.outs_recorded, p.batters_faced),
                            (runs, 3, mask.count_ones() + ab)
                        );
                        assert_eq!(p.at_bats_against, Some(ab));
                        assert_eq!(
                            a.batters[half.offense()]
                                .values()
                                .map(|b| b.plate_appearances)
                                .sum::<u32>(),
                            mask.count_ones() + ab
                        );
                        assert_eq!(
                            p.classified_hits_allowed,
                            Some(if outs == 2 { 1 } else { 2 })
                        );
                        assert_eq!(a.runs[half.offense()], runs);
                        let facts = log[index].event().ordinary_ground_throw.as_ref().unwrap();
                        assert_eq!(a.fielders[side][&facts.receiver_id].credited_putouts, 1);
                        assert_eq!(a.fielders[side][&facts.thrower_id].credited_assists, 1);
                    }
                }
            }
        }
    }
}

#[test]
fn bound_ground_er_does_not_repair_missing_evidence_history_or_completion() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            let (clean, ground_index) = clean_ground_half(7, outs, half, true, true);
            let side = 1 - half.offense();
            let pitcher = clean[ground_index].event().participants.pitcher.clone();
            for missing in 0..3 {
                let log: Vec<_> = clean
                    .iter()
                    .enumerate()
                    .map(|(index, t)| {
                        let mut action = t.event().clone();
                        if index == ground_index {
                            if missing == 0 {
                                action.ordinary_ground_throw = None;
                            }
                            if missing == 1 {
                                action.fielding_review = FieldingReview::Unknown;
                            }
                        }
                        if missing == 2 && index == usize::from(half == Half::Bottom) * 3 {
                            action.fielding_review = FieldingReview::Unknown;
                        }
                        apply(t.before(), action).unwrap()
                    })
                    .collect();
                assert_eq!(
                    derive_accounting(&log).unwrap().pitchers[side][&pitcher].earned_runs,
                    None
                );
            }
            // Keep a positive charged run in this segment. Dropping its only
            // scorer would correctly yield Some(0), not an unknown ER total.
            let start = usize::from(half == Half::Bottom) * 3;
            let partial: Vec<_> = clean[start..]
                .iter()
                .map(|t| {
                    let mut before = t.before().clone();
                    before.inning += 1;
                    apply(&before, t.event().clone()).unwrap()
                })
                .collect();
            let p = &derive_accounting(&partial).unwrap().pitchers[side][&pitcher];
            assert!(p.runs_allowed > 0);
            assert_eq!(p.earned_runs, None);
            assert_eq!(
                derive_accounting(&clean[..clean.len() - 1])
                    .unwrap()
                    .pitchers[side][&pitcher]
                    .earned_runs,
                None
            );
        }
    }
}

// Reuse the observed origin/mask/outs prefix, but replace the ground event
// with an exact protected award. Nonexact controls remain physically legal.
fn forced_award_half(
    mask: u8,
    outs: u8,
    half: Half,
    kind: CreditKind,
    explicit_holds: bool,
    reverse: bool,
    nonexact: bool,
) -> (Vec<EventTransition>, usize) {
    let (template, index) = clean_ground_half(mask, outs, half, false, false);
    let mut log = template[..index].to_vec();
    let mut before = log.last().unwrap().after().clone();
    let batter = before.teams[half.offense()].batting_order
        [before.teams[half.offense()].next_batter as usize]
        .clone();
    let forced = before.bases.iter().take_while(|b| b.is_some()).count();
    let destinations = [
        Destination::First,
        Destination::Second,
        Destination::Third,
        Destination::Home,
    ];
    let mut moves = vec![batter_move(&before, Destination::First)];
    let mut credits = vec![credit(&batter, kind)];
    let mut runs = 0;
    for (i, from) in [Base::First, Base::Second, Base::Third]
        .into_iter()
        .enumerate()
    {
        if let Some(runner) = &before.bases[i] {
            let excess = nonexact && ((mask == 1 && i == 0) || (mask == 2 && i == 1));
            if i < forced || explicit_holds || excess {
                let to = if excess {
                    Destination::Third
                } else {
                    destinations[i + usize::from(i < forced)]
                };
                if to == Destination::Home {
                    runs += 1;
                    credits.push(credit(&runner.player_id, CreditKind::Run));
                }
                moves.push(RunnerMove {
                    runner: runner.clone(),
                    from,
                    to,
                });
            }
        }
    }
    if reverse {
        moves.reverse();
    }
    let award = event(
        &before,
        EventKind::PlateAppearance,
        moves,
        0,
        runs,
        credits,
        vec![],
    );
    before = record_clean(&mut log, &before, award);
    let batter = before.teams[half.offense()].batting_order
        [before.teams[half.offense()].next_batter as usize]
        .clone();
    let mut moves = vec![batter_move(&before, Destination::Home)];
    let mut credits = vec![
        credit(&batter, CreditKind::PlateAppearance),
        credit(&batter, CreditKind::Hit),
        credit(&batter, CreditKind::Run),
    ];
    for (i, from) in [Base::First, Base::Second, Base::Third]
        .into_iter()
        .enumerate()
    {
        if let Some(runner) = &before.bases[i] {
            credits.push(credit(&runner.player_id, CreditKind::Run));
            moves.push(RunnerMove {
                runner: runner.clone(),
                from,
                to: Destination::Home,
            });
        }
    }
    let runs = moves.len() as u8;
    let homer = event(
        &before,
        EventKind::OutOfParkHomeRun,
        moves,
        0,
        runs,
        credits,
        vec![],
    );
    before = record_clean(&mut log, &before, homer);
    while before.half == half {
        let batter = before.teams[half.offense()].batting_order
            [before.teams[half.offense()].next_batter as usize]
            .clone();
        let out = event(
            &before,
            EventKind::PlateAppearance,
            vec![batter_move(&before, Destination::Out)],
            1,
            0,
            vec![credit(&batter, CreditKind::Strikeout)],
            vec![],
        );
        before = record_clean(&mut log, &before, out);
    }
    (log, index)
}

#[test]
fn exact_forced_award_er_has_literal_observed_mask_out_half_and_award_lines() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            for mask in 0u8..8 {
                for kind in [CreditKind::Walk, CreditKind::HitByPitch] {
                    for holds in [false, true] {
                        for reverse in [false, true] {
                            let (log, index) =
                                forced_award_half(mask, outs, half, kind, holds, reverse, false);
                            for t in &log {
                                assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
                            }
                            let a = derive_accounting(&log).unwrap();
                            let p = &a.pitchers[1 - half.offense()]
                                [&log[index].event().participants.pitcher];
                            let runners = mask.count_ones();
                            assert_eq!(p.earned_runs, Some(runners+3), "{half:?} outs{outs} mask{mask} {kind:?} holds{holds} reverse{reverse}");
                            assert_eq!(
                                (
                                    p.runs_allowed,
                                    p.outs_recorded,
                                    p.batters_faced,
                                    p.at_bats_against,
                                    p.classified_hits_allowed
                                ),
                                (runners + 3, 3, runners + 6, Some(5), Some(2))
                            );
                            assert_eq!(p.credited_hits_allowed, 2);
                            assert_eq!(
                                p.credited_walks_allowed,
                                runners + u32::from(kind == CreditKind::Walk)
                            );
                            assert_eq!(
                                p.credited_hit_batters,
                                u32::from(kind == CreditKind::HitByPitch)
                            );
                            let batters = &a.batters[half.offense()];
                            assert_eq!(
                                batters.values().map(|b| b.plate_appearances).sum::<u32>(),
                                runners + 6
                            );
                            assert_eq!(
                                batters.values().map(|b| b.at_bats.unwrap()).sum::<u32>(),
                                5
                            );
                            assert_eq!(batters.values().map(|b| b.credited_hits).sum::<u32>(), 2);
                            assert_eq!(
                                batters.values().map(|b| b.credited_walks).sum::<u32>(),
                                runners + u32::from(kind == CreditKind::Walk)
                            );
                            assert_eq!(
                                batters
                                    .values()
                                    .map(|b| b.credited_hit_by_pitch)
                                    .sum::<u32>(),
                                u32::from(kind == CreditKind::HitByPitch)
                            );
                            assert_eq!(log[index].event().delta.runs, u8::from(mask == 7));
                            assert_eq!(a.runs[half.offense()], runners + 3);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn forced_award_er_keeps_nonexact_and_missing_evidence_unknown() {
    for half in [Half::Top, Half::Bottom] {
        for kind in [CreditKind::Walk, CreditKind::HitByPitch] {
            for mask in [1, 2] {
                let (log, _) = forced_award_half(mask, 1, half, kind, true, true, true);
                let p = &derive_accounting(&log).unwrap().pitchers[1 - half.offense()]
                    [&log[0].before().teams[1 - half.offense()].pitcher];
                assert_eq!(p.runs_allowed, mask.count_ones() + 3);
                assert_eq!(p.earned_runs, None);
            }
            let (clean, index) = forced_award_half(7, 1, half, kind, true, true, false);
            let pitcher = clean[index].event().participants.pitcher.clone();
            let side = 1 - half.offense();
            for missing in 0..3 {
                let log: Vec<_> = clean
                    .iter()
                    .enumerate()
                    .map(|(i, t)| {
                        let mut action = t.event().clone();
                        if i == index {
                            if missing == 0 {
                                action.fielding_review = FieldingReview::Unknown;
                            }
                            if missing == 1 {
                                action.delta.credits.retain(|c| c.kind != kind);
                            }
                        }
                        if missing == 2 && i == usize::from(half == Half::Bottom) * 3 {
                            action.fielding_review = FieldingReview::Unknown;
                        }
                        apply(t.before(), action).unwrap()
                    })
                    .collect();
                assert_eq!(
                    derive_accounting(&log).unwrap().pitchers[side][&pitcher].earned_runs,
                    None
                );
            }
            let start = usize::from(half == Half::Bottom) * 3;
            let partial: Vec<_> = clean[start..]
                .iter()
                .map(|t| {
                    let mut before = t.before().clone();
                    before.inning += 1;
                    apply(&before, t.event().clone()).unwrap()
                })
                .collect();
            assert_eq!(
                derive_accounting(&partial).unwrap().pitchers[side][&pitcher].earned_runs,
                None
            );
            assert_eq!(
                derive_accounting(&clean[..clean.len() - 1])
                    .unwrap()
                    .pitchers[side][&pitcher]
                    .earned_runs,
                None
            );
        }
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

// Actual observed origins/occupancy from the earlier independent half fixture;
// replace only the retirement, preserving every physical before/after state.
fn clean_caught_half(
    mask: u8,
    outs: u8,
    half: Half,
    holds: bool,
    reverse_moves: bool,
    reverse_credits: bool,
) -> (Vec<EventTransition>, usize) {
    let (mut log, index) = clean_ground_half(mask, outs, half, holds, reverse_moves);
    let original = &log[index];
    let before = original.before();
    let mut e = original.event().clone();
    let batter = e.participants.batter.clone().unwrap();
    let catcher = before.teams[1 - half.offense()].batting_order[0].clone();
    e.ordinary_ground_throw = None;
    e.caught_strikeout = Some(bbsim_engine::CaughtStrikeoutFacts {
        batter_id: batter.clone(),
        catcher_id: catcher.clone(),
    });
    e.delta.credits = vec![
        credit(&batter, CreditKind::PlateAppearance),
        credit(&batter, CreditKind::AtBat),
        credit(&batter, CreditKind::NoHit),
        credit(&batter, CreditKind::Strikeout),
        credit(&catcher, CreditKind::Putout),
    ];
    if reverse_credits {
        e.delta.credits.reverse();
    }
    let replacement = apply(before, e).unwrap();
    assert_eq!(replacement.after(), original.after());
    log[index] = replacement;
    (log, index)
}

#[test]
fn bound_caught_k_er_has_literal_observed_lines_in_384_hold_and_order_frames() {
    let mut frames = 0;
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            for mask in 0u8..8 {
                for holds in [false, true] {
                    for reverse_moves in [false, true] {
                        for reverse_credits in [false, true] {
                            let (log, index) = clean_caught_half(
                                mask,
                                outs,
                                half,
                                holds,
                                reverse_moves,
                                reverse_credits,
                            );
                            for t in &log {
                                assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
                            }
                            let a = derive_accounting(&log).unwrap();
                            let side = 1 - half.offense();
                            let pitcher = &log[index].event().participants.pitcher;
                            let catcher = &log[index]
                                .event()
                                .caught_strikeout
                                .as_ref()
                                .unwrap()
                                .catcher_id;
                            let p = &a.pitchers[side][pitcher];
                            let runs = if outs == 2 { 1 } else { mask.count_ones() + 2 };
                            let ab = if outs == 2 { 4 } else { 5 };
                            assert_eq!(p.earned_runs, Some(runs),
                                "{half:?} outs{outs} mask{mask} holds{holds} moves{reverse_moves} credits{reverse_credits}");
                            assert_eq!(
                                (p.runs_allowed, p.outs_recorded, p.batters_faced),
                                (runs, 3, mask.count_ones() + ab)
                            );
                            assert_eq!(p.at_bats_against, Some(ab));
                            assert_eq!(
                                p.classified_hits_allowed,
                                Some(if outs == 2 { 1 } else { 2 })
                            );
                            assert_eq!(p.credited_strikeouts, 3);
                            assert_eq!(a.runs[half.offense()], runs);
                            assert_eq!(
                                a.batters[half.offense()]
                                    .values()
                                    .map(|b| b.plate_appearances)
                                    .sum::<u32>(),
                                mask.count_ones() + ab
                            );
                            assert_eq!(
                                a.batters[half.offense()]
                                    .values()
                                    .map(|b| b.credited_strikeouts)
                                    .sum::<u32>(),
                                3
                            );
                            assert_eq!(a.fielders[side][catcher].credited_putouts, 1);
                            assert_eq!(a.fielders[side][catcher].credited_assists, 0);
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
fn bound_caught_k_er_does_not_repair_missing_facts_review_origin_or_completion() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            let (clean, index) = clean_caught_half(7, outs, half, true, true, true);
            let side = 1 - half.offense();
            let pitcher = &clean[index].event().participants.pitcher;
            let start = usize::from(half == Half::Bottom) * 3;
            for missing in 0..3 {
                let mut log = clean.clone();
                let position = if missing == 2 { start } else { index };
                let mut e = log[position].event().clone();
                if missing == 0 {
                    e.caught_strikeout = None;
                } else {
                    e.fielding_review = FieldingReview::Unknown;
                }
                let replacement = apply(log[position].before(), e).unwrap();
                assert_eq!(replacement.after(), log[position].after());
                log[position] = replacement;
                let a = derive_accounting(&log).unwrap();
                assert!(a.pitchers[side][pitcher].runs_allowed > 0);
                assert_eq!(a.pitchers[side][pitcher].earned_runs, None);
                assert_eq!(a.runs, derive_accounting(&clean).unwrap().runs);
                assert_eq!(a.fielders, derive_accounting(&clean).unwrap().fielders);
            }
            let partial: Vec<_> = clean[start..]
                .iter()
                .map(|t| {
                    let mut before = t.before().clone();
                    before.inning += 1;
                    apply(&before, t.event().clone()).unwrap()
                })
                .collect();
            let a = derive_accounting(&partial).unwrap();
            assert!(a.pitchers[side][pitcher].runs_allowed > 0);
            assert_eq!(a.pitchers[side][pitcher].earned_runs, None);
            assert_eq!(
                derive_accounting(&clean[..clean.len() - 1])
                    .unwrap()
                    .pitchers[side][pitcher]
                    .earned_runs,
                None
            );
        }
    }
}

#[test]
fn generic_single_movement_strikeout_retains_legacy_clean_er_proof() {
    for half in [Half::Top, Half::Bottom] {
        for holds in [false, true] {
            let (mut log, index) = clean_caught_half(0, 1, half, holds, true, true);
            let pitcher = log[index].event().participants.pitcher.clone();
            let mut e = log[index].event().clone();
            e.caught_strikeout = None;
            log[index] = apply(log[index].before(), e).unwrap();
            let a = derive_accounting(&log).unwrap();
            assert_eq!(
                a.pitchers[1 - half.offense()][&pitcher].earned_runs,
                Some(2)
            );
        }
    }
}

fn clean_imposed_balk_half(
    mask: u8,
    outs: u8,
    half: Half,
    reverse_moves: bool,
    reverse_credits: bool,
    relief: bool,
    occurrences: u8,
) -> (Vec<EventTransition>, usize) {
    let (mut log, index) = clean_ground_half(mask, outs, half, false, false);
    log.truncate(index);
    let mut before = log.last().unwrap().after().clone();
    if relief {
        let defense = 1 - half.offense();
        let change = event(
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
        );
        before = record_clean(&mut log, &before, change);
    }
    let award_index = log.len();
    for _ in 0..occurrences {
        let mut moves = vec![];
        let mut credits = vec![credit(
            &before.teams[1 - half.offense()].pitcher,
            CreditKind::Balk,
        )];
        for (i, runner) in before.bases.iter().enumerate() {
            if let Some(runner) = runner {
                moves.push(RunnerMove {
                    runner: runner.clone(),
                    from: [Base::First, Base::Second, Base::Third][i],
                    to: [Destination::Second, Destination::Third, Destination::Home][i],
                });
                if i == 2 {
                    credits.push(credit(&runner.player_id, CreditKind::Run));
                }
            }
        }
        if reverse_moves {
            moves.reverse();
        }
        if reverse_credits {
            credits.reverse();
        }
        let mut award = event(
            &before,
            EventKind::RunnerOnly(RunnerEvent::Balk),
            moves,
            0,
            u8::from(before.bases[2].is_some()),
            credits,
            vec![],
        );
        award.balk = Some(bbsim_engine::BalkFacts {
            cause: bbsim_engine::BalkCause::PitcherAction602a,
            disposition: bbsim_engine::BalkDisposition::PenaltyApplied,
            resolution: bbsim_engine::BalkResolution::ExactProtectedAwards,
        });
        let saved = before.clone();
        let next = record_clean(&mut log, &before, award);
        assert_eq!(before, saved);
        assert_eq!(next.teams, before.teams);
        assert_eq!(next.outs, before.outs);
        before = next;
    }
    let batter = before.teams[half.offense()].batting_order
        [before.teams[half.offense()].next_batter as usize]
        .clone();
    let mut moves = vec![batter_move(&before, Destination::Home)];
    let mut credits = vec![
        credit(&batter, CreditKind::PlateAppearance),
        credit(&batter, CreditKind::Hit),
        credit(&batter, CreditKind::Run),
    ];
    for (i, runner) in before.bases.iter().enumerate() {
        if let Some(runner) = runner {
            moves.push(RunnerMove {
                runner: runner.clone(),
                from: [Base::First, Base::Second, Base::Third][i],
                to: Destination::Home,
            });
            credits.push(credit(&runner.player_id, CreditKind::Run));
        }
    }
    let homer = event(
        &before,
        EventKind::OutOfParkHomeRun,
        moves,
        0,
        (before.bases.iter().flatten().count() + 1) as u8,
        credits,
        vec![],
    );
    before = record_clean(&mut log, &before, homer);
    while before.half == half {
        let batter = before.teams[half.offense()].batting_order
            [before.teams[half.offense()].next_batter as usize]
            .clone();
        let out = event(
            &before,
            EventKind::PlateAppearance,
            vec![batter_move(&before, Destination::Out)],
            1,
            0,
            vec![credit(&batter, CreditKind::Strikeout)],
            vec![],
        );
        before = record_clean(&mut log, &before, out);
    }
    (log, award_index)
}

#[test]
fn exact_imposed_balk_er_has_literal_observed_lines_in_168_order_frames() {
    let mut frames = 0;
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            for mask in 1u8..8 {
                for reverse_moves in [false, true] {
                    for reverse_credits in [false, true] {
                        let (log, index) = clean_imposed_balk_half(
                            mask,
                            outs,
                            half,
                            reverse_moves,
                            reverse_credits,
                            false,
                            1,
                        );
                        for t in &log {
                            assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
                        }
                        let a = derive_accounting(&log).unwrap();
                        let pitcher = &log[index].event().participants.pitcher;
                        let p = &a.pitchers[1 - half.offense()][pitcher];
                        let runs = mask.count_ones() + 2;
                        assert_eq!(p.earned_runs, Some(runs), "{half:?} outs{outs} mask{mask} moves{reverse_moves} credits{reverse_credits}");
                        assert_eq!(
                            (
                                p.runs_allowed,
                                p.outs_recorded,
                                p.batters_faced,
                                p.credited_balks
                            ),
                            (runs, 3, mask.count_ones() + 5, 1)
                        );
                        assert_eq!(p.at_bats_against, Some(5));
                        assert_eq!(p.classified_hits_allowed, Some(2));
                        assert_eq!(p.credited_strikeouts, 3);
                        assert_eq!(a.runs[half.offense()], runs);
                        assert_eq!(
                            a.batters[half.offense()]
                                .values()
                                .map(|b| b.plate_appearances)
                                .sum::<u32>(),
                            mask.count_ones() + 5
                        );
                        frames += 1;
                    }
                }
            }
        }
    }
    assert_eq!(frames, 168);
}

#[test]
fn exact_imposed_balk_er_keeps_partial_review_origin_and_completion_unknown() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            let (clean, index) = clean_imposed_balk_half(7, outs, half, true, true, false, 1);
            let pitcher = &clean[index].event().participants.pitcher;
            let side = 1 - half.offense();
            let start = usize::from(half == Half::Bottom) * 3;
            for missing in 0..6 {
                let mut log = clean.clone();
                let position = if missing == 2 { start } else { index };
                let mut e = log[position].event().clone();
                match missing {
                    0 => e.balk = None,
                    1 | 2 => e.fielding_review = FieldingReview::Unknown,
                    3 => e.balk.as_mut().unwrap().cause = bbsim_engine::BalkCause::Unknown,
                    4 => {
                        e.balk.as_mut().unwrap().disposition =
                            bbsim_engine::BalkDisposition::Unknown
                    }
                    5 => {
                        e.balk.as_mut().unwrap().resolution = bbsim_engine::BalkResolution::Unknown
                    }
                    _ => unreachable!(),
                }
                if missing == 0 || missing >= 3 {
                    e.delta.credits.retain(|c| c.kind != CreditKind::Balk);
                }
                let t = apply(log[position].before(), e).unwrap();
                assert_eq!(t.after(), log[position].after());
                log[position] = t;
                let a = derive_accounting(&log).unwrap();
                assert_eq!(a.pitchers[side][pitcher].runs_allowed, 5);
                assert_eq!(a.pitchers[side][pitcher].earned_runs, None);
            }
            let late: Vec<_> = clean[start..]
                .iter()
                .map(|t| {
                    let mut before = t.before().clone();
                    before.inning += 1;
                    apply(&before, t.event().clone()).unwrap()
                })
                .collect();
            assert_eq!(
                derive_accounting(&late).unwrap().pitchers[side][pitcher].earned_runs,
                None
            );
            assert_eq!(
                derive_accounting(&clean[..clean.len() - 1])
                    .unwrap()
                    .pitchers[side][pitcher]
                    .earned_runs,
                None
            );
        }
    }
}

#[test]
fn exact_imposed_balk_er_retains_inherited_charges_and_distinct_occurrences() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            for occurrences in [1, 2] {
                let (log, index) =
                    clean_imposed_balk_half(7, outs, half, true, true, true, occurrences);
                let a = derive_accounting(&log).unwrap();
                let side = 1 - half.offense();
                let prior = &log[index - 1].event().participants.pitcher;
                let current = &log[index].event().participants.pitcher;
                assert_ne!(prior, current);
                let old = &a.pitchers[side][prior];
                let new = &a.pitchers[side][current];
                assert_eq!(
                    (
                        old.runs_allowed,
                        old.earned_runs,
                        old.outs_recorded,
                        old.batters_faced,
                        old.credited_balks
                    ),
                    (4, Some(4), u32::from(outs), 4 + u32::from(outs), 0)
                );
                assert_eq!(
                    (
                        new.runs_allowed,
                        new.earned_runs,
                        new.outs_recorded,
                        new.batters_faced,
                        new.credited_balks
                    ),
                    (
                        1,
                        Some(1),
                        3 - u32::from(outs),
                        4 - u32::from(outs),
                        u32::from(occurrences)
                    )
                );
                assert_eq!(a.runs[half.offense()], 5);
            }
        }
    }
}

fn clean_sole_safe_hit_half(
    mask: u8,
    outs: u8,
    half: Half,
    value: usize,
    hold_or_station: bool,
    reverse_moves: bool,
    reverse_credits: bool,
) -> (Vec<EventTransition>, usize) {
    let (mut log, index) = clean_ground_half(mask, outs, half, false, false);
    log.truncate(index);
    let mut before = log.last().unwrap().after().clone();
    let batter = before.teams[half.offense()].batting_order
        [before.teams[half.offense()].next_batter as usize]
        .clone();
    let destinations = [
        Destination::First,
        Destination::Second,
        Destination::Third,
        Destination::Home,
    ];
    let hold = hold_or_station && ((value == 1 && mask & 1 == 0) || (value == 2 && mask == 4));
    let mut moves = vec![batter_move(&before, destinations[value - 1])];
    let mut credits = vec![
        credit(&batter, CreditKind::Hit),
        credit(&batter, CreditKind::AtBat),
        credit(
            &batter,
            [CreditKind::Single, CreditKind::Double, CreditKind::Triple][value - 1],
        ),
    ];
    let mut runs = 0;
    for (i, runner) in before.bases.iter().enumerate() {
        if let Some(runner) = runner {
            let to = if hold {
                destinations[i]
            } else if hold_or_station {
                destinations[(i + value).min(3)]
            } else {
                Destination::Home
            };
            moves.push(RunnerMove {
                runner: runner.clone(),
                from: [Base::First, Base::Second, Base::Third][i],
                to,
            });
            if to == Destination::Home {
                runs += 1;
                credits.push(credit(&runner.player_id, CreditKind::Run));
                credits.push(credit(&batter, CreditKind::Rbi));
            }
        }
    }
    let mut hit = event(
        &before,
        EventKind::PlateAppearance,
        moves,
        0,
        runs,
        credits,
        vec![],
    );
    hit.sole_safe_hit = Some(bbsim_engine::SoleSafeHitFacts { batter_id: batter });
    if reverse_moves {
        hit.delta.moves.reverse();
    }
    if reverse_credits {
        hit.delta.credits.reverse();
    }
    before = record_clean(&mut log, &before, hit);
    let batter = before.teams[half.offense()].batting_order
        [before.teams[half.offense()].next_batter as usize]
        .clone();
    let mut moves = vec![batter_move(&before, Destination::Home)];
    let mut credits = vec![
        credit(&batter, CreditKind::PlateAppearance),
        credit(&batter, CreditKind::Hit),
        credit(&batter, CreditKind::Run),
    ];
    for (i, runner) in before.bases.iter().enumerate() {
        if let Some(runner) = runner {
            moves.push(RunnerMove {
                runner: runner.clone(),
                from: [Base::First, Base::Second, Base::Third][i],
                to: Destination::Home,
            });
            credits.push(credit(&runner.player_id, CreditKind::Run));
        }
    }
    let homer = event(
        &before,
        EventKind::OutOfParkHomeRun,
        moves,
        0,
        (before.bases.iter().flatten().count() + 1) as u8,
        credits,
        vec![],
    );
    before = record_clean(&mut log, &before, homer);
    while before.half == half {
        let batter = before.teams[half.offense()].batting_order
            [before.teams[half.offense()].next_batter as usize]
            .clone();
        let out = event(
            &before,
            EventKind::PlateAppearance,
            vec![batter_move(&before, Destination::Out)],
            1,
            0,
            vec![credit(&batter, CreditKind::Strikeout)],
            vec![],
        );
        before = record_clean(&mut log, &before, out);
    }
    (log, index)
}

#[test]
fn sole_safe_hit_er_has_literal_observed_lines_in_1152_legal_policy_frames() {
    let mut frames = 0;
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            for mask in 0u8..8 {
                for value in 1..=3 {
                    for policy in [false, true] {
                        for reverse_moves in [false, true] {
                            for reverse_credits in [false, true] {
                                let (log, index) = clean_sole_safe_hit_half(
                                    mask,
                                    outs,
                                    half,
                                    value,
                                    policy,
                                    reverse_moves,
                                    reverse_credits,
                                );
                                for t in &log {
                                    assert_eq!(
                                        EventTransition::from_bytes(&t.to_bytes()).unwrap(),
                                        *t
                                    );
                                    assert_eq!(
                                        EventTransition::try_new(
                                            t.before().clone(),
                                            t.event().clone(),
                                            t.after().clone()
                                        )
                                        .unwrap(),
                                        *t
                                    );
                                }
                                let a = derive_accounting(&log).unwrap();
                                let pitcher = &log[index].event().participants.pitcher;
                                let p = &a.pitchers[1 - half.offense()][pitcher];
                                let runs = mask.count_ones() + 3;
                                assert_eq!(p.earned_runs, Some(runs), "{half:?} outs{outs} mask{mask} value{value} policy{policy} moves{reverse_moves} credits{reverse_credits}");
                                assert_eq!(
                                    (p.runs_allowed, p.outs_recorded, p.batters_faced),
                                    (runs, 3, mask.count_ones() + 6)
                                );
                                assert_eq!(p.at_bats_against, Some(6));
                                assert_eq!(p.classified_hits_allowed, Some(3));
                                assert_eq!(p.credited_strikeouts, 3);
                                assert_eq!(a.runs[half.offense()], runs);
                                assert_eq!(
                                    a.batters[half.offense()]
                                        .values()
                                        .map(|b| b.plate_appearances)
                                        .sum::<u32>(),
                                    mask.count_ones() + 6
                                );
                                frames += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(frames, 1152);
}

#[test]
fn sole_safe_hit_er_keeps_nonstation_missing_review_cause_and_history_unknown() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            let (clean, index) = clean_sole_safe_hit_half(7, outs, half, 1, false, true, true);
            let pitcher = &clean[index].event().participants.pitcher;
            let side = 1 - half.offense();
            let start = usize::from(half == Half::Bottom) * 3;
            for missing in 0..4 {
                let mut log = clean.clone();
                let position = if missing == 2 { start } else { index };
                let mut e = log[position].event().clone();
                if missing == 0 {
                    e.sole_safe_hit = None;
                } else {
                    e.fielding_review = FieldingReview::Unknown;
                }
                if missing == 3 {
                    e.delta.credits.push(credit(
                        &log[position].before().teams[side].batting_order[0],
                        CreditKind::Error,
                    ));
                }
                let t = apply(log[position].before(), e).unwrap();
                assert_eq!(t.after(), log[position].after());
                assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
                log[position] = t;
                let a = derive_accounting(&log).unwrap();
                assert_eq!(a.pitchers[side][pitcher].runs_allowed, 6);
                assert_eq!(a.pitchers[side][pitcher].earned_runs, None);
                if missing == 3 {
                    assert_eq!(
                        log[index]
                            .rbi_contribution()
                            .unwrap()
                            .runs_batted_in
                            .values()
                            .copied()
                            .sum::<u32>(),
                        3
                    );
                    assert_eq!(
                        a.fielders[side]
                            .values()
                            .map(|f| f.credited_errors)
                            .sum::<u32>(),
                        1
                    );
                }
            }
            let late: Vec<_> = clean[start..]
                .iter()
                .map(|t| {
                    let mut before = t.before().clone();
                    before.inning += 1;
                    apply(&before, t.event().clone()).unwrap()
                })
                .collect();
            assert_eq!(
                derive_accounting(&late).unwrap().pitchers[side][pitcher].earned_runs,
                None
            );
            assert_eq!(
                derive_accounting(&clean[..clean.len() - 1])
                    .unwrap()
                    .pitchers[side][pitcher]
                    .earned_runs,
                None
            );
        }
    }
}

// Fixed authored play script, not a second statistic accumulator/oracle.
fn literal_reviewed_game(reverse_moves: bool, reverse_credits: bool) -> Vec<EventTransition> {
    let mut before = state();
    before.inning = 1;
    before.outs = 0;
    before.bases = [None, None, None];
    let mut log = vec![];
    for frame in 0..17 {
        let offense = before.half.offense();
        let script = if frame == 0 {
            vec![
                CreditKind::OrdinaryWalk,
                CreditKind::Single,
                CreditKind::Double,
                CreditKind::Triple,
                CreditKind::Hit,
            ]
        } else if frame == 1 {
            vec![
                CreditKind::HitByPitch,
                CreditKind::OrdinaryWalk,
                CreditKind::IntentionalWalk,
                CreditKind::Single,
                CreditKind::Double,
                CreditKind::Triple,
                CreditKind::Hit,
            ]
        } else {
            vec![]
        };
        for outcome in
            script
                .into_iter()
                .chain([CreditKind::AtBat, CreditKind::Strikeout, CreditKind::AtBat])
        {
            let batter = before.teams[offense].batting_order
                [before.teams[offense].next_batter as usize]
                .clone();
            let defense = &before.teams[1 - offense];
            let retirement = matches!(outcome, CreditKind::AtBat | CreditKind::Strikeout);
            let award = matches!(
                outcome,
                CreditKind::OrdinaryWalk | CreditKind::IntentionalWalk | CreditKind::HitByPitch
            );
            let homer = outcome == CreditKind::Hit;
            let destination = match outcome {
                CreditKind::AtBat | CreditKind::Strikeout => Destination::Out,
                CreditKind::Double => Destination::Second,
                CreditKind::Triple => Destination::Third,
                CreditKind::Hit => Destination::Home,
                _ => Destination::First,
            };
            let mut moves = vec![batter_move(&before, destination)];
            let mut credits = vec![credit(&batter, outcome)];
            if award && outcome != CreditKind::HitByPitch {
                credits.push(credit(&batter, CreditKind::Walk));
            }
            if retirement {
                credits.push(credit(&batter, CreditKind::NoHit));
                credits.push(credit(
                    &defense.batting_order[usize::from(outcome == CreditKind::Strikeout)],
                    CreditKind::Putout,
                ));
                if outcome == CreditKind::AtBat {
                    credits.push(credit(&defense.pitcher, CreditKind::Assist));
                }
            } else if !award && !homer {
                credits.push(credit(&batter, CreditKind::Hit));
                credits.push(credit(&batter, CreditKind::AtBat));
            }
            let mut runs = u8::from(homer);
            if homer {
                credits.push(credit(&batter, CreditKind::Run));
                credits.push(credit(&batter, CreditKind::Rbi));
                credits.push(credit(&batter, CreditKind::PlateAppearance));
            }
            for (i, runner) in before.bases.iter().enumerate() {
                if let Some(runner) = runner {
                    let to = if award {
                        [Destination::Second, Destination::Third, Destination::Home][i]
                    } else if frame == 0 && outcome == CreditKind::Double {
                        Destination::Third
                    } else {
                        Destination::Home
                    };
                    moves.push(RunnerMove {
                        runner: runner.clone(),
                        from: [Base::First, Base::Second, Base::Third][i],
                        to,
                    });
                    if to == Destination::Home {
                        runs += 1;
                        credits.push(credit(&runner.player_id, CreditKind::Run));
                        credits.push(credit(&batter, CreditKind::Rbi));
                    }
                }
            }
            let mut action = event(
                &before,
                if homer {
                    EventKind::OutOfParkHomeRun
                } else {
                    EventKind::PlateAppearance
                },
                moves,
                u8::from(retirement),
                runs,
                credits,
                vec![],
            );
            if retirement && before.outs == 2 {
                action.delta.third_out = Some(ThirdOut {
                    runner_id: batter.clone(),
                    kind: ThirdOutKind::BatterBeforeFirst,
                    home_before_out: vec![],
                });
            }
            if award {
                action.protected_award =
                    Some(bbsim_engine::ProtectedAwardFacts { batter_id: batter });
            } else if homer {
                action.sole_home_run = Some(bbsim_engine::SoleHomeRunFacts { batter_id: batter });
            } else if outcome == CreditKind::AtBat {
                action.ordinary_ground_throw = Some(bbsim_engine::OrdinaryGroundThrowFacts {
                    batter_id: batter,
                    thrower_id: defense.pitcher.clone(),
                    receiver_id: defense.batting_order[0].clone(),
                });
            } else if outcome == CreditKind::Strikeout {
                action.caught_strikeout = Some(bbsim_engine::CaughtStrikeoutFacts {
                    batter_id: batter,
                    catcher_id: defense.batting_order[1].clone(),
                });
            } else {
                action.sole_safe_hit = Some(bbsim_engine::SoleSafeHitFacts { batter_id: batter });
            }
            if reverse_moves {
                action.delta.moves.reverse();
            }
            if reverse_credits {
                action.delta.credits.reverse();
            }
            let t = apply(&before, reviewed(action)).unwrap_or_else(|error| {
                panic!(
                    "literal frame{frame} outcome{outcome:?} outs{}: {error:?}",
                    before.outs
                )
            });
            before = t.after().clone();
            log.push(t);
        }
    }
    assert!(before.over);
    assert_eq!(
        (before.inning, before.half, before.score),
        (9, Half::Top, [5, 7])
    );
    log
}

#[test]
fn literal_reviewed_game_reconciles_player_lines_and_independent_coverage() {
    use bbsim_engine::accounting::{
        BatterLine, FielderLine, FieldingCoverageLine, HitBreakdown, PitcherLine,
    };
    // PA, AB, H, K, R, RBI, BB: authored from the fixed batting rotation.
    let away = [
        (4, 3, 0, 3, 1, 0, 1),
        (4, 4, 1, 0, 1, 1, 0),
        (4, 4, 1, 0, 1, 0, 0),
        (4, 4, 1, 3, 1, 2, 0),
        (4, 4, 1, 0, 1, 2, 0),
        (3, 3, 0, 0, 0, 0, 0),
        (3, 3, 0, 3, 0, 0, 0),
        (3, 3, 0, 0, 0, 0, 0),
        (3, 3, 0, 0, 0, 0, 0),
    ];
    let home = [
        (4, 3, 0, 0, 1, 0, 0),
        (4, 3, 0, 0, 1, 0, 1),
        (4, 3, 0, 3, 1, 0, 1),
        (4, 4, 1, 0, 1, 3, 0),
        (3, 3, 1, 0, 1, 1, 0),
        (3, 3, 1, 2, 1, 1, 0),
        (3, 3, 1, 0, 1, 2, 0),
        (3, 3, 0, 0, 0, 0, 0),
        (3, 3, 0, 3, 0, 0, 0),
    ];
    for reverse_moves in [false, true] {
        for reverse_credits in [false, true] {
            let log = literal_reviewed_game(reverse_moves, reverse_credits);
            assert_eq!(log.len(), 63);
            for (i, t) in log.iter().enumerate() {
                assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
                assert_eq!(
                    EventTransition::try_new(
                        t.before().clone(),
                        t.event().clone(),
                        t.after().clone()
                    )
                    .unwrap(),
                    *t
                );
                if i > 0 {
                    t.follows(&log[i - 1]).unwrap();
                }
            }
            let a = derive_accounting(&log).unwrap();
            assert!(a.complete_game);
            assert_eq!(a.runs, [5, 7]);
            let mut innings =
                std::collections::BTreeMap::from([(1, [Some(5), Some(7)]), (9, [Some(0), None])]);
            for inning in 2..=8 {
                innings.insert(inning, [Some(0), Some(0)]);
            }
            assert_eq!(a.innings, innings);
            for (side, rows) in [away, home].iter().enumerate() {
                assert_eq!(a.batters[side].len(), 9);
                for (slot, &(pa, ab, hits, k, runs, rbi, walks)) in rows.iter().enumerate() {
                    let id = &log[0].before().teams[side].batting_order[slot];
                    let start = if side == 0 { 1 } else { 3 };
                    let breakdown = HitBreakdown {
                        singles: u32::from(slot == start),
                        doubles: u32::from(slot == start + 1),
                        triples: u32::from(slot == start + 2),
                        home_runs: u32::from(slot == start + 3),
                    };
                    assert_eq!(
                        a.batters[side][id],
                        BatterLine {
                            plate_appearances: pa,
                            at_bats: Some(ab),
                            credited_hits: hits,
                            classified_hits: Some(hits),
                            credited_hit_breakdown: Some(breakdown),
                            credited_walks: walks,
                            credited_intentional_walks: Some(u32::from(side == 1 && slot == 2)),
                            credited_strikeouts: k,
                            credited_runs: runs,
                            credited_rbi: rbi,
                            classified_rbi: Some(rbi),
                            credited_hit_by_pitch: u32::from(side == 1 && slot == 0),
                            ..BatterLine::default()
                        },
                        "side{side} slot{slot}"
                    );
                }
                let (outs, bf, ab, runs, walks, ibb, k, hbp, ground) = if side == 0 {
                    (24, 31, 28, 7, 2, 1, 8, 1, 16)
                } else {
                    (27, 32, 31, 5, 1, 0, 9, 0, 18)
                };
                let team = &log[0].before().teams[side];
                assert_eq!(a.pitchers[side].len(), 1);
                assert_eq!(
                    a.pitchers[side][&team.pitcher],
                    PitcherLine {
                        outs_recorded: outs,
                        batters_faced: bf,
                        at_bats_against: Some(ab),
                        runs_allowed: runs,
                        earned_runs: Some(runs),
                        credited_hits_allowed: 4,
                        classified_hits_allowed: Some(4),
                        credited_hit_breakdown_allowed: Some(HitBreakdown {
                            singles: 1,
                            doubles: 1,
                            triples: 1,
                            home_runs: 1
                        }),
                        credited_walks_allowed: walks,
                        credited_intentional_walks_allowed: Some(ibb),
                        credited_strikeouts: k,
                        credited_hit_batters: hbp,
                        ..PitcherLine::default()
                    }
                );
                assert_eq!(a.classified_fielding[side].len(), 9);
                for (id, line) in &a.classified_fielding[side] {
                    let po = if id == &team.batting_order[0] {
                        ground
                    } else if id == &team.batting_order[1] {
                        k
                    } else {
                        0
                    };
                    let assists = if id == &team.pitcher { ground } else { 0 };
                    assert_eq!(
                        *line,
                        FieldingCoverageLine {
                            observed_events: bf,
                            putouts: Some(po),
                            assists: Some(assists),
                            errors: Some(0),
                            passed_balls: Some(0)
                        }
                    );
                    let credited = a.fielders[side].get(id).cloned().unwrap_or_default();
                    assert_eq!(
                        credited,
                        FielderLine {
                            credited_putouts: po,
                            credited_assists: assists,
                            credited_errors: 0,
                            credited_passed_balls: 0
                        }
                    );
                }
                assert!(!a.classified_fielding[side].contains_key(&team.batting_order[8]));
                assert_eq!(
                    a.classified_fielding[side]
                        .values()
                        .map(|f| f.putouts.unwrap())
                        .sum::<u32>(),
                    outs
                );
                assert_eq!(
                    a.batters[1 - side]
                        .values()
                        .map(|b| b.plate_appearances)
                        .sum::<u32>(),
                    bf
                );
                assert_eq!(
                    a.batters[1 - side]
                        .values()
                        .map(|b| b.credited_runs)
                        .sum::<u32>(),
                    runs
                );
                assert_eq!(
                    a.batters[1 - side]
                        .values()
                        .map(|b| b.credited_hit_breakdown.as_ref().unwrap().total_bases())
                        .sum::<u32>(),
                    10
                );
            }
        }
    }
}

#[test]
fn literal_reviewed_game_missing_facts_preserve_physics_not_complete_scoring() {
    let clean = literal_reviewed_game(true, true);
    let expected = derive_accounting(&clean).unwrap();
    for missing in 0..4 {
        // Away single, ground retirement or HR: all precede later complete plays.
        let index = [1, 1, 5, 4][missing];
        let mut log = clean.clone();
        let mut e = log[index].event().clone();
        match missing {
            0 => e.sole_safe_hit = None,
            1 => e.fielding_review = FieldingReview::Unknown,
            2 => e.ordinary_ground_throw = None,
            _ => e.sole_home_run = None,
        }
        let t = apply(log[index].before(), e).unwrap();
        assert_eq!(t.after(), log[index].after());
        assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
        log[index] = t;
        let a = derive_accounting(&log).unwrap();
        assert!(a.complete_game);
        assert_eq!(a.runs, expected.runs);
        assert_eq!(a.innings, expected.innings);
        assert_eq!(a.fielders, expected.fielders);
        assert_eq!(a.batters[1], expected.batters[1]);
        assert_eq!(a.pitchers[0], expected.pitchers[0]);
        assert_eq!(a.classified_fielding[0], expected.classified_fielding[0]);
        assert_eq!(
            a.pitchers[1]["home_starter"].earned_runs,
            if missing == 3 { Some(5) } else { None }
        );
        let batter = clean[index].event().participants.batter.as_ref().unwrap();
        assert_eq!(
            a.batters[0][batter].classified_rbi,
            if missing == 0 || missing == 3 {
                None
            } else {
                expected.batters[0][batter].classified_rbi
            }
        );
        for f in a.classified_fielding[1].values() {
            assert_eq!((f.putouts, f.assists), (None, None));
            assert_eq!(
                (f.errors, f.passed_balls),
                if missing == 1 {
                    (None, None)
                } else {
                    (Some(0), Some(0))
                }
            );
            assert_eq!(f.observed_events, 32);
        }
        // Normalize only the separately asserted evidence-sensitive cells;
        // every other credited/statistical/coverage value must remain exact.
        let mut normalized = a;
        normalized.classified_fielding[1] = expected.classified_fielding[1].clone();
        normalized.pitchers[1]
            .get_mut("home_starter")
            .unwrap()
            .earned_runs = Some(5);
        normalized.batters[0]
            .get_mut(batter)
            .unwrap()
            .classified_rbi = expected.batters[0][batter].classified_rbi;
        assert_eq!(normalized, expected);
    }
    for (index, kind) in [(1, CreditKind::Rbi), (5, CreditKind::Putout)] {
        let mut e = clean[index].event().clone();
        e.delta.credits.retain(|c| c.kind != kind);
        assert_eq!(
            apply(clean[index].before(), e.clone()),
            Err(TransitionError::IllegalCredit)
        );
        assert_eq!(
            EventTransition::try_new(
                clean[index].before().clone(),
                e,
                clean[index].after().clone()
            ),
            Err(TransitionError::IllegalCredit)
        );
    }
}

#[test]
fn sole_ground_muff_has_literal_positive_fielding_in_384_review_order_frames() {
    use bbsim_engine::accounting::{FieldingContribution, MisplayContribution};
    let mut frames = 0;
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            for mask in 0u8..8 {
                for reverse_moves in [false, true] {
                    for reverse_credits in [false, true] {
                        for complete in [false, true] {
                            let mut before = state();
                            before.half = half;
                            before.outs = outs;
                            let offense = half.offense();
                            let side = 1 - offense;
                            before.bases = std::array::from_fn(|i| {
                                (mask & (1 << i) != 0).then(|| Runner {
                                    // Future batter slot1 must not remain on base after this PA.
                                    player_id: before.teams[offense].batting_order[i + 5].clone(),
                                    responsible_pitcher: before.teams[side].pitcher.clone(),
                                })
                            });
                            let batter = before.teams[offense].batting_order[0].clone();
                            let fielder = before.teams[side].batting_order[3].clone();
                            let mut credits = vec![
                                credit(&batter, CreditKind::AtBat),
                                credit(&batter, CreditKind::NoHit),
                                credit(&fielder, CreditKind::Error),
                            ];
                            let mut moves = vec![batter_move(&before, Destination::First)];
                            for (i, runner) in before.bases.iter().enumerate() {
                                if let Some(runner) = runner {
                                    let to = [
                                        Destination::Second,
                                        Destination::Third,
                                        Destination::Home,
                                    ][i];
                                    moves.push(RunnerMove {
                                        runner: runner.clone(),
                                        from: [Base::First, Base::Second, Base::Third][i],
                                        to,
                                    });
                                    if to == Destination::Home {
                                        credits.push(credit(&runner.player_id, CreditKind::Run));
                                    }
                                }
                            }
                            let mut e = event(
                                &before,
                                EventKind::PlateAppearance,
                                moves,
                                0,
                                u8::from(mask & 4 != 0),
                                credits,
                                vec![],
                            );
                            e.muffed_ground_ball = Some(bbsim_engine::MuffedGroundBallFacts {
                                batter_id: batter,
                                fielder_id: fielder.clone(),
                            });
                            e.fielding_review = if complete {
                                FieldingReview::SoleGroundMuff
                            } else {
                                FieldingReview::Unknown
                            };
                            if reverse_moves {
                                e.delta.moves.reverse();
                            }
                            if reverse_credits {
                                e.delta.credits.reverse();
                            }
                            let t = apply(&before, e).unwrap_or_else(|err| {
                                panic!("muff frame {half:?} outs{outs} mask{mask} moves{reverse_moves} credits{reverse_credits} complete{complete}: {err:?}")
                            });
                            assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
                            assert_eq!(
                                EventTransition::try_new(
                                    before.clone(),
                                    t.event().clone(),
                                    t.after().clone()
                                )
                                .unwrap(),
                                t
                            );
                            let fielding = complete.then(|| FieldingContribution {
                                putouts: Default::default(),
                                assists: Default::default(),
                            });
                            let misplays = complete.then(|| MisplayContribution {
                                errors: std::collections::BTreeMap::from([(fielder.clone(), 1)]),
                                passed_balls: Default::default(),
                            });
                            assert_eq!(t.muffed_ground_fielding(), fielding);
                            assert_eq!(t.fielding_contribution(), fielding);
                            assert_eq!(t.muffed_ground_misplays(), misplays);
                            assert_eq!(t.misplay_contribution(), misplays);
                            assert_eq!(t.reviewed_misplay_zeros(), None);
                            let a = derive_accounting(std::slice::from_ref(&t)).unwrap();
                            assert_eq!(a.fielders[side][&fielder].credited_errors, 1);
                            assert_eq!(a.classified_fielding[side].len(), 9);
                            for (id, line) in &a.classified_fielding[side] {
                                assert_eq!(line.observed_events, 1);
                                assert_eq!(
                                    (line.putouts, line.assists, line.passed_balls),
                                    if complete {
                                        (Some(0), Some(0), Some(0))
                                    } else {
                                        (None, None, None)
                                    }
                                );
                                assert_eq!(
                                    line.errors,
                                    complete.then_some(u32::from(id == &fielder))
                                );
                            }
                            assert_eq!(before.outs, outs);
                            frames += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(frames, 384);
}

fn sole_muff_empty_frame() -> (GameState, Event) {
    let mut before = state();
    before.bases = [None, None, None];
    let batter = before.teams[0].batting_order[0].clone();
    let fielder = before.teams[1].batting_order[3].clone();
    let mut e = event(
        &before,
        EventKind::PlateAppearance,
        vec![batter_move(&before, Destination::First)],
        0,
        0,
        vec![
            credit(&batter, CreditKind::AtBat),
            credit(&batter, CreditKind::NoHit),
            credit(&fielder, CreditKind::Error),
        ],
        vec![],
    );
    e.muffed_ground_ball = Some(bbsim_engine::MuffedGroundBallFacts {
        batter_id: batter,
        fielder_id: fielder,
    });
    e.fielding_review = FieldingReview::SoleGroundMuff;
    (before, e)
}

#[test]
fn sole_ground_muff_has_an_independent_named_error_and_zero_pb() {
    let (before, e) = sole_muff_empty_frame();
    let t = apply(&before, e).unwrap();
    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
    let expected = bbsim_engine::accounting::MisplayContribution {
        errors: std::collections::BTreeMap::from([("home3".into(), 1)]),
        passed_balls: Default::default(),
    };
    assert_eq!(t.muffed_ground_misplays(), Some(expected.clone()));
    assert_eq!(t.misplay_contribution(), Some(expected));
    let a = derive_accounting(&[t]).unwrap();
    assert_eq!(a.classified_fielding[1]["home3"].errors, Some(1));
    assert_eq!(a.classified_fielding[1]["home3"].passed_balls, Some(0));
}

#[test]
fn sole_ground_muff_known_unit_actor_and_missing_binding_are_not_uncertainty() {
    let (before, control) = sole_muff_empty_frame();
    let valid = apply(&before, control.clone()).unwrap();
    assert_eq!(
        EventTransition::from_bytes(&valid.to_bytes()).unwrap(),
        valid
    );
    for defect in 0..9 {
        let mut e = control.clone();
        let expected = match defect {
            0 => {
                e.delta.credits.retain(|c| c.kind != CreditKind::Error);
                TransitionError::IllegalCredit
            }
            1 => {
                e.delta.credits.push(credit("home3", CreditKind::Error));
                TransitionError::IllegalCredit
            }
            2 => {
                e.muffed_ground_ball.as_mut().unwrap().batter_id = "away1".into();
                TransitionError::InvalidParticipants
            }
            3 => {
                e.muffed_ground_ball.as_mut().unwrap().fielder_id = "home8".into();
                TransitionError::IllegalCredit
            }
            4 => {
                e.muffed_ground_ball = None;
                TransitionError::UnsupportedRule
            }
            5 => {
                e.fielding_review = FieldingReview::NoMisplay;
                TransitionError::IllegalCredit
            }
            6 => {
                e.muffed_ground_ball.as_mut().unwrap().fielder_id = "home_reliever".into();
                TransitionError::IllegalCredit
            }
            7 => {
                e.delta
                    .credits
                    .iter_mut()
                    .find(|c| c.kind == CreditKind::Error)
                    .unwrap()
                    .player_id = "home0".into();
                TransitionError::IllegalCredit
            }
            _ => {
                e.delta.credits.retain(|c| c.kind != CreditKind::NoHit);
                TransitionError::IllegalCredit
            }
        };
        assert_eq!(
            apply(&before, e.clone()),
            Err(expected.clone()),
            "defect{defect}"
        );
        assert_eq!(
            EventTransition::try_new(before.clone(), e, valid.after().clone()),
            Err(expected),
            "defect{defect}"
        );
    }
}

#[test]
fn sole_ground_muff_unknown_prior_foul_error_units_have_no_universal_cap() {
    let (before, mut e) = sole_muff_empty_frame();
    e.fielding_review = FieldingReview::Unknown;
    e.delta.credits.push(credit("home3", CreditKind::Error));
    e.delta.credits.push(credit("home0", CreditKind::Assist));
    let t = apply(&before, e).unwrap();
    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
    assert_eq!(t.muffed_ground_misplays(), None);
    assert_eq!(t.misplay_contribution(), None);
    let a = derive_accounting(&[t]).unwrap();
    assert_eq!(a.fielders[1]["home3"].credited_errors, 2);
    assert_eq!(a.fielders[1]["home0"].credited_assists, 1);
    assert_eq!(a.classified_fielding[1]["home3"].errors, None);
}

#[test]
fn sole_ground_muff_semantic_wire_actors_binding_and_encoding_have_parsed_controls() {
    let (before, e) = sole_muff_empty_frame();
    let t = apply(&before, e).unwrap();
    let bytes = t.to_bytes();
    assert_eq!(EventTransition::from_bytes(&bytes), Ok(t));
    let mut marker = vec![1];
    for id in ["away0", "home3"] {
        marker.extend((id.len() as u64).to_le_bytes());
        marker.extend(id.as_bytes());
    }
    let positions: Vec<_> = bytes
        .windows(marker.len())
        .enumerate()
        .filter_map(|(i, b)| (b == marker).then_some(i))
        .collect();
    assert_eq!(positions.len(), 1);
    let offset = positions[0];
    let mut wrong_batter = bytes.clone();
    wrong_batter[offset + 1 + 8 + 4] = b'1';
    let mut wrong_fielder = bytes.clone();
    wrong_fielder[offset + marker.len() - 1] = b'8';
    let mut missing = bytes.clone();
    missing.splice(offset..offset + marker.len(), [0]);
    let results =
        [wrong_batter, wrong_fielder, missing].map(|b| EventTransition::from_bytes(&b).err());
    assert_eq!(
        results,
        [
            Some(TransitionError::InvalidParticipants),
            Some(TransitionError::IllegalCredit),
            Some(TransitionError::UnsupportedRule)
        ]
    );
    let mut invalid = bytes;
    invalid[offset] = 2;
    assert_eq!(
        EventTransition::from_bytes(&invalid),
        Err(TransitionError::InvalidEncoding)
    );
}

#[test]
fn sole_ground_muff_geometry_and_unknown_actor_facts_are_canonical_not_generic_bans() {
    let (before, control) = sole_muff_empty_frame();
    let mut extra = control.clone();
    extra.delta.moves[0].to = Destination::Second;
    assert_eq!(
        apply(&before, extra.clone()),
        Err(TransitionError::DeltaMismatch)
    );
    extra.muffed_ground_ball = None;
    extra.fielding_review = FieldingReview::Unknown;
    assert!(apply(&before, extra).is_ok());
    let mut wrong = control;
    wrong.fielding_review = FieldingReview::Unknown;
    wrong.muffed_ground_ball.as_mut().unwrap().batter_id = "away1".into();
    assert_eq!(
        apply(&before, wrong),
        Err(TransitionError::InvalidParticipants)
    );
}

#[test]
fn sole_ground_muff_distinct_chances_accumulate_but_do_not_repair_unknowns() {
    for uncertain_middle in [false, true] {
        let (mut before, template) = sole_muff_empty_frame();
        let mut log = vec![];
        for step in 0..3 {
            let batter =
                before.teams[0].batting_order[before.teams[0].next_batter as usize].clone();
            let mut e = template.clone();
            e.participants.batter = Some(batter.clone());
            e.muffed_ground_ball.as_mut().unwrap().batter_id = batter.clone();
            e.delta.credits = vec![
                credit(&batter, CreditKind::PlateAppearance),
                credit(&batter, CreditKind::AtBat),
                credit(&batter, CreditKind::NoHit),
                credit("home3", CreditKind::Error),
            ];
            e.delta.moves = vec![batter_move(&before, Destination::First)];
            for (i, runner) in before.bases.iter().enumerate() {
                if let Some(runner) = runner {
                    e.delta.moves.push(RunnerMove {
                        runner: runner.clone(),
                        from: [Base::First, Base::Second, Base::Third][i],
                        to: [Destination::Second, Destination::Third, Destination::Home][i],
                    });
                }
            }
            if uncertain_middle && step == 1 {
                e.fielding_review = FieldingReview::Unknown;
            }
            let frozen = before.clone();
            let t = apply(&before, e).unwrap();
            assert_eq!(before, frozen);
            assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
            before = t.after().clone();
            log.push(t);
        }
        let a = derive_accounting(&log).unwrap();
        assert_eq!(a.fielders[1]["home3"].credited_errors, 3);
        for (id, f) in &a.classified_fielding[1] {
            assert_eq!(f.observed_events, 3);
            assert_eq!(
                f.errors,
                if uncertain_middle {
                    None
                } else {
                    Some(if id == "home3" { 3 } else { 0 })
                }
            );
            assert_eq!(
                (f.putouts, f.assists, f.passed_balls),
                if uncertain_middle {
                    (None, None, None)
                } else {
                    (Some(0), Some(0), Some(0))
                }
            );
        }
    }
}

#[test]
fn sole_ground_muff_incoming_fielder_has_independent_positive_error_coverage() {
    let (before, mut first) = sole_muff_empty_frame();
    first.fielding_review = FieldingReview::Unknown;
    let first = apply(&before, first).unwrap();
    let change = event(
        first.after(),
        EventKind::Substitution,
        vec![],
        0,
        0,
        vec![],
        vec![PersonnelChange::BattingSlot {
            team: TeamSide::Home,
            slot: 3,
            outgoing: "home3".into(),
            incoming: "home_reliever".into(),
        }],
    );
    let substitution = apply(first.after(), change).unwrap();
    let before = substitution.after();
    let batter = before.teams[0].batting_order[before.teams[0].next_batter as usize].clone();
    let mut e = event(
        before,
        EventKind::PlateAppearance,
        vec![
            batter_move(before, Destination::First),
            RunnerMove {
                runner: before.bases[0].clone().unwrap(),
                from: Base::First,
                to: Destination::Second,
            },
        ],
        0,
        0,
        vec![
            credit(&batter, CreditKind::AtBat),
            credit(&batter, CreditKind::NoHit),
            credit("home_reliever", CreditKind::Error),
        ],
        vec![],
    );
    e.muffed_ground_ball = Some(bbsim_engine::MuffedGroundBallFacts {
        batter_id: batter,
        fielder_id: "home_reliever".into(),
    });
    e.fielding_review = FieldingReview::SoleGroundMuff;
    let next = apply(before, e).unwrap();
    let a = derive_accounting(&[first, substitution, next]).unwrap();
    assert_eq!(a.classified_fielding[1]["home3"].observed_events, 1);
    assert_eq!(a.classified_fielding[1]["home3"].errors, None);
    assert_eq!(
        a.classified_fielding[1]["home_reliever"],
        bbsim_engine::accounting::FieldingCoverageLine {
            observed_events: 1,
            putouts: Some(0),
            assists: Some(0),
            errors: Some(1),
            passed_balls: Some(0),
        }
    );
    assert_eq!(a.fielders[1]["home3"].credited_errors, 1);
    assert_eq!(a.fielders[1]["home_reliever"].credited_errors, 1);
    assert_eq!(a.classified_fielding[1]["home_starter"].observed_events, 2);
    assert_eq!(a.classified_fielding[1]["home_starter"].errors, None);
}

#[test]
fn sole_ground_muff_known_facts_and_units_precede_unbound_balk_history() {
    let (before, control) = sole_muff_empty_frame();
    let valid = apply(&before, control.clone()).unwrap();
    let mut partial = control;
    partial.fielding_review = FieldingReview::Unknown;
    partial.balk = Some(bbsim_engine::BalkFacts {
        cause: bbsim_engine::BalkCause::Unknown,
        disposition: bbsim_engine::BalkDisposition::Unknown,
        resolution: bbsim_engine::BalkResolution::Unknown,
    });
    assert_eq!(
        apply(&before, partial.clone()),
        Err(TransitionError::UnsupportedRule)
    );
    for defect in 0..4 {
        let mut e = partial.clone();
        let expected = match defect {
            0 => {
                e.muffed_ground_ball.as_mut().unwrap().batter_id = "away1".into();
                TransitionError::InvalidParticipants
            }
            1 => {
                e.muffed_ground_ball.as_mut().unwrap().fielder_id = "home8".into();
                TransitionError::IllegalCredit
            }
            2 => {
                e.delta.credits.retain(|c| c.kind != CreditKind::Error);
                TransitionError::IllegalCredit
            }
            _ => {
                e.delta.moves[0].runner.responsible_pitcher = "away_pitcher".into();
                TransitionError::DeltaMismatch
            }
        };
        assert_eq!(
            apply(&before, e.clone()),
            Err(expected.clone()),
            "defect{defect}"
        );
        assert_eq!(
            EventTransition::try_new(before.clone(), e, valid.after().clone()),
            Err(expected),
            "defect{defect}"
        );
    }
}

#[test]
fn sole_ground_muff_missing_binding_cannot_invent_a_universal_assist_ban() {
    let (before, mut e) = sole_muff_empty_frame();
    e.muffed_ground_ball = None;
    e.fielding_review = FieldingReview::Unknown;
    e.delta.credits.push(credit("home0", CreditKind::Assist));
    let partial = apply(&before, e.clone()).unwrap();
    assert_eq!(
        EventTransition::from_bytes(&partial.to_bytes()).unwrap(),
        partial
    );
    let bytes = partial.to_bytes();
    let mut marker = vec![1];
    for id in [
        e.participants.batter.as_ref().unwrap(),
        &e.participants.pitcher,
    ] {
        marker.extend((id.len() as u64).to_le_bytes());
        marker.extend(id.as_bytes());
    }
    marker.push(0); // parsed Unknown review
    let review_offset = marker.len() - 1;
    // Include the one-move Delta prefix: the same runner also appears in after.bases.
    marker.extend(1u64.to_le_bytes());
    let batter = e.participants.batter.as_ref().unwrap();
    marker.extend((batter.len() as u64).to_le_bytes());
    marker.extend(batter.as_bytes());
    let positions: Vec<_> = bytes
        .windows(marker.len())
        .enumerate()
        .filter_map(|(i, b)| (b == marker).then_some(i))
        .collect();
    assert_eq!(positions.len(), 1);
    let mut forged = bytes;
    forged[positions[0] + review_offset] = 2; // SoleGroundMuff, still no binding
    e.fielding_review = FieldingReview::SoleGroundMuff;
    assert_eq!(
        [
            apply(&before, e.clone()).err(),
            EventTransition::try_new(before, e, partial.after().clone()).err(),
            EventTransition::from_bytes(&forged).err(),
        ],
        [
            Some(TransitionError::UnsupportedRule),
            Some(TransitionError::UnsupportedRule),
            Some(TransitionError::UnsupportedRule)
        ]
    );
}

#[test]
fn sole_ground_muff_two_out_positive_rbi_is_known_and_unbound_stays_partial() {
    for outs in 0..3 {
        for review in [FieldingReview::Unknown, FieldingReview::SoleGroundMuff] {
            let (mut before, mut e) = sole_muff_empty_frame();
            before.outs = outs;
            let runner = Runner {
                player_id: "runner".into(),
                responsible_pitcher: "home_starter".into(),
            };
            before.bases[2] = Some(runner.clone());
            e.fielding_review = review;
            e.delta.moves.push(RunnerMove {
                runner,
                from: Base::Third,
                to: Destination::Home,
            });
            e.delta.runs = 1;
            e.delta.credits.push(credit("runner", CreditKind::Run));
            let control = apply(&before, e.clone()).unwrap();
            assert_eq!(
                EventTransition::from_bytes(&control.to_bytes()).unwrap(),
                control
            );
            assert_eq!(
                control.rbi_contribution(),
                (outs == 2).then(|| bbsim_engine::accounting::RbiContribution {
                    runs_batted_in: [("away0".into(), 0)].into(),
                })
            );
            assert_eq!(
                derive_accounting(std::slice::from_ref(&control))
                    .unwrap()
                    .batters[0]["away0"]
                    .classified_rbi,
                (outs == 2).then_some(0)
            );
            let mut bad = e.clone();
            bad.delta.credits.push(credit("away0", CreditKind::Rbi));
            let mut unbound = bad.clone();
            unbound.muffed_ground_ball = None;
            unbound.fielding_review = FieldingReview::Unknown;
            let legacy = apply(&before, unbound).unwrap();
            assert_eq!(legacy.before(), control.before());
            assert_eq!(legacy.after(), control.after());
            assert_eq!(legacy.rbi_contribution(), None);
            if outs != 2 {
                let partial = apply(&before, bad).unwrap();
                assert_eq!(partial.after(), control.after());
                assert_eq!(partial.rbi_contribution(), None);
                continue;
            }
            // Forge only one supplied RBI in a fully parsed bound record.
            let mut marker = (e.delta.credits.len() as u64).to_le_bytes().to_vec();
            for c in &e.delta.credits {
                marker.extend((c.player_id.len() as u64).to_le_bytes());
                marker.extend(c.player_id.as_bytes());
                marker.push(match c.kind {
                    CreditKind::PlateAppearance => 0,
                    CreditKind::Run => 4,
                    CreditKind::Error => 10,
                    CreditKind::AtBat => 12,
                    CreditKind::NoHit => 25,
                    _ => panic!("unexpected control credit"),
                });
            }
            let bytes = control.to_bytes();
            let positions: Vec<_> = bytes
                .windows(marker.len())
                .enumerate()
                .filter_map(|(i, b)| (b == marker).then_some(i))
                .collect();
            assert_eq!(positions.len(), 1);
            let mut replaced = marker.clone();
            replaced[..8].copy_from_slice(&((e.delta.credits.len() + 1) as u64).to_le_bytes());
            replaced.extend(5u64.to_le_bytes());
            replaced.extend(b"away0");
            replaced.push(5);
            let mut forged = bytes;
            forged.splice(positions[0]..positions[0] + marker.len(), replaced);
            assert_eq!(
                [
                    apply(&before, bad.clone()).err(),
                    EventTransition::try_new(before.clone(), bad, control.after().clone()).err(),
                    EventTransition::from_bytes(&forged).err(),
                ],
                [
                    Some(TransitionError::IllegalCredit),
                    Some(TransitionError::IllegalCredit),
                    Some(TransitionError::IllegalCredit)
                ]
            );
        }
    }
}

#[test]
fn sole_ground_muff_unbound_label_does_not_hide_a_foreign_steal_actor() {
    let (before, mut e) = sole_muff_empty_frame();
    e.muffed_ground_ball = None;
    e.fielding_review = FieldingReview::Unknown;
    let control = apply(&before, e.clone()).unwrap();
    let mut bytes = control.to_bytes();
    assert_eq!(EventTransition::from_bytes(&bytes).unwrap(), control);
    let mut review = vec![1];
    for id in [
        e.participants.batter.as_ref().unwrap(),
        &e.participants.pitcher,
    ] {
        review.extend((id.len() as u64).to_le_bytes());
        review.extend(id.as_bytes());
    }
    review.push(0);
    let review_offset = review.len() - 1;
    review.extend(1u64.to_le_bytes());
    review.extend(5u64.to_le_bytes());
    review.extend(b"away0");
    let positions: Vec<_> = bytes
        .windows(review.len())
        .enumerate()
        .filter_map(|(i, b)| (b == review).then_some(i))
        .collect();
    assert_eq!(positions.len(), 1);
    bytes[positions[0] + review_offset] = 2;
    let mut credits = (e.delta.credits.len() as u64).to_le_bytes().to_vec();
    for c in &e.delta.credits {
        credits.extend((c.player_id.len() as u64).to_le_bytes());
        credits.extend(c.player_id.as_bytes());
        credits.push(match c.kind {
            CreditKind::AtBat => 12,
            CreditKind::NoHit => 25,
            CreditKind::Error => 10,
            CreditKind::PlateAppearance => 0,
            _ => panic!("unexpected control credit"),
        });
    }
    let positions: Vec<_> = bytes
        .windows(credits.len())
        .enumerate()
        .filter_map(|(i, b)| (b == credits).then_some(i))
        .collect();
    assert_eq!(positions.len(), 1);
    let mut replacement = credits.clone();
    replacement[..8].copy_from_slice(&((e.delta.credits.len() + 1) as u64).to_le_bytes());
    replacement.extend(5u64.to_le_bytes());
    replacement.extend(b"ghost");
    replacement.push(6);
    bytes.splice(positions[0]..positions[0] + credits.len(), replacement);
    e.fielding_review = FieldingReview::SoleGroundMuff;
    e.delta
        .credits
        .push(credit("ghost", CreditKind::StolenBase));
    assert_eq!(
        [
            apply(&before, e.clone()).err(),
            EventTransition::try_new(before, e, control.after().clone()).err(),
            EventTransition::from_bytes(&bytes).err(),
        ],
        [
            Some(TransitionError::IllegalCredit),
            Some(TransitionError::IllegalCredit),
            Some(TransitionError::IllegalCredit)
        ]
    );
    // Presence is not net-advance eligibility: a present held runner may have
    // unresolved intermediate history. Do not call that definitely foreign.
    let (mut before, mut held) = sole_muff_empty_frame();
    let runner = Runner {
        player_id: "runner".into(),
        responsible_pitcher: "home_starter".into(),
    };
    before.bases[0] = Some(runner.clone());
    held.muffed_ground_ball = None;
    held.fielding_review = FieldingReview::Unknown;
    held.delta.moves[0].to = Destination::Second;
    held.delta.moves.push(RunnerMove {
        runner,
        from: Base::First,
        to: Destination::First,
    });
    let physical = apply(&before, held.clone()).unwrap();
    held.fielding_review = FieldingReview::SoleGroundMuff;
    held.delta
        .credits
        .push(credit("runner", CreditKind::StolenBase));
    assert_eq!(
        apply(&before, held.clone()),
        Err(TransitionError::UnsupportedRule)
    );
    assert_eq!(
        EventTransition::try_new(before, held, physical.after().clone()),
        Err(TransitionError::UnsupportedRule)
    );
}

fn sole_ground_muff_complete_game() -> Vec<EventTransition> {
    let (mut before, first_muff) = sole_muff_empty_frame();
    before.inning = 1;
    before.outs = 0;
    let mut log = vec![];
    while !before.over {
        let offense = before.half.offense();
        let batter =
            before.teams[offense].batting_order[before.teams[offense].next_batter as usize].clone();
        let mut e = if log.is_empty() {
            first_muff.clone()
        } else if before.inning == 1
            && before.half == Half::Bottom
            && before.teams[1].next_batter == 0
        {
            let mut e = event(
                &before,
                EventKind::OutOfParkHomeRun,
                vec![batter_move(&before, Destination::Home)],
                0,
                1,
                vec![
                    credit(&batter, CreditKind::PlateAppearance),
                    credit(&batter, CreditKind::Hit),
                    credit(&batter, CreditKind::Run),
                    credit(&batter, CreditKind::Rbi),
                ],
                vec![],
            );
            e.sole_home_run = Some(bbsim_engine::SoleHomeRunFacts {
                batter_id: batter.clone(),
            });
            reviewed(e)
        } else {
            let catcher = before.teams[1 - offense].batting_order[0].clone();
            let mut moves = vec![batter_move(&before, Destination::Out)];
            for (i, runner) in before.bases.iter().enumerate() {
                if let Some(runner) = runner {
                    moves.push(RunnerMove {
                        runner: runner.clone(),
                        from: [Base::First, Base::Second, Base::Third][i],
                        to: [Destination::First, Destination::Second, Destination::Third][i],
                    });
                }
            }
            let mut e = event(
                &before,
                EventKind::PlateAppearance,
                moves,
                1,
                0,
                vec![
                    credit(&batter, CreditKind::Strikeout),
                    credit(&batter, CreditKind::NoHit),
                    credit(&catcher, CreditKind::Putout),
                ],
                vec![],
            );
            e.caught_strikeout = Some(bbsim_engine::CaughtStrikeoutFacts {
                batter_id: batter.clone(),
                catcher_id: catcher,
            });
            reviewed(e)
        };
        if before.outs == 2 {
            e.delta.third_out = Some(ThirdOut {
                runner_id: batter,
                kind: ThirdOutKind::BatterBeforeFirst,
                home_before_out: vec![],
            });
        }
        let t = apply(&before, e).unwrap();
        assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
        before = t.after().clone();
        log.push(t);
    }
    assert_eq!(log.len(), 53);
    assert_eq!(
        (before.inning, before.half, before.score),
        (9, Half::Top, [0, 1])
    );
    log
}

#[test]
fn sole_ground_muff_complete_physics_cannot_repair_missing_fielding_evidence() {
    let log = sole_ground_muff_complete_game();
    let full = derive_accounting(&log).unwrap();
    assert!(full.complete_game);
    assert_eq!(full.runs, [0, 1]);
    assert_eq!(full.pitchers[1]["home_starter"].earned_runs, Some(0));
    assert_eq!(full.pitchers[0]["away_pitcher"].earned_runs, Some(1));
    assert_eq!(full.classified_fielding[1]["home0"].putouts, Some(27));
    assert_eq!(full.classified_fielding[0]["away0"].putouts, Some(24));
    for side in 0..2 {
        assert_eq!(full.classified_fielding[side].len(), 9);
        for (id, line) in &full.classified_fielding[side] {
            assert_eq!(line.observed_events, if side == 1 { 28 } else { 25 });
            assert_eq!(line.errors, Some(u32::from(id == "home3")));
            assert_eq!(line.assists, Some(0));
            assert_eq!(line.passed_balls, Some(0));
        }
    }
    for drop_fact in [false, true] {
        let mut partial = log.clone();
        let mut e = partial[0].event().clone();
        e.fielding_review = FieldingReview::Unknown;
        if drop_fact {
            e.muffed_ground_ball = None;
        }
        partial[0] = apply(partial[0].before(), e).unwrap();
        for (a, b) in partial.iter().zip(&log) {
            assert_eq!(a.before(), b.before());
            assert_eq!(a.after(), b.after());
            assert_eq!(a.event().delta, b.event().delta);
        }
        let mut a = derive_accounting(&partial).unwrap();
        assert!(a.complete_game);
        assert_eq!(a.fielders[1]["home3"].credited_errors, 1);
        assert_eq!(a.classified_fielding[0], full.classified_fielding[0]);
        for (id, line) in &a.classified_fielding[1] {
            assert_eq!(line.observed_events, 28);
            assert_eq!(
                (line.putouts, line.assists, line.errors, line.passed_balls),
                (None, None, None, None)
            );
            assert!(full.classified_fielding[1][id].errors.is_some());
        }
        a.classified_fielding = full.classified_fielding.clone();
        assert_eq!(a, full);
    }
}

#[test]
fn two_out_muff_rbi_has_literal_named_zero_in_384_causal_frames() {
    let mut frames = 0;
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            for mask in 0u8..8 {
                for complete in [false, true] {
                    for reverse_moves in [false, true] {
                        for reverse_credits in [false, true] {
                            let mut before = state();
                            before.half = half;
                            before.outs = outs;
                            let offense = half.offense();
                            let side = 1 - offense;
                            before.bases = std::array::from_fn(|i| {
                                (mask & (1 << i) != 0).then(|| Runner {
                                    player_id: before.teams[offense].batting_order[i + 5].clone(),
                                    responsible_pitcher: before.teams[side].pitcher.clone(),
                                })
                            });
                            let batter = before.teams[offense].batting_order[0].clone();
                            let fielder = before.teams[side].batting_order[3].clone();
                            let mut moves = vec![batter_move(&before, Destination::First)];
                            let mut credits = vec![
                                credit(&batter, CreditKind::AtBat),
                                credit(&batter, CreditKind::NoHit),
                                credit(&fielder, CreditKind::Error),
                            ];
                            for (i, r) in before.bases.iter().enumerate() {
                                if let Some(r) = r {
                                    let to = [
                                        Destination::Second,
                                        Destination::Third,
                                        Destination::Home,
                                    ][i];
                                    moves.push(RunnerMove {
                                        runner: r.clone(),
                                        from: [Base::First, Base::Second, Base::Third][i],
                                        to,
                                    });
                                    if i == 2 {
                                        credits.push(credit(&r.player_id, CreditKind::Run));
                                    }
                                }
                            }
                            let mut e = event(
                                &before,
                                EventKind::PlateAppearance,
                                moves,
                                0,
                                u8::from(mask & 4 != 0),
                                credits,
                                vec![],
                            );
                            e.muffed_ground_ball = Some(bbsim_engine::MuffedGroundBallFacts {
                                batter_id: batter.clone(),
                                fielder_id: fielder.clone(),
                            });
                            e.fielding_review = if complete {
                                FieldingReview::SoleGroundMuff
                            } else {
                                FieldingReview::Unknown
                            };
                            if reverse_moves {
                                e.delta.moves.reverse();
                            }
                            if reverse_credits {
                                e.delta.credits.reverse();
                            }
                            let t = apply(&before, e.clone()).unwrap();
                            assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
                            assert_eq!(
                                EventTransition::try_new(
                                    before.clone(),
                                    e.clone(),
                                    t.after().clone()
                                )
                                .unwrap(),
                                t
                            );
                            let expected = if mask & 4 == 0 || outs == 2 {
                                Some(0)
                            } else {
                                None
                            };
                            assert_eq!(
                                t.rbi_contribution(),
                                expected.map(|value| bbsim_engine::accounting::RbiContribution {
                                    runs_batted_in: [(batter.clone(), value)].into()
                                }),
                                "{half:?} outs{outs} mask{mask} complete{complete}"
                            );
                            let bound = derive_accounting(std::slice::from_ref(&t)).unwrap();
                            assert_eq!(bound.batters[offense][&batter].classified_rbi, expected);
                            let mut legacy = e;
                            legacy.muffed_ground_ball = None;
                            legacy.fielding_review = FieldingReview::Unknown;
                            let legacy = apply(&before, legacy).unwrap();
                            assert_eq!(legacy.before(), t.before());
                            assert_eq!(legacy.after(), t.after());
                            assert_eq!(legacy.event().delta, t.event().delta);
                            assert_eq!(
                                legacy.rbi_contribution(),
                                (mask & 4 == 0).then(|| {
                                    bbsim_engine::accounting::RbiContribution {
                                        runs_batted_in: [(batter.clone(), 0)].into(),
                                    }
                                })
                            );
                            let mut partial = derive_accounting(&[legacy]).unwrap();
                            assert_eq!(
                                partial.batters[offense][&batter].classified_rbi,
                                if mask & 4 == 0 { Some(0) } else { None }
                            );
                            partial.batters[offense]
                                .get_mut(&batter)
                                .unwrap()
                                .classified_rbi = expected;
                            for (id, line) in &bound.classified_fielding[side] {
                                assert_eq!(line.observed_events, 1);
                                assert_eq!(
                                    (line.putouts, line.assists, line.errors, line.passed_balls),
                                    if complete {
                                        (Some(0), Some(0), Some(u32::from(id == &fielder)), Some(0))
                                    } else {
                                        (None, None, None, None)
                                    }
                                );
                                let p = &partial.classified_fielding[side][id];
                                assert_eq!(
                                    (p.putouts, p.assists, p.errors, p.passed_balls),
                                    (None, None, None, None)
                                );
                            }
                            partial.classified_fielding = bound.classified_fielding.clone();
                            assert_eq!(partial, bound);
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
fn two_out_muff_rbi_prior_foul_units_do_not_supply_whole_chance_review() {
    let (mut before, mut e) = sole_muff_empty_frame();
    before.outs = 2;
    let runner = Runner {
        player_id: "runner".into(),
        responsible_pitcher: "home_starter".into(),
    };
    before.bases[2] = Some(runner.clone());
    e.delta.moves.push(RunnerMove {
        runner,
        from: Base::Third,
        to: Destination::Home,
    });
    e.delta.runs = 1;
    e.delta.credits.push(credit("runner", CreditKind::Run));
    e.delta.credits.push(credit("home3", CreditKind::Error));
    e.delta.credits.push(credit("home0", CreditKind::Assist));
    e.fielding_review = FieldingReview::Unknown;
    let t = apply(&before, e).unwrap();
    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
    assert_eq!(
        t.rbi_contribution(),
        Some(bbsim_engine::accounting::RbiContribution {
            runs_batted_in: [("away0".into(), 0)].into()
        })
    );
    assert_eq!(t.fielding_contribution(), None);
    assert_eq!(t.misplay_contribution(), None);
    let a = derive_accounting(&[t]).unwrap();
    assert_eq!(a.batters[0]["away0"].classified_rbi, Some(0));
    assert_eq!(a.fielders[1]["home3"].credited_errors, 2);
    assert_eq!(a.fielders[1]["home0"].credited_assists, 1);
    assert_eq!(a.classified_fielding[1]["home3"].errors, None);
    assert_eq!(a.pitchers[1]["home_starter"].earned_runs, None);
}

#[test]
fn two_out_muff_rbi_missing_positive_cause_taints_only_its_batter_irreversibly() {
    for missing_middle in [false, true] {
        let (mut before, template) = sole_muff_empty_frame();
        before.outs = 2;
        before.bases[2] = Some(Runner {
            player_id: "runner".into(),
            responsible_pitcher: "home_starter".into(),
        });
        let mut log = vec![];
        for step in 0..19 {
            let batter =
                before.teams[0].batting_order[before.teams[0].next_batter as usize].clone();
            let mut moves = vec![batter_move(&before, Destination::First)];
            let mut credits = vec![
                credit(&batter, CreditKind::AtBat),
                credit(&batter, CreditKind::NoHit),
                credit("home3", CreditKind::Error),
            ];
            let mut runs = 0;
            for (i, r) in before.bases.iter().enumerate() {
                if let Some(r) = r {
                    moves.push(RunnerMove {
                        runner: r.clone(),
                        from: [Base::First, Base::Second, Base::Third][i],
                        to: [Destination::Second, Destination::Third, Destination::Home][i],
                    });
                    if i == 2 {
                        runs += 1;
                        credits.push(credit(&r.player_id, CreditKind::Run));
                    }
                }
            }
            let mut e = event(
                &before,
                EventKind::PlateAppearance,
                moves,
                0,
                runs,
                credits,
                vec![],
            );
            e.muffed_ground_ball = template.muffed_ground_ball.clone();
            e.muffed_ground_ball.as_mut().unwrap().batter_id = batter;
            e.fielding_review = FieldingReview::SoleGroundMuff;
            if missing_middle && step == 9 {
                assert_eq!(runs, 1);
                e.muffed_ground_ball = None;
                e.fielding_review = FieldingReview::Unknown;
            }
            let t = apply(&before, e).unwrap();
            assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
            before = t.after().clone();
            log.push(t);
        }
        let a = derive_accounting(&log).unwrap();
        assert_eq!(a.batters[0]["away0"].plate_appearances, 3);
        assert_eq!(
            a.batters[0]["away0"].classified_rbi,
            if missing_middle { None } else { Some(0) }
        );
        for (id, line) in &a.batters[0] {
            if id != "away0" {
                assert_eq!(line.classified_rbi, Some(0));
            }
        }
    }
}

fn two_out_muff_er_half(
    mask: u8,
    half: Half,
    reverse_moves: bool,
    reverse_credits: bool,
    two_homers: bool,
    prefix_scoring: bool,
    accepted_outs: u8,
) -> (Vec<EventTransition>, usize) {
    let mut before = state();
    before.inning = 1;
    before.outs = 0;
    before.bases = [None, None, None];
    let mut log = vec![];
    if half == Half::Bottom {
        finish_with_three_strikeouts(before, &mut log, FieldingReview::NoMisplay);
        before = log.last().unwrap().after().clone();
    }
    if prefix_scoring {
        let homer = reviewed_homer_from(before.clone()).remove(0);
        before = homer.after().clone();
        log.push(homer);
    }
    // Establish exact masks without a fabricated occupied origin or prefix run.
    for base in (0..3).rev() {
        if mask & (1 << base) == 0 {
            continue;
        }
        let batter = before.teams[half.offense()].batting_order
            [before.teams[half.offense()].next_batter as usize]
            .clone();
        let walk = event(
            &before,
            EventKind::PlateAppearance,
            vec![batter_move(&before, Destination::First)],
            0,
            0,
            vec![
                credit(&batter, CreditKind::Walk),
                credit(&batter, CreditKind::OrdinaryWalk),
                credit(&batter, CreditKind::NoHit),
            ],
            vec![],
        );
        before = record_clean(&mut log, &before, walk);
        for step in 0..base {
            let e = event(
                &before,
                EventKind::RunnerOnly(RunnerEvent::WildPitch),
                vec![RunnerMove {
                    runner: before.bases[step].clone().unwrap(),
                    from: [Base::First, Base::Second][step],
                    to: [Destination::Second, Destination::Third][step],
                }],
                0,
                0,
                vec![],
                vec![],
            );
            before = record_clean(&mut log, &before, e);
        }
    }
    // Two actual accepted outs, then one lost routine before-first retirement.
    for step in 0..((if two_homers { 6 } else { 5 }) + usize::from(2 - accepted_outs)) {
        if step < 2 && step >= usize::from(accepted_outs) {
            continue;
        }
        let offense = half.offense();
        let defense = 1 - offense;
        let batter =
            before.teams[offense].batting_order[before.teams[offense].next_batter as usize].clone();
        // The shared event helper inserts PA for PlateAppearance only.
        let mut credits = vec![];
        let mut moves;
        let mut runs = 0;
        let mut e;
        if step == 2 {
            moves = vec![batter_move(&before, Destination::First)];
            credits.extend([
                credit(&batter, CreditKind::AtBat),
                credit(&batter, CreditKind::NoHit),
                credit(&before.teams[defense].batting_order[3], CreditKind::Error),
            ]);
            for (i, r) in before.bases.iter().enumerate() {
                if let Some(r) = r {
                    let to = [Destination::Second, Destination::Third, Destination::Home][i];
                    moves.push(RunnerMove {
                        runner: r.clone(),
                        from: [Base::First, Base::Second, Base::Third][i],
                        to,
                    });
                    if i == 2 {
                        runs += 1;
                        credits.push(credit(&r.player_id, CreditKind::Run));
                    }
                }
            }
            e = event(
                &before,
                EventKind::PlateAppearance,
                moves,
                0,
                runs,
                credits,
                vec![],
            );
            e.muffed_ground_ball = Some(bbsim_engine::MuffedGroundBallFacts {
                batter_id: batter,
                fielder_id: before.teams[defense].batting_order[3].clone(),
            });
            e.fielding_review = FieldingReview::SoleGroundMuff;
        } else if step == 3 || (two_homers && step == 4) {
            moves = vec![batter_move(&before, Destination::Home)];
            runs = 1;
            credits.extend([
                credit(&batter, CreditKind::PlateAppearance),
                credit(&batter, CreditKind::Hit),
                credit(&batter, CreditKind::Run),
                credit(&batter, CreditKind::Rbi),
            ]);
            for (i, r) in before.bases.iter().enumerate() {
                if let Some(r) = r {
                    moves.push(RunnerMove {
                        runner: r.clone(),
                        from: [Base::First, Base::Second, Base::Third][i],
                        to: Destination::Home,
                    });
                    runs += 1;
                    credits.push(credit(&r.player_id, CreditKind::Run));
                    credits.push(credit(&batter, CreditKind::Rbi));
                }
            }
            e = reviewed(event(
                &before,
                EventKind::OutOfParkHomeRun,
                moves,
                0,
                runs,
                credits,
                vec![],
            ));
            e.sole_home_run = Some(bbsim_engine::SoleHomeRunFacts { batter_id: batter });
        } else {
            let catcher = before.teams[defense].batting_order[0].clone();
            moves = vec![batter_move(&before, Destination::Out)];
            credits.extend([
                credit(&batter, CreditKind::Strikeout),
                credit(&batter, CreditKind::NoHit),
                credit(&catcher, CreditKind::Putout),
            ]);
            for (i, r) in before.bases.iter().enumerate() {
                if let Some(r) = r {
                    moves.push(RunnerMove {
                        runner: r.clone(),
                        from: [Base::First, Base::Second, Base::Third][i],
                        to: [Destination::First, Destination::Second, Destination::Third][i],
                    });
                }
            }
            e = reviewed(event(
                &before,
                EventKind::PlateAppearance,
                moves,
                1,
                0,
                credits,
                vec![],
            ));
            e.caught_strikeout = Some(bbsim_engine::CaughtStrikeoutFacts {
                batter_id: batter.clone(),
                catcher_id: catcher,
            });
            if before.outs == 2 {
                e.delta.third_out = Some(ThirdOut {
                    runner_id: batter,
                    kind: ThirdOutKind::BatterBeforeFirst,
                    home_before_out: vec![],
                });
            }
        }
        if reverse_moves {
            e.delta.moves.reverse();
        }
        if reverse_credits {
            e.delta.credits.reverse();
        }
        let t = apply(&before, e).unwrap();
        before = t.after().clone();
        log.push(t);
    }
    let index = log
        .iter()
        .position(|t| t.event().muffed_ground_ball.is_some())
        .unwrap();
    (log, index)
}

#[test]
fn two_out_muff_er_literal_rule_916_example_has_three_runs_and_zero_earned() {
    let (log, index) = two_out_muff_er_half(0, Half::Top, false, false, true, false, 2);
    assert_eq!((log.len(), index), (6, 2));
    for t in &log {
        assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
        assert_eq!(
            EventTransition::try_new(t.before().clone(), t.event().clone(), t.after().clone())
                .unwrap(),
            *t
        );
    }
    let a = derive_accounting(&log).unwrap();
    assert_eq!(a.runs, [3, 0]);
    assert_eq!(a.innings[&1], [Some(3), None]);
    let p = &a.pitchers[1]["home_starter"];
    assert_eq!(
        (
            p.runs_allowed,
            p.outs_recorded,
            p.batters_faced,
            p.at_bats_against,
            p.classified_hits_allowed,
            p.credited_strikeouts
        ),
        (3, 3, 6, Some(6), Some(2), 3)
    );
    assert_eq!(a.classified_fielding[1]["home0"].putouts, Some(3));
    assert_eq!(a.classified_fielding[1]["home3"].errors, Some(1));
    for l in a.classified_fielding[1].values() {
        assert_eq!(l.observed_events, 6);
        assert_eq!(l.assists, Some(0));
        assert_eq!(l.passed_balls, Some(0));
    }
    for id in ["away0", "away1", "away5"] {
        assert_eq!(a.batters[0][id].credited_strikeouts, 1);
    }
    assert_eq!(a.batters[0]["away2"].credited_runs, 1);
    assert_eq!(a.batters[0]["away2"].classified_rbi, Some(0));
    assert_eq!(a.batters[0]["away3"].classified_rbi, Some(2));
    assert_eq!(a.batters[0]["away4"].classified_rbi, Some(1));
    assert_eq!(p.earned_runs, Some(0));
}

#[test]
fn two_out_muff_er_zero_has_literal_observed_masks_halves_and_order_frames() {
    let mut frames = 0;
    for half in [Half::Top, Half::Bottom] {
        for mask in 0u8..8 {
            for reverse_moves in [false, true] {
                for reverse_credits in [false, true] {
                    let (log, index) = two_out_muff_er_half(
                        mask,
                        half,
                        reverse_moves,
                        reverse_credits,
                        false,
                        false,
                        2,
                    );
                    for t in &log {
                        assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
                    }
                    let a = derive_accounting(&log).unwrap();
                    let p =
                        &a.pitchers[1 - half.offense()][&log[index].event().participants.pitcher];
                    assert_eq!(p.runs_allowed, mask.count_ones() + 2);
                    assert_eq!(
                        (
                            p.outs_recorded,
                            p.batters_faced,
                            p.at_bats_against,
                            p.classified_hits_allowed,
                            p.credited_strikeouts
                        ),
                        (3, mask.count_ones() + 5, Some(5), Some(1), 3)
                    );
                    assert_eq!(
                        a.classified_fielding[1 - half.offense()]
                            .values()
                            .next()
                            .unwrap()
                            .observed_events,
                        log.len() as u32 - u32::from(half == Half::Bottom) * 3
                    );
                    assert_eq!(
                        p.earned_runs,
                        Some(0),
                        "{half:?} mask{mask} moves{reverse_moves} credits{reverse_credits}"
                    );
                    frames += 1;
                }
            }
        }
    }
    assert_eq!(frames, 64);
}

#[test]
fn two_out_muff_er_missing_review_origin_completion_or_bounded_shape_stays_unknown() {
    let (control, index) = two_out_muff_er_half(0, Half::Top, false, false, true, false, 2);
    for defect in 0..7 {
        let mut log = control.clone();
        let target = match defect {
            0 | 1 => index,
            2 => 0,
            3 | 6 => index + 1,
            _ => 0,
        };
        if defect == 4 {
            log = log[1..].to_vec();
        } else if defect == 5 {
            log.pop();
        } else {
            let mut e = log[target].event().clone();
            e.fielding_review = FieldingReview::Unknown;
            if defect == 0 {
                e.muffed_ground_ball = None;
            }
            if defect == 6 {
                e.delta.credits.push(credit("home3", CreditKind::Error));
            }
            let t = apply(log[target].before(), e).unwrap();
            assert_eq!(t.before(), log[target].before());
            assert_eq!(t.after(), log[target].after());
            log[target] = t;
        }
        for t in &log {
            assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
        }
        let a = derive_accounting(&log).unwrap();
        assert_eq!(a.pitchers[1]["home_starter"].runs_allowed, 3);
        assert_eq!(
            a.pitchers[1]["home_starter"].earned_runs, None,
            "defect{defect}"
        );
    }
    let (scored, _) = two_out_muff_er_half(0, Half::Top, false, false, true, true, 2);
    let a = derive_accounting(&scored).unwrap();
    assert_eq!(a.pitchers[1]["home_starter"].runs_allowed, 4);
    assert_eq!(a.pitchers[1]["home_starter"].earned_runs, None);
    for outs in 0..2 {
        let (log, _) = two_out_muff_er_half(0, Half::Top, false, false, true, false, outs);
        let a = derive_accounting(&log).unwrap();
        assert_eq!(a.pitchers[1]["home_starter"].outs_recorded, 3);
        assert_eq!(a.pitchers[1]["home_starter"].runs_allowed, 3);
        // Separately preregistered empty-base reconstruction: the error runner
        // is unearned, but both HR batters score before virtual inning end.
        assert_eq!(a.pitchers[1]["home_starter"].earned_runs, Some(2));
    }
    // Rule9.16(i): a reliever cannot inherit the predecessor's missed out.
    // Two inherited scorers remain charged to the old pitcher; the HR batter
    // belongs to the incoming pitcher. This bounded proof covers neither ER.
    let (model, index) = two_out_muff_er_half(1, Half::Top, false, false, false, false, 2);
    let mut log = model[..=index].to_vec();
    let mut before = log.last().unwrap().after().clone();
    let substitution = reviewed(event(
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
    ));
    let t = apply(&before, substitution).unwrap();
    before = t.after().clone();
    log.push(t);
    for t in &model[index + 1..] {
        let mut e = t.event().clone();
        e.participants.pitcher = "home_reliever".into();
        for m in &mut e.delta.moves {
            if m.from == Base::Plate {
                m.runner.responsible_pitcher = "home_reliever".into();
            }
        }
        let t = apply(&before, e).unwrap();
        assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
        before = t.after().clone();
        log.push(t);
    }
    let a = derive_accounting(&log).unwrap();
    assert_eq!(a.pitchers[1]["home_starter"].runs_allowed, 2);
    assert_eq!(a.pitchers[1]["home_reliever"].runs_allowed, 1);
    assert_eq!(a.pitchers[1]["home_starter"].earned_runs, None);
    assert_eq!(a.pitchers[1]["home_reliever"].earned_runs, None);
    let mut log = model[..=index].to_vec();
    let mut before = log.last().unwrap().after().clone();
    let swap = reviewed(event(
        &before,
        EventKind::Substitution,
        vec![],
        0,
        0,
        vec![],
        vec![PersonnelChange::BattingSlot {
            team: TeamSide::Home,
            slot: 3,
            outgoing: "home3".into(),
            incoming: "home_reliever".into(),
        }],
    ));
    let t = apply(&before, swap).unwrap();
    before = t.after().clone();
    log.push(t);
    for t in &model[index + 1..] {
        let t = apply(&before, t.event().clone()).unwrap();
        before = t.after().clone();
        log.push(t);
    }
    assert_eq!(
        derive_accounting(&log).unwrap().pitchers[1]["home_starter"].earned_runs,
        None
    );
}

#[test]
fn two_out_muff_er_known_zero_accumulates_but_never_repairs_prior_unknowns() {
    let (model, index) = two_out_muff_er_half(0, Half::Top, false, false, true, false, 2);
    let mut positive = model.clone();
    finish_with_three_strikeouts(
        positive.last().unwrap().after().clone(),
        &mut positive,
        FieldingReview::NoMisplay,
    );
    positive.extend(reviewed_homer_from(
        positive.last().unwrap().after().clone(),
    ));
    let a = derive_accounting(&positive).unwrap();
    assert_eq!(a.pitchers[1]["home_starter"].runs_allowed, 4);
    assert_eq!(a.pitchers[1]["home_starter"].earned_runs, Some(1));
    for unknown_first in [false, true] {
        let mut log = model.clone();
        if unknown_first {
            let mut e = log[index].event().clone();
            e.muffed_ground_ball = None;
            e.fielding_review = FieldingReview::Unknown;
            log[index] = apply(log[index].before(), e).unwrap();
        }
        finish_with_three_strikeouts(
            log.last().unwrap().after().clone(),
            &mut log,
            FieldingReview::NoMisplay,
        );
        let mut before = log.last().unwrap().after().clone();
        let offset = before.teams[0].next_batter as usize;
        let ids: std::collections::BTreeMap<_, _> = model
            .iter()
            .enumerate()
            .map(|(i, t)| {
                (
                    t.event().participants.batter.clone().unwrap(),
                    before.teams[0].batting_order[(offset + i) % 9].clone(),
                )
            })
            .collect();
        for t in &model {
            let mut e = t.event().clone();
            e.participants.batter = Some(ids[e.participants.batter.as_ref().unwrap()].clone());
            for c in &mut e.delta.credits {
                if let Some(id) = ids.get(&c.player_id) {
                    c.player_id = id.clone();
                }
            }
            for m in &mut e.delta.moves {
                if let Some(id) = ids.get(&m.runner.player_id) {
                    m.runner.player_id = id.clone();
                }
            }
            if let Some(f) = &mut e.muffed_ground_ball {
                f.batter_id = ids[&f.batter_id].clone();
            }
            if let Some(f) = &mut e.caught_strikeout {
                f.batter_id = ids[&f.batter_id].clone();
            }
            if let Some(f) = &mut e.sole_home_run {
                f.batter_id = ids[&f.batter_id].clone();
            }
            if let Some(f) = &mut e.delta.third_out {
                f.runner_id = ids[&f.runner_id].clone();
            }
            let t = apply(&before, e).unwrap();
            assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
            before = t.after().clone();
            log.push(t);
        }
        let a = derive_accounting(&log).unwrap();
        assert_eq!(a.pitchers[1]["home_starter"].runs_allowed, 6);
        assert_eq!(
            a.pitchers[1]["home_starter"].earned_runs,
            if unknown_first { None } else { Some(0) }
        );
        assert_eq!(a.pitchers[0]["away_pitcher"].earned_runs, Some(0));
    }
}

#[test]
fn two_out_muff_er_terminal_error_walkoff_keeps_actual_runs_but_no_earned_run() {
    // Existing legal ending plus the preregistered terminal-half exclusion;
    // integration evidence, not a fabricated new regression red.
    let mut before = state();
    before.inning = 1;
    before.outs = 0;
    before.bases = [None, None, None];
    let mut log = vec![];
    for _ in 0..17 {
        finish_with_three_strikeouts(before, &mut log, FieldingReview::NoMisplay);
        before = log.last().unwrap().after().clone();
    }
    assert_eq!(
        (before.inning, before.half, before.score),
        (9, Half::Bottom, [0, 0])
    );
    let runner = before.teams[1].batting_order[before.teams[1].next_batter as usize].clone();
    let walk = event(
        &before,
        EventKind::PlateAppearance,
        vec![batter_move(&before, Destination::First)],
        0,
        0,
        vec![
            credit(&runner, CreditKind::Walk),
            credit(&runner, CreditKind::OrdinaryWalk),
            credit(&runner, CreditKind::NoHit),
        ],
        vec![],
    );
    before = record_clean(&mut log, &before, walk);
    for step in 0..2 {
        let wp = event(
            &before,
            EventKind::RunnerOnly(RunnerEvent::WildPitch),
            vec![RunnerMove {
                runner: before.bases[step].clone().unwrap(),
                from: [Base::First, Base::Second][step],
                to: [Destination::Second, Destination::Third][step],
            }],
            0,
            0,
            vec![],
            vec![],
        );
        before = record_clean(&mut log, &before, wp);
    }
    for _ in 0..2 {
        let batter = before.teams[1].batting_order[before.teams[1].next_batter as usize].clone();
        let out = event(
            &before,
            EventKind::PlateAppearance,
            vec![batter_move(&before, Destination::Out)],
            1,
            0,
            vec![
                credit(&batter, CreditKind::Strikeout),
                credit(&batter, CreditKind::NoHit),
            ],
            vec![],
        );
        before = record_clean(&mut log, &before, out);
    }
    let batter = before.teams[1].batting_order[before.teams[1].next_batter as usize].clone();
    let mut e = event(
        &before,
        EventKind::PlateAppearance,
        vec![
            batter_move(&before, Destination::First),
            RunnerMove {
                runner: before.bases[2].clone().unwrap(),
                from: Base::Third,
                to: Destination::Home,
            },
        ],
        0,
        1,
        vec![
            credit(&batter, CreditKind::AtBat),
            credit(&batter, CreditKind::NoHit),
            credit(&runner, CreditKind::Run),
            credit("away3", CreditKind::Error),
        ],
        vec![],
    );
    e.muffed_ground_ball = Some(bbsim_engine::MuffedGroundBallFacts {
        batter_id: batter,
        fielder_id: "away3".into(),
    });
    e.fielding_review = FieldingReview::SoleGroundMuff;
    let t = apply(&before, e).unwrap();
    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
    assert_eq!(
        (
            t.after().over,
            t.after().inning,
            t.after().half,
            t.after().outs,
            t.after().score
        ),
        (true, 9, Half::Bottom, 2, [0, 1])
    );
    log.push(t);
    let a = derive_accounting(&log).unwrap();
    assert!(a.complete_game);
    assert_eq!(log.len(), 57);
    assert_eq!(a.runs, [0, 1]);
    assert_eq!(a.innings[&9], [Some(0), Some(1)]);
    assert_eq!(
        (
            a.pitchers[0]["away_pitcher"].runs_allowed,
            a.pitchers[0]["away_pitcher"].outs_recorded,
            a.pitchers[0]["away_pitcher"].earned_runs
        ),
        (1, 26, Some(0))
    );
}

fn empty_muff_er_ordered_half(
    half: Half,
    outs: u8,
    pattern: usize,
    reverse_moves: bool,
    reverse_credits: bool,
) -> Vec<EventTransition> {
    let (model, index) =
        two_out_muff_er_half(0, half, reverse_moves, reverse_credits, true, false, outs);
    let mut log = model[..=index].to_vec();
    let mut before = log.last().unwrap().after().clone();
    let orders: [&[bool]; 4] = [
        &[true, true],
        &[false, true],
        &[true, false, true],
        &[true; 9],
    ];
    for homer in orders[pattern].iter().copied().chain([false; 3]) {
        let offense = half.offense();
        let defense = 1 - offense;
        let batter =
            before.teams[offense].batting_order[before.teams[offense].next_batter as usize].clone();
        let mut moves = vec![batter_move(
            &before,
            if homer {
                Destination::Home
            } else {
                Destination::Out
            },
        )];
        let mut credits = vec![];
        let mut runs = u8::from(homer);
        for (i, r) in before.bases.iter().enumerate() {
            if let Some(r) = r {
                moves.push(RunnerMove {
                    runner: r.clone(),
                    from: [Base::First, Base::Second, Base::Third][i],
                    to: if homer {
                        Destination::Home
                    } else {
                        [Destination::First, Destination::Second, Destination::Third][i]
                    },
                });
                if homer {
                    runs += 1;
                    credits.push(credit(&r.player_id, CreditKind::Run));
                    credits.push(credit(&batter, CreditKind::Rbi));
                }
            }
        }
        let mut e = if homer {
            credits.extend([
                credit(&batter, CreditKind::PlateAppearance),
                credit(&batter, CreditKind::Hit),
                credit(&batter, CreditKind::Run),
                credit(&batter, CreditKind::Rbi),
            ]);
            let mut e = reviewed(event(
                &before,
                EventKind::OutOfParkHomeRun,
                moves,
                0,
                runs,
                credits,
                vec![],
            ));
            e.sole_home_run = Some(bbsim_engine::SoleHomeRunFacts {
                batter_id: batter.clone(),
            });
            e
        } else {
            let catcher = before.teams[defense].batting_order[0].clone();
            credits.extend([
                credit(&batter, CreditKind::Strikeout),
                credit(&batter, CreditKind::NoHit),
                credit(&catcher, CreditKind::Putout),
            ]);
            let mut e = reviewed(event(
                &before,
                EventKind::PlateAppearance,
                moves,
                1,
                0,
                credits,
                vec![],
            ));
            e.caught_strikeout = Some(bbsim_engine::CaughtStrikeoutFacts {
                batter_id: batter.clone(),
                catcher_id: catcher,
            });
            if before.outs == 2 {
                e.delta.third_out = Some(ThirdOut {
                    runner_id: batter,
                    kind: ThirdOutKind::BatterBeforeFirst,
                    home_before_out: vec![],
                });
            }
            e
        };
        if reverse_moves {
            e.delta.moves.reverse();
        }
        if reverse_credits {
            e.delta.credits.reverse();
        }
        let t = apply(&before, e).unwrap();
        before = t.after().clone();
        log.push(t);
        if before.half != half {
            break;
        }
    }
    assert_ne!(before.half, half);
    log
}

#[test]
fn empty_muff_er_has_literal_earned_and_unearned_runs_in_48_order_frames() {
    let mut frames = 0;
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..2 {
            for pattern in 0..3 {
                for rm in [false, true] {
                    for rc in [false, true] {
                        let log = empty_muff_er_ordered_half(half, outs, pattern, rm, rc);
                        for t in &log {
                            assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
                            assert_eq!(
                                EventTransition::try_new(
                                    t.before().clone(),
                                    t.event().clone(),
                                    t.after().clone()
                                )
                                .unwrap(),
                                *t
                            );
                        }
                        let a = derive_accounting(&log).unwrap();
                        let defense = 1 - half.offense();
                        let pitcher = &log.last().unwrap().event().participants.pitcher;
                        let p = &a.pitchers[defense][pitcher];
                        let homers = if pattern == 1 { 1 } else { 2 };
                        assert_eq!(
                            (
                                p.runs_allowed,
                                p.outs_recorded,
                                p.batters_faced,
                                p.at_bats_against,
                                p.classified_hits_allowed,
                                p.credited_strikeouts
                            ),
                            (1 + homers, 3, 4 + homers, Some(4 + homers), Some(homers), 3)
                        );
                        assert_eq!(
                            a.classified_fielding[defense]
                                .values()
                                .map(|l| l.errors.unwrap())
                                .sum::<u32>(),
                            1
                        );
                        assert_eq!(
                            a.classified_fielding[defense]
                                .values()
                                .map(|l| l.putouts.unwrap())
                                .sum::<u32>(),
                            3
                        );
                        for l in a.classified_fielding[defense].values() {
                            assert_eq!(l.observed_events, 4 + homers);
                            assert_eq!(l.assists, Some(0));
                            assert_eq!(l.passed_balls, Some(0));
                        }
                        let expected = match pattern {
                            0 => 2,
                            1 => u32::from(outs == 0),
                            _ => 2 - u32::from(outs == 1),
                        };
                        assert_eq!(
                            p.earned_runs,
                            Some(expected),
                            "{half:?} outs{outs} pattern{pattern} moves{rm} credits{rc}"
                        );
                        frames += 1;
                    }
                }
            }
        }
    }
    assert_eq!(frames, 48);
}

#[test]
fn empty_muff_er_missing_or_outside_history_retains_unknown() {
    let control = empty_muff_er_ordered_half(Half::Top, 1, 2, false, false);
    let index = control
        .iter()
        .position(|t| t.event().muffed_ground_ball.is_some())
        .unwrap();
    for defect in 0..9 {
        let mut log = control.clone();
        if defect == 6 {
            log.remove(0);
        } else if defect == 7 {
            log.pop();
        } else {
            let target = match defect {
                0 | 1 => index,
                2 => 0,
                4 => index + 2,
                _ => index + 1,
            };
            let mut e = log[target].event().clone();
            match defect {
                0 => {
                    e.muffed_ground_ball = None;
                    e.fielding_review = FieldingReview::Unknown;
                }
                4 => e.caught_strikeout = None,
                5 => e.sole_home_run = None,
                8 => {
                    e.fielding_review = FieldingReview::Unknown;
                    e.delta.credits.push(credit("home3", CreditKind::Error));
                }
                _ => e.fielding_review = FieldingReview::Unknown,
            }
            let t = apply(log[target].before(), e).unwrap();
            assert_eq!(t.after(), log[target].after());
            log[target] = t;
        }
        for t in &log {
            assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
        }
        let a = derive_accounting(&log).unwrap();
        assert_eq!(a.pitchers[1]["home_starter"].runs_allowed, 3);
        assert_eq!(
            a.pitchers[1]["home_starter"].earned_runs, None,
            "defect{defect}"
        );
    }
    for outs in 0..2 {
        let (log, _) = two_out_muff_er_half(1, Half::Top, false, false, true, false, outs);
        let a = derive_accounting(&log).unwrap();
        assert_eq!(a.pitchers[1]["home_starter"].runs_allowed, 4);
        assert_eq!(a.pitchers[1]["home_starter"].earned_runs, None);
    }
    let (scored, _) = two_out_muff_er_half(0, Half::Top, false, false, true, true, 1);
    assert_eq!(
        derive_accounting(&scored).unwrap().pitchers[1]["home_starter"].earned_runs,
        None
    );
    for pitcher_change in [false, true] {
        let mut log = control[..=index].to_vec();
        let mut before = log.last().unwrap().after().clone();
        let change = if pitcher_change {
            PersonnelChange::Pitcher {
                team: TeamSide::Home,
                outgoing: "home_starter".into(),
                incoming: "home_reliever".into(),
            }
        } else {
            PersonnelChange::BattingSlot {
                team: TeamSide::Home,
                slot: 3,
                outgoing: "home3".into(),
                incoming: "home_reliever".into(),
            }
        };
        let t = apply(
            &before,
            reviewed(event(
                &before,
                EventKind::Substitution,
                vec![],
                0,
                0,
                vec![],
                vec![change],
            )),
        )
        .unwrap();
        before = t.after().clone();
        log.push(t);
        for t in &control[index + 1..] {
            let mut e = t.event().clone();
            if pitcher_change {
                e.participants.pitcher = "home_reliever".into();
                for m in &mut e.delta.moves {
                    if m.from == Base::Plate {
                        m.runner.responsible_pitcher = "home_reliever".into();
                    }
                }
            }
            let t = apply(&before, e).unwrap();
            before = t.after().clone();
            log.push(t);
        }
        let a = derive_accounting(&log).unwrap();
        assert_eq!(a.pitchers[1]["home_starter"].earned_runs, None);
        if pitcher_change {
            assert_eq!(a.pitchers[1]["home_starter"].runs_allowed, 1);
            assert_eq!(a.pitchers[1]["home_reliever"].runs_allowed, 2);
            assert_eq!(a.pitchers[1]["home_reliever"].earned_runs, None);
        }
    }
    // A reviewed WP is clean in the old vocabulary, but lies outside this
    // deliberately HR/caught-K-only suffix and cannot silently widen it.
    let mut log = control[..=index].to_vec();
    let mut before = log.last().unwrap().after().clone();
    let wp = event(
        &before,
        EventKind::RunnerOnly(RunnerEvent::WildPitch),
        vec![RunnerMove {
            runner: before.bases[0].clone().unwrap(),
            from: Base::First,
            to: Destination::Second,
        }],
        0,
        0,
        vec![],
        vec![],
    );
    before = record_clean(&mut log, &before, wp);
    for (n, t) in control[index + 1..].iter().enumerate() {
        let mut e = t.event().clone();
        if n == 0 {
            for m in &mut e.delta.moves {
                if m.from == Base::First {
                    m.from = Base::Second;
                }
            }
        }
        let t = apply(&before, e).unwrap();
        before = t.after().clone();
        log.push(t);
    }
    let a = derive_accounting(&log).unwrap();
    assert_eq!(a.pitchers[1]["home_starter"].runs_allowed, 3);
    assert_eq!(a.pitchers[1]["home_starter"].earned_runs, None);
}

fn append_rebased_muff_er_half(log: &mut Vec<EventTransition>, model: &[EventTransition]) {
    let mut before = log.last().unwrap().after().clone();
    let offense = before.half.offense();
    let offset = before.teams[offense].next_batter as usize;
    let ids: std::collections::BTreeMap<_, _> = model
        .iter()
        .enumerate()
        .map(|(i, t)| {
            (
                t.event().participants.batter.clone().unwrap(),
                before.teams[offense].batting_order[(offset + i) % 9].clone(),
            )
        })
        .collect();
    for t in model {
        let mut e = t.event().clone();
        e.participants.batter = Some(ids[e.participants.batter.as_ref().unwrap()].clone());
        for c in &mut e.delta.credits {
            if let Some(id) = ids.get(&c.player_id) {
                c.player_id = id.clone();
            }
        }
        for m in &mut e.delta.moves {
            if let Some(id) = ids.get(&m.runner.player_id) {
                m.runner.player_id = id.clone();
            }
        }
        if let Some(f) = &mut e.muffed_ground_ball {
            f.batter_id = ids[&f.batter_id].clone();
        }
        if let Some(f) = &mut e.caught_strikeout {
            f.batter_id = ids[&f.batter_id].clone();
        }
        if let Some(f) = &mut e.sole_home_run {
            f.batter_id = ids[&f.batter_id].clone();
        }
        if let Some(f) = &mut e.delta.third_out {
            f.runner_id = ids[&f.runner_id].clone();
        }
        let t = apply(&before, e).unwrap();
        assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
        before = t.after().clone();
        log.push(t);
    }
}

#[test]
fn empty_muff_er_contributions_accumulate_without_repairing_prior_unknowns() {
    let model = empty_muff_er_ordered_half(Half::Top, 0, 0, false, false);
    let mut positive = model.clone();
    finish_with_three_strikeouts(
        positive.last().unwrap().after().clone(),
        &mut positive,
        FieldingReview::NoMisplay,
    );
    positive.extend(reviewed_homer_from(
        positive.last().unwrap().after().clone(),
    ));
    let a = derive_accounting(&positive).unwrap();
    assert_eq!(a.pitchers[1]["home_starter"].runs_allowed, 4);
    assert_eq!(a.pitchers[1]["home_starter"].earned_runs, Some(3));
    for unknown_first in [false, true] {
        let mut log = model.clone();
        if unknown_first {
            let mut e = log[0].event().clone();
            e.muffed_ground_ball = None;
            e.fielding_review = FieldingReview::Unknown;
            log[0] = apply(log[0].before(), e).unwrap();
        }
        finish_with_three_strikeouts(
            log.last().unwrap().after().clone(),
            &mut log,
            FieldingReview::NoMisplay,
        );
        append_rebased_muff_er_half(&mut log, &model);
        let a = derive_accounting(&log).unwrap();
        assert_eq!(a.pitchers[1]["home_starter"].runs_allowed, 6);
        assert_eq!(
            a.pitchers[1]["home_starter"].earned_runs,
            if unknown_first { None } else { Some(4) }
        );
        assert_eq!(a.pitchers[0]["away_pitcher"].earned_runs, Some(0));
    }
}

#[test]
fn empty_muff_er_terminal_homer_has_one_earned_and_one_unearned_run() {
    for outs in 0..2 {
        let mut before = state();
        before.inning = 1;
        before.outs = 0;
        before.bases = [None, None, None];
        let mut log = vec![];
        for _ in 0..17 {
            finish_with_three_strikeouts(before, &mut log, FieldingReview::NoMisplay);
            before = log.last().unwrap().after().clone();
        }
        let (model, index) = two_out_muff_er_half(0, Half::Bottom, false, false, true, false, outs);
        append_rebased_muff_er_half(&mut log, &model[3..=index + 1]);
        let t = log.last().unwrap();
        assert_eq!(
            (
                t.after().over,
                t.after().inning,
                t.after().half,
                t.after().outs,
                t.after().score
            ),
            (true, 9, Half::Bottom, outs, [0, 2])
        );
        let a = derive_accounting(&log).unwrap();
        assert!(a.complete_game);
        assert_eq!(a.runs, [0, 2]);
        assert_eq!(
            (
                a.pitchers[0]["away_pitcher"].runs_allowed,
                a.pitchers[0]["away_pitcher"].outs_recorded,
                a.pitchers[0]["away_pitcher"].earned_runs
            ),
            (2, 24 + u32::from(outs), Some(1))
        );
    }
}

#[test]
fn empty_muff_er_error_reach_is_not_a_permanent_unearned_actor_label() {
    let log = empty_muff_er_ordered_half(Half::Top, 0, 3, false, false);
    let a = derive_accounting(&log).unwrap();
    let p = &a.pitchers[1]["home_starter"];
    assert_eq!(log.len(), 13);
    for t in &log {
        assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
    }
    assert_eq!(
        (
            p.runs_allowed,
            p.outs_recorded,
            p.batters_faced,
            p.at_bats_against,
            p.classified_hits_allowed,
            p.credited_strikeouts
        ),
        (10, 3, 13, Some(13), Some(9), 3)
    );
    assert_eq!(a.batters[0]["away0"].credited_runs, 2);
    assert_eq!(a.batters[0]["away0"].classified_hits, Some(1));
    assert_eq!(log[9].event().participants.batter.as_deref(), Some("away0"));
    assert_eq!(log[9].event().kind, EventKind::OutOfParkHomeRun);
    assert_eq!(log[9].event().delta.moves[0].from, Base::Plate);
    assert_eq!(p.earned_runs, Some(9));
}
