use std::collections::BTreeMap;

use bbsim_engine::{
    accounting::{derive_accounting, FieldingContribution, FieldingCoverageLine},
    apply, Base, CaughtStrikeoutFacts, Credit, CreditKind, Delta, Destination, Event, EventKind,
    EventTransition, FieldingReview, GameState, Half, OrdinaryGroundThrowFacts, Participants,
    PersonnelChange, ProtectedAwardFacts, Runner, RunnerEvent, RunnerMove, SoleHomeRunFacts,
    SoleSafeHitFacts, TeamSide, TeamState, ThirdOut, ThirdOutKind,
};

#[derive(Clone, Copy, Debug)]
enum Shape {
    Ground,
    Caught,
    Single,
    Award,
    HomeRun,
}

fn state(mask: u8, outs: u8, half: Half, dh: bool) -> GameState {
    let team = |prefix: &str| {
        let pitcher = format!("{prefix}p");
        let mut batting_order = std::array::from_fn(|i| format!("{prefix}{i}"));
        if !dh {
            batting_order[8] = pitcher.clone();
        }
        TeamState {
            batting_order,
            next_batter: 0,
            pitcher,
            dh_slot: dh.then_some(8),
            dh_must_bat: false,
            bench: vec![format!("{prefix}x"), format!("{prefix}q")],
            removed: vec![],
        }
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
fn event(before: &GameState, shape: Shape) -> Event {
    let offense = &before.teams[before.half.offense()];
    let defense = &before.teams[1 - before.half.offense()];
    let batter = offense.batting_order[offense.next_batter as usize].clone();
    let retirement = matches!(shape, Shape::Ground | Shape::Caught);
    let target = match shape {
        Shape::Ground | Shape::Caught => Destination::Out,
        Shape::Single | Shape::Award => Destination::First,
        Shape::HomeRun => Destination::Home,
    };
    let mut moves = vec![RunnerMove {
        runner: Runner {
            player_id: batter.clone(),
            responsible_pitcher: defense.pitcher.clone(),
        },
        from: Base::Plate,
        to: target,
    }];
    let mut credits = vec![credit(&batter, CreditKind::PlateAppearance)];
    match shape {
        Shape::Ground => credits.extend([
            credit(&batter, CreditKind::AtBat),
            credit(&batter, CreditKind::NoHit),
            credit(&defense.batting_order[1], CreditKind::Putout),
            credit(&defense.batting_order[3], CreditKind::Assist),
        ]),
        Shape::Caught => credits.extend([
            credit(&batter, CreditKind::AtBat),
            credit(&batter, CreditKind::NoHit),
            credit(&batter, CreditKind::Strikeout),
            credit(&defense.batting_order[0], CreditKind::Putout),
        ]),
        Shape::Single => credits.extend([
            credit(&batter, CreditKind::Hit),
            credit(&batter, CreditKind::Single),
        ]),
        Shape::Award => credits.extend([
            credit(&batter, CreditKind::NoHit),
            credit(&batter, CreditKind::HitByPitch),
        ]),
        Shape::HomeRun => credits.push(credit(&batter, CreditKind::Hit)),
    }
    let destinations = [
        Destination::First,
        Destination::Second,
        Destination::Third,
        Destination::Home,
    ];
    let mut forced = true;
    for (i, from) in [Base::First, Base::Second, Base::Third]
        .into_iter()
        .enumerate()
    {
        if let Some(runner) = &before.bases[i] {
            let to = match shape {
                Shape::Ground | Shape::Caught => destinations[i],
                Shape::Single => destinations[i + 1],
                Shape::Award => {
                    if forced {
                        destinations[i + 1]
                    } else {
                        destinations[i]
                    }
                }
                Shape::HomeRun => Destination::Home,
            };
            moves.push(RunnerMove {
                runner: runner.clone(),
                from,
                to,
            });
        } else {
            forced = false;
        }
    }
    let mut runs = 0;
    for movement in &moves {
        if movement.to == Destination::Home {
            runs += 1;
            credits.extend([
                credit(&movement.runner.player_id, CreditKind::Run),
                credit(&batter, CreditKind::Rbi),
            ]);
        }
    }
    Event {
        kind: if matches!(shape, Shape::HomeRun) {
            EventKind::OutOfParkHomeRun
        } else {
            EventKind::PlateAppearance
        },
        participants: Participants {
            batter: Some(batter.clone()),
            pitcher: defense.pitcher.clone(),
        },
        delta: Delta {
            moves,
            outs: u8::from(retirement),
            runs,
            credits,
            changes: vec![],
            third_out: (retirement && before.outs == 2).then(|| ThirdOut {
                runner_id: batter.clone(),
                kind: ThirdOutKind::BatterBeforeFirst,
                home_before_out: vec![],
            }),
        },
        fielding_review: FieldingReview::NoMisplay,
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        ordinary_ground_throw: matches!(shape, Shape::Ground).then(|| OrdinaryGroundThrowFacts {
            batter_id: batter.clone(),
            thrower_id: defense.batting_order[3].clone(),
            receiver_id: defense.batting_order[1].clone(),
        }),
        caught_strikeout: matches!(shape, Shape::Caught).then(|| CaughtStrikeoutFacts {
            batter_id: batter.clone(),
            catcher_id: defense.batting_order[0].clone(),
        }),
        protected_award: matches!(shape, Shape::Award).then(|| ProtectedAwardFacts {
            batter_id: batter.clone(),
        }),
        sole_home_run: matches!(shape, Shape::HomeRun).then(|| SoleHomeRunFacts {
            batter_id: batter.clone(),
        }),
        sole_safe_hit: matches!(shape, Shape::Single)
            .then_some(SoleSafeHitFacts { batter_id: batter }),
    }
}
fn checked(before: &GameState, event: Event) -> EventTransition {
    let frozen = before.clone();
    let t = apply(before, event.clone()).expect("independently built legal fixture");
    assert_eq!(*before, frozen);
    assert_eq!(
        EventTransition::try_new(before.clone(), event, t.after().clone()),
        Ok(t.clone())
    );
    assert_eq!(EventTransition::from_bytes(&t.to_bytes()), Ok(t.clone()));
    t
}
fn contribution(prefix: &str, shape: Shape) -> FieldingContribution {
    match shape {
        Shape::Ground => FieldingContribution {
            putouts: [(format!("{prefix}1"), 1)].into(),
            assists: [(format!("{prefix}3"), 1)].into(),
        },
        Shape::Caught => FieldingContribution {
            putouts: [(format!("{prefix}0"), 1)].into(),
            assists: BTreeMap::new(),
        },
        _ => FieldingContribution {
            putouts: BTreeMap::new(),
            assists: BTreeMap::new(),
        },
    }
}
fn covered(prefix: &str, shape: Shape, count: u32) -> BTreeMap<String, FieldingCoverageLine> {
    // Literal fixture defense: 0..7 plus p, never the DH8/bench/old charge pitcher.
    (0..8)
        .map(|i| format!("{prefix}{i}"))
        .chain([format!("{prefix}p")])
        .map(|id| {
            let po = if (matches!(shape, Shape::Ground) && id == format!("{prefix}1"))
                || (matches!(shape, Shape::Caught) && id == format!("{prefix}0"))
            {
                count
            } else {
                0
            };
            let assists = if matches!(shape, Shape::Ground) && id == format!("{prefix}3") {
                count
            } else {
                0
            };
            (
                id,
                FieldingCoverageLine {
                    observed_events: count,
                    putouts: Some(po),
                    assists: Some(assists),
                },
            )
        })
        .collect()
}

#[test]
fn five_proofs_fold_literal_before_defense_in_1920_independent_frames() {
    let mut frames = 0;
    for shape in [
        Shape::Ground,
        Shape::Caught,
        Shape::Single,
        Shape::Award,
        Shape::HomeRun,
    ] {
        for mask in 0..8 {
            for outs in 0..3 {
                for half in [Half::Top, Half::Bottom] {
                    for dh in [false, true] {
                        let before = state(mask, outs, half, dh);
                        let prefix = if half == Half::Top { "h" } else { "a" };
                        for reverse_moves in [false, true] {
                            for reverse_credits in [false, true] {
                                let mut e = event(&before, shape);
                                if reverse_moves {
                                    e.delta.moves.reverse();
                                }
                                if reverse_credits {
                                    e.delta.credits.reverse();
                                }
                                let t = checked(&before, e);
                                assert_eq!(
                                    t.fielding_contribution(),
                                    Some(contribution(prefix, shape))
                                );
                                let lines = derive_accounting(&[t]).unwrap();
                                assert_eq!(
                                    lines.classified_fielding[1 - half.offense()],
                                    covered(prefix, shape, 1)
                                );
                                assert!(lines.classified_fielding[half.offense()].is_empty());
                                let credited_count = match shape {
                                    Shape::Ground => 2,
                                    Shape::Caught => 1,
                                    _ => 0,
                                };
                                assert_eq!(
                                    lines.fielders[1 - half.offense()].len(),
                                    credited_count
                                );
                                assert!(lines.fielders[half.offense()].is_empty());
                                frames += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(frames, 1920);
}

#[test]
fn missing_facts_or_review_taint_all_observed_defenders_not_credited_maps() {
    let before = state(0, 0, Half::Top, true);
    for missing_fact in [false, true] {
        let mut e = event(&before, Shape::Ground);
        if missing_fact {
            e.ordinary_ground_throw = None;
        } else {
            e.fielding_review = FieldingReview::Unknown;
        }
        let t = checked(&before, e);
        assert_eq!(t.fielding_contribution(), None);
        let lines = derive_accounting(&[t]).unwrap();
        assert_eq!(lines.classified_fielding[1].len(), 9);
        for line in lines.classified_fielding[1].values() {
            assert_eq!(
                *line,
                FieldingCoverageLine {
                    observed_events: 1,
                    putouts: None,
                    assists: None
                }
            );
        }
        assert_eq!(lines.fielders[1].len(), 2);
        assert_eq!(lines.fielders[1]["h1"].credited_putouts, 1);
        assert_eq!(lines.fielders[1]["h3"].credited_assists, 1);
    }
}

#[test]
fn unknown_is_irreversible_and_boundary_uses_before_defense() {
    for unknown_index in [None, Some(0), Some(1), Some(2)] {
        let mut before = state(0, 0, Half::Top, false);
        let mut log = vec![];
        for index in 0..3 {
            let mut e = event(&before, Shape::Ground);
            if unknown_index == Some(index) {
                e.ordinary_ground_throw = None;
            }
            let t = checked(&before, e);
            before = t.after().clone();
            log.push(t);
        }
        assert_eq!(before.half, Half::Bottom);
        let lines = derive_accounting(&log).unwrap();
        assert!(lines.classified_fielding[0].is_empty());
        if unknown_index.is_none() {
            assert_eq!(lines.classified_fielding[1], covered("h", Shape::Ground, 3));
        } else {
            for row in lines.classified_fielding[1].values() {
                assert_eq!(row.observed_events, 3);
                assert_eq!(row.putouts, None);
                assert_eq!(row.assists, None);
            }
        }
        let t = checked(&before, event(&before, Shape::Caught));
        log.push(t);
        let extended = derive_accounting(&log).unwrap();
        assert_eq!(
            extended.classified_fielding[1],
            lines.classified_fielding[1]
        );
        assert_eq!(
            extended.classified_fielding[0],
            covered("a", Shape::Caught, 1)
        );
    }
}

fn substitution(before: &GameState) -> Event {
    let mut e = event(before, Shape::Ground);
    e.kind = EventKind::Substitution;
    e.ordinary_ground_throw = None;
    e.delta = Delta {
        moves: vec![],
        outs: 0,
        runs: 0,
        credits: vec![],
        third_out: None,
        changes: vec![
            PersonnelChange::BattingSlot {
                team: TeamSide::Home,
                slot: 3,
                outgoing: "h3".into(),
                incoming: "hx".into(),
            },
            PersonnelChange::Pitcher {
                team: TeamSide::Home,
                outgoing: "hp".into(),
                incoming: "hq".into(),
            },
        ],
    };
    e
}
fn runner_only(before: &GameState) -> Event {
    let mut e = event(before, Shape::Ground);
    e.kind = EventKind::RunnerOnly(RunnerEvent::Pickoff);
    e.ordinary_ground_throw = None;
    e.delta = Delta {
        moves: vec![RunnerMove {
            runner: before.bases[0].as_ref().unwrap().clone(),
            from: Base::First,
            to: Destination::First,
        }],
        outs: 0,
        runs: 0,
        credits: vec![],
        changes: vec![],
        third_out: None,
    };
    e
}

#[test]
fn runner_only_taints_but_personnel_does_not_taint_incoming_actors() {
    let before = state(1, 0, Half::Top, true);
    let first = checked(&before, event(&before, Shape::Ground));
    let runner = checked(first.after(), runner_only(first.after()));
    assert_eq!(runner.fielding_contribution(), None);
    let replace = checked(runner.after(), substitution(runner.after()));
    assert_eq!(replace.fielding_contribution(), None);
    let last = checked(replace.after(), event(replace.after(), Shape::Ground));
    let lines = derive_accounting(&[first, runner, replace, last]).unwrap();
    assert_eq!(
        lines.classified_fielding[1]["h3"],
        FieldingCoverageLine {
            observed_events: 2,
            putouts: None,
            assists: None
        }
    );
    assert_eq!(
        lines.classified_fielding[1]["hp"],
        FieldingCoverageLine {
            observed_events: 2,
            putouts: None,
            assists: None
        }
    );
    assert_eq!(
        lines.classified_fielding[1]["hx"],
        FieldingCoverageLine {
            observed_events: 1,
            putouts: Some(0),
            assists: Some(1)
        }
    );
    assert_eq!(
        lines.classified_fielding[1]["hq"],
        FieldingCoverageLine {
            observed_events: 1,
            putouts: Some(0),
            assists: Some(0)
        }
    );
    assert_eq!(
        lines.classified_fielding[1]["h1"],
        FieldingCoverageLine {
            observed_events: 3,
            putouts: None,
            assists: None
        }
    );
    assert!(!lines.classified_fielding[1].contains_key("h8"));
    assert!(!lines.classified_fielding[1].contains_key("oldp"));
    assert!(lines.classified_fielding[0].is_empty());
    assert_eq!(lines.fielders[1].len(), 3);
    assert_eq!(lines.fielders[1]["h1"].credited_putouts, 2);
}

#[test]
fn substitution_only_has_no_observed_fielders() {
    let before = state(0, 0, Half::Top, true);
    let t = checked(&before, substitution(&before));
    assert_eq!(t.fielding_contribution(), None);
    let lines = derive_accounting(&[t]).unwrap();
    assert!(lines.classified_fielding.iter().all(BTreeMap::is_empty));
    assert!(lines.fielders.iter().all(BTreeMap::is_empty));
}

#[test]
fn irreversible_dh_loss_does_not_double_count_the_already_observed_pitcher() {
    let before = state(0, 0, Half::Top, true);
    let first = checked(&before, event(&before, Shape::Ground));
    let mut loss = substitution(first.after());
    loss.delta.changes = vec![PersonnelChange::BattingSlot {
        team: TeamSide::Home,
        slot: 8,
        outgoing: "h8".into(),
        incoming: "hp".into(),
    }];
    let change = checked(first.after(), loss);
    assert_eq!(change.after().teams[1].dh_slot, None);
    assert_eq!(change.after().teams[1].batting_order[8], "hp");
    let last = checked(change.after(), event(change.after(), Shape::Ground));
    let lines = derive_accounting(&[first, change, last]).unwrap();
    assert_eq!(lines.classified_fielding[1], covered("h", Shape::Ground, 2));
    assert_eq!(lines.classified_fielding[1]["hp"].observed_events, 2);
    assert!(!lines.classified_fielding[1].contains_key("h8"));
    assert_eq!(lines.fielders[1].len(), 2);
}

#[test]
fn inherited_scoring_charges_are_not_actual_defensive_opportunities() {
    let before = state(7, 0, Half::Bottom, false);
    let t = checked(&before, event(&before, Shape::HomeRun));
    let lines = derive_accounting(&[t]).unwrap();
    assert_eq!(
        lines.classified_fielding[0],
        covered("a", Shape::HomeRun, 1)
    );
    assert_eq!(lines.pitchers[0]["oldp"].runs_allowed, 3);
    assert_eq!(lines.pitchers[0]["ap"].runs_allowed, 1);
    assert!(!lines.classified_fielding[0].contains_key("oldp"));
}

#[test]
fn physical_complete_game_does_not_override_a_missing_fielding_proof() {
    for omit_one in [false, true] {
        let mut before = state(0, 0, Half::Top, true);
        before.inning = 1;
        let mut scored = false;
        let mut log = vec![];
        while !before.over {
            let shape = if !scored && before.half == Half::Bottom {
                scored = true;
                Shape::HomeRun
            } else {
                Shape::Ground
            };
            let mut e = event(&before, shape);
            if omit_one && before.inning == 2 && before.half == Half::Top && before.outs == 0 {
                e.ordinary_ground_throw = None;
            }
            let t = checked(&before, e);
            before = t.after().clone();
            log.push(t);
        }
        assert_eq!(log.len(), 52);
        let lines = derive_accounting(&log).unwrap();
        assert!(lines.complete_game);
        assert_eq!(lines.runs, [0, 1]);
        assert_eq!(lines.classified_fielding[0]["a1"].putouts, Some(24));
        assert_eq!(lines.classified_fielding[0]["a3"].assists, Some(24));
        for row in lines.classified_fielding[0].values() {
            assert_eq!(row.observed_events, 25);
        }
        for (id, row) in &lines.classified_fielding[1] {
            assert_eq!(row.observed_events, 27);
            assert_eq!(
                row.putouts,
                (!omit_one).then_some(if id == "h1" { 27 } else { 0 })
            );
            assert_eq!(
                row.assists,
                (!omit_one).then_some(if id == "h3" { 27 } else { 0 })
            );
        }
        assert_eq!(lines.fielders[1]["h1"].credited_putouts, 27);
        assert_eq!(lines.fielders[1]["h3"].credited_assists, 27);
        assert_eq!(lines.fielders[0]["a1"].credited_putouts, 24);
        assert_eq!(lines.fielders[0]["a3"].credited_assists, 24);
    }
}
