//! Supplied decisions, not generated event frequencies or historical causality.
#[path = "../examples/support/contact_consequence.rs"]
mod contact_consequence;
use bbsim_engine::*;
use contact_consequence::{project, ContactDecision, POLICY_VERSION};

fn state(mask: u8, outs: u8) -> GameState {
    let teams = ["a", "h"].map(|prefix| TeamState {
        batting_order: std::array::from_fn(|i| format!("{prefix}{i}")),
        next_batter: 0,
        pitcher: format!("{prefix}p"),
        dh_slot: Some(8),
        dh_must_bat: false,
        bench: vec![format!("{prefix}bench")],
        removed: vec![format!("{prefix}removed")],
    });
    GameState {
        inning: 2,
        half: Half::Top,
        outs,
        score: [0, 0],
        bases: std::array::from_fn(|i| {
            (mask & (1 << i) != 0).then(|| Runner {
                player_id: format!("a{}", i + 3),
                responsible_pitcher: format!("inherited-{i}"),
            })
        }),
        teams,
        over: false,
    }
}
fn roundtrip(transition: &EventTransition, version: &[u8]) {
    assert_eq!(POLICY_VERSION, "supplied-contact-consequence-projection-v1");
    let bytes = transition.to_bytes();
    assert!(bytes.starts_with(version));
    let decoded = EventTransition::from_bytes(&bytes).unwrap();
    assert_eq!(&decoded, transition);
    assert_eq!(format!("{decoded:?}"), format!("{transition:?}"));
    assert_eq!(format!("{decoded:#?}"), format!("{transition:#?}"));
}

#[test]
fn supplied_caught_sacrifice_fly_constructs_one_canonical_pa() {
    let before = state(4, 0);
    let facts = CaughtSacrificeFlyFacts {
        scoring_runner_id: "a5".into(),
        handling: SacrificeFlyHandling::QualifyingOutfielderOrInfielderInOutfield,
        retouch: SacrificeFlyRetouch::Legal,
        timing: SacrificeFlyRunTiming::AfterCatch,
    };
    let catch = OrdinaryAirCatchFacts {
        batter_id: "a0".into(),
        fielder_id: "h5".into(),
        contact: AirContact::Fly,
    };
    let decision = ContactDecision::CaughtSacrificeFly {
        catch: catch.clone(),
        facts: facts.clone(),
    };
    let saved = decision.clone();
    let transition = project(&before, &decision).unwrap();
    let mut expected = before.clone();
    expected.outs = 1;
    expected.score[0] = 1;
    expected.bases[2] = None;
    expected.teams[0].next_batter = 1;
    assert_eq!(transition.before(), &before);
    assert_eq!(transition.after(), &expected);
    assert_eq!(transition.caught_sacrifice_fly_facts(), Some(&facts));
    assert_eq!(transition.event().ordinary_air_catch, Some(catch));
    assert_eq!(decision, saved);
    roundtrip(&transition, b"BBE28");
}

#[test]
fn supplied_sacrifice_bunt_constructs_one_canonical_pa() {
    let before = state(5, 0);
    let facts = SacrificeBuntThrowFacts {
        batter_id: "a0".into(),
        advancing_runner_id: "a3".into(),
        thrower_id: "h3".into(),
        receiver_id: "h1".into(),
        judgment: SacrificeBuntJudgment::SacrificingOwnChance,
    };
    let decision = ContactDecision::SacrificeBunt(facts.clone());
    let saved = decision.clone();
    let transition = project(&before, &decision).unwrap();
    let mut expected = before.clone();
    expected.outs = 1;
    expected.bases[1] = expected.bases[0].take();
    expected.teams[0].next_batter = 1;
    assert_eq!(transition.before(), &before);
    assert_eq!(transition.after(), &expected);
    assert_eq!(transition.sacrifice_bunt_throw_facts(), Some(&facts));
    assert_eq!(decision, saved);
    roundtrip(&transition, b"BBE29");
}

