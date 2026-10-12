use bbsim_engine::{
    accounting::{derive_accounting, RbiContribution},
    apply, BalkCause, BalkDisposition, BalkFacts, BalkResolution, Base, Credit, CreditKind, Delta,
    Destination, Event, EventKind, EventTransition, FieldingReview, GameState, Half, Participants,
    Runner, RunnerMove, SoleSafeHitFacts, TeamState, ThirdOutKind, TransitionError,
};

fn state(mask: u8, outs: u8, half: Half) -> GameState {
    let team = |prefix: &str| TeamState {
        batting_order: std::array::from_fn(|i| format!("{prefix}{i}")),
        next_batter: 0,
        pitcher: format!("{prefix}p"),
        dh_slot: Some(8),
        dh_must_bat: false,
        bench: vec![],
        removed: vec![],
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
fn hit(before: &GameState, value: usize, cause: bool, review: FieldingReview) -> Event {
    let offense = &before.teams[before.half.offense()];
    let id = offense.batting_order[offense.next_batter as usize].clone();
    let pitcher = before.teams[1 - before.half.offense()].pitcher.clone();
    let targets = [
        Destination::First,
        Destination::Second,
        Destination::Third,
        Destination::Home,
    ];
    let mut moves = vec![RunnerMove {
        runner: Runner {
            player_id: id.clone(),
            responsible_pitcher: pitcher.clone(),
        },
        from: Base::Plate,
        to: targets[value - 1],
    }];
    let modifier = [CreditKind::Single, CreditKind::Double, CreditKind::Triple][value - 1];
    let mut credits = vec![
        credit(&id, CreditKind::PlateAppearance),
        credit(&id, CreditKind::Hit),
        credit(&id, modifier),
    ];
    let mut runs = 0;
    for (i, from) in [Base::First, Base::Second, Base::Third]
        .into_iter()
        .enumerate()
    {
        if let Some(runner) = &before.bases[i] {
            let to = targets[(i + value).min(3)];
            moves.push(RunnerMove {
                runner: runner.clone(),
                from,
                to,
            });
            if to == Destination::Home {
                runs += 1;
                credits.extend([
                    credit(&runner.player_id, CreditKind::Run),
                    credit(&id, CreditKind::Rbi),
                ]);
            }
        }
    }
    Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(id.clone()),
            pitcher,
        },
        delta: Delta {
            moves,
            outs: 0,
            runs,
            credits,
            changes: vec![],
            third_out: None,
        },
        fielding_review: review,
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        ordinary_ground_throw: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: cause.then_some(SoleSafeHitFacts { batter_id: id }),
        muffed_ground_ball: None,
    }
}

fn checked(before: &GameState, event: Event) -> EventTransition {
    let frozen = before.clone();
    let t = apply(before, event.clone()).expect("valid independently built safe-hit fixture");
    assert_eq!(*before, frozen);
    assert_eq!(
        EventTransition::try_new(before.clone(), event, t.after().clone()),
        Ok(t.clone())
    );
    assert_eq!(EventTransition::from_bytes(&t.to_bytes()), Ok(t.clone()));
    assert_eq!(*before, frozen);
    t
}
fn named(id: &str, count: u32) -> Option<RbiContribution> {
    Some(RbiContribution {
        runs_batted_in: [(id.to_string(), count)].into(),
    })
}

