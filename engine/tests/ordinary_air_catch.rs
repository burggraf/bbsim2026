use bbsim_engine::{
    accounting::{derive_accounting, FieldingContribution},
    apply, AirContact, Base, Credit, CreditKind, Delta, Destination, Event, EventKind,
    EventTransition, FieldingReview, GameState, Half, OrdinaryAirCatchFacts, Participants, Runner,
    RunnerMove, TeamState, ThirdOut, ThirdOutKind, TransitionError,
};
use std::collections::BTreeMap;

fn state(mask: u8, outs: u8, half: Half) -> GameState {
    let team = |prefix: &str| TeamState {
        defense: bbsim_engine::DefenseState::LegacyUnassigned,
        batting_order: std::array::from_fn(|i| format!("{prefix}-{i}")),
        next_batter: 0,
        pitcher: (format!("{prefix}-pitcher")).into(),
        dh_slot: Some(8),
        dh_must_bat: false,
        bench: vec![format!("{prefix}-bench")],
        removed: vec![format!("{prefix}-removed")],
    };
    let teams = [team("away"), team("home")];
    GameState {
        inning: 2,
        half,
        outs,
        score: [0, 0],
        bases: std::array::from_fn(|i| {
            (mask & (1 << i) != 0).then(|| Runner {
                player_id: teams[half.offense()].batting_order[i + 3].clone(),
                responsible_pitcher: "prior-pitcher".into(),
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
fn event(before: &GameState, contact: AirContact) -> Event {
    let offense = &before.teams[before.half.offense()];
    let defense = &before.teams[1 - before.half.offense()];
    let batter = offense.batting_order[usize::from(offense.next_batter)].clone();
    let fielder = defense.batting_order[5].clone();
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
            moves: vec![RunnerMove {
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
            }],
            outs: 1,
            runs: 0,
            credits: vec![
                credit(&batter, CreditKind::PlateAppearance),
                credit(&batter, CreditKind::AtBat),
                credit(&batter, CreditKind::NoHit),
                credit(&fielder, CreditKind::Putout),
            ],
            changes: vec![],
            third_out: (before.outs == 2).then(|| ThirdOut {
                runner_id: batter.clone(),
                kind: ThirdOutKind::BatterBeforeFirst,
                home_before_out: vec![],
            }),
        },
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        ordinary_ground_throw: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        muffed_ground_ball: None,
        ordinary_air_catch: Some(OrdinaryAirCatchFacts {
            batter_id: batter,
            fielder_id: fielder,
            contact,
        }),
    }
}

#[test]
fn direct_air_catch_has_independent_named_po_in_all_base_out_contexts() {
    for contact in [AirContact::Fly, AirContact::LineDrive, AirContact::PopUp] {
        for half in [Half::Top, Half::Bottom] {
            for outs in 0..3 {
                for mask in 0..8 {
                    let before = state(mask, outs, half);
                    let action = event(&before, contact);
                    let fielder = action
                        .ordinary_air_catch
                        .as_ref()
                        .unwrap()
                        .fielder_id
                        .clone();
                    let t = apply(&before, action.clone()).unwrap();
                    let expected = Some(FieldingContribution {
                        putouts: BTreeMap::from([(fielder, 1)]),
                        assists: BTreeMap::new(),
                    });
                    assert_eq!(t.ordinary_air_fielding(), expected);
                    assert_eq!(t.fielding_contribution(), expected);
                    assert_eq!(t.ordinary_ground_fielding(), None);
                    assert_eq!(t.caught_strikeout_fielding(), None);
                    assert_eq!(
                        EventTransition::try_new(before.clone(), action, t.after().clone())
                            .unwrap(),
                        t
                    );
                    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
                    assert_eq!(t.event().delta.outs, 1);
                    assert_eq!(t.after().teams[half.offense()].next_batter, 1);
                }
            }
        }
    }
}

#[test]
fn held_runner_facts_reject_an_otherwise_legal_extra_advance() {
    let before = state(1, 1, Half::Top);
    let mut action = event(&before, AirContact::Fly);
    action.delta.moves.push(RunnerMove {
        runner: before.bases[0].clone().unwrap(),
        from: Base::First,
        to: Destination::Second,
    });
    assert_eq!(apply(&before, action), Err(TransitionError::DeltaMismatch));
}

#[test]
fn reviewed_air_catch_requires_exact_pa_ab_nohit_po_and_zero_assists() {
    let before = state(7, 0, Half::Top);
    for missing in [CreditKind::AtBat, CreditKind::NoHit, CreditKind::Putout] {
        let mut action = event(&before, AirContact::LineDrive);
        action.delta.credits.retain(|c| c.kind != missing);
        assert_eq!(apply(&before, action), Err(TransitionError::IllegalCredit));
    }
    let mut action = event(&before, AirContact::LineDrive);
    action
        .delta
        .credits
        .push(credit("home-4", CreditKind::Assist));
    assert_eq!(apply(&before, action), Err(TransitionError::IllegalCredit));
}

#[test]
fn actor_local_fielding_unknown_is_not_repaired_by_a_later_air_catch() {
    let mut before = state(0, 0, Half::Top);
    let mut log = Vec::new();
    for index in 0..3 {
        let mut action = event(&before, AirContact::PopUp);
        if index == 1 {
            action.ordinary_air_catch = None;
        }
        let t = apply(&before, action).unwrap();
        before = t.after().clone();
        log.push(t);
    }
    let folded = derive_accounting(&log).unwrap();
    let fielder = &folded.classified_fielding[1]["home-5"];
    assert_eq!(fielder.observed_events, 3);
    assert_eq!((fielder.putouts, fielder.assists), (None, None));
    assert_eq!((fielder.errors, fielder.passed_balls), (Some(0), Some(0)));
    assert_eq!(folded.fielders[1]["home-5"].credited_putouts, 3);
    assert_eq!(folded.pitchers[1]["home-pitcher"].outs_recorded, 3);
}

#[test]
fn named_catching_fielder_must_match_the_putout_not_be_inferred_from_it() {
    let before = state(7, 1, Half::Top);
    let mut action = event(&before, AirContact::Fly);
    action.ordinary_air_catch.as_mut().unwrap().fielder_id = "home-4".into();
    assert_eq!(apply(&before, action), Err(TransitionError::IllegalCredit));
}

fn scoring_half(with_facts: bool, review: FieldingReview) -> Vec<EventTransition> {
    let mut before = state(0, 0, Half::Top);
    before.inning = 1;
    let mut home_run = event(&before, AirContact::Fly);
    let batter = home_run.participants.batter.clone().unwrap();
    home_run.ordinary_air_catch = None;
    home_run.kind = EventKind::OutOfParkHomeRun;
    home_run.delta.outs = 0;
    home_run.delta.runs = 1;
    home_run.delta.moves[0].to = Destination::Home;
    home_run
        .delta
        .credits
        .retain(|c| matches!(c.kind, CreditKind::PlateAppearance | CreditKind::AtBat));
    home_run.delta.credits.extend([
        credit(&batter, CreditKind::Hit),
        credit(&batter, CreditKind::Run),
        credit(&batter, CreditKind::Rbi),
    ]);
    home_run.sole_home_run = Some(bbsim_engine::SoleHomeRunFacts { batter_id: batter });
    let t = apply(&before, home_run).unwrap();
    before = t.after().clone();
    let mut log = vec![t];
    for index in 0..3 {
        let mut action = event(&before, AirContact::Fly);
        if index == 0 {
            action.fielding_review = review;
            if !with_facts {
                action.ordinary_air_catch = None;
            }
        }
        let t = apply(&before, action).unwrap();
        before = t.after().clone();
        log.push(t);
    }
    log
}

#[test]
fn fully_observed_clean_scoring_half_counts_air_catches_without_er_uncertainty() {
    let log = scoring_half(true, FieldingReview::NoMisplay);
    let folded = derive_accounting(&log).unwrap();
    let pitcher = &folded.pitchers[1]["home-pitcher"];
    assert_eq!(
        (
            pitcher.batters_faced,
            pitcher.outs_recorded,
            pitcher.runs_allowed
        ),
        (4, 3, 1)
    );
    assert_eq!(pitcher.earned_runs, Some(1));
    assert_eq!(folded.classified_fielding[1]["home-5"].putouts, Some(3));
}

#[test]
fn errorless_er_requires_whole_chance_review_not_air_catch_details() {
    for (facts, review) in [
        (false, FieldingReview::NoMisplay),
        (true, FieldingReview::Unknown),
    ] {
        let log = scoring_half(facts, review);
        let folded = derive_accounting(&log).unwrap();
        assert_eq!(folded.pitchers[1]["home-pitcher"].runs_allowed, 1);
        assert_eq!(
            folded.pitchers[1]["home-pitcher"].earned_runs,
            if review == FieldingReview::NoMisplay {
                Some(1)
            } else {
                None
            }
        );
        assert_eq!(folded.classified_fielding[1]["home-5"].putouts, None);
    }
}

#[test]
fn explicit_air_facts_survive_the_new_codec_version() {
    let before = state(3, 2, Half::Top);
    let t = apply(&before, event(&before, AirContact::LineDrive)).unwrap();
    assert_eq!(&t.to_bytes()[..5], b"BBE26");
    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
}

#[test]
fn no_air_records_keep_legacy_codec_and_debug_shape() {
    let before = state(0, 1, Half::Top);
    let mut action = event(&before, AirContact::Fly);
    action.ordinary_air_catch = None;
    let t = apply(&before, action).unwrap();
    assert_eq!(&t.to_bytes()[..5], b"BBE25");
    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
    assert!(!format!("{:#?}", t.event()).contains("ordinary_air_catch"));
}

#[test]
fn missing_air_facts_and_unknown_review_do_not_certify_fielding() {
    let before = state(5, 1, Half::Bottom);
    let mut action = event(&before, AirContact::PopUp);
    action.ordinary_air_catch = None;
    let legacy = apply(&before, action).unwrap();
    assert_eq!(legacy.ordinary_air_fielding(), None);
    assert_eq!(legacy.fielding_contribution(), None);
    let mut action = event(&before, AirContact::PopUp);
    action.fielding_review = FieldingReview::Unknown;
    let unknown = apply(&before, action).unwrap();
    assert_eq!(unknown.ordinary_air_fielding(), None);
    assert_eq!(unknown.fielding_contribution(), None);
}

#[test]
fn held_movements_and_credit_order_preserve_physical_state_and_independent_boxes() {
    for contact in [AirContact::Fly, AirContact::LineDrive, AirContact::PopUp] {
        for half in [Half::Top, Half::Bottom] {
            for outs in 0..3 {
                for mask in 0..8 {
                    let before = state(mask, outs, half);
                    let reference = apply(&before, event(&before, contact)).unwrap();
                    for holds in [false, true] {
                        for reverse_moves in [false, true] {
                            for reverse_credits in [false, true] {
                                let mut action = event(&before, contact);
                                if holds {
                                    for (i, from) in [Base::First, Base::Second, Base::Third]
                                        .into_iter()
                                        .enumerate()
                                    {
                                        if let Some(runner) = &before.bases[i] {
                                            action.delta.moves.push(RunnerMove {
                                                runner: runner.clone(),
                                                from,
                                                to: [
                                                    Destination::First,
                                                    Destination::Second,
                                                    Destination::Third,
                                                ][i],
                                            });
                                        }
                                    }
                                }
                                if reverse_moves {
                                    action.delta.moves.reverse();
                                }
                                if reverse_credits {
                                    action.delta.credits.reverse();
                                }
                                let t = apply(&before, action).unwrap();
                                assert_eq!(t.after(), reference.after());
                                assert_eq!(t.before(), &before);
                                assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
                                let box_score =
                                    derive_accounting(std::slice::from_ref(&t)).unwrap();
                                let side = 1 - half.offense();
                                let defense = &before.teams[side];
                                let pitcher =
                                    &box_score.pitchers[side][defense.pitcher.active_id().unwrap()];
                                assert_eq!(
                                    (
                                        pitcher.batters_faced,
                                        pitcher.outs_recorded,
                                        pitcher.at_bats_against,
                                        pitcher.classified_hits_allowed
                                    ),
                                    (1, 1, Some(1), Some(0))
                                );
                                assert_eq!(box_score.classified_fielding[side].len(), 9);
                                assert!(!box_score.classified_fielding[side]
                                    .contains_key("prior-pitcher"));
                                for (id, line) in &box_score.classified_fielding[side] {
                                    assert_eq!(line.observed_events, 1);
                                    assert_eq!(
                                        line.putouts,
                                        Some(u32::from(id == &defense.batting_order[5]))
                                    );
                                    assert_eq!(
                                        (line.assists, line.errors, line.passed_balls),
                                        (Some(0), Some(0), Some(0))
                                    );
                                }
                                let batter = t.event().participants.batter.as_ref().unwrap();
                                let batting = &box_score.batters[half.offense()][batter];
                                assert_eq!(
                                    (
                                        batting.plate_appearances,
                                        batting.at_bats,
                                        batting.classified_hits,
                                        batting.classified_rbi
                                    ),
                                    (1, Some(1), Some(0), Some(0))
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn catcher_actor_eligibility_uses_before_defense_including_a_pitcher_catch() {
    let before = state(7, 2, Half::Top);
    for bad in [
        "",
        "home-8",
        "home-bench",
        "home-removed",
        "away-5",
        "prior-pitcher",
    ] {
        let mut action = event(&before, AirContact::PopUp);
        action.ordinary_air_catch.as_mut().unwrap().fielder_id = bad.into();
        assert_eq!(apply(&before, action), Err(TransitionError::IllegalCredit));
    }
    let mut action = event(&before, AirContact::PopUp);
    action.ordinary_air_catch.as_mut().unwrap().fielder_id = "home-pitcher".into();
    action
        .delta
        .credits
        .iter_mut()
        .find(|c| c.kind == CreditKind::Putout)
        .unwrap()
        .player_id = "home-pitcher".into();
    let t = apply(&before, action).unwrap();
    assert_eq!(
        t.ordinary_air_fielding().unwrap().putouts,
        BTreeMap::from([("home-pitcher".into(), 1)])
    );
}

#[test]
fn no_dh_pitcher_catch_is_observed_once_not_as_two_defensive_actors() {
    let mut before = state(0, 0, Half::Top);
    let pitcher_id = before.teams[1]
        .pitcher
        .active_id()
        .expect("legacy fixture pitcher")
        .to_owned();
    let defense = &mut before.teams[1];
    defense.dh_slot = None;
    defense.batting_order[0] = defense
        .pitcher
        .active_id()
        .expect("legacy fixture pitcher")
        .to_owned();
    let mut action = event(&before, AirContact::PopUp);
    action.ordinary_air_catch.as_mut().unwrap().fielder_id = pitcher_id.clone();
    action
        .delta
        .credits
        .iter_mut()
        .find(|c| c.kind == CreditKind::Putout)
        .unwrap()
        .player_id = pitcher_id;
    let t = apply(&before, action).unwrap();
    let folded = derive_accounting(std::slice::from_ref(&t)).unwrap();
    assert_eq!(folded.classified_fielding[1].len(), 9);
    let pitcher = &folded.classified_fielding[1]["home-pitcher"];
    assert_eq!(
        (pitcher.observed_events, pitcher.putouts, pitcher.assists),
        (1, Some(1), Some(0))
    );
}

#[test]
fn air_actor_and_credit_forgeries_reach_constructor_reducer_and_decoder() {
    let before = state(3, 1, Half::Top);
    let reference = apply(&before, event(&before, AirContact::Fly)).unwrap();
    let mut action = reference.event().clone();
    action.ordinary_air_catch.as_mut().unwrap().batter_id = "away-1".into();
    assert_eq!(
        apply(&before, action.clone()),
        Err(TransitionError::InvalidParticipants)
    );
    assert_eq!(
        EventTransition::try_new(before.clone(), action, reference.after().clone()),
        Err(TransitionError::InvalidParticipants)
    );
    let mut encoded = reference.to_bytes();
    let offset = encoded
        .windows(6)
        .rposition(|bytes| bytes == b"away-0")
        .unwrap();
    encoded[offset + 5] = b'1';
    assert_eq!(
        EventTransition::from_bytes(&encoded),
        Err(TransitionError::InvalidParticipants)
    );
    let mut action = reference.event().clone();
    action
        .delta
        .credits
        .push(credit("away-0", CreditKind::Strikeout));
    assert_eq!(
        apply(&before, action.clone()),
        Err(TransitionError::IllegalCredit)
    );
    assert_eq!(
        EventTransition::try_new(before, action, reference.after().clone()),
        Err(TransitionError::IllegalCredit)
    );
}

#[test]
fn competing_ground_cause_is_not_a_direct_air_catch_even_with_unknown_review() {
    let before = state(0, 1, Half::Top);
    let mut action = event(&before, AirContact::Fly);
    action.fielding_review = FieldingReview::Unknown;
    action.ordinary_ground_throw = Some(bbsim_engine::OrdinaryGroundThrowFacts {
        batter_id: "away-0".into(),
        thrower_id: "home-3".into(),
        receiver_id: "home-5".into(),
    });
    action
        .delta
        .credits
        .push(credit("home-3", CreditKind::Assist));
    let mut ground = action.clone();
    ground.ordinary_air_catch = None;
    assert!(apply(&before, ground).is_ok());
    assert_eq!(apply(&before, action), Err(TransitionError::IllegalCredit));
}

#[test]
fn unknown_prior_review_does_not_impose_assist_error_or_legacy_pb_unit_caps() {
    let before = state(0, 0, Half::Top);
    let mut action = event(&before, AirContact::Fly);
    action.fielding_review = FieldingReview::Unknown;
    for (kind, id, units) in [
        (CreditKind::Assist, "home-3", 7),
        (CreditKind::Error, "home-4", 6),
        (CreditKind::PassedBall, "home-0", 5),
    ] {
        for _ in 0..units {
            action.delta.credits.push(credit(id, kind));
        }
    }
    let t = apply(&before, action).unwrap();
    assert_eq!(t.fielding_contribution(), None);
    assert_eq!(t.misplay_contribution(), None);
    let folded = derive_accounting(std::slice::from_ref(&t)).unwrap();
    assert_eq!(folded.fielders[1]["home-3"].credited_assists, 7);
    assert_eq!(folded.fielders[1]["home-4"].credited_errors, 6);
    assert_eq!(folded.fielders[1]["home-0"].credited_passed_balls, 5);
}

#[test]
fn bound_air_retirement_cannot_hide_known_legal_first_touch_behind_unknown_balk_history() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..2 {
            let before = state(1, outs, half);
            let mut action = event(&before, AirContact::Fly);
            let batter = action.participants.batter.clone().unwrap();
            action.fielding_review = FieldingReview::Unknown;
            action.balk = Some(bbsim_engine::BalkFacts {
                cause: bbsim_engine::BalkCause::Unknown,
                disposition: bbsim_engine::BalkDisposition::Unknown,
                resolution: bbsim_engine::BalkResolution::ActualMinimumProgress(
                    bbsim_engine::BalkProgressFacts {
                        batter_first_touch: bbsim_engine::NamedBaseTouch {
                            runner_id: batter,
                            base: Destination::First,
                        },
                        runner_next_touches: vec![],
                    },
                ),
            });
            let mut generic = action.clone();
            generic.ordinary_air_catch = None;
            assert_eq!(
                apply(&before, generic),
                Err(TransitionError::UnsupportedRule)
            );
            assert_eq!(apply(&before, action), Err(TransitionError::IllegalCredit));
        }
    }
}

#[test]
fn terminal_bottom_ninth_catch_keeps_before_defense_coverage() {
    let mut before = state(7, 2, Half::Bottom);
    before.inning = 9;
    before.score = [2, 1];
    let t = apply(&before, event(&before, AirContact::LineDrive)).unwrap();
    assert!(t.after().over);
    assert_eq!(t.after().bases, [None, None, None]);
    let folded = derive_accounting(std::slice::from_ref(&t)).unwrap();
    assert_eq!(folded.classified_fielding[0]["away-5"].putouts, Some(1));
    assert!(folded.classified_fielding[1].is_empty());
}

#[test]
fn air_codec_rejects_invalid_tags_versions_missing_facts_and_named_actor_forgery() {
    let before = state(0, 1, Half::Top);
    let t = apply(&before, event(&before, AirContact::Fly)).unwrap();
    let encoded = t.to_bytes();
    let mut bad = encoded.clone();
    *bad.last_mut().unwrap() = 255;
    assert_eq!(
        EventTransition::from_bytes(&bad),
        Err(TransitionError::InvalidEncoding)
    );
    let mut bad = encoded.clone();
    bad[4] = b'7';
    assert_eq!(
        EventTransition::from_bytes(&bad),
        Err(TransitionError::InvalidEncoding)
    );
    let mut bad = encoded.clone();
    let offset = bad
        .windows(6)
        .rposition(|bytes| bytes == b"home-5")
        .unwrap();
    bad[offset + 5] = b'4';
    assert_eq!(
        EventTransition::from_bytes(&bad),
        Err(TransitionError::IllegalCredit)
    );
    let mut action = event(&before, AirContact::Fly);
    action.ordinary_air_catch = None;
    let mut fake_new = apply(&before, action).unwrap().to_bytes();
    fake_new[4] = b'6';
    assert_eq!(
        EventTransition::from_bytes(&fake_new),
        Err(TransitionError::InvalidEncoding)
    );
}