#[test]
fn supplied_force_relay_constructs_one_canonical_pa_and_half_cleanup() {
    let before = state(5, 1);
    let facts = GroundForceDoublePlayFacts {
        batter_id: "a0".into(),
        forced_runner_id: "a3".into(),
        initial_fielder_id: "h3".into(),
        pivot_id: "h2".into(),
        first_receiver_id: "h1".into(),
    };
    let decision = ContactDecision::GroundForceDoublePlay(facts.clone());
    let saved = decision.clone();
    let transition = project(&before, &decision).unwrap();
    let mut expected = before.clone();
    expected.outs = 0;
    expected.half = Half::Bottom;
    expected.bases = [None, None, None];
    expected.teams[0].next_batter = 1;
    assert_eq!(transition.before(), &before);
    assert_eq!(transition.after(), &expected);
    assert_eq!(transition.ground_force_double_play_facts(), Some(&facts));
    assert_eq!(decision, saved);
    roundtrip(&transition, b"BBE30");
}

fn matrix_state(mask: u8, outs: u8, half: Half, inning: u16, cursor: u8, dh: bool) -> GameState {
    let mut before = state(mask, outs);
    before.half = half;
    before.inning = inning;
    for team in &mut before.teams {
        team.next_batter = cursor;
        team.dh_slot = dh.then_some(8);
        if !dh {
            team.batting_order[7] = team.pitcher.clone();
        }
    }
    before.bases = std::array::from_fn(|i| {
        (mask & (1 << i) != 0).then(|| Runner {
            player_id: before.teams[half.offense()].batting_order
                [(usize::from(cursor) + i + 3) % 9]
                .clone(),
            responsible_pitcher: format!("inherited-{i}"),
        })
    });
    before
}
fn decision(before: &GameState, family: u8, role: u8, contact: AirContact) -> ContactDecision {
    let offense = &before.teams[before.half.offense()];
    let defense = &before.teams[1 - before.half.offense()];
    let batter = offense.batting_order[usize::from(offense.next_batter)].clone();
    match family {
        0 => ContactDecision::CaughtSacrificeFly {
            catch: OrdinaryAirCatchFacts {
                batter_id: batter,
                fielder_id: if role == 2 {
                    defense.pitcher.clone()
                } else {
                    defense.batting_order[5 + usize::from(role)].clone()
                },
                contact,
            },
            facts: CaughtSacrificeFlyFacts {
                scoring_runner_id: before.bases[2].as_ref().unwrap().player_id.clone(),
                handling: SacrificeFlyHandling::QualifyingOutfielderOrInfielderInOutfield,
                retouch: SacrificeFlyRetouch::Legal,
                timing: SacrificeFlyRunTiming::AfterCatch,
            },
        },
        1 => ContactDecision::SacrificeBunt(SacrificeBuntThrowFacts {
            batter_id: batter,
            advancing_runner_id: before.bases[0].as_ref().unwrap().player_id.clone(),
            thrower_id: if role == 1 {
                defense.pitcher.clone()
            } else {
                defense.batting_order[3].clone()
            },
            receiver_id: if role == 2 {
                defense.pitcher.clone()
            } else {
                defense.batting_order[1].clone()
            },
            judgment: SacrificeBuntJudgment::SacrificingOwnChance,
        }),
        2 => ContactDecision::GroundForceDoublePlay(GroundForceDoublePlayFacts {
            batter_id: batter,
            forced_runner_id: before.bases[0].as_ref().unwrap().player_id.clone(),
            initial_fielder_id: if matches!(role, 2 | 5) {
                defense.pitcher.clone()
            } else if role == 1 {
                defense.batting_order[1].clone()
            } else {
                defense.batting_order[3].clone()
            },
            pivot_id: if role == 3 {
                defense.pitcher.clone()
            } else {
                defense.batting_order[2].clone()
            },
            first_receiver_id: if matches!(role, 4 | 5) {
                defense.pitcher.clone()
            } else {
                defense.batting_order[1].clone()
            },
        }),
        _ => panic!("invalid test family"),
    }
}
// Literal oracle: deliberately no canonical apply/ending/projector helper calls.
fn expected(before: &GameState, family: u8) -> Option<GameState> {
    if family == 2
        && before.outs == 1
        && before.half == Half::Top
        && before.inning >= 9
        && before.score[1] > before.score[0]
    {
        return None;
    }
    let mut after = before.clone();
    let offense = before.half.offense();
    after.teams[offense].next_batter = (before.teams[offense].next_batter + 1) % 9;
    after.outs = before.outs + if family == 2 { 2 } else { 1 };
    match family {
        0 => {
            after.score[offense] += 1;
            after.bases[2] = None;
        }
        1 => {
            after.bases[1] = after.bases[0].take();
        }
        2 => {
            after.bases[0] = None;
        }
        _ => panic!("invalid test family"),
    }
    if after.outs == 3 {
        after.outs = 0;
        after.bases = [None, None, None];
        if before.half == Half::Top {
            after.half = Half::Bottom;
        } else {
            after.half = Half::Top;
            after.inning = before.inning.checked_add(1).unwrap();
        }
    }
    Some(after)
}
fn rejected(before: &GameState, choice: &ContactDecision) {
    let saved = before.clone();
    let original = choice.clone();
    assert!(project(before, choice).is_err());
    assert_eq!(*before, saved);
    assert_eq!(*choice, original);
}
fn checked(before: &GameState, choice: &ContactDecision, family: u8) {
    let Some(after) = expected(before, family) else {
        rejected(before, choice);
        return;
    };
    let original = choice.clone();
    let canonical = canonical_fixture(before, choice).unwrap();
    assert_eq!(canonical.after(), &after);
    let transition = project(before, choice).unwrap();
    let normalized = |mut event: Event| {
        event.delta.moves.sort_by_key(|value| format!("{value:?}"));
        event
            .delta
            .credits
            .sort_by_key(|value| format!("{value:?}"));
        event
    };
    assert_eq!(
        normalized(transition.event().clone()),
        normalized(canonical.event().clone())
    );
    assert_eq!(transition.rbi_contribution(), canonical.rbi_contribution());
    assert_eq!(transition.before(), before);
    assert_eq!(transition.after(), &after);
    assert_eq!(*choice, original);
    let (putouts, assists) = match choice {
        ContactDecision::CaughtSacrificeFly { catch, facts } => {
            assert_eq!(transition.caught_sacrifice_fly_facts(), Some(facts));
            assert_eq!(transition.event().ordinary_air_catch.as_ref(), Some(catch));
            (vec![(catch.fielder_id.clone(), 1)], vec![])
        }
        ContactDecision::SacrificeBunt(facts) => {
            assert_eq!(transition.sacrifice_bunt_throw_facts(), Some(facts));
            (
                vec![(facts.receiver_id.clone(), 1)],
                vec![(facts.thrower_id.clone(), 1)],
            )
        }
        ContactDecision::GroundForceDoublePlay(facts) => {
            assert_eq!(transition.ground_force_double_play_facts(), Some(facts));
            (
                vec![
                    (facts.pivot_id.clone(), 1),
                    (facts.first_receiver_id.clone(), 1),
                ],
                vec![
                    (facts.initial_fielder_id.clone(), 1),
                    (facts.pivot_id.clone(), 1),
                ],
            )
        }
    };
    assert_eq!(
        transition.fielding_contribution(),
        Some(accounting::FieldingContribution {
            putouts: putouts.into_iter().collect(),
            assists: assists.into_iter().collect()
        })
    );
    roundtrip(
        &transition,
        [b"BBE28", b"BBE29", b"BBE30"][usize::from(family)],
    );
    let stats = accounting::derive_accounting(&[transition]).unwrap();
    let offense = before.half.offense();
    let defense = 1 - offense;
    let batter =
        &before.teams[offense].batting_order[usize::from(before.teams[offense].next_batter)];
    let line = &stats.batters[offense][batter];
    assert_eq!(
        (
            line.plate_appearances,
            line.at_bats,
            line.classified_hits,
            line.classified_rbi
        ),
        (
            1,
            Some(u32::from(family == 2)),
            Some(0),
            Some(u32::from(family == 0))
        )
    );
    assert_eq!(
        (
            line.credited_sacrifice_flies,
            line.credited_sacrifice_bunts,
            line.credited_grounded_into_double_plays,
            line.credited_runs
        ),
        (
            u32::from(family == 0),
            u32::from(family == 1),
            u32::from(family == 2),
            0
        )
    );
    let pitcher = &stats.pitchers[defense][&before.teams[defense].pitcher];
    assert_eq!(
        (
            pitcher.batters_faced,
            pitcher.outs_recorded,
            pitcher.at_bats_against,
            pitcher.classified_hits_allowed,
            pitcher.runs_allowed
        ),
        (
            1,
            if family == 2 { 2 } else { 1 },
            Some(u32::from(family == 2)),
            Some(0),
            0
        )
    );
    assert_eq!(stats.runs[offense], u32::from(family == 0));
    assert_eq!(stats.runs[defense], 0);
    assert_eq!(
        stats.fielders[defense]
            .values()
            .map(|line| line.credited_putouts)
            .sum::<u32>(),
        if family == 2 { 2 } else { 1 }
    );
    assert_eq!(
        stats.fielders[defense]
            .values()
            .map(|line| line.credited_assists)
            .sum::<u32>(),
        [0, 1, 2][usize::from(family)]
    );
}