// Independent whole-event encoder, limited to the explicitly used fixture
// vocabulary. Controls prove correct framing before any semantic rejection.
fn event_bytes(event: &Event) -> Vec<u8> {
    fn string(out: &mut Vec<u8>, value: &str) {
        out.extend((value.len() as u64).to_le_bytes());
        out.extend(value.as_bytes());
    }
    let mut out = vec![match event.kind {
        EventKind::PlateAppearance => 0,
        EventKind::OutOfParkHomeRun => 3,
        _ => panic!("unrepresented fixture event"),
    }];
    out.push(1);
    string(&mut out, event.participants.batter.as_ref().unwrap());
    string(&mut out, &event.participants.pitcher);
    out.push(u8::from(event.fielding_review == FieldingReview::NoMisplay));
    out.extend((event.delta.moves.len() as u64).to_le_bytes());
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
    out.extend((event.delta.credits.len() as u64).to_le_bytes());
    for c in &event.delta.credits {
        string(&mut out, &c.player_id);
        out.push(match c.kind {
            CreditKind::PlateAppearance => 0,
            CreditKind::Hit => 3,
            CreditKind::Run => 4,
            CreditKind::Rbi => 5,
            CreditKind::Error => 10,
            CreditKind::PassedBall => 11,
            CreditKind::Single => 17,
            CreditKind::Double => 18,
            CreditKind::Triple => 19,
            CreditKind::StolenBase => 6,
            CreditKind::CaughtStealing => 7,
            CreditKind::Putout => 8,
            CreditKind::Assist => 9,
            CreditKind::WildPitch => 23,
            CreditKind::Balk => 24,
            CreditKind::NoHit => 25,
            _ => panic!("unrepresented fixture credit"),
        });
    }
    assert!(event.delta.changes.is_empty());
    out.extend(0u64.to_le_bytes());
    match &event.delta.third_out {
        None => out.push(0),
        Some(third) => {
            assert_eq!(third.kind, ThirdOutKind::BatterBeforeFirst);
            assert!(third.home_before_out.is_empty());
            out.push(1);
            string(&mut out, &third.runner_id);
            out.push(0);
            out.extend(0u64.to_le_bytes());
        }
    }
    assert!(
        event.two_out_contact.is_none()
            && event.single_delivery.is_none()
            && event.caught_strikeout.is_none()
            && event.protected_award.is_none()
    );
    out.extend([0; 2]);
    match &event.balk {
        None => out.push(0),
        Some(balk) => {
            assert_eq!(balk.cause, BalkCause::PitcherAction602a);
            assert_eq!(
                balk.disposition,
                BalkDisposition::PlayProceedsWithoutReference
            );
            out.extend([1, 1, 2]);
            match &balk.resolution {
                BalkResolution::Unknown => out.push(0),
                BalkResolution::ActualMinimumProgress(progress) => {
                    assert_eq!(progress.batter_first_touch.base, Destination::First);
                    assert!(progress.runner_next_touches.is_empty());
                    out.push(2);
                    string(&mut out, &progress.batter_first_touch.runner_id);
                    out.push(0);
                    out.extend(0u64.to_le_bytes());
                }
                _ => panic!("unrepresented fixture balk resolution"),
            }
        }
    }
    match &event.ordinary_ground_throw {
        None => out.push(0),
        Some(facts) => {
            out.push(1);
            string(&mut out, &facts.batter_id);
            string(&mut out, &facts.thrower_id);
            string(&mut out, &facts.receiver_id);
        }
    }
    out.extend([0; 2]); // caught K, protected award
    assert!(event.sole_home_run.is_none());
    out.push(0);
    match &event.sole_safe_hit {
        None => out.push(0),
        Some(facts) => {
            out.push(1);
            string(&mut out, &facts.batter_id);
        }
    }
    assert!(event.muffed_ground_ball.is_none());
    out.push(0);
    out
}
fn splice(t: &EventTransition, event: &Event) -> Vec<u8> {
    let mut bytes = t.to_bytes();
    assert_eq!(&bytes[..5], b"BBE25");
    let original = event_bytes(t.event());
    let positions: Vec<_> = bytes
        .windows(original.len())
        .enumerate()
        .filter_map(|(i, b)| (b == original).then_some(i))
        .collect();
    assert_eq!(positions.len(), 1);
    bytes.splice(
        positions[0]..positions[0] + original.len(),
        event_bytes(event),
    );
    bytes
}
fn reject(before: &GameState, valid: &Event, bad: Event, expected: TransitionError) {
    let frozen = before.clone();
    let t = checked(before, valid.clone());
    assert_eq!(
        EventTransition::from_bytes(&splice(&t, valid)),
        Ok(t.clone())
    );
    let results = [
        apply(before, bad.clone()).err(),
        EventTransition::try_new(before.clone(), bad.clone(), t.after().clone()).err(),
        EventTransition::from_bytes(&splice(&t, &bad)).err(),
    ];
    assert_eq!(
        results,
        [
            Some(expected.clone()),
            Some(expected.clone()),
            Some(expected)
        ]
    );
    assert_eq!(*before, frozen);
}

