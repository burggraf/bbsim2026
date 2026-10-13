use bbsim_engine::{
    accounting::{derive_accounting, FieldingContribution},
    apply, BalkCause, BalkDisposition, BalkFacts, BalkProgressFacts, BalkResolution, Base,
    CaughtStrikeoutFacts, Credit, CreditKind, DeliveryContext, Delta, Destination, Event,
    EventKind, EventTransition, FieldingReview, GameState, Half, HandlingAssessment,
    NamedBaseTouch, Participants, RecoveryReview, Runner, RunnerMove, SingleDeliveryFacts,
    TeamState, ThirdOut, ThirdOutKind, TransitionError,
};
use std::collections::BTreeMap;

fn state(mask: u8, outs: u8, half: Half, dh: bool) -> GameState {
    let team = |prefix: &str| {
        let pitcher = format!("{prefix}-pitcher");
        let mut batting_order = std::array::from_fn(|i| format!("{prefix}-{i}"));
        if !dh {
            batting_order[0] = pitcher.clone();
        }
        TeamState {
            batting_order,
            next_batter: 0,
            pitcher,
            dh_slot: dh.then_some(8),
            dh_must_bat: false,
            bench: vec![format!("{prefix}-bench")],
            removed: vec![format!("{prefix}-removed")],
        }
    };
    let teams = [team("away"), team("home")];
    let bases = std::array::from_fn(|i| {
        (mask & (1 << i) != 0).then(|| Runner {
            player_id: teams[half.offense()].batting_order[i + 3].clone(),
            responsible_pitcher: "prior-pitcher".into(),
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
fn event(before: &GameState, catcher_slot: usize, holds: bool, reverse: bool) -> Event {
    let offense = &before.teams[before.half.offense()];
    let defense = &before.teams[1 - before.half.offense()];
    let batter = offense.batting_order[offense.next_batter as usize].clone();
    let catcher = defense.batting_order[catcher_slot].clone();
    let mut moves = vec![RunnerMove {
        runner: Runner {
            player_id: batter.clone(),
            responsible_pitcher: defense.pitcher.clone(),
        },
        from: Base::Plate,
        to: Destination::Out,
    }];
    if holds {
        for (i, from) in [Base::First, Base::Second, Base::Third]
            .into_iter()
            .enumerate()
        {
            if let Some(runner) = &before.bases[i] {
                moves.push(RunnerMove {
                    runner: runner.clone(),
                    from,
                    to: [Destination::First, Destination::Second, Destination::Third][i],
                });
            }
        }
    }
    if reverse {
        moves.reverse();
    }
    Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(batter.clone()),
            pitcher: defense.pitcher.clone(),
        },
        delta: Delta {
            moves,
            outs: 1,
            runs: 0,
            credits: vec![
                credit(&batter, CreditKind::PlateAppearance),
                credit(&batter, CreditKind::Strikeout),
                credit(&catcher, CreditKind::Putout),
            ],
            changes: vec![],
            third_out: (before.outs == 2).then(|| ThirdOut {
                runner_id: batter.clone(),
                kind: ThirdOutKind::BatterBeforeFirst,
                home_before_out: vec![],
            }),
        },
        fielding_review: FieldingReview::NoMisplay,
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        ordinary_ground_throw: None,
        caught_strikeout: Some(CaughtStrikeoutFacts {
            batter_id: batter,
            catcher_id: catcher,
        }),
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        ordinary_air_catch: None,
        muffed_ground_ball: None,
    }
}

#[test]
fn caught_k_has_independent_catcher_po_without_pitcher_assist() {
    for half in [Half::Top, Half::Bottom] {
        for dh in [false, true] {
            for outs in 0..3 {
                for mask in 0u8..8 {
                    for slot in [1, 2] {
                        for holds in [false, true] {
                            for reverse in [false, true] {
                                let before = state(mask, outs, half, dh);
                                let snapshot = before.clone();
                                let action = event(&before, slot, holds, reverse);
                                let catcher =
                                    action.caught_strikeout.as_ref().unwrap().catcher_id.clone();
                                let t = apply(&before, action.clone()).unwrap();
                                assert_eq!(before, snapshot);
                                assert_eq!(
                                    EventTransition::try_new(
                                        before.clone(),
                                        action,
                                        t.after().clone()
                                    )
                                    .unwrap(),
                                    t
                                );
                                assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
                                assert_eq!(
                                    t.caught_strikeout_fielding(),
                                    Some(FieldingContribution {
                                        putouts: BTreeMap::from([(catcher.clone(), 1)]),
                                        assists: BTreeMap::new(),
                                    })
                                );
                                assert_eq!(t.ordinary_ground_fielding(), None);
                                let a = derive_accounting(std::slice::from_ref(&t)).unwrap();
                                let side = 1 - half.offense();
                                let p = &a.pitchers[side][&t.event().participants.pitcher];
                                assert_eq!(
                                    (
                                        p.batters_faced,
                                        p.outs_recorded,
                                        p.credited_strikeouts,
                                        p.at_bats_against,
                                        p.classified_hits_allowed
                                    ),
                                    (1, 1, 1, Some(1), Some(0))
                                );
                                assert_eq!(
                                    (
                                        a.fielders[side][&catcher].credited_putouts,
                                        a.fielders[side][&catcher].credited_assists
                                    ),
                                    (1, 0)
                                );
                                let b = &a.batters[half.offense()]
                                    [t.event().participants.batter.as_ref().unwrap()];
                                assert_eq!(
                                    (
                                        b.plate_appearances,
                                        b.at_bats,
                                        b.credited_strikeouts,
                                        b.classified_hits
                                    ),
                                    (1, Some(1), 1, Some(0))
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
fn missing_catch_or_review_does_not_create_known_fielding_maps() {
    let before = state(7, 1, Half::Top, true);
    for missing in 0..3 {
        let mut action = event(&before, 1, true, true);
        if missing == 0 {
            action.caught_strikeout = None;
        }
        if missing > 0 {
            action.fielding_review = FieldingReview::Unknown;
        }
        if missing == 2 {
            action
                .delta
                .credits
                .retain(|c| c.kind != CreditKind::Putout);
            action.delta.credits.push(credit(
                &before.teams[1].batting_order[3],
                CreditKind::Assist,
            ));
        }
        let t = apply(&before, action).unwrap();
        assert_eq!(t.caught_strikeout_fielding(), None);
        assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
    }
}

fn bytes_string(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend((value.len() as u64).to_le_bytes());
    bytes.extend(value.as_bytes());
}
fn event_bytes(action: &Event) -> Vec<u8> {
    assert_eq!(action.kind, EventKind::PlateAppearance);
    let mut bytes = vec![0, 1];
    bytes_string(&mut bytes, action.participants.batter.as_ref().unwrap());
    bytes_string(&mut bytes, &action.participants.pitcher);
    bytes.push(u8::from(
        action.fielding_review == FieldingReview::NoMisplay,
    ));
    bytes.extend((action.delta.moves.len() as u64).to_le_bytes());
    for m in &action.delta.moves {
        bytes_string(&mut bytes, &m.runner.player_id);
        bytes_string(&mut bytes, &m.runner.responsible_pitcher);
        bytes.push(match m.from {
            Base::Plate => 0,
            Base::First => 1,
            Base::Second => 2,
            Base::Third => 3,
        });
        bytes.push(match m.to {
            Destination::First => 0,
            Destination::Second => 1,
            Destination::Third => 2,
            Destination::Home => 3,
            Destination::Out => 4,
        });
    }
    bytes.extend([action.delta.outs, action.delta.runs]);
    bytes.extend((action.delta.credits.len() as u64).to_le_bytes());
    for c in &action.delta.credits {
        bytes_string(&mut bytes, &c.player_id);
        bytes.push(match c.kind {
            CreditKind::PlateAppearance => 0,
            CreditKind::Strikeout => 1,
            CreditKind::Putout => 8,
            CreditKind::Assist => 9,
            CreditKind::Hit => 3,
            CreditKind::StolenBase => 6,
            CreditKind::CaughtStealing => 7,
            CreditKind::Error => 10,
            _ => panic!("unsupported test credit"),
        });
    }
    assert!(action.delta.changes.is_empty());
    bytes.extend(0u64.to_le_bytes());
    match &action.delta.third_out {
        None => bytes.push(0),
        Some(out) => {
            assert!(out.home_before_out.is_empty());
            bytes.push(1);
            bytes_string(&mut bytes, &out.runner_id);
            bytes.push(match out.kind {
                ThirdOutKind::BatterBeforeFirst => 0,
                ThirdOutKind::Force => 1,
                ThirdOutKind::Timing => 2,
                ThirdOutKind::PrecedingRunnerAppeal => 3,
                ThirdOutKind::BatterAfterFirstTiming => 4,
            });
            bytes.extend(0u64.to_le_bytes());
        }
    }
    assert!(action.two_out_contact.is_none() && action.ordinary_ground_throw.is_none());
    bytes.push(0);
    match &action.single_delivery {
        None => bytes.push(0),
        Some(facts) => {
            assert_eq!(facts.context, DeliveryContext::UncaughtThirdStrike);
            assert_eq!(facts.handling, HandlingAssessment::PitcherWild);
            assert!(facts.advances.is_empty() && facts.pre_delivery_steals.is_none());
            assert_eq!(facts.recovery, RecoveryReview::Unknown);
            bytes.extend([1, 1, 0]);
            bytes_string(&mut bytes, &facts.catcher_id);
            bytes.extend(0u64.to_le_bytes());
            bytes.extend([0, 0]);
        }
    }
    match &action.balk {
        None => bytes.push(0),
        Some(facts) => {
            bytes.push(1);
            bytes.push(match facts.cause {
                BalkCause::Unknown => 0,
                BalkCause::PitcherAction602a => 1,
                _ => panic!("unsupported test cause"),
            });
            bytes.push(match facts.disposition {
                BalkDisposition::Unknown => 0,
                BalkDisposition::PlayProceedsWithoutReference => 2,
                _ => panic!("unsupported test disposition"),
            });
            match &facts.resolution {
                BalkResolution::Unknown => bytes.push(0),
                BalkResolution::ActualMinimumProgress(progress) => {
                    assert_eq!(progress.batter_first_touch.base, Destination::First);
                    assert!(progress.runner_next_touches.is_empty());
                    bytes.push(2);
                    bytes_string(&mut bytes, &progress.batter_first_touch.runner_id);
                    bytes.push(0);
                    bytes.extend(0u64.to_le_bytes());
                }
                _ => panic!("unsupported test resolution"),
            }
        }
    }
    bytes.push(0);
    match &action.caught_strikeout {
        None => bytes.push(0),
        Some(facts) => {
            bytes.push(1);
            bytes_string(&mut bytes, &facts.batter_id);
            bytes_string(&mut bytes, &facts.catcher_id);
        }
    }
    assert!(
        action.protected_award.is_none()
            && action.sole_home_run.is_none()
            && action.sole_safe_hit.is_none()
            && action.muffed_ground_ball.is_none()
    );
    bytes.extend([0, 0, 0, 0]);
    bytes
}
fn splice_event(t: &EventTransition, original: &Event, replacement: &Event) -> Vec<u8> {
    let mut bytes = t.to_bytes();
    let marker = event_bytes(original);
    let positions: Vec<_> = bytes
        .windows(marker.len())
        .enumerate()
        .filter_map(|(i, value)| (value == marker).then_some(i))
        .collect();
    assert_eq!(positions.len(), 1);
    bytes.splice(
        positions[0]..positions[0] + marker.len(),
        event_bytes(replacement),
    );
    bytes
}
// The complete event lies between before/after snapshots. Unique whole-event
// splices and an actual valid factual parse control isolate semantic forgeries.
fn factual_wire(before: &GameState, action: &Event) -> Vec<u8> {
    let mut generic = action.clone();
    generic.caught_strikeout = None;
    let t = apply(before, generic.clone()).unwrap();
    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
    assert_eq!(&t.to_bytes()[..5], b"BBE25");
    let mut positive = action.clone();
    positive.caught_strikeout.as_mut().unwrap().catcher_id =
        before.teams[1 - before.half.offense()].batting_order[1].clone();
    assert!(EventTransition::from_bytes(&splice_event(&t, &generic, &positive)).is_ok());
    splice_event(&t, &generic, action)
}

fn reject_all(before: &GameState, control: &Event, bad: &Event, expected: TransitionError) {
    let t = apply(before, control.clone()).unwrap();
    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
    let result = (
        apply(before, bad.clone()),
        EventTransition::try_new(before.clone(), bad.clone(), t.after().clone()),
        EventTransition::from_bytes(&splice_event(&t, control, bad)),
    );
    assert_eq!(
        result,
        (Err(expected.clone()), Err(expected.clone()), Err(expected))
    );
}

#[test]
fn caught_fact_option_and_obsolete_version_cannot_bypass_validation() {
    let before = state(0, 0, Half::Top, true);
    let action = event(&before, 1, false, false);
    let t = apply(&before, action.clone()).unwrap();
    let mut bytes = t.to_bytes();
    assert_eq!(EventTransition::from_bytes(&bytes).unwrap(), t);
    bytes[..5].copy_from_slice(b"BBE20");
    assert_eq!(
        EventTransition::from_bytes(&bytes),
        Err(TransitionError::InvalidEncoding)
    );
    let marker = event_bytes(&action);
    let bytes = t.to_bytes();
    let positions: Vec<_> = bytes
        .windows(marker.len())
        .enumerate()
        .filter_map(|(i, value)| (value == marker).then_some(i))
        .collect();
    assert_eq!(positions.len(), 1);
    let mut generic = action.clone();
    generic.caught_strikeout = None;
    let prefix = event_bytes(&generic).len() - 5;
    let mut bad = bytes;
    assert_eq!(bad[positions[0] + prefix], 1);
    bad[positions[0] + prefix] = 2;
    assert_eq!(
        EventTransition::from_bytes(&bad),
        Err(TransitionError::InvalidEncoding)
    );
}

#[test]
fn qualification_units_and_actual_batter_cannot_be_forged() {
    let before = state(0, 0, Half::Top, true);
    let control = event(&before, 1, false, false);
    for missing in [
        CreditKind::Strikeout,
        CreditKind::PlateAppearance,
        CreditKind::Putout,
    ] {
        let mut bad = control.clone();
        bad.delta.credits.retain(|c| c.kind != missing);
        reject_all(&before, &control, &bad, TransitionError::IllegalCredit);
    }
    for extra in [
        CreditKind::Strikeout,
        CreditKind::PlateAppearance,
        CreditKind::Putout,
        CreditKind::Assist,
        CreditKind::Hit,
        CreditKind::Error,
    ] {
        let mut bad = control.clone();
        let id = match extra {
            CreditKind::Putout => before.teams[1].batting_order[1].clone(),
            CreditKind::Assist | CreditKind::Error => before.teams[1].batting_order[3].clone(),
            _ => bad.participants.batter.clone().unwrap(),
        };
        bad.delta.credits.push(credit(&id, extra));
        reject_all(&before, &control, &bad, TransitionError::IllegalCredit);
    }
    let mut bad = control.clone();
    bad.delta
        .credits
        .iter_mut()
        .find(|c| c.kind == CreditKind::Putout)
        .unwrap()
        .player_id = before.teams[1].batting_order[2].clone();
    reject_all(&before, &control, &bad, TransitionError::IllegalCredit);
    let mut bad = control.clone();
    bad.caught_strikeout.as_mut().unwrap().batter_id = before.teams[0].batting_order[1].clone();
    reject_all(
        &before,
        &control,
        &bad,
        TransitionError::InvalidParticipants,
    );
}

#[test]
fn held_charge_and_before_first_third_out_are_bound() {
    let before = state(1, 2, Half::Top, true);
    let control = event(&before, 1, true, false);
    let mut bad = control.clone();
    bad.delta.moves[1].to = Destination::Second;
    reject_all(&before, &control, &bad, TransitionError::DeltaMismatch);
    let mut bad = control.clone();
    bad.delta.moves[1].runner.responsible_pitcher = "different-pitcher".into();
    reject_all(&before, &control, &bad, TransitionError::DeltaMismatch);
    let mut bad = control.clone();
    bad.delta.third_out = None;
    reject_all(&before, &control, &bad, TransitionError::DeltaMismatch);
    let mut bad = control.clone();
    bad.delta.third_out.as_mut().unwrap().kind = ThirdOutKind::BatterAfterFirstTiming;
    reject_all(&before, &control, &bad, TransitionError::DeltaMismatch);
    let mut bad = control.clone();
    bad.delta.third_out.as_mut().unwrap().runner_id =
        before.bases[0].as_ref().unwrap().player_id.clone();
    reject_all(&before, &control, &bad, TransitionError::DeltaMismatch);
}

#[test]
fn terminal_uncaught_context_or_legal_first_touch_is_not_missing_catch_history() {
    let before = state(2, 0, Half::Top, true);
    let control = event(&before, 1, true, false);
    let mut bad = control.clone();
    bad.single_delivery = Some(SingleDeliveryFacts {
        context: DeliveryContext::UncaughtThirdStrike,
        handling: HandlingAssessment::PitcherWild,
        catcher_id: before.teams[1].batting_order[1].clone(),
        advances: vec![],
        pre_delivery_steals: None,
        recovery: RecoveryReview::Unknown,
    });
    reject_all(&before, &control, &bad, TransitionError::IllegalCredit);
    let mut bad = control.clone();
    bad.balk = Some(BalkFacts {
        cause: BalkCause::PitcherAction602a,
        disposition: BalkDisposition::PlayProceedsWithoutReference,
        resolution: BalkResolution::ActualMinimumProgress(BalkProgressFacts {
            batter_first_touch: NamedBaseTouch {
                runner_id: control.participants.batter.clone().unwrap(),
                base: Destination::First,
            },
            runner_next_touches: vec![],
        }),
    });
    reject_all(&before, &control, &bad, TransitionError::IllegalCredit);
    let mut partial = control.clone();
    partial.balk = Some(BalkFacts {
        cause: BalkCause::Unknown,
        disposition: BalkDisposition::Unknown,
        resolution: BalkResolution::Unknown,
    });
    reject_all(
        &before,
        &control,
        &partial,
        TransitionError::UnsupportedRule,
    );
    partial.fielding_review = FieldingReview::Unknown;
    partial
        .delta
        .credits
        .retain(|c| c.kind != CreditKind::Putout);
    partial.caught_strikeout.as_mut().unwrap().catcher_id = before.teams[1].pitcher.clone();
    reject_all(&before, &control, &partial, TransitionError::IllegalCredit);
}

#[test]
fn asserted_catcher_roles_are_known_before_missing_chance_review() {
    for half in [Half::Top, Half::Bottom] {
        let before = state(0, 0, half, true);
        let defense = &before.teams[1 - half.offense()];
        for id in [
            &defense.pitcher,
            &defense.batting_order[8],
            &defense.bench[0],
            &defense.removed[0],
            &before.teams[half.offense()].batting_order[1],
        ] {
            let mut action = event(&before, 1, false, false);
            action.fielding_review = FieldingReview::Unknown;
            action
                .delta
                .credits
                .retain(|c| c.kind != CreditKind::Putout);
            action.caught_strikeout.as_mut().unwrap().catcher_id = id.clone();
            let mut generic = action.clone();
            generic.caught_strikeout = None;
            let control = apply(&before, generic).unwrap();
            assert_eq!(
                (
                    apply(&before, action.clone()),
                    EventTransition::try_new(
                        before.clone(),
                        action.clone(),
                        control.after().clone()
                    ),
                    EventTransition::from_bytes(&factual_wire(&before, &action)),
                ),
                (
                    Err(TransitionError::IllegalCredit),
                    Err(TransitionError::IllegalCredit),
                    Err(TransitionError::IllegalCredit),
                )
            );
        }
    }
}
