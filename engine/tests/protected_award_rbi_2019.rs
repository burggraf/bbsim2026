use bbsim_engine::{
    accounting::{derive_accounting, RbiContribution},
    apply, BalkCause, BalkDisposition, BalkFacts, BalkResolution, Base, Credit, CreditKind, Delta,
    Destination, Event, EventKind, EventTransition, FieldingReview, GameState, Half, Participants,
    ProtectedAwardFacts, Runner, RunnerMove, TeamState, ThirdOut, ThirdOutKind, TransitionError,
};

fn state(mask: u8, outs: u8, half: Half) -> GameState {
    let team = |prefix: &str| TeamState {
        batting_order: std::array::from_fn(|i| format!("{prefix}-{i}")),
        next_batter: 0,
        pitcher: format!("{prefix}-pitcher"),
        dh_slot: Some(8),
        dh_must_bat: false,
        bench: vec![],
        removed: vec![],
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
fn award(
    before: &GameState,
    hbp: bool,
    holds: bool,
    reverse: bool,
    review: FieldingReview,
) -> Event {
    let offense = &before.teams[before.half.offense()];
    let defense = &before.teams[1 - before.half.offense()];
    let batter = offense.batting_order[offense.next_batter as usize].clone();
    let mut credits = vec![
        credit(&batter, CreditKind::PlateAppearance),
        credit(&batter, CreditKind::NoHit),
    ];
    if hbp {
        credits.push(credit(&batter, CreditKind::HitByPitch));
    } else {
        credits.push(credit(&batter, CreditKind::Walk));
        credits.push(credit(&batter, CreditKind::OrdinaryWalk));
    }
    let mut moves = vec![RunnerMove {
        runner: Runner {
            player_id: batter.clone(),
            responsible_pitcher: defense.pitcher.clone(),
        },
        from: Base::Plate,
        to: Destination::First,
    }];
    let mut forced = true;
    let mut runs = 0;
    for (i, from) in [Base::First, Base::Second, Base::Third]
        .into_iter()
        .enumerate()
    {
        if let Some(runner) = &before.bases[i] {
            if forced || holds {
                let to = if forced {
                    [Destination::Second, Destination::Third, Destination::Home][i]
                } else {
                    [Destination::First, Destination::Second, Destination::Third][i]
                };
                moves.push(RunnerMove {
                    runner: runner.clone(),
                    from,
                    to,
                });
                if to == Destination::Home {
                    runs += 1;
                    credits.push(credit(&runner.player_id, CreditKind::Run));
                    credits.push(credit(&batter, CreditKind::Rbi));
                }
            }
        } else {
            forced = false;
        }
    }
    if reverse {
        moves.reverse();
        credits.reverse();
    }
    Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(batter.clone()),
            pitcher: defense.pitcher.clone(),
        },
        fielding_review: review,
        delta: Delta {
            moves,
            outs: 0,
            runs,
            credits,
            changes: vec![],
            third_out: None,
        },
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        ordinary_ground_throw: None,
        caught_strikeout: None,
        protected_award: Some(ProtectedAwardFacts { batter_id: batter }),
    }
}