#[test]
fn literal_rbi_across_three_values_masks_outs_halves_and_independent_orders() {
    let mut frames = 0;
    for value in 1..=3 {
        for mask in 0..8 {
            for outs in 0..3 {
                for half in [Half::Top, Half::Bottom] {
                    for reverse_moves in [false, true] {
                        for reverse_credits in [false, true] {
                            let before = state(mask, outs, half);
                            let mut event = hit(&before, value, true, FieldingReview::NoMisplay);
                            if reverse_moves {
                                event.delta.moves.reverse();
                            }
                            if reverse_credits {
                                event.delta.credits.reverse();
                            }
                            let t = checked(&before, event);
                            let id = &before.teams[half.offense()].batting_order[0];
                            let runs = (0..3)
                                .filter(|i| mask & (1 << i) != 0 && i + value >= 3)
                                .count() as u32;
                            assert_eq!(t.sole_safe_hit_rbi(), named(id, runs));
                            assert_eq!(t.rbi_contribution(), named(id, runs));
                            assert_eq!(t.after().score[half.offense()], runs as u16);
                            let lines = derive_accounting(&[t]).unwrap();
                            assert_eq!(
                                lines.batters[half.offense()][id].classified_rbi,
                                Some(runs)
                            );
                            assert_eq!(lines.batters[half.offense()][id].credited_rbi, runs);
                            assert_eq!(lines.batters[half.offense()][id].plate_appearances, 1);
                            assert_eq!(lines.batters[half.offense()][id].at_bats, Some(1));
                            assert_eq!(
                                lines.pitchers[1 - half.offense()]
                                    [&before.teams[1 - half.offense()].pitcher]
                                    .batters_faced,
                                1
                            );
                            if runs > 0 {
                                assert_eq!(
                                    lines.pitchers[1 - half.offense()]["oldp"].runs_allowed,
                                    runs
                                );
                            }
                            frames += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(frames, 576);
}

#[test]
fn missing_cause_is_not_reconstructed_from_hit_value_shape_or_supplied_rbi() {
    for value in 1..=3 {
        let before = state(7, 0, Half::Top);
        for supplied in [true, false] {
            let mut event = hit(&before, value, false, FieldingReview::NoMisplay);
            if !supplied {
                event.delta.credits.retain(|c| c.kind != CreditKind::Rbi);
            }
            let t = checked(&before, event);
            assert_eq!(t.sole_safe_hit_rbi(), None);
            assert_eq!(t.rbi_contribution(), None);
            assert_eq!(
                derive_accounting(&[t]).unwrap().batters[0]["a0"].classified_rbi,
                None
            );
        }
    }
    let before = state(0, 0, Half::Top);
    let t = checked(&before, hit(&before, 1, false, FieldingReview::Unknown));
    assert_eq!(t.sole_safe_hit_rbi(), None);
    assert_eq!(t.rbi_contribution(), named("a0", 0));
}

#[test]
fn subtype_current_participants_and_legal_run_conservation_are_required() {
    let before = state(7, 0, Half::Top);
    let valid = hit(&before, 1, true, FieldingReview::NoMisplay);
    let mut subtype = valid.clone();
    subtype.kind = EventKind::OutOfParkHomeRun;
    reject(
        &before,
        &valid,
        subtype,
        TransitionError::IllegalStateChange,
    );
    let mut actor = valid.clone();
    actor.participants.batter = Some("a2".into());
    actor.sole_safe_hit.as_mut().unwrap().batter_id = "a2".into();
    reject(&before, &valid, actor, TransitionError::InvalidParticipants);
    let mut pitcher = valid.clone();
    pitcher.participants.pitcher = "oldp".into();
    reject(
        &before,
        &valid,
        pitcher,
        TransitionError::InvalidParticipants,
    );
    let mut runs = valid.clone();
    runs.delta.runs = 2;
    runs.delta.credits.push(credit("a0", CreditKind::Rbi));
    reject(&before, &valid, runs, TransitionError::DeltaMismatch);
}

#[test]
fn exact_actual_actor_and_rbi_units_are_required_on_all_canonical_paths() {
    let before = state(7, 1, Half::Top);
    let valid = hit(&before, 3, true, FieldingReview::NoMisplay);
    let mut wrong = valid.clone();
    wrong.sole_safe_hit.as_mut().unwrap().batter_id = "a2".into();
    reject(&before, &valid, wrong, TransitionError::InvalidParticipants);
    for alteration in 0..4 {
        let mut wrong = valid.clone();
        let position = wrong
            .delta
            .credits
            .iter()
            .position(|c| c.kind == CreditKind::Rbi)
            .unwrap();
        match alteration {
            0 => {
                wrong.delta.credits.remove(position);
            }
            1 => wrong.delta.credits.push(credit("a0", CreditKind::Rbi)),
            2 => wrong.delta.credits[position].player_id = "a2".into(),
            _ => wrong.delta.credits.retain(|c| c.kind != CreditKind::Rbi),
        }
        reject(&before, &valid, wrong, TransitionError::IllegalCredit);
    }
}

#[test]
fn explicit_hit_classification_and_complete_forward_safe_geometry_are_required() {
    let before = state(7, 0, Half::Top);
    let valid = hit(&before, 1, true, FieldingReview::NoMisplay);
    for kind in [CreditKind::Hit, CreditKind::Single] {
        let mut missing = valid.clone();
        missing.delta.credits.retain(|c| c.kind != kind);
        reject(&before, &valid, missing, TransitionError::IllegalCredit);
        let mut duplicate = valid.clone();
        duplicate.delta.credits.push(credit("a0", kind));
        reject(&before, &valid, duplicate, TransitionError::IllegalCredit);
        let mut foreign = valid.clone();
        foreign
            .delta
            .credits
            .iter_mut()
            .find(|c| c.kind == kind)
            .unwrap()
            .player_id = "a2".into();
        reject(&before, &valid, foreign, TransitionError::IllegalCredit);
    }
    let mut charge = valid.clone();
    charge.delta.moves[1].runner.responsible_pitcher = "hp".into();
    reject(&before, &valid, charge, TransitionError::DeltaMismatch);
    let mut origin = valid.clone();
    origin.delta.moves[1].from = Base::Third;
    reject(&before, &valid, origin, TransitionError::DeltaMismatch);
    let mut missing = valid.clone();
    missing.delta.moves.remove(1);
    reject(&before, &valid, missing, TransitionError::DeltaMismatch);
    let mut retreat = valid.clone();
    retreat.delta.moves[2].to = Destination::First;
    reject(&before, &valid, retreat, TransitionError::DeltaMismatch);
    let mut out = valid.clone();
    out.delta.moves[1].to = Destination::Out;
    out.delta.outs = 1;
    reject(&before, &valid, out, TransitionError::DeltaMismatch);
    let mut home = valid.clone();
    home.delta.moves[0].to = Destination::Home;
    home.delta.runs += 1;
    home.delta
        .credits
        .extend([credit("a0", CreditKind::Run), credit("a0", CreditKind::Rbi)]);
    reject(&before, &valid, home, TransitionError::DeltaMismatch);
}

#[test]
fn held_and_unequal_continuous_advances_derive_rbi_from_named_movements() {
    let before = state(7, 0, Half::Top);
    let mut event = hit(&before, 1, true, FieldingReview::Unknown);
    event.delta.moves[1].to = Destination::Home;
    event.delta.moves[2].to = Destination::Second;
    event.delta.runs = 2;
    event
        .delta
        .credits
        .extend([credit("a3", CreditKind::Run), credit("a0", CreditKind::Rbi)]);
    let t = checked(&before, event);
    assert_eq!(t.sole_safe_hit_rbi(), named("a0", 2));
    let lines = derive_accounting(&[t]).unwrap();
    assert_eq!(lines.batters[0]["a0"].classified_rbi, Some(2));
    assert_eq!(lines.pitchers[1]["oldp"].earned_runs, None);
}

#[test]
fn prior_foul_chance_error_does_not_erase_an_independent_unaided_single_cause() {
    let before = state(7, 2, Half::Top);
    let mut event = hit(&before, 1, true, FieldingReview::Unknown);
    event.delta.credits.push(credit("h4", CreditKind::Error));
    let t = checked(&before, event.clone());
    assert_eq!(t.sole_safe_hit_rbi(), named("a0", 1));
    assert_eq!(
        derive_accounting(&[t]).unwrap().pitchers[1]["oldp"].earned_runs,
        None
    );
    let valid = event.clone();
    event.fielding_review = FieldingReview::NoMisplay;
    reject(&before, &valid, event, TransitionError::IllegalCredit);
}

#[test]
fn competing_units_and_known_errors_precede_missing_balk_chronology() {
    let before = state(7, 0, Half::Top);
    let valid = hit(&before, 1, true, FieldingReview::Unknown);
    for (actor, kind) in [
        ("a3", CreditKind::StolenBase),
        ("a3", CreditKind::CaughtStealing),
        ("hp", CreditKind::WildPitch),
        ("h4", CreditKind::PassedBall),
        ("hp", CreditKind::Balk),
        ("h1", CreditKind::Putout),
        ("h3", CreditKind::Assist),
    ] {
        let mut bad = valid.clone();
        bad.delta.credits.push(credit(actor, kind));
        reject(&before, &valid, bad, TransitionError::IllegalCredit);
    }
    let mut compound = valid.clone();
    compound.balk = Some(BalkFacts {
        cause: BalkCause::PitcherAction602a,
        disposition: BalkDisposition::PlayProceedsWithoutReference,
        resolution: BalkResolution::Unknown,
    });
    let mut actor = compound.clone();
    actor.sole_safe_hit.as_mut().unwrap().batter_id = "a2".into();
    reject(&before, &valid, actor, TransitionError::InvalidParticipants);
    let mut units = compound.clone();
    units.delta.credits.retain(|c| c.kind != CreditKind::Rbi);
    reject(&before, &valid, units, TransitionError::IllegalCredit);
    reject(
        &before,
        &valid,
        compound.clone(),
        TransitionError::UnsupportedRule,
    );
    compound.sole_safe_hit = None;
    assert_eq!(
        apply(&before, compound).err(),
        Some(TransitionError::UnsupportedRule)
    );
}

#[test]
fn ordinary_walk_off_keeps_only_the_permitted_winning_run() {
    for inning in [9, 11] {
        let mut before = state(6, 2, Half::Bottom);
        before.inning = inning;
        let mut event = hit(&before, 1, true, FieldingReview::NoMisplay);
        let t = checked(&before, event.clone());
        assert!(t.after().over);
        assert_eq!(t.rbi_contribution(), named("h0", 1));
        // The existing winning policy can hold the trailing runners rather
        // than credit a full station advance after the winning run.
        event.delta.moves[1].to = Destination::Second;
        let t = checked(&before, event);
        assert!(t.after().over);
        assert_eq!(t.sole_safe_hit_rbi(), named("h0", 1));
    }
}

#[test]
fn a_known_other_family_actor_error_precedes_unrelated_extra_base_review() {
    let before = state(7, 0, Half::Top);
    let valid = hit(&before, 2, true, FieldingReview::NoMisplay);
    let mut wrong = valid.clone();
    wrong.fielding_review = FieldingReview::Unknown;
    wrong.ordinary_ground_throw = Some(bbsim_engine::OrdinaryGroundThrowFacts {
        batter_id: "a2".into(),
        thrower_id: "h3".into(),
        receiver_id: "h1".into(),
    });
    reject(&before, &valid, wrong, TransitionError::InvalidParticipants);
}

#[test]
fn known_wrong_balk_touch_precedes_unrelated_extra_base_review() {
    let before = state(7, 0, Half::Top);
    let valid = hit(&before, 2, true, FieldingReview::NoMisplay);
    let mut wrong = valid.clone();
    wrong.fielding_review = FieldingReview::Unknown;
    wrong.balk = Some(BalkFacts {
        cause: BalkCause::PitcherAction602a,
        disposition: BalkDisposition::PlayProceedsWithoutReference,
        resolution: BalkResolution::ActualMinimumProgress(bbsim_engine::BalkProgressFacts {
            batter_first_touch: bbsim_engine::NamedBaseTouch {
                runner_id: "a2".into(),
                base: Destination::First,
            },
            runner_next_touches: vec![],
        }),
    });
    reject(&before, &valid, wrong, TransitionError::IllegalCredit);
}

#[test]
fn ambiguous_ordinary_walk_off_value_is_not_expanded_by_the_rbi_cause() {
    let mut before = state(6, 1, Half::Bottom);
    before.inning = 9;
    before.score = [1, 0];
    let valid = hit(&before, 1, true, FieldingReview::NoMisplay);
    let ambiguous = hit(&before, 2, true, FieldingReview::NoMisplay);
    reject(&before, &valid, ambiguous, TransitionError::UnsupportedRule);
}

#[test]
fn known_units_precede_extra_base_review_uncertainty_without_expanding_it() {
    let before = state(7, 0, Half::Top);
    let valid = hit(&before, 2, true, FieldingReview::NoMisplay);
    let mut unknown = valid.clone();
    unknown.fielding_review = FieldingReview::Unknown;
    let mut wrong = unknown.clone();
    wrong.delta.credits.retain(|c| c.kind != CreditKind::Rbi);
    reject(&before, &valid, wrong, TransitionError::IllegalCredit);
    reject(&before, &valid, unknown, TransitionError::UnsupportedRule);
}

#[test]
fn causal_rbi_accumulates_but_never_repairs_an_actors_unknown_pa() {
    for first_known in [false, true] {
        let before = state(4, 0, Half::Top);
        let first = checked(
            &before,
            hit(&before, 1, first_known, FieldingReview::NoMisplay),
        );
        let mut current = first.after().clone();
        let mut log = vec![first];
        for _ in 0..8 {
            let t = checked(&current, hit(&current, 3, true, FieldingReview::NoMisplay));
            current = t.after().clone();
            log.push(t);
        }
        assert_eq!(current.teams[0].next_batter, 0);
        let repeated = checked(&current, hit(&current, 3, true, FieldingReview::NoMisplay));
        assert_eq!(repeated.sole_safe_hit_rbi(), named("a0", 1));
        log.push(repeated);
        let lines = derive_accounting(&log).unwrap();
        assert_eq!(
            lines.batters[0]["a0"].classified_rbi,
            first_known.then_some(2)
        );
        assert_eq!(lines.batters[0]["a0"].credited_rbi, 2);
        assert_eq!(lines.batters[0]["a0"].plate_appearances, 2);
        assert_eq!(lines.batters[0]["a1"].classified_rbi, Some(1));
    }
}

#[test]
fn previous_prefix_and_invalid_new_option_are_encoding_not_semantic_errors() {
    let before = state(7, 0, Half::Top);
    let generic = checked(&before, hit(&before, 1, false, FieldingReview::NoMisplay));
    let original = event_bytes(generic.event());
    let bytes = generic.to_bytes();
    let offset = bytes
        .windows(original.len())
        .position(|w| w == original)
        .unwrap()
        + original.len()
        - 2; // safe-hit option precedes the new muff None
    let mut invalid = bytes.clone();
    assert_eq!(invalid[offset], 0);
    invalid[offset] = 2;
    assert_eq!(
        EventTransition::from_bytes(&invalid),
        Err(TransitionError::InvalidEncoding)
    );
    let mut previous = bytes.clone();
    previous[..5].copy_from_slice(b"BBE24");
    assert_eq!(
        EventTransition::from_bytes(&previous),
        Err(TransitionError::InvalidEncoding)
    );
    let mut old = bytes;
    old[..5].copy_from_slice(b"BBE23");
    assert_eq!(
        EventTransition::from_bytes(&old),
        Err(TransitionError::InvalidEncoding)
    );
}

#[test]
fn reviewed_exclusive_safe_hits_prove_zero_po_a_in_576_independent_frames() {
    let mut frames = 0;
    for value in 1..=3 {
        for mask in 0..8 {
            for outs in 0..3 {
                for half in [Half::Top, Half::Bottom] {
                    let before = state(mask, outs, half);
                    for reverse_moves in [false, true] {
                        for reverse_credits in [false, true] {
                            let mut event = hit(&before, value, true, FieldingReview::NoMisplay);
                            if reverse_moves {
                                event.delta.moves.reverse();
                            }
                            if reverse_credits {
                                event.delta.credits.reverse();
                            }
                            assert_eq!(event.delta.outs, 0);
                            assert!(event.delta.moves.iter().all(|m| m.to != Destination::Out));
                            assert!(!event.delta.credits.iter().any(|c| matches!(
                                c.kind,
                                CreditKind::Putout | CreditKind::Assist
                            )));
                            let t = checked(&before, event);
                            let expected = bbsim_engine::accounting::FieldingContribution {
                                putouts: std::collections::BTreeMap::new(),
                                assists: std::collections::BTreeMap::new(),
                            };
                            assert_eq!(t.sole_safe_hit_fielding(), Some(expected.clone()));
                            // Qualified absent actors are zero only in this before-defense.
                            let defense = &before.teams[1 - half.offense()];
                            let qualified = t.sole_safe_hit_fielding().unwrap();
                            let fielders: Vec<_> = defense
                                .batting_order
                                .iter()
                                .enumerate()
                                .filter(|(slot, _)| Some(*slot as u8) != defense.dh_slot)
                                .map(|(_, id)| id)
                                .chain([&defense.pitcher])
                                .collect();
                            assert_eq!(fielders.len(), 9);
                            for id in fielders {
                                assert_eq!(qualified.putouts.get(id).copied().unwrap_or(0), 0);
                                assert_eq!(qualified.assists.get(id).copied().unwrap_or(0), 0);
                            }
                            assert_eq!(t.caught_strikeout_fielding(), None);
                            assert_eq!(t.ordinary_ground_fielding(), None);
                            frames += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(frames, 576);
}

#[test]
fn zero_po_a_requires_exclusive_cause_and_full_review_not_final_safe_bases() {
    for value in 1..=3 {
        let before = state(7, 1, Half::Top);
        let bare = checked(
            &before,
            hit(&before, value, false, FieldingReview::NoMisplay),
        );
        assert_eq!(bare.sole_safe_hit_fielding(), None);
        assert_eq!(bare.sole_safe_hit_rbi(), None);
    }
    let before = state(7, 2, Half::Top);
    for prior_error in [false, true] {
        let mut event = hit(&before, 1, true, FieldingReview::Unknown);
        if prior_error {
            event.delta.credits.push(credit("h4", CreditKind::Error));
        }
        let t = checked(&before, event);
        assert_eq!(t.sole_safe_hit_fielding(), None);
        assert_eq!(t.sole_safe_hit_rbi(), named("a0", 1));
        let lines = derive_accounting(&[t]).unwrap();
        assert_eq!(lines.batters[0]["a0"].classified_rbi, Some(1));
        assert_eq!(lines.pitchers[1]["oldp"].earned_runs, None);
        assert_eq!(
            lines.fielders[1].get("h4").map(|f| f.credited_errors),
            prior_error.then_some(1)
        );
    }
}

#[test]
fn reviewed_held_and_unequal_continuous_advances_have_zero_po_a() {
    let before = state(7, 0, Half::Top);
    let mut event = hit(&before, 1, true, FieldingReview::NoMisplay);
    event.delta.moves[1].to = Destination::Home;
    event.delta.moves[2].to = Destination::Second;
    event.delta.runs = 2;
    event
        .delta
        .credits
        .extend([credit("a3", CreditKind::Run), credit("a0", CreditKind::Rbi)]);
    let t = checked(&before, event);
    assert_eq!(t.sole_safe_hit_rbi(), named("a0", 2));
    assert_eq!(
        t.sole_safe_hit_fielding(),
        Some(bbsim_engine::accounting::FieldingContribution {
            putouts: std::collections::BTreeMap::new(),
            assists: std::collections::BTreeMap::new(),
        })
    );
    // This fielding getter does not broaden the station-only clean ER proof.
    assert_eq!(
        derive_accounting(&[t]).unwrap().pitchers[1]["oldp"].earned_runs,
        None
    );
}

#[test]
fn legal_ordinary_walk_off_preserves_zero_po_a_scope_without_full_box_claim() {
    for inning in [9, 11] {
        let mut before = state(6, 2, Half::Bottom);
        before.inning = inning;
        let mut event = hit(&before, 1, true, FieldingReview::NoMisplay);
        event.delta.moves[1].to = Destination::Second;
        let t = checked(&before, event);
        assert!(t.after().over);
        assert_eq!(t.sole_safe_hit_rbi(), named("h0", 1));
        assert_eq!(
            t.sole_safe_hit_fielding(),
            Some(bbsim_engine::accounting::FieldingContribution {
                putouts: std::collections::BTreeMap::new(),
                assists: std::collections::BTreeMap::new(),
            })
        );
        let lines = derive_accounting(&[t]).unwrap();
        assert!(!lines.complete_game);
        assert_eq!(lines.pitchers[0]["oldp"].earned_runs, None);
    }
}
