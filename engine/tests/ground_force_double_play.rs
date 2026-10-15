//! Supplied direct force/First relay facts, not inferred from GIDP/PO/A labels.
use bbsim_engine::*;

fn state(mask: u8, outs: u8, half: Half, cursor: u8, dh: bool) -> GameState {
    let teams = ["a", "h"].map(|prefix| {
        let pitcher = format!("{prefix}p");
        let mut batting_order = std::array::from_fn(|i| format!("{prefix}{i}"));
        if !dh {
            batting_order[7] = pitcher.clone();
        }
        TeamState {
            batting_order,
            next_batter: cursor,
            pitcher,
            dh_slot: dh.then_some(8),
            dh_must_bat: false,
            bench: vec![format!("{prefix}bench")],
            removed: vec![format!("{prefix}removed")],
        }
    });
    let bases = std::array::from_fn(|i| {
        (mask & (1 << i) != 0).then(|| Runner {
            player_id: teams[half.offense()].batting_order[(usize::from(cursor) + i + 3) % 9]
                .clone(),
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
fn batter(before: &GameState) -> String {
    let team = &before.teams[before.half.offense()];
    team.batting_order[usize::from(team.next_batter)].clone()
}
fn credit(actor: &str, kind: CreditKind) -> Credit {
    Credit {
        player_id: actor.into(),
        kind,
    }
}
fn fixture(before: &GameState, role: u8) -> (Event, GroundForceDoublePlayFacts) {
    let defense = &before.teams[1 - before.half.offense()];
    let initial = if matches!(role, 2 | 5) {
        defense.pitcher.clone()
    } else if role == 1 {
        defense.batting_order[1].clone()
    } else {
        defense.batting_order[3].clone()
    };
    let pivot = if role == 3 {
        defense.pitcher.clone()
    } else {
        defense.batting_order[2].clone()
    };
    let receiver = if matches!(role, 4 | 5) {
        defense.pitcher.clone()
    } else {
        defense.batting_order[1].clone()
    };
    let batter = batter(before);
    let runner = before.bases[0].as_ref().expect("First occupied");
    let facts = GroundForceDoublePlayFacts {
        batter_id: batter.clone(),
        forced_runner_id: runner.player_id.clone(),
        initial_fielder_id: initial.clone(),
        pivot_id: pivot.clone(),
        first_receiver_id: receiver.clone(),
    };
    let mut moves = vec![
        RunnerMove {
            runner: Runner {
                player_id: batter.clone(),
                responsible_pitcher: defense.pitcher.clone(),
            },
            from: Base::Plate,
            to: Destination::Out,
        },
        RunnerMove {
            runner: runner.clone(),
            from: Base::First,
            to: Destination::Out,
        },
    ];
    if let Some(third) = &before.bases[2] {
        moves.push(RunnerMove {
            runner: third.clone(),
            from: Base::Third,
            to: Destination::Third,
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
            outs: 2,
            runs: 0,
            credits: vec![
                credit(&batter, CreditKind::PlateAppearance),
                credit(&batter, CreditKind::AtBat),
                credit(&batter, CreditKind::NoHit),
                credit(&batter, CreditKind::GroundedIntoDoublePlay),
                credit(&pivot, CreditKind::Putout),
                credit(&receiver, CreditKind::Putout),
                credit(&initial, CreditKind::Assist),
                credit(&pivot, CreditKind::Assist),
            ],
            changes: vec![],
            third_out: (before.outs == 1).then(|| ThirdOut {
                runner_id: batter.clone(),
                kind: ThirdOutKind::BatterBeforeFirst,
                home_before_out: vec![],
            }),
        },
        two_out_contact: Some(TwoOutContactFacts {
            contact: ContactClass::Grounded,
            continuity: ContactContinuity::Continuous,
            first_out: RetirementFact {
                runner_id: runner.player_id.clone(),
                cause: RetirementCause::Force {
                    at: Destination::Second,
                },
            },
            second_out: RetirementFact {
                runner_id: batter,
                cause: RetirementCause::BatterPutoutBeforeFirstTouch,
            },
            error_placement: Some(ReviewedErrorPlacement {
                between_outs: vec![],
                outside_interval: vec![],
            }),
        }),
        ordinary_ground_throw: None,
        ordinary_air_catch: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        muffed_ground_ball: None,
        single_delivery: None,
        balk: None,
    };
    (event, facts)
}

#[test]
fn supplied_direct_force_relay_qualifies_named_fielding() {
    let before = state(5, 0, Half::Top, 0, true);
    let (event, supplied) = fixture(&before, 0);
    let legacy = apply(&before, event.clone()).unwrap();
    let transition =
        apply_with_ground_force_double_play_facts(&before, event.clone(), supplied.clone())
            .unwrap();
    assert_eq!(transition.after(), legacy.after());
    assert_eq!(transition.after().outs, 2);
    assert_eq!(
        transition.after().bases,
        [None, None, before.bases[2].clone()]
    );
    assert_eq!(transition.after().teams[0].next_batter, 1);
    assert_eq!(transition.ground_force_double_play_facts(), Some(&supplied));
    let proof = transition.ground_force_double_play_fielding().unwrap();
    assert_eq!(proof.putouts, [("h2".into(), 1), ("h1".into(), 1)].into());
    assert_eq!(proof.assists, [("h3".into(), 1), ("h2".into(), 1)].into());
    assert_eq!(transition.fielding_contribution(), Some(proof));
    assert_eq!(
        transition.ground_force_double_play_rbi(),
        Some(accounting::RbiContribution {
            runs_batted_in: [("a0".into(), 0)].into(),
        })
    );
    let canonical = EventTransition::try_new_with_ground_force_double_play_facts(
        before,
        event,
        transition.after().clone(),
        supplied,
    )
    .unwrap();
    assert_eq!(canonical, transition);
    assert_eq!(&transition.to_bytes()[..5], b"BBE30");
    assert_eq!(
        EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
        transition
    );
}

#[test]
fn legacy_qualified_gidp_still_has_unknown_named_fielding() {
    let before = state(1, 0, Half::Top, 0, true);
    let (event, _) = fixture(&before, 0);
    let legacy = apply(&before, event).unwrap();
    assert!(legacy.ground_force_double_play_facts().is_none());
    assert!(legacy.ground_force_double_play_fielding().is_none());
    assert!(legacy.fielding_contribution().is_none());
    assert_eq!(
        legacy.rbi_contribution(),
        Some(accounting::RbiContribution {
            runs_batted_in: [("a0".into(), 0)].into(),
        })
    );
    assert!(legacy.ground_force_double_play_rbi().is_none());
    assert_eq!(&legacy.to_bytes()[..5], b"BBE25");
}

// Independent state oracle: no call to apply/ending to predict support or cleanup.
fn expected_after(before: &GameState) -> GameState {
    let mut after = before.clone();
    let side = before.half.offense();
    after.teams[side].next_batter = (before.teams[side].next_batter + 1) % 9;
    after.outs = before.outs + 2;
    after.bases = [None, None, before.bases[2].clone()];
    if after.outs == 3 {
        after.outs = 0;
        after.bases = [None, None, None];
        if before.half == Half::Top && before.inning >= 9 && before.score[1] > before.score[0] {
            after.over = true;
        } else if before.half == Half::Top {
            after.half = Half::Bottom;
        } else {
            after.half = Half::Top;
            after.inning = before
                .inning
                .checked_add(1)
                .expect("oracle fixture cannot overflow");
        }
    }
    after
}
fn rejected(before: &GameState, event: &Event, facts: &GroundForceDoublePlayFacts) {
    let saved = before.clone();
    assert!(
        apply_with_ground_force_double_play_facts(before, event.clone(), facts.clone()).is_err()
    );
    assert!(
        EventTransition::try_new_with_ground_force_double_play_facts(
            before.clone(),
            event.clone(),
            expected_after(before),
            facts.clone()
        )
        .is_err()
    );
    assert_eq!(*before, saved);
}
fn checked(
    before: &GameState,
    event: &Event,
    facts: &GroundForceDoublePlayFacts,
) -> EventTransition {
    let t =
        apply_with_ground_force_double_play_facts(before, event.clone(), facts.clone()).unwrap();
    assert_eq!(t.after(), &expected_after(before));
    assert_eq!(t.ground_force_double_play_facts(), Some(facts));
    assert_eq!(
        t,
        EventTransition::try_new_with_ground_force_double_play_facts(
            before.clone(),
            event.clone(),
            expected_after(before),
            facts.clone()
        )
        .unwrap()
    );
    let proof = accounting::FieldingContribution {
        putouts: [
            (facts.pivot_id.clone(), 1),
            (facts.first_receiver_id.clone(), 1),
        ]
        .into(),
        assists: [
            (facts.initial_fielder_id.clone(), 1),
            (facts.pivot_id.clone(), 1),
        ]
        .into(),
    };
    assert_eq!(t.ground_force_double_play_fielding(), Some(proof.clone()));
    assert_eq!(t.fielding_contribution(), Some(proof));
    assert_eq!(
        t.ground_force_double_play_rbi(),
        Some(accounting::RbiContribution {
            runs_batted_in: [(batter(before), 0)].into(),
        })
    );
    assert_eq!(&t.to_bytes()[..5], b"BBE30");
    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
    t
}
#[test]
fn cartesian_state_and_independent_accounting_oracle() {
    let mut accepted = 0;
    let mut terminal = 0;
    for (inning, half) in [
        (2, Half::Top),
        (2, Half::Bottom),
        (9, Half::Top),
        (65535, Half::Top),
    ] {
        for outs in 0..2 {
            for mask in [1, 5] {
                for cursor in 0..9 {
                    for dh in [false, true] {
                        for score in [[0, 0], [2, 0], [0, 2]] {
                            for role in 0..6 {
                                for reversed in [false, true] {
                                    let mut before = state(mask, outs, half, cursor, dh);
                                    before.inning = inning;
                                    before.score = score;
                                    let (mut event, facts) = fixture(&before, role);
                                    if reversed {
                                        event.delta.moves.reverse();
                                        event.delta.credits.reverse();
                                    }
                                    if inning >= 9 && outs == 1 && score[1] > score[0] {
                                        rejected(&before, &event, &facts);
                                        terminal += 1;
                                        continue;
                                    }
                                    let t = checked(&before, &event, &facts);
                                    let stats = accounting::derive_accounting(&[t]).unwrap();
                                    let offense = half.offense();
                                    let defense = 1 - offense;
                                    let line = &stats.batters[offense][&facts.batter_id];
                                    assert_eq!(
                                        (
                                            line.plate_appearances,
                                            line.at_bats,
                                            line.classified_hits,
                                            line.credited_grounded_into_double_plays,
                                            line.classified_rbi
                                        ),
                                        (1, Some(1), Some(0), 1, Some(0))
                                    );
                                    assert_eq!(
                                        (
                                            line.credited_runs,
                                            line.credited_sacrifice_bunts,
                                            line.credited_sacrifice_flies
                                        ),
                                        (0, 0, 0)
                                    );
                                    let pitcher =
                                        &stats.pitchers[defense][&before.teams[defense].pitcher];
                                    assert_eq!(
                                        (
                                            pitcher.batters_faced,
                                            pitcher.outs_recorded,
                                            pitcher.at_bats_against,
                                            pitcher.classified_hits_allowed,
                                            pitcher.runs_allowed
                                        ),
                                        (1, 2, Some(1), Some(0), 0)
                                    );
                                    assert_eq!(stats.runs, [0, 0]);
                                    assert_eq!(
                                        stats.fielders[defense]
                                            .values()
                                            .map(|f| f.credited_putouts)
                                            .sum::<u32>(),
                                        2
                                    );
                                    assert_eq!(
                                        stats.fielders[defense]
                                            .values()
                                            .map(|f| f.credited_assists)
                                            .sum::<u32>(),
                                        2
                                    );
                                    for f in stats.classified_fielding[defense].values() {
                                        assert_eq!((f.errors, f.passed_balls), (Some(0), Some(0)));
                                    }
                                    accepted += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!((accepted, terminal), (9504, 864));
}
#[test]
fn known_contradictions_and_eligible_named_roles() {
    let before = state(1, 0, Half::Top, 0, true);
    let (event, facts) = fixture(&before, 0);
    let mut invalid = before.clone();
    invalid.inning = 0;
    assert_eq!(
        apply_with_ground_force_double_play_facts(&invalid, event.clone(), facts.clone()),
        Err(TransitionError::InvalidState)
    );
    let mut bad = event.clone();
    bad.participants.batter = Some("wrong".into());
    bad.two_out_contact = None;
    assert_eq!(
        apply_with_ground_force_double_play_facts(&before, bad, facts.clone()),
        Err(TransitionError::InvalidParticipants)
    );
    let mut bad = event.clone();
    bad.delta.credits[0].player_id = "h0".into();
    bad.two_out_contact = None;
    assert_eq!(
        apply_with_ground_force_double_play_facts(&before, bad, facts.clone()),
        Err(TransitionError::IllegalCredit)
    );
    for dh in [false, true] {
        let before = state(1, 0, Half::Top, 0, dh);
        let defense = &before.teams[1];
        let candidates = defense.batting_order.iter().cloned().chain([
            defense.pitcher.clone(),
            "hbench".into(),
            "hremoved".into(),
            "foreign".into(),
            "a0".into(),
        ]);
        for actor in candidates {
            for position in 0..3 {
                let (mut e, mut f) = fixture(&before, 0);
                match position {
                    0 => f.initial_fielder_id = actor.clone(),
                    1 => f.pivot_id = actor.clone(),
                    _ => f.first_receiver_id = actor.clone(),
                }
                e.delta.credits.truncate(4);
                e.delta.credits.extend([
                    credit(&f.pivot_id, CreditKind::Putout),
                    credit(&f.first_receiver_id, CreditKind::Putout),
                    credit(&f.initial_fielder_id, CreditKind::Assist),
                    credit(&f.pivot_id, CreditKind::Assist),
                ]);
                let eligible = actor == defense.pitcher
                    || defense
                        .batting_order
                        .iter()
                        .enumerate()
                        .any(|(i, id)| *id == actor && defense.dh_slot != Some(i as u8));
                if eligible
                    && f.initial_fielder_id != f.pivot_id
                    && f.first_receiver_id != f.pivot_id
                {
                    checked(&before, &e, &f);
                } else {
                    rejected(&before, &e, &f);
                }
            }
        }
    }
    let mut ambiguous = before.clone();
    ambiguous.teams[0]
        .removed
        .push(facts.initial_fielder_id.clone());
    assert!(apply(&ambiguous, event.clone()).is_ok());
    rejected(&ambiguous, &event, &facts);
}
#[test]
fn late_bottom_and_unqualified_episodes_refuse_without_fallback() {
    for inning in 9..=u16::MAX {
        for outs in 0..2 {
            for score in [[0, 0], [2, 0], [0, 2]] {
                let mut before = state(1, outs, Half::Bottom, 0, true);
                before.inning = inning;
                before.score = score;
                let (e, f) = fixture(&before, 0);
                // Standalone guard is checked even where a legal after cannot exist.
                assert!(apply_with_ground_force_double_play_facts(&before, e, f).is_err());
            }
        }
    }
    let before = state(5, 1, Half::Top, 0, true);
    let (event, facts) = fixture(&before, 0);
    for i in 0..8 {
        let mut bad = event.clone();
        bad.delta.credits.remove(i);
        rejected(&before, &bad, &facts);
        let mut bad = event.clone();
        bad.delta.credits.push(event.delta.credits[i].clone());
        rejected(&before, &bad, &facts);
        let mut bad = event.clone();
        bad.delta.credits[i].player_id = "wrong".into();
        rejected(&before, &bad, &facts);
    }
    for i in 0..29 {
        let mut bad = event.clone();
        match i {
            0 => bad.two_out_contact = None,
            1 => bad.two_out_contact.as_mut().unwrap().continuity = ContactContinuity::Unknown,
            2 => {
                bad.two_out_contact.as_mut().unwrap().continuity = ContactContinuity::Discontinuous
            }
            3 => bad.two_out_contact.as_mut().unwrap().error_placement = None,
            4 => bad
                .two_out_contact
                .as_mut()
                .unwrap()
                .error_placement
                .as_mut()
                .unwrap()
                .outside_interval
                .push("h1".into()),
            5 => {
                let c = bad.two_out_contact.as_mut().unwrap();
                std::mem::swap(&mut c.first_out, &mut c.second_out);
            }
            6 => {
                bad.two_out_contact.as_mut().unwrap().first_out.cause = RetirementCause::Force {
                    at: Destination::Third,
                }
            }
            7 => {
                bad.two_out_contact.as_mut().unwrap().first_out.cause = RetirementCause::Tag {
                    at: Destination::Second,
                    force_loss: ForceLoss::Unknown,
                }
            }
            8 => {
                bad.two_out_contact.as_mut().unwrap().second_out.cause =
                    RetirementCause::RunnerInterference
            }
            9 => bad.delta.third_out = None,
            10 => bad.delta.third_out.as_mut().unwrap().runner_id = facts.forced_runner_id.clone(),
            11 => bad.delta.moves[0].runner.responsible_pitcher = "wrong-charge".into(),
            12 => bad.delta.moves[1].runner.responsible_pitcher = "wrong-charge".into(),
            13 => {
                bad.delta.moves.pop();
            }
            14 => bad.delta.moves[2].to = Destination::Home,
            15 => bad.delta.outs = 1,
            16 => bad.delta.runs = 1,
            17 => bad.fielding_review = FieldingReview::Unknown,
            18 => {
                bad.ordinary_ground_throw = Some(OrdinaryGroundThrowFacts {
                    batter_id: facts.batter_id.clone(),
                    thrower_id: facts.initial_fielder_id.clone(),
                    receiver_id: facts.first_receiver_id.clone(),
                })
            }
            19 => {
                bad.sole_safe_hit = Some(SoleSafeHitFacts {
                    batter_id: facts.batter_id.clone(),
                })
            }
            20 => bad.delta.changes.push(PersonnelChange::Pitcher {
                team: TeamSide::Home,
                outgoing: "hp".into(),
                incoming: "hbench".into(),
            }),
            21 => {
                bad.two_out_contact.as_mut().unwrap().second_out.runner_id =
                    facts.forced_runner_id.clone()
            }
            22 => bad.two_out_contact.as_mut().unwrap().contact = ContactClass::NotGrounded,
            23 => {
                bad.two_out_contact.as_mut().unwrap().second_out.cause =
                    RetirementCause::BatterAfterFirstTag {
                        at: Destination::First,
                    }
            }
            24 => bad.delta.credits.push(credit("h1", CreditKind::Error)),
            25 => bad
                .two_out_contact
                .as_mut()
                .unwrap()
                .error_placement
                .as_mut()
                .unwrap()
                .between_outs
                .push("h1".into()),
            26 => bad.delta.third_out.as_mut().unwrap().kind = ThirdOutKind::Timing,
            27 => bad.participants.pitcher = "wrong".into(),
            _ => bad.delta.moves[1].from = Base::Second,
        }
        rejected(&before, &bad, &facts);
    }
    for i in 0..5 {
        let mut f = facts.clone();
        match i {
            0 => f.batter_id = "wrong".into(),
            1 => f.forced_runner_id = "wrong".into(),
            2 => f.initial_fielder_id = f.pivot_id.clone(),
            3 => f.first_receiver_id = f.pivot_id.clone(),
            _ => f.pivot_id = "hbench".into(),
        }
        rejected(&before, &event, &f);
    }
    for mask in [0, 2, 3, 4, 6, 7] {
        rejected(&state(mask, 1, Half::Top, 0, true), &event, &facts);
    }
    let mut badbefore = before.clone();
    badbefore.outs = 2;
    rejected(&badbefore, &event, &facts);
    let mut badbefore = before.clone();
    badbefore.over = true;
    rejected(&badbefore, &event, &facts);
    let bunt = SacrificeBuntThrowFacts {
        batter_id: facts.batter_id.clone(),
        advancing_runner_id: facts.forced_runner_id.clone(),
        thrower_id: facts.initial_fielder_id.clone(),
        receiver_id: facts.first_receiver_id.clone(),
        judgment: SacrificeBuntJudgment::SacrificingOwnChance,
    };
    assert!(apply_with_sacrifice_bunt_throw_facts(&before, event.clone(), bunt).is_err());
    let sf = CaughtSacrificeFlyFacts {
        scoring_runner_id: facts.forced_runner_id.clone(),
        handling: SacrificeFlyHandling::QualifyingOutfielderOrInfielderInOutfield,
        retouch: SacrificeFlyRetouch::Legal,
        timing: SacrificeFlyRunTiming::AfterCatch,
    };
    assert!(apply_with_caught_sacrifice_fly_facts(&before, event.clone(), sf).is_err());
    let walkoff = WalkoffHitFacts {
        winning_runner_id: facts.forced_runner_id.clone(),
        batter_touch: NamedBaseTouch {
            runner_id: facts.batter_id.clone(),
            base: Destination::First,
        },
        timing: NaturalCourseTouchTiming::ByWinningRun,
    };
    assert!(apply_with_walkoff_hit_facts(&before, event, walkoff).is_err());
}
#[test]
fn context_ownership_wire_tail_and_after_state_revalidate() {
    let before = state(5, 0, Half::Top, 0, true);
    let (event, mut f) = fixture(&before, 0);
    let original = f.clone();
    let t = checked(&before, &event, &f);
    f.initial_fielder_id = "changed".into();
    assert_eq!(t.ground_force_double_play_facts(), Some(&original));
    let bytes = t.to_bytes();
    let mut version_only = apply(&before, event.clone()).unwrap().to_bytes();
    version_only[..5].copy_from_slice(b"BBE30");
    assert!(EventTransition::from_bytes(&version_only).is_err());
    for len in 0..bytes.len() {
        assert!(EventTransition::from_bytes(&bytes[..len]).is_err());
    }
    let mut bad = bytes.clone();
    bad.push(0);
    assert!(EventTransition::from_bytes(&bad).is_err());
    for version in [b"BBE25", b"BBE26", b"BBE27", b"BBE28", b"BBE29", b"BBE31"] {
        let mut bad = bytes.clone();
        bad[..5].copy_from_slice(version);
        assert!(EventTransition::from_bytes(&bad).is_err());
    }
    let names = [
        &original.batter_id,
        &original.forced_runner_id,
        &original.initial_fielder_id,
        &original.pivot_id,
        &original.first_receiver_id,
    ];
    let tail: usize = names.iter().map(|id| 8 + id.len()).sum();
    let mut start = bytes.len() - tail;
    for id in names {
        let mut bad = bytes.clone();
        bad[start + 8] ^= 32;
        assert!(EventTransition::from_bytes(&bad).is_err());
        start += 8 + id.len();
    }
    // GameState ends with over: a valid bool corrupted in the supplied AFTER.
    let mut bad = bytes.clone();
    bad[bytes.len() - tail - 1] = 1;
    assert!(EventTransition::from_bytes(&bad).is_err());
    for i in 0..5 {
        let mut after = expected_after(&before);
        match i {
            0 => after.score[0] += 1,
            1 => after.outs = 1,
            2 => after.teams[0].next_batter = 0,
            3 => after.bases[2] = None,
            _ => after.over = true,
        }
        assert!(
            EventTransition::try_new_with_ground_force_double_play_facts(
                before.clone(),
                event.clone(),
                after,
                original.clone()
            )
            .is_err()
        );
    }
}
fn permutations<T: Clone>(values: &[T]) -> Vec<Vec<T>> {
    if values.is_empty() {
        return vec![vec![]];
    }
    let mut result = vec![];
    for i in 0..values.len() {
        let mut remaining = values.to_vec();
        let head = remaining.remove(i);
        for mut tail in permutations(&remaining) {
            tail.insert(0, head.clone());
            result.push(tail);
        }
    }
    result
}
#[test]
fn every_credit_and_movement_order_preserves_the_episode() {
    let mut frames = 0;
    for (mask, role) in [(5, 0), (1, 1)] {
        let before = state(mask, 0, Half::Top, 0, true);
        let (event, f) = fixture(&before, role);
        let expected = checked(&before, &event, &f);
        for credits in permutations(&event.delta.credits) {
            for moves in permutations(&event.delta.moves) {
                let mut e = event.clone();
                e.delta.credits = credits.clone();
                e.delta.moves = moves;
                let t = apply_with_ground_force_double_play_facts(&before, e, f.clone()).unwrap();
                assert_eq!(t.after(), expected.after());
                assert_eq!(t.fielding_contribution(), expected.fielding_contribution());
                assert_eq!(t.rbi_contribution(), expected.rbi_contribution());
                frames += 1;
            }
        }
    }
    assert_eq!(frames, 322560);
}
fn simple_pa(
    before: &GameState,
    moves: Vec<RunnerMove>,
    outs: u8,
    runs: u8,
    credits: Vec<Credit>,
) -> Event {
    Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(batter(before)),
            pitcher: before.teams[1 - before.half.offense()].pitcher.clone(),
        },
        fielding_review: FieldingReview::NoMisplay,
        delta: Delta {
            moves,
            outs,
            runs,
            credits,
            changes: vec![],
            third_out: (before.outs + outs == 3).then(|| ThirdOut {
                runner_id: batter(before),
                kind: ThirdOutKind::BatterBeforeFirst,
                home_before_out: vec![],
            }),
        },
        ordinary_ground_throw: None,
        ordinary_air_catch: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        muffed_ground_ball: None,
        two_out_contact: None,
        single_delivery: None,
        balk: None,
    }
}
fn plate(before: &GameState, to: Destination) -> RunnerMove {
    RunnerMove {
        runner: Runner {
            player_id: batter(before),
            responsible_pitcher: before.teams[1 - before.half.offense()].pitcher.clone(),
        },
        from: Base::Plate,
        to,
    }
}
fn single(before: &GameState, classified: bool) -> Event {
    assert!(before.bases.iter().all(Option::is_none));
    let id = batter(before);
    let mut e = simple_pa(
        before,
        vec![plate(before, Destination::First)],
        0,
        0,
        vec![
            credit(&id, CreditKind::PlateAppearance),
            credit(&id, CreditKind::AtBat),
            credit(&id, CreditKind::Hit),
            credit(&id, CreditKind::Single),
        ],
    );
    e.sole_safe_hit = classified.then_some(SoleSafeHitFacts { batter_id: id });
    e
}
fn held_ground(before: &GameState) -> Event {
    let id = batter(before);
    let defense = &before.teams[1 - before.half.offense()];
    let thrower = defense.batting_order[3].clone();
    let receiver = defense.batting_order[1].clone();
    let mut moves = vec![plate(before, Destination::Out)];
    for (i, r) in before.bases.iter().enumerate() {
        if let Some(r) = r {
            moves.push(RunnerMove {
                runner: r.clone(),
                from: [Base::First, Base::Second, Base::Third][i],
                to: [Destination::First, Destination::Second, Destination::Third][i],
            });
        }
    }
    let mut e = simple_pa(
        before,
        moves,
        1,
        0,
        vec![
            credit(&id, CreditKind::PlateAppearance),
            credit(&id, CreditKind::AtBat),
            credit(&id, CreditKind::NoHit),
            credit(&receiver, CreditKind::Putout),
            credit(&thrower, CreditKind::Assist),
        ],
    );
    e.ordinary_ground_throw = Some(OrdinaryGroundThrowFacts {
        batter_id: id,
        thrower_id: thrower,
        receiver_id: receiver,
    });
    e
}
fn home_run(before: &GameState) -> Event {
    let id = batter(before);
    let mut moves = vec![plate(before, Destination::Home)];
    let mut credits = vec![
        credit(&id, CreditKind::PlateAppearance),
        credit(&id, CreditKind::AtBat),
        credit(&id, CreditKind::Hit),
        credit(&id, CreditKind::Run),
        credit(&id, CreditKind::Rbi),
    ];
    for (i, r) in before.bases.iter().enumerate() {
        if let Some(r) = r {
            moves.push(RunnerMove {
                runner: r.clone(),
                from: [Base::First, Base::Second, Base::Third][i],
                to: Destination::Home,
            });
            credits.extend([
                credit(&r.player_id, CreditKind::Run),
                credit(&id, CreditKind::Rbi),
            ]);
        }
    }
    let mut e = simple_pa(before, moves.clone(), 0, moves.len() as u8, credits);
    e.kind = EventKind::OutOfParkHomeRun;
    e.sole_home_run = Some(SoleHomeRunFacts { batter_id: id });
    e
}
fn completed_half(start: GameState, use_dp: bool, classified_first: bool) -> Vec<EventTransition> {
    let mut log = vec![apply(&start, single(&start, classified_first)).unwrap()];
    let before = log.last().unwrap().after();
    log.push(apply(before, home_run(before)).unwrap());
    let before = log.last().unwrap().after();
    log.push(apply(before, single(before, true)).unwrap());
    let before = log.last().unwrap().after();
    if use_dp {
        let (e, f) = fixture(before, 0);
        log.push(apply_with_ground_force_double_play_facts(before, e, f).unwrap());
    } else {
        log.push(apply(before, held_ground(before)).unwrap());
        let before = log.last().unwrap().after();
        log.push(apply(before, held_ground(before)).unwrap());
    }
    let before = log.last().unwrap().after();
    log.push(apply(before, held_ground(before)).unwrap());
    assert_eq!(log.last().unwrap().after().half, Half::Bottom);
    assert_eq!(log.last().unwrap().after().outs, 0);
    for pair in log.windows(2) {
        assert_eq!(pair[0].after(), pair[1].before());
    }
    log
}
#[test]
fn positive_completed_half_er_control_and_prior_unknowns_are_not_repaired() {
    let mut start = state(0, 0, Half::Top, 0, true);
    start.inning = 1;
    let control = completed_half(start.clone(), false, true);
    let c = accounting::derive_accounting(&control).unwrap();
    assert_eq!(c.runs, [2, 0]);
    assert_eq!(c.pitchers[1]["hp"].earned_runs, Some(2));
    let log = completed_half(start.clone(), true, true);
    let stats = accounting::derive_accounting(&log).unwrap();
    assert_eq!(stats.runs, [2, 0]);
    assert_eq!(stats.pitchers[1]["hp"].runs_allowed, 2);
    assert_eq!(stats.pitchers[1]["hp"].earned_runs, None);
    assert_eq!(stats.classified_fielding[1]["h1"].putouts, Some(2));
    assert_eq!(stats.classified_fielding[1]["h3"].assists, Some(2));
    let unknown = completed_half(start.clone(), true, false);
    let stats = accounting::derive_accounting(&unknown).unwrap();
    assert_eq!(stats.classified_fielding[1]["h1"].putouts, None);
    assert_eq!(stats.classified_fielding[1]["h3"].assists, None);
    // A prior positive uncertain half remains None even after a later clean,
    // independently certifiable positive half. Qualified new fielding cannot repair it.
    let mut prior = completed_half(start, false, false);
    // Missing sole-hit fielding proof does NOT remove clean station-hit ER
    // coverage. Confirm that existing positive control, then supply genuinely
    // unknown whole-chance review for this separate prior-ER fixture.
    assert_eq!(
        accounting::derive_accounting(&prior).unwrap().pitchers[1]["hp"].earned_runs,
        Some(2)
    );
    let first_after = prior[0].after().clone();
    let mut first = prior[0].event().clone();
    first.fielding_review = FieldingReview::Unknown;
    prior[0] = apply(prior[0].before(), first).unwrap();
    assert_eq!(prior[0].after(), &first_after);
    assert_eq!(
        accounting::derive_accounting(&prior).unwrap().pitchers[1]["hp"].earned_runs,
        None
    );
    for _ in 0..3 {
        let b = prior.last().unwrap().after();
        prior.push(apply(b, held_ground(b)).unwrap());
    }
    let next = completed_half(prior.last().unwrap().after().clone(), false, true);
    prior.extend(next);
    let stats = accounting::derive_accounting(&prior).unwrap();
    assert_eq!(stats.pitchers[1]["hp"].runs_allowed, 4);
    assert_eq!(stats.pitchers[1]["hp"].earned_runs, None);
}
