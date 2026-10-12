use bbsim_engine::{
    accounting::derive_accounting, apply, Base, Credit, CreditKind, Delta, Destination, Event,
    EventKind, EventTransition, FieldingReview, GameState, Half, OrdinaryGroundThrowFacts,
    Participants, Runner, RunnerMove, TeamState, ThirdOut, ThirdOutKind, TransitionError,
};

fn state(mask: usize, outs: u8, half: Half, dh: bool) -> GameState {
    let mut teams = ["away", "home"].map(|prefix| TeamState {
        batting_order: std::array::from_fn(|i| format!("{prefix}{i}")),
        next_batter: 0,
        pitcher: format!("{prefix}_pitcher"),
        dh_slot: Some(8),
        dh_must_bat: false,
        bench: vec![format!("{prefix}_bench")],
        removed: vec![format!("{prefix}_removed")],
    });
    let offense = half.offense();
    if !dh {
        teams[1 - offense].dh_slot = None;
        teams[1 - offense].batting_order[8] = teams[1 - offense].pitcher.clone();
    }
    let bases = std::array::from_fn(|i| {
        (mask & (1 << i) != 0).then(|| Runner {
            player_id: teams[offense].batting_order[i + 5].clone(),
            responsible_pitcher: teams[1 - offense].pitcher.clone(),
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
fn credit(id: &str, kind: CreditKind) -> Credit {
    Credit {
        player_id: id.into(),
        kind,
    }
}
fn play(before: &GameState, holds: bool, pitcher_throws: bool, pitcher_receives: bool) -> Event {
    let offense = before.half.offense();
    let defense = &before.teams[1 - offense];
    let batter = before.teams[offense].batting_order[0].clone();
    let thrower = if pitcher_throws {
        &defense.pitcher
    } else {
        &defense.batting_order[3]
    };
    let receiver = if pitcher_receives {
        &defense.pitcher
    } else {
        &defense.batting_order[1]
    };
    let mut moves = vec![RunnerMove {
        runner: Runner {
            player_id: batter.clone(),
            responsible_pitcher: defense.pitcher.clone(),
        },
        from: Base::Plate,
        to: Destination::Out,
    }];
    if holds {
        for (i, (from, to)) in [
            (Base::First, Destination::First),
            (Base::Second, Destination::Second),
            (Base::Third, Destination::Third),
        ]
        .into_iter()
        .enumerate()
        {
            if let Some(runner) = &before.bases[i] {
                moves.push(RunnerMove {
                    runner: runner.clone(),
                    from,
                    to,
                });
            }
        }
    }
    Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(batter.clone()),
            pitcher: defense.pitcher.clone(),
        },
        fielding_review: FieldingReview::NoMisplay,
        delta: Delta {
            moves,
            outs: 1,
            runs: 0,
            credits: vec![
                credit(&batter, CreditKind::PlateAppearance),
                credit(&batter, CreditKind::AtBat),
                credit(&batter, CreditKind::NoHit),
                credit(receiver, CreditKind::Putout),
                credit(thrower, CreditKind::Assist),
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
        caught_strikeout: None,
        ordinary_ground_throw: Some(OrdinaryGroundThrowFacts {
            batter_id: batter,
            thrower_id: thrower.clone(),
            receiver_id: receiver.clone(),
        }),
    }
}
fn string(out: &mut Vec<u8>, value: &str) {
    out.extend_from_slice(&(value.len() as u64).to_le_bytes());
    out.extend_from_slice(value.as_bytes());
}
fn event_bytes(event: &Event) -> Vec<u8> {
    assert!(event.delta.changes.is_empty());
    let mut out = match event.kind {
        EventKind::PlateAppearance => vec![0],
        EventKind::Substitution => vec![2],
        EventKind::OutOfParkHomeRun => vec![3],
        EventKind::FieldersChoice => vec![4],
        EventKind::RunnerOnly(kind) => vec![
            1,
            match kind {
                bbsim_engine::RunnerEvent::WildPitch => 0,
                bbsim_engine::RunnerEvent::PassedBall => 1,
                bbsim_engine::RunnerEvent::StealAttempt => 2,
                bbsim_engine::RunnerEvent::Pickoff => 3,
                bbsim_engine::RunnerEvent::Balk => 4,
            },
        ],
    };
    if let Some(batter) = &event.participants.batter {
        out.push(1);
        string(&mut out, batter);
    } else {
        out.push(0);
    }
    string(&mut out, &event.participants.pitcher);
    out.push(u8::from(event.fielding_review == FieldingReview::NoMisplay));
    out.extend_from_slice(&(event.delta.moves.len() as u64).to_le_bytes());
    for m in &event.delta.moves {
        string(&mut out, &m.runner.player_id);
        string(&mut out, &m.runner.responsible_pitcher);
        out.push(match m.from {
            Base::Plate => 0,
            Base::First => 1,
            Base::Second => 2,
            Base::Third => 3,
        });
        out.push(match m.to {
            Destination::First => 0,
            Destination::Second => 1,
            Destination::Third => 2,
            Destination::Home => 3,
            Destination::Out => 4,
        });
    }
    out.extend([event.delta.outs, event.delta.runs]);
    out.extend_from_slice(&(event.delta.credits.len() as u64).to_le_bytes());
    for c in &event.delta.credits {
        string(&mut out, &c.player_id);
        out.push(match c.kind {
            CreditKind::PlateAppearance => 0,
            CreditKind::Strikeout => 1,
            CreditKind::Walk => 2,
            CreditKind::Hit => 3,
            CreditKind::Run => 4,
            CreditKind::Rbi => 5,
            CreditKind::StolenBase => 6,
            CreditKind::CaughtStealing => 7,
            CreditKind::Putout => 8,
            CreditKind::Assist => 9,
            CreditKind::Error => 10,
            CreditKind::PassedBall => 11,
            CreditKind::AtBat => 12,
            CreditKind::HitByPitch => 13,
            CreditKind::SacrificeBunt => 14,
            CreditKind::SacrificeFly => 15,
            CreditKind::InterferenceAward => 16,
            CreditKind::Single => 17,
            CreditKind::Double => 18,
            CreditKind::Triple => 19,
            CreditKind::OrdinaryWalk => 20,
            CreditKind::IntentionalWalk => 21,
            CreditKind::GroundedIntoDoublePlay => 22,
            CreditKind::WildPitch => 23,
            CreditKind::Balk => 24,
            CreditKind::NoHit => 25,
        });
    }
    out.extend_from_slice(&0u64.to_le_bytes());
    if let Some(third) = &event.delta.third_out {
        out.push(1);
        string(&mut out, &third.runner_id);
        out.push(match third.kind {
            ThirdOutKind::BatterBeforeFirst => 0,
            ThirdOutKind::Force => 1,
            ThirdOutKind::Timing => 2,
            ThirdOutKind::PrecedingRunnerAppeal => 3,
            ThirdOutKind::BatterAfterFirstTiming => 4,
        });
        out.extend_from_slice(&(third.home_before_out.len() as u64).to_le_bytes());
        for id in &third.home_before_out {
            string(&mut out, id);
        }
    } else {
        out.push(0);
    }
    if let Some(contact) = &event.two_out_contact {
        assert_eq!(contact.contact, bbsim_engine::ContactClass::Grounded);
        assert_eq!(
            contact.first_out.cause,
            bbsim_engine::RetirementCause::BatterPutoutBeforeFirstTouch
        );
        assert_eq!(
            contact.second_out.cause,
            bbsim_engine::RetirementCause::Force {
                at: Destination::Second
            }
        );
        assert_eq!(contact.continuity, bbsim_engine::ContactContinuity::Unknown);
        assert!(contact.error_placement.is_none());
        out.extend([1, 0]);
        string(&mut out, &contact.first_out.runner_id);
        out.push(0);
        string(&mut out, &contact.second_out.runner_id);
        out.extend([1, 1, 2, 0]);
    } else {
        out.push(0);
    }
    if let Some(delivery) = &event.single_delivery {
        assert!(delivery.advances.is_empty());
        assert!(delivery.pre_delivery_steals.is_none());
        assert_eq!(delivery.recovery, bbsim_engine::RecoveryReview::Unknown);
        out.extend([
            1,
            match delivery.context {
                bbsim_engine::DeliveryContext::RunnerOnly => 0,
                bbsim_engine::DeliveryContext::UncaughtThirdStrike => 1,
                bbsim_engine::DeliveryContext::DeliveredFourthBall => 2,
            },
            match delivery.handling {
                bbsim_engine::HandlingAssessment::PitcherWild => 0,
                bbsim_engine::HandlingAssessment::CatcherPassed => 1,
            },
        ]);
        string(&mut out, &delivery.catcher_id);
        out.extend_from_slice(&0u64.to_le_bytes());
        out.extend([0, 0]); // missing steal review and unknown recovery
    } else {
        out.push(0);
    }
    if let Some(balk) = &event.balk {
        assert_eq!(balk.cause, bbsim_engine::BalkCause::Unknown);
        assert_eq!(balk.disposition, bbsim_engine::BalkDisposition::Unknown);
        out.extend([1, 0, 0]);
        match &balk.resolution {
            bbsim_engine::BalkResolution::Unknown => out.push(0),
            bbsim_engine::BalkResolution::ExactProtectedAwards => out.push(1),
            bbsim_engine::BalkResolution::ActualMinimumProgress(progress) => {
                out.push(2);
                let touch = |out: &mut Vec<u8>, t: &bbsim_engine::NamedBaseTouch| {
                    string(out, &t.runner_id);
                    out.push(match t.base {
                        Destination::First => 0,
                        Destination::Second => 1,
                        Destination::Third => 2,
                        Destination::Home => 3,
                        Destination::Out => 4,
                    });
                };
                touch(&mut out, &progress.batter_first_touch);
                out.extend_from_slice(&(progress.runner_next_touches.len() as u64).to_le_bytes());
                for t in &progress.runner_next_touches {
                    touch(&mut out, t);
                }
            }
        }
    } else {
        out.push(0);
    }
    if let Some(f) = &event.ordinary_ground_throw {
        out.push(1);
        string(&mut out, &f.batter_id);
        string(&mut out, &f.thrower_id);
        string(&mut out, &f.receiver_id);
    } else {
        out.push(0);
    }
    assert!(event.caught_strikeout.is_none());
    out.push(0);
    out
}
fn wire(valid: &EventTransition, replacement: &Event) -> Vec<u8> {
    let original = event_bytes(valid.event());
    let mut bytes = valid.to_bytes();
    let offsets: Vec<_> = bytes
        .windows(original.len())
        .enumerate()
        .filter_map(|(i, b)| (b == original).then_some(i))
        .collect();
    assert_eq!(offsets.len(), 1, "whole-event splice is unambiguous");
    bytes.splice(
        offsets[0]..offsets[0] + original.len(),
        event_bytes(replacement),
    );
    bytes
}
fn reject(valid: &EventTransition, forged: Event, error: TransitionError) {
    assert_eq!(
        EventTransition::from_bytes(&wire(valid, valid.event())).unwrap(),
        *valid,
        "positive parse control"
    );
    let reduced = apply(valid.before(), forged.clone()).map(|_| ());
    let constructed = EventTransition::try_new(
        valid.before().clone(),
        forged.clone(),
        valid.after().clone(),
    )
    .map(|_| ());
    let decoded = EventTransition::from_bytes(&wire(valid, &forged)).map(|_| ());
    assert_eq!(
        (reduced, constructed, decoded),
        (Err(error.clone()), Err(error.clone()), Err(error))
    );
}

#[test]
fn direct_ground_throw_has_independent_event_local_po_a_in_all_contexts() {
    for half in [Half::Top, Half::Bottom] {
        for dh in [true, false] {
            for mask in 0..8 {
                for outs in 0..3 {
                    for holds in [false, true] {
                        for (p_throw, p_receive) in [(false, false), (true, false), (false, true)] {
                            let before = state(mask, outs, half, dh);
                            let event = play(&before, holds, p_throw, p_receive);
                            let f = event.ordinary_ground_throw.clone().unwrap();
                            for reverse in [false, true] {
                                let mut event = event.clone();
                                if reverse {
                                    event.delta.moves.reverse();
                                    event.delta.credits.reverse();
                                }
                                let t = apply(&before, event).unwrap();
                                let contribution = t
                                    .ordinary_ground_fielding()
                                    .expect("qualified factual contribution");
                                assert_eq!(
                                    contribution.putouts,
                                    [(f.receiver_id.clone(), 1)].into()
                                );
                                assert_eq!(
                                    contribution.assists,
                                    [(f.thrower_id.clone(), 1)].into()
                                );
                                assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
                                assert_eq!(
                                    EventTransition::from_bytes(&wire(&t, t.event())).unwrap(),
                                    t
                                );
                                let box_lines =
                                    derive_accounting(std::slice::from_ref(&t)).unwrap();
                                let batter = &box_lines.batters[half.offense()][&f.batter_id];
                                let pitcher = &box_lines.pitchers[1 - half.offense()]
                                    [&before.teams[1 - half.offense()].pitcher];
                                assert_eq!(
                                    (
                                        batter.plate_appearances,
                                        batter.at_bats,
                                        batter.classified_hits
                                    ),
                                    (1, Some(1), Some(0))
                                );
                                assert_eq!(
                                    (
                                        pitcher.batters_faced,
                                        pitcher.outs_recorded,
                                        pitcher.classified_hits_allowed
                                    ),
                                    (1, 1, Some(0))
                                );
                                assert_eq!(box_lines.runs, [0, 0]);
                                assert_eq!(
                                    box_lines.fielders[1 - half.offense()][&f.receiver_id]
                                        .credited_putouts,
                                    1
                                );
                                assert_eq!(
                                    box_lines.fielders[1 - half.offense()][&f.thrower_id]
                                        .credited_assists,
                                    1
                                );
                                if outs < 2 {
                                    assert_eq!(t.after().bases, before.bases);
                                    assert_eq!(t.after().outs, outs + 1);
                                } else {
                                    assert!(t.after().bases.iter().all(Option::is_none));
                                    assert_eq!(t.after().outs, 0);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn known_actor_and_credit_forgeries_reach_all_canonical_paths() {
    for half in [Half::Top, Half::Bottom] {
        let before = state(7, 2, half, true);
        let valid = apply(&before, play(&before, true, false, false)).unwrap();
        let defense = &before.teams[1 - half.offense()];
        for actor in [
            &defense.batting_order[8],
            &defense.bench[0],
            &defense.removed[0],
            &before.teams[half.offense()].batting_order[4],
            &String::new(),
        ] {
            for receiver in [false, true] {
                let mut bad = valid.event().clone();
                let f = bad.ordinary_ground_throw.as_mut().unwrap();
                if receiver {
                    f.receiver_id = actor.clone();
                } else {
                    f.thrower_id = actor.clone();
                }
                reject(&valid, bad, TransitionError::IllegalCredit);
            }
        }
        let mut bad = valid.event().clone();
        bad.ordinary_ground_throw.as_mut().unwrap().batter_id =
            before.teams[half.offense()].batting_order[1].clone();
        reject(&valid, bad, TransitionError::InvalidParticipants);
        let mut bad = valid.event().clone();
        bad.ordinary_ground_throw.as_mut().unwrap().receiver_id = bad
            .ordinary_ground_throw
            .as_ref()
            .unwrap()
            .thrower_id
            .clone();
        reject(&valid, bad, TransitionError::DeltaMismatch);
        for kind in [
            CreditKind::Putout,
            CreditKind::Assist,
            CreditKind::AtBat,
            CreditKind::NoHit,
        ] {
            let mut bad = valid.event().clone();
            bad.delta.credits.retain(|c| c.kind != kind);
            reject(&valid, bad, TransitionError::IllegalCredit);
            let mut bad = valid.event().clone();
            bad.delta.credits.push(
                bad.delta
                    .credits
                    .iter()
                    .find(|c| c.kind == kind)
                    .unwrap()
                    .clone(),
            );
            reject(&valid, bad, TransitionError::IllegalCredit);
        }
        for kind in [CreditKind::Putout, CreditKind::Assist] {
            let mut bad = valid.event().clone();
            bad.delta
                .credits
                .iter_mut()
                .find(|c| c.kind == kind)
                .unwrap()
                .player_id = defense.batting_order[2].clone();
            reject(&valid, bad, TransitionError::IllegalCredit);
        }
        for kind in [
            CreditKind::Strikeout,
            CreditKind::SacrificeBunt,
            CreditKind::SacrificeFly,
            CreditKind::GroundedIntoDoublePlay,
            CreditKind::HitByPitch,
            CreditKind::Hit,
        ] {
            let mut bad = valid.event().clone();
            bad.delta.credits.push(credit(
                valid.event().participants.batter.as_ref().unwrap(),
                kind,
            ));
            reject(&valid, bad, TransitionError::IllegalCredit);
        }
    }
}

#[test]
fn false_movements_and_third_out_claims_are_not_qualified() {
    let before = state(7, 2, Half::Top, true);
    let valid = apply(&before, play(&before, true, false, false)).unwrap();
    for kind in [
        ThirdOutKind::Force,
        ThirdOutKind::Timing,
        ThirdOutKind::BatterAfterFirstTiming,
        ThirdOutKind::PrecedingRunnerAppeal,
    ] {
        let mut bad = valid.event().clone();
        bad.delta.third_out.as_mut().unwrap().kind = kind;
        reject(&valid, bad, TransitionError::DeltaMismatch);
    }
    let mut bad = valid.event().clone();
    bad.delta.third_out = None;
    reject(&valid, bad, TransitionError::DeltaMismatch);
    let mut bad = valid.event().clone();
    bad.delta.third_out.as_mut().unwrap().runner_id = "wrong".into();
    reject(&valid, bad, TransitionError::DeltaMismatch);
    let mut bad = valid.event().clone();
    bad.delta
        .third_out
        .as_mut()
        .unwrap()
        .home_before_out
        .push(before.bases[2].as_ref().unwrap().player_id.clone());
    reject(&valid, bad, TransitionError::DeltaMismatch);
    for index in 0..valid.event().delta.moves.len() {
        let mut bad = valid.event().clone();
        bad.delta.moves[index].to = Destination::Home;
        reject(&valid, bad, TransitionError::DeltaMismatch);
        let mut bad = valid.event().clone();
        bad.delta.moves[index].runner.responsible_pitcher = "wrong_charge".into();
        reject(&valid, bad, TransitionError::DeltaMismatch);
    }
    let mut bad = valid.event().clone();
    bad.delta.runs = 1;
    reject(&valid, bad, TransitionError::DeltaMismatch);
    let mut bad = valid.event().clone();
    bad.delta.outs = 2;
    reject(&valid, bad, TransitionError::DeltaMismatch);
    let before = state(0, 0, Half::Top, true);
    let valid = apply(&before, play(&before, false, false, false)).unwrap();
    let mut bad = valid.event().clone();
    bad.delta.third_out = Some(ThirdOut {
        runner_id: bad.participants.batter.clone().unwrap(),
        kind: ThirdOutKind::BatterBeforeFirst,
        home_before_out: vec![],
    });
    reject(&valid, bad, TransitionError::DeltaMismatch);
}

#[test]
fn unbound_episode_uncertainty_cannot_mask_known_credit_roles() {
    let before = state(7, 0, Half::Top, true);
    let valid = apply(&before, play(&before, false, false, false)).unwrap();
    let mut partial = valid.event().clone();
    partial.fielding_review = FieldingReview::Unknown;
    partial.balk = Some(bbsim_engine::BalkFacts {
        cause: bbsim_engine::BalkCause::Unknown,
        disposition: bbsim_engine::BalkDisposition::Unknown,
        resolution: bbsim_engine::BalkResolution::Unknown,
    });
    reject(&valid, partial.clone(), TransitionError::UnsupportedRule);
    for kind in [CreditKind::Error, CreditKind::Assist, CreditKind::WildPitch] {
        let mut bad = partial.clone();
        bad.delta.credits.push(credit("away4", kind));
        reject(&valid, bad, TransitionError::IllegalCredit);
    }
    let mut bad = partial;
    bad.delta.credits.push(credit("away0", CreditKind::NoHit));
    reject(&valid, bad, TransitionError::IllegalCredit);
}

#[test]
fn bound_ground_facts_do_not_hide_existing_delivery_context_guards() {
    let before = state(7, 0, Half::Top, true);
    let valid = apply(&before, play(&before, false, false, false)).unwrap();
    for context in [
        bbsim_engine::DeliveryContext::RunnerOnly,
        bbsim_engine::DeliveryContext::UncaughtThirdStrike,
        bbsim_engine::DeliveryContext::DeliveredFourthBall,
    ] {
        let mut bad = valid.event().clone();
        bad.fielding_review = FieldingReview::Unknown;
        bad.single_delivery = Some(bbsim_engine::SingleDeliveryFacts {
            context,
            handling: bbsim_engine::HandlingAssessment::PitcherWild,
            catcher_id: "home0".into(),
            advances: vec![],
            pre_delivery_steals: None,
            recovery: bbsim_engine::RecoveryReview::Unknown,
        });
        let mut legacy = bad.clone();
        legacy.ordinary_ground_throw = None;
        reject(&valid, legacy, TransitionError::IllegalCredit);
        reject(&valid, bad, TransitionError::IllegalCredit);
    }
}

#[test]
fn actual_first_acquisition_cannot_coexist_with_bound_before_first_retirement() {
    let before = state(1, 0, Half::Top, true);
    let valid = apply(&before, play(&before, false, false, false)).unwrap();
    let mut bad = valid.event().clone();
    bad.fielding_review = FieldingReview::Unknown;
    bad.balk = Some(bbsim_engine::BalkFacts {
        cause: bbsim_engine::BalkCause::Unknown,
        disposition: bbsim_engine::BalkDisposition::Unknown,
        resolution: bbsim_engine::BalkResolution::ActualMinimumProgress(
            bbsim_engine::BalkProgressFacts {
                batter_first_touch: bbsim_engine::NamedBaseTouch {
                    runner_id: "away0".into(),
                    base: Destination::First,
                },
                runner_next_touches: vec![bbsim_engine::NamedBaseTouch {
                    runner_id: "away5".into(),
                    base: Destination::Second,
                }],
            },
        ),
    });
    let mut generic = bad.clone();
    generic.ordinary_ground_throw = None;
    reject(&valid, generic, TransitionError::UnsupportedRule);
    reject(&valid, bad, TransitionError::IllegalCredit);
}

#[test]
fn a_second_asserted_retirement_cannot_hide_behind_compound_uncertainty() {
    let before = state(1, 0, Half::Top, true);
    let valid = apply(&before, play(&before, false, false, false)).unwrap();
    let mut bad = valid.event().clone();
    bad.fielding_review = FieldingReview::Unknown;
    bad.two_out_contact = Some(bbsim_engine::TwoOutContactFacts {
        contact: bbsim_engine::ContactClass::Grounded,
        first_out: bbsim_engine::RetirementFact {
            runner_id: "away0".into(),
            cause: bbsim_engine::RetirementCause::BatterPutoutBeforeFirstTouch,
        },
        second_out: bbsim_engine::RetirementFact {
            runner_id: "away5".into(),
            cause: bbsim_engine::RetirementCause::Force {
                at: Destination::Second,
            },
        },
        continuity: bbsim_engine::ContactContinuity::Unknown,
        error_placement: None,
    });
    let mut generic = bad.clone();
    generic.ordinary_ground_throw = None;
    reject(&valid, generic, TransitionError::DeltaMismatch);
    reject(&valid, bad, TransitionError::DeltaMismatch);
}

fn before_first_steal_credit_is_known(kind: CreditKind) {
    let before = state(1, 0, Half::Top, true);
    let valid = apply(&before, play(&before, false, false, false)).unwrap();
    {
        let mut bad = valid.event().clone();
        bad.fielding_review = FieldingReview::Unknown;
        bad.delta.credits.push(Credit {
            player_id: "away0".into(),
            kind,
        });
        reject(&valid, bad.clone(), TransitionError::IllegalCredit);
        bad.balk = Some(bbsim_engine::BalkFacts {
            cause: bbsim_engine::BalkCause::Unknown,
            disposition: bbsim_engine::BalkDisposition::Unknown,
            resolution: bbsim_engine::BalkResolution::Unknown,
        });
        let mut generic = bad.clone();
        generic.ordinary_ground_throw = None;
        reject(
            &valid,
            generic,
            if kind == CreditKind::CaughtStealing {
                // Existing canonical CS eligibility requires a before-base runner.
                TransitionError::IllegalCredit
            } else {
                TransitionError::UnsupportedRule
            },
        );
        reject(&valid, bad, TransitionError::IllegalCredit);
    }
}

#[test]
fn before_first_batter_cannot_supply_a_completed_steal() {
    before_first_steal_credit_is_known(CreditKind::StolenBase);
}

#[test]
fn before_first_batter_cannot_supply_caught_stealing() {
    before_first_steal_credit_is_known(CreditKind::CaughtStealing);
}

#[test]
fn facts_do_not_change_inherited_charges_or_make_other_out_methods_impossible() {
    let mut before = state(7, 0, Half::Bottom, true);
    for runner in before.bases.iter_mut().flatten() {
        runner.responsible_pitcher = "earlier_pitcher".into();
    }
    let valid = apply(&before, play(&before, true, false, false)).unwrap();
    assert_eq!(valid.after().bases, before.bases);
    assert!(valid.ordinary_ground_fielding().is_some());
    let box_lines = derive_accounting(std::slice::from_ref(&valid)).unwrap();
    assert!(!box_lines.pitchers[0].contains_key("earlier_pitcher"));
    for kind in [
        EventKind::Substitution,
        EventKind::FieldersChoice,
        EventKind::OutOfParkHomeRun,
        EventKind::RunnerOnly(bbsim_engine::RunnerEvent::Pickoff),
    ] {
        let mut bad = valid.event().clone();
        bad.kind = kind;
        reject(&valid, bad, TransitionError::IllegalStateChange);
    }
    let mut generic = valid.event().clone();
    generic.ordinary_ground_throw = None;
    generic
        .delta
        .credits
        .retain(|c| c.kind != CreditKind::Assist);
    let unassisted_partial = apply(&before, generic).unwrap();
    assert_eq!(unassisted_partial.ordinary_ground_fielding(), None);
    assert_eq!(
        EventTransition::from_bytes(&unassisted_partial.to_bytes()).unwrap(),
        unassisted_partial
    );
}

#[test]
fn valid_wire_controls_isolate_version_truncation_and_invalid_fact_tag() {
    let before = state(0, 0, Half::Top, true);
    let t = apply(&before, play(&before, false, false, false)).unwrap();
    let bytes = t.to_bytes();
    assert_eq!(&bytes[..5], b"BBE21");
    assert_eq!(EventTransition::from_bytes(&bytes).unwrap(), t);
    let mut earlier = bytes.clone();
    earlier[..5].copy_from_slice(b"BBE19");
    assert_eq!(
        EventTransition::from_bytes(&earlier),
        Err(TransitionError::InvalidEncoding)
    );
    assert_eq!(
        EventTransition::from_bytes(&bytes[..bytes.len() - 1]),
        Err(TransitionError::InvalidEncoding)
    );
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert_eq!(
        EventTransition::from_bytes(&trailing),
        Err(TransitionError::InvalidEncoding)
    );
    let event = event_bytes(t.event());
    let mut no_facts = t.event().clone();
    no_facts.ordinary_ground_throw = None;
    let prefix_len = event_bytes(&no_facts).len() - 2;
    let offsets: Vec<_> = bytes
        .windows(event.len())
        .enumerate()
        .filter_map(|(i, b)| (b == event).then_some(i))
        .collect();
    assert_eq!(offsets.len(), 1);
    let mut invalid = bytes;
    assert_eq!(invalid[offsets[0] + prefix_len], 1);
    invalid[offsets[0] + prefix_len] = 2;
    assert_eq!(
        EventTransition::from_bytes(&invalid),
        Err(TransitionError::InvalidEncoding)
    );
    let generic = EventTransition::from_bytes(&wire(&t, &no_facts)).unwrap();
    assert_eq!(generic.ordinary_ground_fielding(), None);
}

#[test]
fn missing_review_or_facts_never_means_complete_fielding() {
    let before = state(7, 0, Half::Top, true);
    let event = play(&before, false, false, false);
    for facts in [false, true] {
        for reviewed in [false, true] {
            if facts && reviewed {
                continue;
            }
            let mut partial = event.clone();
            if !facts {
                partial.ordinary_ground_throw = None;
            }
            if !reviewed {
                partial.fielding_review = FieldingReview::Unknown;
            }
            let t = apply(&before, partial).unwrap();
            assert_eq!(t.ordinary_ground_fielding(), None);
            assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
        }
    }
    let mut partial = event.clone();
    partial.fielding_review = FieldingReview::Unknown;
    partial.delta.credits.push(credit(
        &before.teams[1].batting_order[2],
        CreditKind::Assist,
    ));
    let t = apply(&before, partial).unwrap();
    assert_eq!(
        t.ordinary_ground_fielding(),
        None,
        "unreviewed earlier missed chance is not impossible"
    );
    let mut partial = event;
    partial.fielding_review = FieldingReview::Unknown;
    partial
        .delta
        .credits
        .retain(|c| c.kind != CreditKind::Assist);
    let t = apply(&before, partial).unwrap();
    assert_eq!(
        t.ordinary_ground_fielding(),
        None,
        "absent assist is not a reviewed zero"
    );
}
