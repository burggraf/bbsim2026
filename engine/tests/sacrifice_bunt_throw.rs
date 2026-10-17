//! Supplied bunt/scorer/throw assertions, never inferred from a source SH label.
use bbsim_engine::{
    accounting::derive_accounting, apply, apply_with_sacrifice_bunt_throw_facts, Base, Credit,
    CreditKind, Delta, Destination, Event, EventKind, EventTransition, FieldingReview, GameState,
    Half, OrdinaryGroundThrowFacts, Participants, Runner, RunnerMove, SacrificeBuntJudgment,
    SacrificeBuntThrowFacts, TeamState, ThirdOut, ThirdOutKind, TransitionError,
};
fn state(mask: u8, outs: u8, half: Half, cursor: u8, dh: bool) -> GameState {
    let teams = ["a", "h"].map(|prefix| {
        let pitcher = format!("{prefix}p");
        let mut order = std::array::from_fn(|i| format!("{prefix}{i}"));
        if !dh {
            order[7] = pitcher.clone();
        }
        TeamState {
            batting_order: order,
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
fn credit(actor: &str, kind: CreditKind) -> Credit {
    Credit {
        player_id: actor.into(),
        kind,
    }
}
fn batter(before: &GameState) -> String {
    let team = &before.teams[before.half.offense()];
    team.batting_order[usize::from(team.next_batter)].clone()
}
fn plate_move(before: &GameState, to: Destination) -> RunnerMove {
    RunnerMove {
        runner: Runner {
            player_id: batter(before),
            responsible_pitcher: before.teams[1 - before.half.offense()].pitcher.clone(),
        },
        from: Base::Plate,
        to,
    }
}
fn plain(
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
fn facts(before: &GameState, role: u8) -> SacrificeBuntThrowFacts {
    let defense = &before.teams[1 - before.half.offense()];
    SacrificeBuntThrowFacts {
        batter_id: batter(before),
        advancing_runner_id: before.bases[0]
            .as_ref()
            .map_or("absent".into(), |r| r.player_id.clone()),
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
    }
}
fn action(before: &GameState, facts: &SacrificeBuntThrowFacts) -> Event {
    let mut moves = vec![plate_move(before, Destination::Out)];
    for (i, runner) in before.bases.iter().enumerate() {
        if let Some(runner) = runner {
            moves.push(RunnerMove {
                runner: runner.clone(),
                from: [Base::First, Base::Second, Base::Third][i],
                to: if i == 0 {
                    Destination::Second
                } else {
                    [Destination::First, Destination::Second, Destination::Third][i]
                },
            });
        }
    }
    plain(
        before,
        moves,
        1,
        0,
        vec![
            credit(&facts.batter_id, CreditKind::PlateAppearance),
            credit(&facts.batter_id, CreditKind::NoHit),
            credit(&facts.batter_id, CreditKind::SacrificeBunt),
            credit(&facts.receiver_id, CreditKind::Putout),
            credit(&facts.thrower_id, CreditKind::Assist),
        ],
    )
}
fn after(before: &GameState) -> GameState {
    let mut result = before.clone();
    result.outs += 1;
    result.bases[1] = result.bases[0].take();
    let offense = before.half.offense();
    result.teams[offense].next_batter = (result.teams[offense].next_batter + 1) % 9;
    result
}
fn rejected(before: &GameState, event: &Event, facts: &SacrificeBuntThrowFacts) {
    let saved = before.clone();
    assert!(
        apply_with_sacrifice_bunt_throw_facts(before, event.clone(), facts.clone()).is_err(),
        "{before:?}/{event:?}/{facts:?}"
    );
    assert_eq!(before, &saved);
}
fn checked(before: &GameState, event: &Event, facts: &SacrificeBuntThrowFacts) -> EventTransition {
    let saved = before.clone();
    let transition =
        apply_with_sacrifice_bunt_throw_facts(before, event.clone(), facts.clone()).unwrap();
    assert_eq!(before, &saved);
    assert_eq!(transition.before(), before);
    assert_eq!(transition.after(), &after(before));
    assert_eq!(transition.sacrifice_bunt_throw_facts(), Some(facts));
    assert!(transition.caught_sacrifice_fly_facts().is_none());
    assert!(transition.walkoff_hit_facts().is_none());
    assert_eq!(
        EventTransition::try_new_with_sacrifice_bunt_throw_facts(
            before.clone(),
            event.clone(),
            after(before),
            facts.clone()
        )
        .unwrap(),
        transition
    );
    assert_eq!(&transition.to_bytes()[..5], b"BBE29");
    assert_eq!(
        EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
        transition
    );
    let proof = transition.sacrifice_bunt_throw_fielding().unwrap();
    assert_eq!(proof.putouts.len(), 1);
    assert_eq!(proof.putouts[&facts.receiver_id], 1);
    assert_eq!(proof.assists.len(), 1);
    assert_eq!(proof.assists[&facts.thrower_id], 1);
    assert_eq!(transition.fielding_contribution(), Some(proof));
    assert_eq!(
        transition.sacrifice_bunt_throw_rbi(),
        transition.rbi_contribution()
    );
    let rbi = transition.sacrifice_bunt_throw_rbi().unwrap();
    assert_eq!(rbi.runs_batted_in.len(), 1);
    assert_eq!(rbi.runs_batted_in[&facts.batter_id], 0);
    let stats = derive_accounting(std::slice::from_ref(&transition)).unwrap();
    let offense = before.half.offense();
    let defense = 1 - offense;
    let line = &stats.batters[offense][&facts.batter_id];
    assert_eq!(
        (
            line.plate_appearances,
            line.at_bats,
            line.classified_hits,
            line.credited_sacrifice_bunts,
            line.credited_rbi,
            line.classified_rbi
        ),
        (1, Some(0), Some(0), 1, 0, Some(0))
    );
    let pitch = &stats.pitchers[defense][&event.participants.pitcher];
    assert_eq!(
        (
            pitch.batters_faced,
            pitch.outs_recorded,
            pitch.at_bats_against,
            pitch.credited_sacrifice_bunts,
            pitch.runs_allowed
        ),
        (1, 1, Some(0), 1, 0)
    );
    assert_eq!(stats.runs, [0, 0]);
    for (id, po, assists) in [(&facts.receiver_id, 1, 0), (&facts.thrower_id, 0, 1)] {
        let field = &stats.classified_fielding[defense][id];
        assert_eq!(
            (
                field.putouts,
                field.assists,
                field.errors,
                field.passed_balls
            ),
            (Some(po), Some(assists), Some(0), Some(0))
        );
    }
    transition
}
#[test]
fn independent_geometry_cursors_dh_fielding_roles_and_vector_permutations() {
    let mut frames = 0;
    for (half, inning) in [
        (Half::Top, 2),
        (Half::Bottom, 2),
        (Half::Top, 9),
        (Half::Top, 10),
    ] {
        for dh in [false, true] {
            for cursor in 0..9 {
                for outs in 0..2 {
                    for mask in [1, 5] {
                        for role in 0..3 {
                            let mut before = state(mask, outs, half, cursor, dh);
                            before.inning = inning;
                            let facts = facts(&before, role);
                            let event = action(&before, &facts);
                            let first = checked(&before, &event, &facts);
                            frames += 1;
                            let mut reverse = event;
                            reverse.delta.moves.reverse();
                            reverse.delta.credits.reverse();
                            let second = checked(&before, &reverse, &facts);
                            frames += 1;
                            assert_eq!(first.after(), second.after());
                            assert_eq!(
                                derive_accounting(&[first]).unwrap(),
                                derive_accounting(&[second]).unwrap()
                            );
                        }
                    }
                }
            }
        }
    }
    assert_eq!(frames, 1728);
}
#[test]
fn absent_context_keeps_generic_sh_and_non_bunt_throw_semantics_separate() {
    let before = state(5, 0, Half::Top, 0, true);
    let facts = facts(&before, 0);
    let event = action(&before, &facts);
    let generic = apply(&before, event.clone()).unwrap();
    assert!(generic.sacrifice_bunt_throw_facts().is_none());
    assert!(generic.sacrifice_bunt_throw_fielding().is_none());
    assert!(generic.sacrifice_bunt_throw_rbi().is_none());
    assert!(generic.fielding_contribution().is_none());
    let conserved_zero = generic.rbi_contribution().unwrap();
    assert_eq!(conserved_zero.runs_batted_in.len(), 1);
    assert_eq!(conserved_zero.runs_batted_in[&facts.batter_id], 0);
    assert_eq!(&generic.to_bytes()[..5], b"BBE25");
    assert!(!format!("{generic:?}").contains("sacrifice_bunt_throw"));
    assert!(!format!("{generic:#?}").contains("sacrifice_bunt_throw"));
    let mut wrong = event;
    wrong.ordinary_ground_throw = Some(OrdinaryGroundThrowFacts {
        batter_id: facts.batter_id.clone(),
        thrower_id: facts.thrower_id.clone(),
        receiver_id: facts.receiver_id.clone(),
    });
    assert_eq!(
        apply(&before, wrong.clone()),
        Err(TransitionError::IllegalCredit)
    );
    rejected(&before, &wrong, &facts);
}
#[test]
fn missing_unknown_wrong_units_participants_charges_and_geometry_refuse() {
    let before = state(5, 0, Half::Top, 0, true);
    let good = facts(&before, 0);
    let event = action(&before, &good);
    for judgment in [
        SacrificeBuntJudgment::Unknown,
        SacrificeBuntJudgment::ExclusivelyForBaseHit,
    ] {
        let mut bad = good.clone();
        bad.judgment = judgment;
        rejected(&before, &event, &bad);
    }
    for field in 0..4 {
        let mut bad = good.clone();
        match field {
            0 => bad.batter_id = "foreign".into(),
            1 => bad.advancing_runner_id = "foreign".into(),
            2 => bad.thrower_id = "foreign".into(),
            _ => bad.receiver_id = "foreign".into(),
        }
        rejected(&before, &event, &bad);
    }
    for index in 0..event.delta.credits.len() {
        let mut bad = event.clone();
        bad.delta.credits.remove(index);
        rejected(&before, &bad, &good);
        let mut bad = event.clone();
        bad.delta.credits.push(event.delta.credits[index].clone());
        rejected(&before, &bad, &good);
        let mut bad = event.clone();
        bad.delta.credits[index].player_id = "foreign".into();
        rejected(&before, &bad, &good);
    }
    for kind in [
        CreditKind::AtBat,
        CreditKind::Hit,
        CreditKind::SacrificeFly,
        CreditKind::Rbi,
        CreditKind::Run,
        CreditKind::StolenBase,
        CreditKind::CaughtStealing,
        CreditKind::Error,
        CreditKind::PassedBall,
        CreditKind::GroundedIntoDoublePlay,
        CreditKind::Walk,
        CreditKind::Strikeout,
        CreditKind::HitByPitch,
    ] {
        let mut bad = event.clone();
        bad.delta.credits.push(credit(&good.batter_id, kind));
        rejected(&before, &bad, &good);
    }
    for defect in 0..10 {
        let mut bad = event.clone();
        match defect {
            0 => bad.fielding_review = FieldingReview::Unknown,
            1 => bad.delta.moves[0].to = Destination::First,
            2 => bad.delta.moves[1].to = Destination::First,
            3 => bad.delta.moves[1].to = Destination::Third,
            4 => bad.delta.moves[1].runner.responsible_pitcher = "wrong-charge".into(),
            5 => {
                bad.delta.moves.pop();
            }
            6 => bad.participants.pitcher = "foreign".into(),
            7 => bad.participants.batter = Some("a1".into()),
            8 => bad.kind = EventKind::OutOfParkHomeRun,
            _ => bad.delta.outs = 0,
        }
        rejected(&before, &bad, &good);
    }
    for mask in [0, 2, 3, 4, 6, 7] {
        let invalid = state(mask, 0, Half::Top, 0, true);
        let supplied = facts(&invalid, 0);
        rejected(&invalid, &action(&invalid, &supplied), &supplied);
    }
    for defect in 0..3 {
        let mut invalid = before.clone();
        match defect {
            0 => invalid.inning = 0,
            1 => invalid.over = true,
            _ => invalid.teams[0].next_batter = 9,
        }
        rejected(&invalid, &event, &good);
    }
    let mut bottom = state(1, 0, Half::Bottom, 0, true);
    let supplied = facts(&bottom, 0);
    let event = action(&bottom, &supplied);
    for inning in 9..=u16::MAX {
        bottom.inning = inning;
        let t = apply_with_sacrifice_bunt_throw_facts(&bottom, event.clone(), supplied.clone())
            .unwrap();
        assert_eq!(t.after(), &after(&bottom));
    }
}
#[test]
fn named_fielders_must_be_distinct_before_eligible_actors() {
    for half in [Half::Top, Half::Bottom] {
        for dh in [false, true] {
            let before = state(5, 1, half, 2, dh);
            let defense = &before.teams[1 - half.offense()];
            for receiver in [false, true] {
                for actor in defense.batting_order.iter().chain([&defense.pitcher]) {
                    let mut supplied = facts(&before, 0);
                    if receiver {
                        supplied.receiver_id = actor.clone();
                    } else {
                        supplied.thrower_id = actor.clone();
                    }
                    let event = action(&before, &supplied);
                    if (dh && actor == &defense.batting_order[8])
                        || supplied.thrower_id == supplied.receiver_id
                    {
                        rejected(&before, &event, &supplied);
                    } else {
                        checked(&before, &event, &supplied);
                    }
                }
            }
            for actor in [
                "",
                "foreign",
                &defense.bench[0],
                &defense.removed[0],
                &before.teams[half.offense()].pitcher,
            ] {
                for receiver in [false, true] {
                    let mut supplied = facts(&before, 0);
                    if receiver {
                        supplied.receiver_id = actor.into();
                    } else {
                        supplied.thrower_id = actor.into();
                    }
                    rejected(&before, &action(&before, &supplied), &supplied);
                }
            }
        }
    }
}
#[test]
fn known_canonical_errors_precede_unknown_scorer_judgment() {
    let before = state(1, 0, Half::Top, 0, true);
    let mut supplied = facts(&before, 0);
    let event = action(&before, &supplied);
    supplied.judgment = SacrificeBuntJudgment::Unknown;
    let mut invalid = before.clone();
    invalid.teams[0].next_batter = 9;
    assert_eq!(
        apply_with_sacrifice_bunt_throw_facts(&invalid, event.clone(), supplied.clone()),
        Err(TransitionError::InvalidState)
    );
    let mut invalid = event.clone();
    invalid.participants.pitcher = "foreign".into();
    assert_eq!(
        apply_with_sacrifice_bunt_throw_facts(&before, invalid, supplied.clone()),
        Err(TransitionError::InvalidParticipants)
    );
    let mut invalid = event.clone();
    invalid.delta.credits.push(credit("h3", CreditKind::Error));
    assert_eq!(
        apply_with_sacrifice_bunt_throw_facts(&before, invalid, supplied.clone()),
        Err(TransitionError::IllegalCredit)
    );
    let mut invalid = event.clone();
    invalid.delta.moves[1].runner.responsible_pitcher = "wrong-charge".into();
    assert_eq!(
        apply_with_sacrifice_bunt_throw_facts(&before, invalid, supplied.clone()),
        Err(TransitionError::DeltaMismatch)
    );
    supplied.advancing_runner_id = "foreign".into();
    assert_eq!(
        apply_with_sacrifice_bunt_throw_facts(&before, event, supplied),
        Err(TransitionError::InvalidParticipants)
    );
}
#[test]
fn competing_causes_personnel_extra_effects_and_terminal_contexts_refuse() {
    let before = state(5, 0, Half::Top, 0, true);
    let supplied = facts(&before, 0);
    let event = action(&before, &supplied);
    for cause in 0..7 {
        let mut bad = event.clone();
        match cause {
            0 => {
                bad.ordinary_ground_throw = Some(OrdinaryGroundThrowFacts {
                    batter_id: supplied.batter_id.clone(),
                    thrower_id: supplied.thrower_id.clone(),
                    receiver_id: supplied.receiver_id.clone(),
                })
            }
            1 => {
                bad.ordinary_air_catch = Some(bbsim_engine::OrdinaryAirCatchFacts {
                    batter_id: supplied.batter_id.clone(),
                    fielder_id: supplied.receiver_id.clone(),
                    contact: bbsim_engine::AirContact::Fly,
                })
            }
            2 => {
                bad.caught_strikeout = Some(bbsim_engine::CaughtStrikeoutFacts {
                    batter_id: supplied.batter_id.clone(),
                    catcher_id: supplied.receiver_id.clone(),
                })
            }
            3 => {
                bad.sole_safe_hit = Some(bbsim_engine::SoleSafeHitFacts {
                    batter_id: supplied.batter_id.clone(),
                })
            }
            4 => {
                bad.sole_home_run = Some(bbsim_engine::SoleHomeRunFacts {
                    batter_id: supplied.batter_id.clone(),
                })
            }
            5 => {
                bad.protected_award = Some(bbsim_engine::ProtectedAwardFacts {
                    batter_id: supplied.batter_id.clone(),
                })
            }
            _ => {
                bad.muffed_ground_ball = Some(bbsim_engine::MuffedGroundBallFacts {
                    batter_id: supplied.batter_id.clone(),
                    fielder_id: supplied.thrower_id.clone(),
                })
            }
        }
        rejected(&before, &bad, &supplied);
        let mut unknown = supplied.clone();
        unknown.judgment = SacrificeBuntJudgment::Unknown;
        assert_eq!(
            apply_with_sacrifice_bunt_throw_facts(&before, bad, unknown),
            Err(TransitionError::IllegalCredit)
        );
    }
    let mut bad = event.clone();
    bad.delta
        .changes
        .push(bbsim_engine::PersonnelChange::Pitcher {
            team: bbsim_engine::TeamSide::Home,
            outgoing: "hp".into(),
            incoming: "hbench".into(),
        });
    rejected(&before, &bad, &supplied);
    let mut bad = event.clone();
    bad.delta.moves[1].to = Destination::Out;
    bad.delta.outs = 2;
    bad.delta
        .credits
        .push(credit(&supplied.receiver_id, CreditKind::Putout));
    rejected(&before, &bad, &supplied);
    let mut bad = event.clone();
    let runner = before.bases[2].as_ref().unwrap();
    bad.delta.moves[2].to = Destination::Home;
    bad.delta.runs = 1;
    bad.delta.credits.extend([
        credit(&runner.player_id, CreditKind::Run),
        credit(&supplied.batter_id, CreditKind::Rbi),
    ]);
    rejected(&before, &bad, &supplied);
    let two_outs = state(1, 2, Half::Top, 0, true);
    let supplied_two = facts(&two_outs, 0);
    rejected(&two_outs, &action(&two_outs, &supplied_two), &supplied_two);
    let mut bad = event.clone();
    bad.single_delivery = Some(bbsim_engine::SingleDeliveryFacts {
        context: bbsim_engine::DeliveryContext::RunnerOnly,
        handling: bbsim_engine::HandlingAssessment::PitcherWild,
        catcher_id: "h1".into(),
        advances: vec![],
        pre_delivery_steals: None,
        recovery: bbsim_engine::RecoveryReview::Unknown,
    });
    rejected(&before, &bad, &supplied);
    let mut bad = event.clone();
    bad.balk = Some(bbsim_engine::BalkFacts {
        cause: bbsim_engine::BalkCause::Unknown,
        disposition: bbsim_engine::BalkDisposition::Unknown,
        resolution: bbsim_engine::BalkResolution::Unknown,
    });
    rejected(&before, &bad, &supplied);
    let terminal = bbsim_engine::WalkoffHitFacts {
        winning_runner_id: supplied.advancing_runner_id.clone(),
        batter_touch: bbsim_engine::NamedBaseTouch {
            runner_id: supplied.batter_id.clone(),
            base: Destination::First,
        },
        timing: bbsim_engine::NaturalCourseTouchTiming::ByWinningRun,
    };
    assert!(
        bbsim_engine::apply_with_walkoff_hit_facts(&before, event.clone(), terminal.clone())
            .is_err()
    );
    assert!(EventTransition::try_new_with_walkoff_hit_facts(
        before.clone(),
        event.clone(),
        after(&before),
        terminal
    )
    .is_err());
    let sf = bbsim_engine::CaughtSacrificeFlyFacts {
        scoring_runner_id: runner.player_id.clone(),
        handling: bbsim_engine::SacrificeFlyHandling::QualifyingOutfielderOrInfielderInOutfield,
        retouch: bbsim_engine::SacrificeFlyRetouch::Legal,
        timing: bbsim_engine::SacrificeFlyRunTiming::AfterCatch,
    };
    assert!(bbsim_engine::apply_with_caught_sacrifice_fly_facts(
        &before,
        event.clone(),
        sf.clone()
    )
    .is_err());
    assert!(EventTransition::try_new_with_caught_sacrifice_fly_facts(
        before.clone(),
        event,
        after(&before),
        sf
    )
    .is_err());
}
#[test]
fn immutable_context_codec_and_supplied_after_state_revalidate() {
    let before = state(5, 1, Half::Top, 0, true);
    let mut supplied = facts(&before, 0);
    let original = supplied.clone();
    let event = action(&before, &supplied);
    let transition = checked(&before, &event, &supplied);
    supplied.thrower_id = "changed".into();
    assert_eq!(transition.sacrifice_bunt_throw_facts(), Some(&original));
    assert!(format!("{transition:?}").contains("sacrifice_bunt_throw"));
    let bytes = transition.to_bytes();
    for length in 0..bytes.len() {
        assert!(EventTransition::from_bytes(&bytes[..length]).is_err());
    }
    let mut extra = bytes.clone();
    extra.push(0);
    assert!(EventTransition::from_bytes(&extra).is_err());
    for tag in [0, 2, 255] {
        let mut bad = bytes.clone();
        let end = bad.len() - 1;
        bad[end] = tag;
        assert!(EventTransition::from_bytes(&bad).is_err());
    }
    let mut bad = bytes.clone();
    let index = bad.len() - 2;
    bad[index] ^= 32;
    assert!(EventTransition::from_bytes(&bad).is_err());
    for version in [b"BBE25", b"BBE26", b"BBE27", b"BBE28", b"BBE30"] {
        let mut bad = bytes.clone();
        bad[..5].copy_from_slice(version);
        assert!(EventTransition::from_bytes(&bad).is_err());
    }
    for defect in 0..5 {
        let mut invalid = after(&before);
        match defect {
            0 => invalid.score[0] += 1,
            1 => invalid.outs = 1,
            2 => invalid.teams[0].next_batter = 0,
            3 => invalid.bases[2] = None,
            _ => invalid.over = true,
        }
        assert!(EventTransition::try_new_with_sacrifice_bunt_throw_facts(
            before.clone(),
            event.clone(),
            invalid,
            original.clone()
        )
        .is_err());
    }
}
fn held_ground(before: &GameState) -> Event {
    let supplied = facts(before, 0);
    let mut event = action(before, &supplied);
    for movement in &mut event.delta.moves {
        if movement.from == Base::First {
            movement.to = Destination::First;
        }
    }
    for unit in &mut event.delta.credits {
        if unit.kind == CreditKind::SacrificeBunt {
            unit.kind = CreditKind::AtBat;
        }
    }
    event.ordinary_ground_throw = Some(OrdinaryGroundThrowFacts {
        batter_id: supplied.batter_id,
        thrower_id: supplied.thrower_id,
        receiver_id: supplied.receiver_id,
    });
    event
}
fn home_run(before: &GameState) -> Event {
    let id = batter(before);
    let mut moves = vec![plate_move(before, Destination::Home)];
    let mut credits = vec![
        credit(&id, CreditKind::PlateAppearance),
        credit(&id, CreditKind::AtBat),
        credit(&id, CreditKind::Hit),
        credit(&id, CreditKind::Run),
        credit(&id, CreditKind::Rbi),
    ];
    for (i, runner) in before.bases.iter().enumerate() {
        if let Some(runner) = runner {
            moves.push(RunnerMove {
                runner: runner.clone(),
                from: [Base::First, Base::Second, Base::Third][i],
                to: Destination::Home,
            });
            credits.extend([
                credit(&runner.player_id, CreditKind::Run),
                credit(&id, CreditKind::Rbi),
            ]);
        }
    }
    let runs = moves.len() as u8;
    let mut event = plain(before, moves, 0, runs, credits);
    event.kind = EventKind::OutOfParkHomeRun;
    event.sole_home_run = Some(bbsim_engine::SoleHomeRunFacts { batter_id: id });
    event
}
fn complete_half(first: EventTransition, with_bunt: bool) -> Vec<EventTransition> {
    let mut log = vec![first];
    let before = log.last().unwrap().after();
    let next = if with_bunt {
        let supplied = facts(before, 0);
        apply_with_sacrifice_bunt_throw_facts(before, action(before, &supplied), supplied).unwrap()
    } else {
        apply(before, held_ground(before)).unwrap()
    };
    log.push(next);
    let before = log.last().unwrap().after();
    log.push(apply(before, home_run(before)).unwrap());
    for _ in 0..2 {
        let before = log.last().unwrap().after();
        log.push(apply(before, held_ground(before)).unwrap());
    }
    assert_eq!(log.last().unwrap().after().half, Half::Bottom);
    assert_eq!(log.last().unwrap().after().outs, 0);
    for pair in log.windows(2) {
        assert_eq!(pair[0].after(), pair[1].before());
    }
    log
}
#[test]
fn completed_half_has_positive_er_control_and_never_repairs_prior_none_fielding() {
    let mut start = state(0, 0, Half::Top, 0, true);
    start.inning = 1;
    let id = batter(&start);
    let mut single = plain(
        &start,
        vec![plate_move(&start, Destination::First)],
        0,
        0,
        vec![
            credit(&id, CreditKind::PlateAppearance),
            credit(&id, CreditKind::AtBat),
            credit(&id, CreditKind::Hit),
            credit(&id, CreditKind::Single),
        ],
    );
    single.sole_safe_hit = Some(bbsim_engine::SoleSafeHitFacts { batter_id: id });
    let first = apply(&start, single.clone()).unwrap();
    let control = complete_half(first.clone(), false);
    let result = derive_accounting(&control).unwrap();
    assert_eq!(result.runs, [2, 0]);
    assert_eq!(result.pitchers[1]["hp"].earned_runs, Some(2));
    let log = complete_half(first, true);
    let stats = derive_accounting(&log).unwrap();
    assert_eq!(stats.runs, [2, 0]);
    assert_eq!(stats.pitchers[1]["hp"].runs_allowed, 2);
    assert_eq!(stats.pitchers[1]["hp"].earned_runs, None);
    assert_eq!(stats.classified_fielding[1]["h1"].putouts, Some(3));
    assert_eq!(stats.classified_fielding[1]["h3"].assists, Some(3));
    single.sole_safe_hit = None;
    let unknown = complete_half(apply(&start, single).unwrap(), true);
    let stats = derive_accounting(&unknown).unwrap();
    assert_eq!(stats.classified_fielding[1]["h1"].putouts, None);
    assert_eq!(stats.classified_fielding[1]["h3"].assists, None);
}