#[test]
fn independent_state_and_accounting_cartesian_grid() {
    let mut attempts = [0; 3];
    let mut terminal = 0;
    for family in 0..3 {
        let masks: &[u8] = if family == 0 { &[4, 5, 6, 7] } else { &[1, 5] };
        for (inning, half) in [
            (2, Half::Top),
            (2, Half::Bottom),
            (9, Half::Top),
            (65535, Half::Top),
        ] {
            for outs in 0..2 {
                for &mask in masks {
                    for cursor in 0..9 {
                        for dh in [false, true] {
                            for score in [[0, 0], [2, 1], [1, 2]] {
                                for role in 0..if family == 2 { 6 } else { 3 } {
                                    for (index, contact) in
                                        [AirContact::Fly, AirContact::LineDrive, AirContact::PopUp]
                                            .into_iter()
                                            .enumerate()
                                    {
                                        if family != 0 && index != 0 {
                                            continue;
                                        }
                                        let mut before =
                                            matrix_state(mask, outs, half, inning, cursor, dh);
                                        before.score = score;
                                        let choice = decision(&before, family, role, contact);
                                        attempts[usize::from(family)] += 1;
                                        terminal += u32::from(expected(&before, family).is_none());
                                        checked(&before, &choice, family);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(attempts, [15552, 2592, 5184]);
    assert_eq!(terminal, 432);
    println!("supplied projection grid: attempts={attempts:?};terminal Top refusals={terminal};accepted=22896");
}

#[test]
fn unsupported_states_and_unknown_facts_refuse_without_mutation() {
    for family in 0..3 {
        let before = state(if family == 0 { 4 } else { 5 }, 0);
        let choice = decision(&before, family, 0, AirContact::Fly);
        for alter in 0..6 {
            let mut bad = before.clone();
            match alter {
                0 => bad.inning = 0,
                1 => bad.over = true,
                2 => bad.outs = 2,
                3 => bad.teams[0].next_batter = 9,
                4 => bad.bases[if family == 0 { 2 } else { 0 }] = None,
                5 => bad.teams[1].batting_order[0] = bad.teams[1].batting_order[1].clone(),
                _ => unreachable!(),
            }
            rejected(&bad, &choice);
        }
        for inning in [9, 10, 65535] {
            for outs in 0..2 {
                for score in [[0, 0], [2, 1], [1, 2]] {
                    let mut bad = matrix_state(
                        if family == 0 { 4 } else { 5 },
                        outs,
                        Half::Bottom,
                        inning,
                        0,
                        true,
                    );
                    bad.score = score;
                    let supplied = decision(&bad, family, 0, AirContact::Fly);
                    rejected(&bad, &supplied);
                }
            }
        }
        if family != 0 {
            let mut occupied = before.clone();
            occupied.bases[1] = state(7, 0).bases[1].clone();
            rejected(&occupied, &choice);
        }
        for actor in ["foreign", "a0", "hbench", "hremoved", "h8"] {
            let mut bad = choice.clone();
            match &mut bad {
                ContactDecision::CaughtSacrificeFly { catch, .. } => {
                    catch.fielder_id = actor.into()
                }
                ContactDecision::SacrificeBunt(facts) => facts.thrower_id = actor.into(),
                ContactDecision::GroundForceDoublePlay(facts) => facts.pivot_id = actor.into(),
            }
            rejected(&before, &bad);
        }
        let mut unknown = choice.clone();
        match &mut unknown {
            ContactDecision::CaughtSacrificeFly { facts, .. } => {
                facts.handling = SacrificeFlyHandling::Unknown
            }
            ContactDecision::SacrificeBunt(facts) => {
                facts.judgment = SacrificeBuntJudgment::Unknown
            }
            ContactDecision::GroundForceDoublePlay(facts) => {
                facts.forced_runner_id = "wrong-runner".into()
            }
        }
        rejected(&before, &unknown);
        for change in 0..5 {
            let mut bad = choice.clone();
            match &mut bad {
                ContactDecision::CaughtSacrificeFly { catch, facts } => match change {
                    0 => facts.retouch = SacrificeFlyRetouch::Unknown,
                    1 => facts.timing = SacrificeFlyRunTiming::Unknown,
                    2 => facts.timing = SacrificeFlyRunTiming::BeforeCatch,
                    3 => facts.scoring_runner_id = "wrong-runner".into(),
                    _ => catch.batter_id = "wrong-batter".into(),
                },
                ContactDecision::SacrificeBunt(facts) => match change {
                    0 => facts.judgment = SacrificeBuntJudgment::ExclusivelyForBaseHit,
                    1 => facts.receiver_id = facts.thrower_id.clone(),
                    2 => facts.receiver_id = "h8".into(),
                    3 => facts.advancing_runner_id = "wrong-runner".into(),
                    _ => facts.batter_id = "wrong-batter".into(),
                },
                ContactDecision::GroundForceDoublePlay(facts) => match change {
                    0 => facts.pivot_id = facts.initial_fielder_id.clone(),
                    1 => facts.first_receiver_id = facts.pivot_id.clone(),
                    2 => facts.initial_fielder_id = "hbench".into(),
                    3 => facts.first_receiver_id = "a0".into(),
                    _ => facts.batter_id = "wrong-batter".into(),
                },
            }
            rejected(&before, &bad);
        }
    }
}

fn unit(actor: &str, kind: CreditKind) -> Credit {
    Credit {
        player_id: actor.into(),
        kind,
    }
}
// Separate explicit event fixture: the new projector is never called here.
fn canonical_fixture(
    before: &GameState,
    choice: &ContactDecision,
) -> Result<EventTransition, TransitionError> {
    let offense = &before.teams[before.half.offense()];
    let pitcher = before.teams[1 - before.half.offense()].pitcher.clone();
    let batter = offense.batting_order[usize::from(offense.next_batter)].clone();
    let family = match choice {
        ContactDecision::CaughtSacrificeFly { .. } => 0,
        ContactDecision::SacrificeBunt(_) => 1,
        ContactDecision::GroundForceDoublePlay(_) => 2,
    };
    let mut moves = vec![RunnerMove {
        runner: Runner {
            player_id: batter.clone(),
            responsible_pitcher: pitcher.clone(),
        },
        from: Base::Plate,
        to: Destination::Out,
    }];
    for (i, runner) in before.bases.iter().enumerate() {
        if let Some(runner) = runner {
            moves.push(RunnerMove {
                runner: runner.clone(),
                from: [Base::First, Base::Second, Base::Third][i],
                to: match (family, i) {
                    (0, 2) => Destination::Home,
                    (1, 0) => Destination::Second,
                    (2, 0) => Destination::Out,
                    _ => [Destination::First, Destination::Second, Destination::Third][i],
                },
            });
        }
    }
    let mut credits = vec![
        unit(&batter, CreditKind::PlateAppearance),
        unit(&batter, CreditKind::NoHit),
    ];
    let mut air = None;
    let mut contact = None;
    match choice {
        ContactDecision::CaughtSacrificeFly { catch, .. } => {
            air = Some(catch.clone());
            credits.extend([
                unit(&batter, CreditKind::SacrificeFly),
                unit(&batter, CreditKind::Rbi),
                unit(
                    &before.bases[2].as_ref().unwrap().player_id,
                    CreditKind::Run,
                ),
                unit(&catch.fielder_id, CreditKind::Putout),
            ]);
        }
        ContactDecision::SacrificeBunt(facts) => credits.extend([
            unit(&batter, CreditKind::SacrificeBunt),
            unit(&facts.receiver_id, CreditKind::Putout),
            unit(&facts.thrower_id, CreditKind::Assist),
        ]),
        ContactDecision::GroundForceDoublePlay(facts) => {
            credits.extend([
                unit(&batter, CreditKind::AtBat),
                unit(&batter, CreditKind::GroundedIntoDoublePlay),
                unit(&facts.first_receiver_id, CreditKind::Putout),
                unit(&facts.pivot_id, CreditKind::Putout),
                unit(&facts.initial_fielder_id, CreditKind::Assist),
                unit(&facts.pivot_id, CreditKind::Assist),
            ]);
            contact = Some(TwoOutContactFacts {
                contact: ContactClass::Grounded,
                continuity: ContactContinuity::Continuous,
                first_out: RetirementFact {
                    runner_id: before.bases[0].as_ref().unwrap().player_id.clone(),
                    cause: RetirementCause::Force {
                        at: Destination::Second,
                    },
                },
                second_out: RetirementFact {
                    runner_id: batter.clone(),
                    cause: RetirementCause::BatterPutoutBeforeFirstTouch,
                },
                error_placement: Some(ReviewedErrorPlacement {
                    between_outs: vec![],
                    outside_interval: vec![],
                }),
            });
        }
    }
    let event = Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(batter.clone()),
            pitcher,
        },
        fielding_review: FieldingReview::NoMisplay,
        delta: Delta {
            moves,
            outs: if family == 2 { 2 } else { 1 },
            runs: u8::from(family == 0),
            credits,
            changes: vec![],
            third_out: (family == 2 && before.outs == 1).then(|| ThirdOut {
                runner_id: batter,
                kind: ThirdOutKind::BatterBeforeFirst,
                home_before_out: vec![],
            }),
        },
        ordinary_air_catch: air,
        two_out_contact: contact,
        single_delivery: None,
        balk: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        muffed_ground_ball: None,
        ordinary_ground_throw: None,
    };
    match choice {
        ContactDecision::CaughtSacrificeFly { facts, .. } => {
            apply_with_caught_sacrifice_fly_facts(before, event, facts.clone())
        }
        ContactDecision::SacrificeBunt(facts) => {
            apply_with_sacrifice_bunt_throw_facts(before, event, facts.clone())
        }
        ContactDecision::GroundForceDoublePlay(facts) => {
            apply_with_ground_force_double_play_facts(before, event, facts.clone())
        }
    }
}
fn ordinary(before: &GameState, walk: bool, steal: bool) -> EventTransition {
    let team = &before.teams[before.half.offense()];
    let batter = team.batting_order[usize::from(team.next_batter)].clone();
    let pitcher = before.teams[1 - before.half.offense()].pitcher.clone();
    let mut moves = Vec::new();
    let credits = if steal {
        let runner = before.bases[1].as_ref().unwrap();
        vec![unit(&runner.player_id, CreditKind::StolenBase)]
    } else {
        moves.push(RunnerMove {
            runner: Runner {
                player_id: batter.clone(),
                responsible_pitcher: pitcher.clone(),
            },
            from: Base::Plate,
            to: if walk {
                Destination::First
            } else {
                Destination::Out
            },
        });
        if walk {
            vec![
                unit(&batter, CreditKind::PlateAppearance),
                unit(&batter, CreditKind::Walk),
            ]
        } else {
            vec![
                unit(&batter, CreditKind::PlateAppearance),
                unit(&batter, CreditKind::AtBat),
                unit(&batter, CreditKind::NoHit),
                unit(&batter, CreditKind::Strikeout),
                unit(&pitcher, CreditKind::Putout),
            ]
        }
    };
    for (i, runner) in before.bases.iter().enumerate() {
        if let Some(runner) = runner {
            moves.push(RunnerMove {
                runner: runner.clone(),
                from: [Base::First, Base::Second, Base::Third][i],
                to: if steal && i == 1 {
                    Destination::Third
                } else {
                    [Destination::First, Destination::Second, Destination::Third][i]
                },
            });
        }
    }
    apply(
        before,
        Event {
            kind: if steal {
                EventKind::RunnerOnly(RunnerEvent::StealAttempt)
            } else {
                EventKind::PlateAppearance
            },
            participants: Participants {
                batter: Some(batter.clone()),
                pitcher,
            },
            fielding_review: if steal {
                FieldingReview::NoMisplay
            } else {
                FieldingReview::Unknown
            },
            delta: Delta {
                moves,
                outs: u8::from(!walk && !steal),
                runs: 0,
                credits,
                changes: vec![],
                third_out: (!walk && !steal && before.outs == 2).then(|| ThirdOut {
                    runner_id: batter,
                    kind: ThirdOutKind::BatterBeforeFirst,
                    home_before_out: vec![],
                }),
            },
            two_out_contact: None,
            single_delivery: None,
            balk: None,
            caught_strikeout: None,
            protected_award: None,
            sole_home_run: None,
            sole_safe_hit: None,
            muffed_ground_ball: None,
            ordinary_ground_throw: None,
            ordinary_air_catch: None,
        },
    )
    .unwrap()
}

#[test]
fn supplied_mixed_prefix_keeps_runner_only_out_of_pa_and_preserves_continuity() {
    let initial = state(5, 0);
    let mut current = initial.clone();
    let mut transitions = Vec::new();
    // SF, bunt, safe runner-only steal, K; then walk, DP, K. Not a generated game.
    for action in 0..7 {
        let transition = match action {
            0 | 1 | 5 => {
                let family = match action {
                    0 => 0,
                    1 => 1,
                    _ => 2,
                };
                let supplied = decision(&current, family, 0, AirContact::Fly);
                let reference = canonical_fixture(&current, &supplied).unwrap();
                let next = project(&current, &supplied).unwrap();
                assert_eq!(next.after(), &expected(&current, family).unwrap());
                assert_eq!(next.after(), reference.after());
                next
            }
            2 => ordinary(&current, false, true),
            4 => ordinary(&current, true, false),
            _ => ordinary(&current, false, false),
        };
        assert_eq!(transition.before(), &current);
        assert_eq!(
            EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
            transition
        );
        current = transition.after().clone();
        transitions.push(transition);
        let stats = accounting::derive_accounting(&transitions).unwrap();
        let pa = [1, 2, 2, 3, 4, 5, 6][action];
        assert_eq!(
            stats
                .batters
                .iter()
                .flat_map(|team| team.values())
                .map(|line| line.plate_appearances)
                .sum::<u32>(),
            pa
        );
        assert_eq!(
            stats
                .pitchers
                .iter()
                .flat_map(|team| team.values())
                .map(|line| line.batters_faced)
                .sum::<u32>(),
            pa
        );
        assert_eq!(
            stats
                .pitchers
                .iter()
                .flat_map(|team| team.values())
                .map(|line| line.outs_recorded)
                .sum::<u32>(),
            [1, 2, 2, 3, 3, 5, 6][action]
        );
        assert_eq!(stats.runs, [1, 0]);
    }
    assert_eq!(
        (current.inning, current.half, current.outs),
        (3, Half::Top, 0)
    );
    assert_eq!(current.score, [1, 0]);
    assert_eq!(current.bases, [None, None, None]);
    assert_eq!(
        [current.teams[0].next_batter, current.teams[1].next_batter],
        [3, 3]
    );
    assert_eq!(transitions[2].before().teams, transitions[2].after().teams);
    assert!(!current.over);
    assert_eq!(transitions[0].before(), &initial);
}