#[test]
fn sole_protected_award_has_named_independent_rbi_in_every_force_context() {
    let mut frames = 0;
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            for mask in 0..8 {
                for hbp in [false, true] {
                    for holds in [false, true] {
                        for reverse in [false, true] {
                            for review in [FieldingReview::Unknown, FieldingReview::NoMisplay] {
                                let before = state(mask, outs, half);
                                let snapshot = before.clone();
                                let event = award(&before, hbp, holds, reverse, review);
                                let batter = event.participants.batter.clone().unwrap();
                                let pitcher = event.participants.pitcher.clone();
                                let rbi = u32::from(mask == 7);
                                let t = apply(&before, event.clone()).unwrap();
                                assert_eq!(before, snapshot);
                                assert_eq!(
                                    EventTransition::try_new(
                                        before.clone(),
                                        event,
                                        t.after().clone()
                                    )
                                    .unwrap(),
                                    t
                                );
                                assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
                                assert_eq!(
                                    t.protected_award_rbi(),
                                    Some(RbiContribution {
                                        runs_batted_in: [(batter.clone(), rbi)].into()
                                    })
                                );
                                let box_score = derive_accounting(&[t]).unwrap();
                                let b = &box_score.batters[half.offense()][&batter];
                                assert_eq!(
                                    (
                                        b.plate_appearances,
                                        b.at_bats,
                                        b.classified_hits,
                                        b.credited_rbi
                                    ),
                                    (1, Some(0), Some(0), rbi)
                                );
                                assert_eq!(
                                    (b.credited_walks, b.credited_hit_by_pitch),
                                    (u32::from(!hbp), u32::from(hbp))
                                );
                                let p = &box_score.pitchers[1 - half.offense()][&pitcher];
                                assert_eq!(
                                    (p.batters_faced, p.outs_recorded, p.runs_allowed),
                                    (1, 0, 0)
                                );
                                assert_eq!(
                                    (p.at_bats_against, p.classified_hits_allowed),
                                    (Some(0), Some(0))
                                );
                                assert_eq!(
                                    (p.credited_walks_allowed, p.credited_hit_batters),
                                    (u32::from(!hbp), u32::from(hbp))
                                );
                                if rbi == 1 {
                                    let old =
                                        &box_score.pitchers[1 - half.offense()]["prior-pitcher"];
                                    assert_eq!(
                                        (old.runs_allowed, old.batters_faced, old.earned_runs),
                                        (1, 0, None)
                                    );
                                }
                                frames += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(frames, 768);
}

#[test]
fn missing_protected_cause_never_certifies_supplied_or_omitted_rbi() {
    for review in [FieldingReview::Unknown, FieldingReview::NoMisplay] {
        let before = state(7, 1, Half::Top);
        let mut event = award(&before, false, false, false, review);
        event.protected_award = None;
        for omitted in [false, true] {
            let mut action = event.clone();
            if omitted {
                action.delta.credits.retain(|c| c.kind != CreditKind::Rbi);
            }
            let t = apply(&before, action).unwrap();
            assert_eq!(t.protected_award_rbi(), None);
            assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
        }
    }
}

// Encode only this isolated PA shape, then splice the unique whole Event
// between untouched before/after snapshots. Positive parses precede forgeries.
fn string(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend((value.len() as u64).to_le_bytes());
    bytes.extend(value.as_bytes());
}
fn event_bytes(event: &Event) -> Vec<u8> {
    assert_eq!(event.kind, EventKind::PlateAppearance);
    let mut bytes = vec![0, 1];
    string(&mut bytes, event.participants.batter.as_ref().unwrap());
    string(&mut bytes, &event.participants.pitcher);
    bytes.push(u8::from(event.fielding_review == FieldingReview::NoMisplay));
    bytes.extend((event.delta.moves.len() as u64).to_le_bytes());
    for m in &event.delta.moves {
        string(&mut bytes, &m.runner.player_id);
        string(&mut bytes, &m.runner.responsible_pitcher);
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
    bytes.extend([event.delta.outs, event.delta.runs]);
    bytes.extend((event.delta.credits.len() as u64).to_le_bytes());
    for c in &event.delta.credits {
        string(&mut bytes, &c.player_id);
        bytes.push(match c.kind {
            CreditKind::PlateAppearance => 0,
            CreditKind::Walk => 2,
            CreditKind::Run => 4,
            CreditKind::Rbi => 5,
            CreditKind::HitByPitch => 13,
            CreditKind::OrdinaryWalk => 20,
            CreditKind::IntentionalWalk => 21,
            CreditKind::NoHit => 25,
            _ => panic!("unrepresented fixture credit"),
        });
    }
    assert!(event.delta.changes.is_empty());
    bytes.extend(0u64.to_le_bytes());
    match &event.delta.third_out {
        None => bytes.push(0),
        Some(out) => {
            assert_eq!(out.kind, ThirdOutKind::BatterBeforeFirst);
            assert!(out.home_before_out.is_empty());
            bytes.push(1);
            string(&mut bytes, &out.runner_id);
            bytes.push(0);
            bytes.extend(0u64.to_le_bytes());
        }
    }
    assert!(
        event.two_out_contact.is_none()
            && event.single_delivery.is_none()
            && event.ordinary_ground_throw.is_none()
            && event.caught_strikeout.is_none()
    );
    bytes.extend([0; 2]); // contact, delivery
    match &event.balk {
        None => bytes.push(0),
        Some(balk) => {
            assert_eq!(balk.cause, BalkCause::Unknown);
            assert_eq!(balk.disposition, BalkDisposition::Unknown);
            assert_eq!(balk.resolution, BalkResolution::Unknown);
            bytes.extend([1, 0, 0, 0]);
        }
    }
    bytes.extend([0; 2]); // ground, caught K
    match &event.protected_award {
        None => bytes.push(0),
        Some(facts) => {
            bytes.push(1);
            string(&mut bytes, &facts.batter_id);
        }
    }
    bytes
}
fn splice(t: &EventTransition, replacement: &Event) -> Vec<u8> {
    let mut bytes = t.to_bytes();
    assert_eq!(&bytes[..5], b"BBE22");
    let original = event_bytes(t.event());
    let offsets: Vec<_> = bytes
        .windows(original.len())
        .enumerate()
        .filter_map(|(i, b)| (b == original).then_some(i))
        .collect();
    assert_eq!(offsets.len(), 1);
    bytes.splice(
        offsets[0]..offsets[0] + original.len(),
        event_bytes(replacement),
    );
    bytes
}
fn reject_all_paths(before: &GameState, valid: Event, forged: Event, expected: TransitionError) {
    let t = apply(before, valid).unwrap();
    assert_eq!(
        EventTransition::from_bytes(&splice(&t, t.event())).unwrap(),
        t
    );
    let outcomes = [
        apply(before, forged.clone()).err(),
        EventTransition::try_new(before.clone(), forged.clone(), t.after().clone()).err(),
        EventTransition::from_bytes(&splice(&t, &forged)).err(),
    ];
    assert_eq!(
        outcomes,
        [
            Some(expected.clone()),
            Some(expected.clone()),
            Some(expected)
        ]
    );
}

#[test]
fn protected_cause_requires_its_rbi_unit_on_all_three_paths() {
    for review in [FieldingReview::Unknown, FieldingReview::NoMisplay] {
        let before = state(7, 0, Half::Bottom);
        let event = award(&before, true, true, true, review);
        let mut omitted = event.clone();
        omitted.delta.credits.retain(|c| c.kind != CreditKind::Rbi);
        reject_all_paths(&before, event, omitted, TransitionError::IllegalCredit);
    }
}

#[test]
fn protected_cause_requires_actual_batter_on_all_three_paths() {
    for review in [FieldingReview::Unknown, FieldingReview::NoMisplay] {
        let before = state(7, 0, Half::Bottom);
        let event = award(&before, false, true, true, review);
        let mut wrong = event.clone();
        wrong.protected_award.as_mut().unwrap().batter_id =
            before.teams[before.half.offense()].batting_order[2].clone();
        reject_all_paths(&before, event, wrong, TransitionError::InvalidParticipants);
    }
}

#[test]
fn award_units_classification_and_force_origins_are_bound_on_all_paths() {
    for review in [FieldingReview::Unknown, FieldingReview::NoMisplay] {
        let before = state(7, 1, Half::Top);
        let event = award(&before, false, true, false, review);
        let batter = event.participants.batter.clone().unwrap();
        let mut duplicate = event.clone();
        duplicate
            .delta
            .credits
            .push(credit(&batter, CreditKind::Rbi));
        reject_all_paths(
            &before,
            event.clone(),
            duplicate,
            TransitionError::IllegalCredit,
        );
        let mut foreign = event.clone();
        foreign
            .delta
            .credits
            .iter_mut()
            .find(|c| c.kind == CreditKind::Rbi)
            .unwrap()
            .player_id = before.teams[0].batting_order[2].clone();
        reject_all_paths(
            &before,
            event.clone(),
            foreign,
            TransitionError::IllegalCredit,
        );
        let mut unclassified = event.clone();
        unclassified
            .delta
            .credits
            .retain(|c| !matches!(c.kind, CreditKind::Walk | CreditKind::OrdinaryWalk));
        reject_all_paths(
            &before,
            event.clone(),
            unclassified,
            TransitionError::IllegalCredit,
        );
        let mut missing_force = event.clone();
        missing_force.delta.moves.retain(|m| m.from != Base::First);
        reject_all_paths(
            &before,
            event.clone(),
            missing_force,
            TransitionError::DeltaMismatch,
        );
        let mut wrong_charge = event.clone();
        wrong_charge
            .delta
            .moves
            .iter_mut()
            .find(|m| m.from == Base::First)
            .unwrap()
            .runner
            .responsible_pitcher = "invented-pitcher".into();
        reject_all_paths(&before, event, wrong_charge, TransitionError::DeltaMismatch);
    }
}

#[test]
fn extra_batter_reach_and_false_third_out_do_not_bind_the_protected_cause() {
    let before = state(0, 2, Half::Top);
    let event = award(&before, false, false, false, FieldingReview::Unknown);
    let mut extra = event.clone();
    extra.delta.moves[0].to = Destination::Second;
    reject_all_paths(
        &before,
        event.clone(),
        extra,
        TransitionError::DeltaMismatch,
    );
    let mut false_out = event.clone();
    false_out.delta.third_out = Some(ThirdOut {
        runner_id: event.participants.batter.clone().unwrap(),
        kind: ThirdOutKind::BatterBeforeFirst,
        home_before_out: vec![],
    });
    reject_all_paths(&before, event, false_out, TransitionError::DeltaMismatch);
}

#[test]
fn protected_fact_option_and_obsolete_prefix_are_encoding_not_semantic_errors() {
    let before = state(7, 0, Half::Top);
    let event = award(&before, false, false, false, FieldingReview::NoMisplay);
    let t = apply(&before, event.clone()).unwrap();
    assert_eq!(EventTransition::from_bytes(&splice(&t, &event)).unwrap(), t);
    let mut old = t.to_bytes();
    old[..5].copy_from_slice(b"BBE21");
    assert_eq!(
        EventTransition::from_bytes(&old),
        Err(TransitionError::InvalidEncoding)
    );
    let original = event_bytes(&event);
    let mut bytes = t.to_bytes();
    let offsets: Vec<_> = bytes
        .windows(original.len())
        .enumerate()
        .filter_map(|(i, b)| (b == original).then_some(i))
        .collect();
    assert_eq!(offsets.len(), 1);
    let mut generic = event;
    generic.protected_award = None;
    let offset = offsets[0] + event_bytes(&generic).len() - 1;
    assert_eq!(bytes[offset], 1);
    bytes[offset] = 2;
    assert_eq!(
        EventTransition::from_bytes(&bytes),
        Err(TransitionError::InvalidEncoding)
    );
}

#[test]
fn unknown_compound_history_cannot_hide_bound_actor_or_missing_rbi() {
    let before = state(7, 1, Half::Top);
    let event = award(&before, false, true, false, FieldingReview::Unknown);
    let unknown = BalkFacts {
        cause: BalkCause::Unknown,
        disposition: BalkDisposition::Unknown,
        resolution: BalkResolution::Unknown,
    };
    let mut omitted = event.clone();
    omitted.delta.credits.retain(|c| c.kind != CreditKind::Rbi);
    omitted.balk = Some(unknown.clone());
    reject_all_paths(
        &before,
        event.clone(),
        omitted,
        TransitionError::IllegalCredit,
    );
    let mut wrong = event.clone();
    wrong.protected_award.as_mut().unwrap().batter_id = "other-batter".into();
    wrong.balk = Some(unknown.clone());
    reject_all_paths(
        &before,
        event.clone(),
        wrong,
        TransitionError::InvalidParticipants,
    );
    let mut unbound = event.clone();
    unbound.balk = Some(unknown);
    reject_all_paths(&before, event, unbound, TransitionError::UnsupportedRule);
}

#[test]
fn ordinary_and_intentional_walkoff_awards_keep_rbi_and_inherited_charge_separate() {
    for outs in 0..3 {
        for hbp in [false, true] {
            for intentional in [false, true] {
                if hbp && intentional {
                    continue;
                }
                let mut before = state(7, outs, Half::Bottom);
                before.inning = 9;
                let mut event = award(&before, hbp, true, false, FieldingReview::Unknown);
                if intentional {
                    for c in &mut event.delta.credits {
                        if c.kind == CreditKind::OrdinaryWalk {
                            c.kind = CreditKind::IntentionalWalk;
                        }
                    }
                }
                let batter = event.participants.batter.clone().unwrap();
                let pitcher = event.participants.pitcher.clone();
                let t = apply(&before, event).unwrap();
                assert!(t.after().over);
                assert_eq!(t.after().score, [0, 1]);
                assert_eq!(
                    t.protected_award_rbi(),
                    Some(RbiContribution {
                        runs_batted_in: [(batter.clone(), 1)].into()
                    })
                );
                assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
                let lines = derive_accounting(&[t]).unwrap();
                assert_eq!(lines.batters[1][&batter].credited_rbi, 1);
                assert_eq!(
                    lines.batters[1][&batter].credited_intentional_walks,
                    Some(u32::from(intentional))
                );
                assert_eq!(
                    (
                        lines.pitchers[0][&pitcher].batters_faced,
                        lines.pitchers[0][&pitcher].runs_allowed
                    ),
                    (1, 0)
                );
                assert_eq!(
                    (
                        lines.pitchers[0]["prior-pitcher"].runs_allowed,
                        lines.pitchers[0]["prior-pitcher"].earned_runs
                    ),
                    (1, None)
                );
            }
        }
    }
}

#[test]
fn unknown_prior_misplay_does_not_erase_the_independent_award_rbi_cause() {
    let before = state(7, 1, Half::Top);
    let mut event = award(&before, false, false, false, FieldingReview::Unknown);
    let fielder = before.teams[1].batting_order[2].clone();
    event
        .delta
        .credits
        .push(credit(&fielder, CreditKind::Error));
    let batter = event.participants.batter.clone().unwrap();
    let t = apply(&before, event).unwrap();
    assert_eq!(
        t.protected_award_rbi(),
        Some(RbiContribution {
            runs_batted_in: [(batter, 1)].into()
        })
    );
    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
    assert_eq!(
        derive_accounting(&[t]).unwrap().pitchers[1]["prior-pitcher"].earned_runs,
        None
    );
}

#[test]
fn unforced_scoring_is_not_an_exclusive_protected_award_cause() {
    for review in [FieldingReview::Unknown, FieldingReview::NoMisplay] {
        let before = state(4, 1, Half::Top);
        let mut extra = award(&before, false, true, false, review);
        let runner = before.bases[2].as_ref().unwrap();
        extra
            .delta
            .moves
            .iter_mut()
            .find(|m| m.from == Base::Third)
            .unwrap()
            .to = Destination::Home;
        extra.delta.runs = 1;
        extra
            .delta
            .credits
            .push(credit(&runner.player_id, CreditKind::Run));
        extra.delta.credits.push(credit(
            extra.participants.batter.as_ref().unwrap(),
            CreditKind::Rbi,
        ));
        let mut legal_partial = extra.clone();
        legal_partial.protected_award = None;
        reject_all_paths(
            &before,
            legal_partial,
            extra,
            TransitionError::DeltaMismatch,
        );
    }
}
