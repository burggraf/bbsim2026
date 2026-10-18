//! Explicit modeled/reviewed caught-SF facts, never inferred source chronology.
use bbsim_engine::{
    accounting::derive_accounting, apply, apply_with_caught_sacrifice_fly_facts, AirContact, Base,
    CaughtSacrificeFlyFacts, Credit, CreditKind, Delta, Destination, Event, EventKind,
    EventTransition, FieldingReview, GameState, Half, OrdinaryAirCatchFacts, Participants, Runner,
    RunnerMove, SacrificeFlyHandling, SacrificeFlyRetouch, SacrificeFlyRunTiming, TeamState,
    ThirdOut, ThirdOutKind,
};
fn state(mask: u8, outs: u8, half: Half, cursor: u8, dh: bool) -> GameState {
    let teams = ["a", "h"].map(|prefix| {
        let pitcher = format!("{prefix}p");
        let mut order = std::array::from_fn(|i| format!("{prefix}{i}"));
        if !dh {
            order[7] = pitcher.clone();
        }
        TeamState {
            defense: bbsim_engine::DefenseState::LegacyUnassigned,
            batting_order: order,
            next_batter: cursor,
            pitcher: (pitcher).into(),
            dh_slot: dh.then_some(8),
            dh_must_bat: false,
            bench: vec![format!("{prefix}bench")],
            removed: vec![format!("{prefix}removed")],
        }
    });
    let bases = std::array::from_fn(|i| {
        (mask & (1 << i) != 0).then(|| Runner {
            // Exclude both the current and next queued batter from held identities.
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
fn action(before: &GameState, contact: AirContact, scoring: bool) -> Event {
    let offense = &before.teams[before.half.offense()];
    let defense = &before.teams[1 - before.half.offense()];
    let batter = offense.batting_order[usize::from(offense.next_batter)].clone();
    let fielder = defense.batting_order[5].clone();
    let mut moves = vec![RunnerMove {
        runner: Runner {
            player_id: batter.clone(),
            responsible_pitcher: defense
                .pitcher
                .active_id()
                .expect("legacy fixture pitcher")
                .to_owned(),
        },
        from: Base::Plate,
        to: Destination::Out,
    }];
    let mut credits = vec![
        credit(&batter, CreditKind::PlateAppearance),
        credit(&batter, CreditKind::NoHit),
        credit(
            &batter,
            if scoring {
                CreditKind::SacrificeFly
            } else {
                CreditKind::AtBat
            },
        ),
        credit(&fielder, CreditKind::Putout),
    ];
    for (i, runner) in before.bases.iter().enumerate() {
        if let Some(runner) = runner {
            let to = if scoring && i == 2 {
                Destination::Home
            } else {
                [Destination::First, Destination::Second, Destination::Third][i]
            };
            moves.push(RunnerMove {
                runner: runner.clone(),
                from: [Base::First, Base::Second, Base::Third][i],
                to,
            });
            if to == Destination::Home {
                credits.push(credit(&runner.player_id, CreditKind::Run));
                credits.push(credit(&batter, CreditKind::Rbi));
            }
        }
    }
    Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(batter.clone()),
            pitcher: Some(
                defense
                    .pitcher
                    .active_id()
                    .expect("legacy fixture pitcher")
                    .to_owned(),
            ),
        },
        fielding_review: FieldingReview::NoMisplay,
        delta: Delta {
            moves,
            outs: 1,
            runs: u8::from(scoring),
            credits,
            changes: vec![],
            third_out: (before.outs == 2).then(|| ThirdOut {
                runner_id: batter.clone(),
                kind: ThirdOutKind::BatterBeforeFirst,
                home_before_out: vec![],
            }),
        },
        ordinary_air_catch: Some(OrdinaryAirCatchFacts {
            batter_id: batter,
            fielder_id: fielder,
            contact,
        }),
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
fn facts(before: &GameState) -> CaughtSacrificeFlyFacts {
    CaughtSacrificeFlyFacts {
        scoring_runner_id: before.bases[2].as_ref().unwrap().player_id.clone(),
        handling: SacrificeFlyHandling::QualifyingOutfielderOrInfielderInOutfield,
        retouch: SacrificeFlyRetouch::Legal,
        timing: SacrificeFlyRunTiming::AfterCatch,
    }
}
fn expected_after(before: &GameState) -> GameState {
    let mut after = before.clone();
    let offense = before.half.offense();
    after.score[offense] += 1;
    after.outs += 1;
    after.bases[2] = None;
    after.teams[offense].next_batter = (after.teams[offense].next_batter + 1) % 9;
    after.over =
        before.half == Half::Bottom && before.inning >= 9 && after.score[1] > after.score[0];
    after
}
fn rejected(before: &GameState, event: &Event, facts: &CaughtSacrificeFlyFacts) {
    let saved = before.clone();
    assert!(
        apply_with_caught_sacrifice_fly_facts(before, event.clone(), facts.clone()).is_err(),
        "{before:?}/{event:?}/{facts:?}"
    );
    assert_eq!(before, &saved);
}
fn checked(before: &GameState, event: &Event, facts: &CaughtSacrificeFlyFacts) -> EventTransition {
    let saved = before.clone();
    let transition =
        apply_with_caught_sacrifice_fly_facts(before, event.clone(), facts.clone()).unwrap();
    assert_eq!(before, &saved);
    assert_eq!(transition.before(), before);
    assert_eq!(transition.after(), &expected_after(before));
    assert_eq!(transition.caught_sacrifice_fly_facts(), Some(facts));
    assert!(transition.walkoff_hit_facts().is_none());
    assert_eq!(
        EventTransition::try_new_with_caught_sacrifice_fly_facts(
            before.clone(),
            event.clone(),
            expected_after(before),
            facts.clone()
        )
        .unwrap(),
        transition
    );
    assert!(apply(before, event.clone()).is_err());
    assert!(
        EventTransition::try_new(before.clone(), event.clone(), expected_after(before)).is_err()
    );
    assert_eq!(&transition.to_bytes()[..5], b"BBE28");
    assert_eq!(
        EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
        transition
    );
    let offense = before.half.offense();
    let defense = 1 - offense;
    let batter = event.participants.batter.as_ref().unwrap();
    let actual = before.teams[defense].pitcher.active_id().unwrap();
    let responsible = &before.bases[2].as_ref().unwrap().responsible_pitcher;
    let fielder = &event.ordinary_air_catch.as_ref().unwrap().fielder_id;
    let stats = derive_accounting(std::slice::from_ref(&transition)).unwrap();
    let line = &stats.batters[offense][batter];
    assert_eq!(
        (
            line.plate_appearances,
            line.at_bats,
            line.classified_hits,
            line.credited_sacrifice_flies,
            line.credited_rbi,
            line.classified_rbi
        ),
        (1, Some(0), Some(0), 1, 1, Some(1))
    );
    assert_eq!(
        (
            stats.pitchers[defense][actual].batters_faced,
            stats.pitchers[defense][actual].outs_recorded,
            stats.pitchers[defense][actual].at_bats_against,
            stats.pitchers[defense][actual].credited_sacrifice_flies
        ),
        (1, 1, Some(0), 1)
    );
    assert_eq!(stats.pitchers[defense][responsible].runs_allowed, 1);
    assert_eq!(
        stats.batters[offense][&facts.scoring_runner_id].credited_runs,
        1
    );
    assert_eq!(stats.runs[offense], 1);
    assert_eq!(stats.runs[defense], 0);
    assert_eq!(
        transition
            .caught_sacrifice_fly_rbi()
            .unwrap()
            .runs_batted_in[batter],
        1
    );
    assert_eq!(
        transition.rbi_contribution().unwrap().runs_batted_in[batter],
        1
    );
    let proof = transition.fielding_contribution().unwrap();
    assert_eq!(proof.putouts.len(), 1);
    assert_eq!(proof.putouts[fielder], 1);
    assert!(proof.assists.is_empty());
    let coverage = &stats.classified_fielding[defense][fielder];
    assert_eq!(
        (
            coverage.putouts,
            coverage.assists,
            coverage.errors,
            coverage.passed_balls
        ),
        (Some(1), Some(0), Some(0), Some(0))
    );
    assert_eq!(stats.pitchers[defense][responsible].earned_runs, None);
    transition
}
#[test]
fn cartesian_caught_sf_geometry_cursors_contacts_dh_credits_and_permutations() {
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
                    for mask in 4..8 {
                        for contact in [AirContact::Fly, AirContact::LineDrive, AirContact::PopUp] {
                            let mut before = state(mask, outs, half, cursor, dh);
                            before.inning = inning;
                            let event = action(&before, contact, true);
                            let facts = facts(&before);
                            let first = checked(&before, &event, &facts);
                            frames += 1;
                            let mut reverse = event.clone();
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
    assert_eq!(frames, 3456);
}
#[test]
fn absent_context_preserves_legacy_generic_sf_and_held_air_without_certification() {
    let before = state(4, 0, Half::Top, 0, true);
    let event = action(&before, AirContact::Fly, true);
    assert!(
        apply(&before, event.clone()).is_err(),
        "SF plus ordinary-air facts still needs new context"
    );
    let mut generic = event;
    generic.ordinary_air_catch = None;
    let legacy = apply(&before, generic).unwrap();
    assert!(legacy.caught_sacrifice_fly_facts().is_none());
    assert!(legacy.caught_sacrifice_fly_rbi().is_none());
    assert!(legacy.rbi_contribution().is_none());
    assert_eq!(&legacy.to_bytes()[..5], b"BBE25");
    assert!(!format!("{legacy:?}").contains("caught_sacrifice_fly"));
    assert!(!format!("{legacy:#?}").contains("caught_sacrifice_fly"));
    let held = apply(&before, action(&before, AirContact::Fly, false)).unwrap();
    assert_eq!(&held.to_bytes()[..5], b"BBE26");
    assert!(held.caught_sacrifice_fly_facts().is_none());
    assert_eq!(held.after().bases, before.bases);
}
#[test]
fn unknown_or_wrong_facts_refuse_and_late_qualifying_sacrifices_end_the_game() {
    let before = state(4, 0, Half::Top, 0, true);
    let event = action(&before, AirContact::Fly, true);
    let good = facts(&before);
    for variant in 0..7 {
        let mut bad = good.clone();
        match variant {
            0 => bad.handling = SacrificeFlyHandling::Unknown,
            1 => bad.retouch = SacrificeFlyRetouch::Unknown,
            2 => bad.timing = SacrificeFlyRunTiming::Unknown,
            3 => bad.timing = SacrificeFlyRunTiming::BeforeCatch,
            4 => bad.scoring_runner_id.clear(),
            5 => bad.scoring_runner_id = "foreign".into(),
            _ => bad.scoring_runner_id = "a0".into(),
        }
        rejected(&before, &event, &bad);
    }
    for index in 0..event.delta.credits.len() {
        let mut bad = event.clone();
        bad.delta.credits.remove(index);
        rejected(&before, &bad, &good);
    }
    for kind in [
        CreditKind::AtBat,
        CreditKind::Hit,
        CreditKind::SacrificeBunt,
        CreditKind::CaughtStealing,
        CreditKind::Assist,
        CreditKind::Error,
    ] {
        let mut bad = event.clone();
        bad.delta
            .credits
            .push(credit(event.participants.batter.as_ref().unwrap(), kind));
        rejected(&before, &bad, &good);
    }
    for index in 0..event.delta.credits.len() {
        let mut bad = event.clone();
        bad.delta.credits.push(event.delta.credits[index].clone());
        rejected(&before, &bad, &good);
    }
    for defect in 0..12 {
        let mut bad = event.clone();
        match defect {
            0 => bad.ordinary_air_catch = None,
            1 => bad.fielding_review = FieldingReview::Unknown,
            2 => bad.delta.runs = 0,
            3 => bad.delta.outs = 0,
            4 => bad.delta.moves[0].to = Destination::First,
            5 => bad.delta.moves[1].to = Destination::Third,
            6 => bad.delta.moves[1].runner.responsible_pitcher = "other".into(),
            7 => bad.participants.batter = Some("a1".into()),
            8 => bad.participants.pitcher = Some("foreign".into()),
            9 => bad.kind = EventKind::OutOfParkHomeRun,
            10 => bad.ordinary_air_catch.as_mut().unwrap().fielder_id = "h8".into(),
            _ => bad.ordinary_air_catch.as_mut().unwrap().batter_id = "a1".into(),
        }
        rejected(&before, &bad, &good);
    }
    for defect in 0..6 {
        let mut bad = before.clone();
        match defect {
            0 => bad.outs = 2,
            1 => bad.over = true,
            2 => bad.inning = 0,
            3 => bad.teams[0].next_batter = 9,
            4 => bad.bases[2] = None,
            _ => bad.score[0] = u16::MAX,
        }
        rejected(&bad, &event, &good);
    }
    for mask in [5, 6] {
        let before = state(mask, 0, Half::Top, 0, true);
        let mut event = action(&before, AirContact::Fly, true);
        event.delta.moves[1].to = if mask == 5 {
            Destination::Second
        } else {
            Destination::Third
        };
        rejected(&before, &event, &facts(&before));
    }
    let nonthird = state(2, 0, Half::Top, 0, true);
    let mut event = action(&nonthird, AirContact::Fly, false);
    let scorer = nonthird.bases[1].as_ref().unwrap();
    event.delta.moves[1].to = Destination::Home;
    event.delta.runs = 1;
    event.delta.credits.retain(|c| c.kind != CreditKind::AtBat);
    event.delta.credits.extend([
        credit("a0", CreditKind::SacrificeFly),
        credit("a0", CreditKind::Rbi),
        credit(&scorer.player_id, CreditKind::Run),
    ]);
    let mut supplied = good.clone();
    supplied.scoring_runner_id = scorer.player_id.clone();
    rejected(&nonthird, &event, &supplied);
    let mut bottom = state(4, 0, Half::Bottom, 0, true);
    let event = action(&bottom, AirContact::Fly, true);
    let facts = facts(&bottom);
    for inning in 9..=u16::MAX {
        bottom.inning = inning;
        let t =
            apply_with_caught_sacrifice_fly_facts(&bottom, event.clone(), facts.clone()).unwrap();
        assert_eq!(t.after(), &expected_after(&bottom));
    }
}
#[test]
fn new_metadata_codec_and_supplied_after_state_revalidate_every_contract() {
    let before = state(7, 1, Half::Top, 0, true);
    let event = action(&before, AirContact::LineDrive, true);
    let mut supplied = facts(&before);
    let original = supplied.clone();
    let transition = checked(&before, &event, &supplied);
    supplied.scoring_runner_id = "changed-after-construction".into();
    assert_eq!(transition.caught_sacrifice_fly_facts(), Some(&original));
    assert!(format!("{transition:?}").contains("caught_sacrifice_fly"));
    let bytes = transition.to_bytes();
    for length in 0..bytes.len() {
        assert!(EventTransition::from_bytes(&bytes[..length]).is_err());
    }
    let mut extra = bytes.clone();
    extra.push(0);
    assert!(EventTransition::from_bytes(&extra).is_err());
    for offset in 1..=3 {
        for tag in [0, 255] {
            let mut bad = bytes.clone();
            let index = bad.len() - offset;
            bad[index] = tag;
            assert!(EventTransition::from_bytes(&bad).is_err());
        }
    }
    let mut early = bytes.clone();
    let end = early.len() - 1;
    early[end] = 2;
    assert!(EventTransition::from_bytes(&early).is_err());
    let mut bad = bytes.clone();
    let index = bad.len() - 4;
    bad[index] ^= 32;
    assert!(EventTransition::from_bytes(&bad).is_err());
    for version in [b"BBE25", b"BBE26", b"BBE27", b"BBE29"] {
        let mut bad = bytes.clone();
        bad[..5].copy_from_slice(version);
        assert!(EventTransition::from_bytes(&bad).is_err());
    }
    for defect in 0..5 {
        let mut after = expected_after(&before);
        match defect {
            0 => after.score[0] = 0,
            1 => after.outs = 1,
            2 => after.teams[0].next_batter = 0,
            3 => after.bases[0] = None,
            _ => after.over = true,
        }
        assert!(EventTransition::try_new_with_caught_sacrifice_fly_facts(
            before.clone(),
            event.clone(),
            after,
            original.clone()
        )
        .is_err());
    }
}
#[test]
fn named_handling_actor_requires_real_before_defense_eligibility() {
    for half in [Half::Top, Half::Bottom] {
        for dh in [false, true] {
            let before = state(6, 1, half, 2, dh);
            let defense = &before.teams[1 - half.offense()];
            for actor in defense
                .batting_order
                .iter()
                .map(String::as_str)
                .chain(defense.pitcher.active_id())
            {
                let mut event = action(&before, AirContact::Fly, true);
                event.ordinary_air_catch.as_mut().unwrap().fielder_id = actor.to_owned();
                event
                    .delta
                    .credits
                    .iter_mut()
                    .find(|c| c.kind == CreditKind::Putout)
                    .unwrap()
                    .player_id = actor.to_owned();
                if dh && actor == defense.batting_order[8] {
                    rejected(&before, &event, &facts(&before));
                } else {
                    checked(&before, &event, &facts(&before));
                }
            }
            for actor in [
                "",
                "foreign",
                &defense.bench[0],
                &defense.removed[0],
                before.teams[half.offense()].pitcher.active_id().unwrap(),
            ] {
                let mut event = action(&before, AirContact::Fly, true);
                event.ordinary_air_catch.as_mut().unwrap().fielder_id = actor.into();
                event
                    .delta
                    .credits
                    .iter_mut()
                    .find(|c| c.kind == CreditKind::Putout)
                    .unwrap()
                    .player_id = actor.into();
                rejected(&before, &event, &facts(&before));
            }
        }
    }
}
#[test]
fn known_canonical_contradictions_precede_unknown_handling_evidence() {
    use bbsim_engine::TransitionError;
    let before = state(4, 0, Half::Top, 0, true);
    let event = action(&before, AirContact::Fly, true);
    let mut unknown = facts(&before);
    unknown.handling = SacrificeFlyHandling::Unknown;
    let mut invalid = before.clone();
    invalid.teams[0].next_batter = 9;
    assert_eq!(
        apply_with_caught_sacrifice_fly_facts(&invalid, event.clone(), unknown.clone()),
        Err(TransitionError::InvalidState)
    );
    let mut invalid = event.clone();
    invalid.participants.pitcher = Some("foreign".into());
    assert_eq!(
        apply_with_caught_sacrifice_fly_facts(&before, invalid, unknown.clone()),
        Err(TransitionError::InvalidParticipants)
    );
    let mut invalid = event.clone();
    invalid.delta.credits.push(credit("h5", CreditKind::Error));
    assert_eq!(
        apply_with_caught_sacrifice_fly_facts(&before, invalid, unknown.clone()),
        Err(TransitionError::IllegalCredit)
    );
    let mut invalid = event.clone();
    invalid.delta.moves[1].runner.responsible_pitcher = "wrong-charge".into();
    assert_eq!(
        apply_with_caught_sacrifice_fly_facts(&before, invalid, unknown.clone()),
        Err(TransitionError::DeltaMismatch)
    );
    let mut invalid = unknown.clone();
    invalid.scoring_runner_id = "foreign".into();
    assert_eq!(
        apply_with_caught_sacrifice_fly_facts(&before, event.clone(), invalid),
        Err(TransitionError::InvalidParticipants)
    );
    unknown.timing = SacrificeFlyRunTiming::BeforeCatch;
    assert_eq!(
        apply_with_caught_sacrifice_fly_facts(&before, event, unknown),
        Err(TransitionError::IllegalCredit)
    );
}
#[test]
fn completed_reviewed_sf_half_earns_run_without_restoring_unknown_fielding() {
    let mut start = state(0, 0, Half::Top, 0, true);
    start.inning = 1;
    let mut triple = action(&start, AirContact::Fly, false);
    let batter = triple.participants.batter.as_ref().unwrap().clone();
    triple.ordinary_air_catch = None;
    triple.delta.outs = 0;
    triple.delta.moves[0].to = Destination::Third;
    triple.delta.credits = vec![
        credit(&batter, CreditKind::PlateAppearance),
        credit(&batter, CreditKind::Hit),
        credit(&batter, CreditKind::Triple),
    ];
    let first = apply(&start, triple).unwrap();
    let before = first.after();
    let event = action(before, AirContact::Fly, true);
    let sf = apply_with_caught_sacrifice_fly_facts(before, event, facts(before)).unwrap();
    let second = apply(sf.after(), action(sf.after(), AirContact::Fly, false)).unwrap();
    let third = apply(
        second.after(),
        action(second.after(), AirContact::Fly, false),
    )
    .unwrap();
    let log = [first, sf, second, third];
    assert_eq!(log[3].after().half, Half::Bottom);
    assert_eq!(log[3].after().outs, 0);
    let stats = derive_accounting(&log).unwrap();
    let pitcher = start.teams[1].pitcher.active_id().unwrap();
    assert_eq!(stats.runs, [1, 0]);
    assert_eq!(stats.pitchers[1][pitcher].earned_runs, Some(1));
    assert_eq!(stats.pitchers[1][pitcher].runs_allowed, 1);
    assert_eq!(
        stats.batters[0][&log[1].event().participants.batter.clone().unwrap()].classified_rbi,
        Some(1)
    );
    assert_eq!(
        stats.classified_fielding[1][&start.teams[1].batting_order[5]].putouts, None,
        "new qualified SF cannot repair prior unknown PO coverage"
    );
}

#[test]
fn competing_causal_facts_personnel_multi_effects_and_terminal_hit_contexts_refuse() {
    let before = state(7, 0, Half::Top, 0, true);
    let event = action(&before, AirContact::Fly, true);
    let good = facts(&before);
    let batter = event.participants.batter.as_ref().unwrap();
    for cause in 0..6 {
        let mut bad = event.clone();
        match cause {
            0 => {
                bad.sole_safe_hit = Some(bbsim_engine::SoleSafeHitFacts {
                    batter_id: batter.clone(),
                })
            }
            1 => {
                bad.sole_home_run = Some(bbsim_engine::SoleHomeRunFacts {
                    batter_id: batter.clone(),
                })
            }
            2 => {
                bad.protected_award = Some(bbsim_engine::ProtectedAwardFacts {
                    batter_id: batter.clone(),
                })
            }
            3 => {
                bad.caught_strikeout = Some(bbsim_engine::CaughtStrikeoutFacts {
                    batter_id: batter.clone(),
                    catcher_id: "h5".into(),
                })
            }
            4 => {
                bad.ordinary_ground_throw = Some(bbsim_engine::OrdinaryGroundThrowFacts {
                    batter_id: batter.clone(),
                    thrower_id: "h4".into(),
                    receiver_id: "h5".into(),
                })
            }
            _ => {
                bad.muffed_ground_ball = Some(bbsim_engine::MuffedGroundBallFacts {
                    batter_id: batter.clone(),
                    fielder_id: "h5".into(),
                })
            }
        }
        rejected(&before, &bad, &good);
        let mut unknown = good.clone();
        unknown.handling = SacrificeFlyHandling::Unknown;
        assert_eq!(
            apply_with_caught_sacrifice_fly_facts(&before, bad, unknown),
            Err(bbsim_engine::TransitionError::IllegalCredit)
        );
    }
    let mut personnel = event.clone();
    personnel
        .delta
        .changes
        .push(bbsim_engine::PersonnelChange::Pitcher {
            team: bbsim_engine::TeamSide::Home,
            outgoing: "hp".into(),
            incoming: "hbench".into(),
        });
    rejected(&before, &personnel, &good);
    let mut extra_out = event.clone();
    extra_out
        .delta
        .moves
        .iter_mut()
        .find(|m| m.from == Base::First)
        .unwrap()
        .to = Destination::Out;
    extra_out.delta.outs = 2;
    extra_out
        .delta
        .credits
        .push(credit("h5", CreditKind::Putout));
    rejected(&before, &extra_out, &good);
    let mut extra_run = event.clone();
    let second = before.bases[1].as_ref().unwrap();
    extra_run
        .delta
        .moves
        .iter_mut()
        .find(|m| m.from == Base::Second)
        .unwrap()
        .to = Destination::Home;
    extra_run.delta.runs = 2;
    extra_run.delta.credits.extend([
        credit(&second.player_id, CreditKind::Run),
        credit(batter, CreditKind::Rbi),
    ]);
    rejected(&before, &extra_run, &good);
    let two_outs = state(4, 2, Half::Top, 0, true);
    rejected(
        &two_outs,
        &action(&two_outs, AirContact::Fly, true),
        &facts(&two_outs),
    );
    let terminal = bbsim_engine::WalkoffHitFacts {
        winning_runner_id: good.scoring_runner_id.clone(),
        batter_touch: bbsim_engine::NamedBaseTouch {
            runner_id: batter.clone(),
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
        event,
        expected_after(&before),
        terminal
    )
    .is_err());
}
