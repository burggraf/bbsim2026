use bbsim_engine::{
    accounting::{derive_accounting, RbiContribution},
    apply, BalkCause, BalkDisposition, BalkFacts, BalkResolution, Base, Credit, CreditKind, Delta,
    Destination, Event, EventKind, EventTransition, FieldingReview, GameState, Half, Participants,
    Runner, RunnerMove, SoleHomeRunFacts, TeamState, ThirdOut, ThirdOutKind, TransitionError,
};

fn state(mask: u8, outs: u8, half: Half) -> GameState {
    let team = |prefix: &str| TeamState {
        defense: bbsim_engine::DefenseState::LegacyUnassigned,
        batting_order: std::array::from_fn(|i| format!("{prefix}{i}")),
        next_batter: 0,
        pitcher: (format!("{prefix}p")).into(),
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
fn hr(before: &GameState, cause: bool, review: FieldingReview) -> Event {
    let offense = &before.teams[before.half.offense()];
    let id = offense.batting_order[offense.next_batter as usize].clone();
    let pitcher = before.teams[1 - before.half.offense()]
        .pitcher
        .active_id()
        .expect("legacy fixture pitcher")
        .to_owned();
    let mut moves = vec![RunnerMove {
        runner: Runner {
            player_id: id.clone(),
            responsible_pitcher: pitcher.clone(),
        },
        from: Base::Plate,
        to: Destination::Home,
    }];
    let mut credits = vec![
        credit(&id, CreditKind::PlateAppearance),
        credit(&id, CreditKind::Hit),
        credit(&id, CreditKind::Run),
        credit(&id, CreditKind::Rbi),
    ];
    for (i, from) in [Base::First, Base::Second, Base::Third]
        .into_iter()
        .enumerate()
    {
        if let Some(runner) = &before.bases[i] {
            moves.push(RunnerMove {
                runner: runner.clone(),
                from,
                to: Destination::Home,
            });
            credits.extend([
                credit(&runner.player_id, CreditKind::Run),
                credit(&id, CreditKind::Rbi),
            ]);
        }
    }
    Event {
        kind: EventKind::OutOfParkHomeRun,
        participants: Participants {
            batter: Some(id.clone()),
            pitcher: Some(pitcher),
        },
        delta: Delta {
            runs: moves.len() as u8,
            moves,
            outs: 0,
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
        sole_home_run: cause.then_some(SoleHomeRunFacts { batter_id: id }),
        sole_safe_hit: None,
        ordinary_air_catch: None,
        muffed_ground_ball: None,
    }
}
fn checked(before: &GameState, event: Event) -> EventTransition {
    let frozen = before.clone();
    let t = apply(before, event.clone()).expect("valid independently built HR fixture");
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
    string(&mut out, event.participants.pitcher.as_deref().unwrap());
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
            CreditKind::Assist => 9,
            CreditKind::Error => 10,
            CreditKind::PassedBall => 11,
            CreditKind::Single => 17,
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
            && event.ordinary_ground_throw.is_none()
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
            assert_eq!(balk.resolution, BalkResolution::Unknown);
            out.extend([1, 1, 2, 0]);
        }
    }
    out.extend([0; 3]); // ground, caught K, protected award
    match &event.sole_home_run {
        None => out.push(0),
        Some(facts) => {
            out.push(1);
            string(&mut out, &facts.batter_id);
        }
    }
    assert!(event.sole_safe_hit.is_none() && event.muffed_ground_ball.is_none());
    out.extend([0, 0]);
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
fn known_participant_origin_classification_and_alternative_cause_errors_are_preserved() {
    let before = state(7, 1, Half::Top);
    let valid = hr(&before, true, FieldingReview::Unknown);
    let mut wrong = valid.clone();
    wrong.participants.batter = Some("a2".into());
    wrong.sole_home_run.as_mut().unwrap().batter_id = "a2".into();
    reject(&before, &valid, wrong, TransitionError::InvalidParticipants);
    let mut wrong = valid.clone();
    wrong.participants.pitcher = Some("oldp".into());
    reject(&before, &valid, wrong, TransitionError::InvalidParticipants);
    let mut origin = valid.clone();
    origin
        .delta
        .moves
        .iter_mut()
        .find(|m| m.from == Base::First)
        .unwrap()
        .from = Base::Third;
    reject(&before, &valid, origin, TransitionError::DeltaMismatch);
    let mut hit = valid.clone();
    hit.delta
        .credits
        .iter_mut()
        .find(|c| c.kind == CreditKind::Hit)
        .unwrap()
        .player_id = "a2".into();
    reject(&before, &valid, hit, TransitionError::IllegalCredit);
    for (actor, kind) in [
        ("a0", CreditKind::NoHit),
        ("a0", CreditKind::Single),
        ("hp", CreditKind::WildPitch),
        ("h4", CreditKind::PassedBall),
        ("hp", CreditKind::Balk),
    ] {
        let mut other = valid.clone();
        other.delta.credits.push(credit(actor, kind));
        reject(&before, &valid, other, TransitionError::IllegalCredit);
    }
    let mut contradiction = valid.clone();
    contradiction.fielding_review = FieldingReview::NoMisplay;
    contradiction
        .delta
        .credits
        .push(credit("h4", CreditKind::Error));
    reject(
        &before,
        &valid,
        contradiction,
        TransitionError::IllegalCredit,
    );
}

#[test]
fn sole_hr_option_corruption_and_previous_prefix_are_encoding_errors() {
    let before = state(7, 1, Half::Top);
    let event = hr(&before, true, FieldingReview::Unknown);
    let t = checked(&before, event.clone());
    assert_eq!(
        EventTransition::from_bytes(&splice(&t, &event)),
        Ok(t.clone())
    );
    let mut old = t.to_bytes();
    old[..5].copy_from_slice(b"BBE22");
    assert_eq!(
        EventTransition::from_bytes(&old),
        Err(TransitionError::InvalidEncoding)
    );
    let original = event_bytes(&event);
    let positions: Vec<_> = t
        .to_bytes()
        .windows(original.len())
        .enumerate()
        .filter_map(|(i, b)| (b == original).then_some(i))
        .collect();
    assert_eq!(positions.len(), 1);
    let mut generic = event;
    generic.sole_home_run = None;
    let mut bad = t.to_bytes();
    let offset = positions[0] + event_bytes(&generic).len() - 3;
    assert_eq!(bad[offset], 1);
    bad[offset] = 2;
    assert_eq!(
        EventTransition::from_bytes(&bad),
        Err(TransitionError::InvalidEncoding)
    );
}

#[test]
fn actual_actor_and_exact_rbi_units_are_required_on_all_three_paths() {
    let before = state(7, 1, Half::Top);
    let valid = hr(&before, true, FieldingReview::Unknown);
    let mut wrong = valid.clone();
    wrong.sole_home_run.as_mut().unwrap().batter_id = "a2".into();
    reject(&before, &valid, wrong, TransitionError::InvalidParticipants);
    let mut omitted = valid.clone();
    omitted.delta.credits.retain(|c| c.kind != CreditKind::Rbi);
    reject(&before, &valid, omitted, TransitionError::IllegalCredit);
    let mut short = valid.clone();
    let index = short
        .delta
        .credits
        .iter()
        .position(|c| c.kind == CreditKind::Rbi)
        .unwrap();
    short.delta.credits.remove(index);
    reject(&before, &valid, short, TransitionError::IllegalCredit);
    let mut duplicate = valid.clone();
    duplicate.delta.credits.push(credit("a0", CreditKind::Rbi));
    reject(&before, &valid, duplicate, TransitionError::IllegalCredit);
    let mut foreign = valid.clone();
    foreign
        .delta
        .credits
        .iter_mut()
        .find(|c| c.kind == CreditKind::Rbi)
        .unwrap()
        .player_id = "a2".into();
    reject(&before, &valid, foreign, TransitionError::IllegalCredit);
}

#[test]
fn sole_hr_requires_explicit_subtype_hit_and_exact_runner_origins() {
    let before = state(7, 1, Half::Top);
    let valid = hr(&before, true, FieldingReview::Unknown);
    let mut wrong = valid.clone();
    wrong.kind = EventKind::PlateAppearance;
    reject(&before, &valid, wrong, TransitionError::IllegalStateChange);
    let mut missing = valid.clone();
    missing.delta.credits.retain(|c| c.kind != CreditKind::Hit);
    reject(&before, &valid, missing, TransitionError::IllegalCredit);
    let mut duplicate = valid.clone();
    duplicate.delta.credits.push(credit("a0", CreditKind::Hit));
    reject(&before, &valid, duplicate, TransitionError::IllegalCredit);
    let mut missing_runner = valid.clone();
    missing_runner.delta.moves.retain(|m| m.from != Base::First);
    reject(
        &before,
        &valid,
        missing_runner,
        TransitionError::DeltaMismatch,
    );
    let mut false_charge = valid.clone();
    false_charge
        .delta
        .moves
        .iter_mut()
        .find(|m| m.from == Base::First)
        .unwrap()
        .runner
        .responsible_pitcher = "invented".into();
    reject(
        &before,
        &valid,
        false_charge,
        TransitionError::DeltaMismatch,
    );
    let mut false_home = valid.clone();
    false_home.delta.moves[0].to = Destination::First;
    reject(&before, &valid, false_home, TransitionError::DeltaMismatch);
    let mut false_out = valid.clone();
    false_out.delta.outs = 1;
    reject(&before, &valid, false_out, TransitionError::DeltaMismatch);
    let mut false_third = valid.clone();
    false_third.delta.third_out = Some(ThirdOut {
        runner_id: "a0".into(),
        kind: ThirdOutKind::BatterBeforeFirst,
        home_before_out: vec![],
    });
    reject(&before, &valid, false_third, TransitionError::DeltaMismatch);
}

#[test]
fn known_causal_contradictions_precede_unrelated_compound_uncertainty() {
    let before = state(7, 1, Half::Top);
    let valid = hr(&before, true, FieldingReview::Unknown);
    let mut compound = valid.clone();
    compound.balk = Some(BalkFacts {
        cause: BalkCause::PitcherAction602a,
        disposition: BalkDisposition::PlayProceedsWithoutReference,
        resolution: BalkResolution::Unknown,
    });
    reject(
        &before,
        &valid,
        compound.clone(),
        TransitionError::UnsupportedRule,
    );
    let mut wrong_actor = compound.clone();
    wrong_actor.sole_home_run.as_mut().unwrap().batter_id = "a2".into();
    reject(
        &before,
        &valid,
        wrong_actor,
        TransitionError::InvalidParticipants,
    );
    let mut wrong_units = compound.clone();
    wrong_units
        .delta
        .credits
        .retain(|c| c.kind != CreditKind::Rbi);
    reject(&before, &valid, wrong_units, TransitionError::IllegalCredit);
    compound.sole_home_run = None;
    reject(&before, &valid, compound, TransitionError::UnsupportedRule);
}

#[test]
fn sole_hr_cause_has_literal_rbi_for_every_occupancy_out_half_review_and_order() {
    for mask in 0u8..8 {
        for outs in 0..3 {
            for half in [Half::Top, Half::Bottom] {
                for review in [FieldingReview::Unknown, FieldingReview::NoMisplay] {
                    for (reverse_moves, reverse_credits) in
                        [(false, false), (false, true), (true, false), (true, true)]
                    {
                        let before = state(mask, outs, half);
                        let mut event = hr(&before, true, review);
                        let id = event.participants.batter.clone().unwrap();
                        let pitcher = event
                            .participants
                            .pitcher
                            .as_deref()
                            .expect("legacy fixture pitcher")
                            .to_owned();
                        if reverse_moves {
                            event.delta.moves.reverse();
                        }
                        if reverse_credits {
                            event.delta.credits.reverse();
                        }
                        let t = checked(&before, event);
                        let runs = 1 + mask.count_ones();
                        assert_eq!(t.sole_home_run_rbi(), named(&id, runs));
                        assert_eq!(t.rbi_contribution(), named(&id, runs));
                        assert_eq!(t.protected_award_rbi(), None);
                        assert_eq!(u32::from(t.after().score[half.offense()]), runs);
                        assert_eq!(t.after().outs, outs);
                        assert_eq!(t.after().bases, [None, None, None]);
                        assert_eq!(t.after().teams[half.offense()].next_batter, 1);
                        let lines = derive_accounting(&[t]).unwrap();
                        let batter = &lines.batters[half.offense()][&id];
                        assert_eq!(batter.plate_appearances, 1);
                        assert_eq!(batter.at_bats, Some(1));
                        assert_eq!(batter.classified_hits, Some(1));
                        assert_eq!(batter.credited_hit_breakdown.as_ref().unwrap().home_runs, 1);
                        assert_eq!(
                            (batter.credited_rbi, batter.classified_rbi),
                            (runs, Some(runs))
                        );
                        assert_eq!(lines.pitchers[1 - half.offense()][&pitcher].runs_allowed, 1);
                        if mask != 0 {
                            assert_eq!(
                                lines.pitchers[1 - half.offense()]["oldp"].runs_allowed,
                                mask.count_ones()
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn bare_hr_with_correct_or_omitted_supplied_units_remains_unknown() {
    for mask in 0u8..8 {
        for omitted in [false, true] {
            let before = state(mask, 0, Half::Top);
            let mut event = hr(&before, false, FieldingReview::NoMisplay);
            let id = event.participants.batter.clone().unwrap();
            if omitted {
                event.delta.credits.retain(|c| c.kind != CreditKind::Rbi);
            }
            let t = checked(&before, event);
            assert_eq!(t.sole_home_run_rbi(), None);
            assert_eq!(t.rbi_contribution(), None);
            let lines = derive_accounting(&[t]).unwrap();
            assert_eq!(lines.batters[0][&id].classified_rbi, None);
        }
    }
}

#[test]
fn unknown_review_and_prior_foul_chance_error_do_not_erase_the_unaided_hr_cause() {
    let before = state(7, 1, Half::Top);
    let mut event = hr(&before, true, FieldingReview::Unknown);
    let id = event.participants.batter.clone().unwrap();
    event
        .delta
        .credits
        .push(credit(&before.teams[1].batting_order[4], CreditKind::Error));
    let t = checked(&before, event);
    assert_eq!(t.sole_home_run_rbi(), named(&id, 4));
    assert_eq!(t.rbi_contribution(), named(&id, 4));
    assert_eq!(t.ordinary_ground_fielding(), None);
    assert_eq!(t.caught_strikeout_fielding(), None);
    let lines = derive_accounting(&[t]).unwrap();
    assert_eq!(lines.batters[0][&id].classified_rbi, Some(4));
    assert_eq!(lines.pitchers[1]["hp"].earned_runs, None);
    assert_eq!(lines.pitchers[1]["oldp"].earned_runs, None);
}

#[test]
fn ninth_and_extra_inning_endings_keep_every_hr_run_and_rbi() {
    for inning in [9, 12] {
        for half in [Half::Top, Half::Bottom] {
            let mut before = state(7, 2, half);
            before.inning = inning;
            before.score = if half == Half::Bottom {
                [13, 10]
            } else {
                [10, 10]
            };
            let event = hr(&before, true, FieldingReview::NoMisplay);
            let id = event.participants.batter.clone().unwrap();
            let t = checked(&before, event);
            assert_eq!(t.after().over, half == Half::Bottom);
            assert_eq!(t.after().inning, inning);
            assert_eq!(t.after().half, half);
            assert_eq!(t.after().outs, 2);
            assert_eq!(
                t.after().score,
                if half == Half::Bottom {
                    [13, 14]
                } else {
                    [14, 10]
                }
            );
            assert_eq!(t.rbi_contribution(), named(&id, 4));
            let lines = derive_accounting(&[t]).unwrap();
            assert_eq!(lines.batters[half.offense()][&id].classified_rbi, Some(4));
            assert_eq!(lines.pitchers[1 - half.offense()]["oldp"].runs_allowed, 3);
        }
    }
}

#[test]
fn mixed_inherited_charges_do_not_become_current_pitcher_participation() {
    let mut before = state(7, 1, Half::Bottom);
    before.bases[1].as_mut().unwrap().responsible_pitcher = "ap".into();
    let event = hr(&before, true, FieldingReview::Unknown);
    let t = checked(&before, event);
    assert_eq!(t.rbi_contribution(), named("h0", 4));
    let lines = derive_accounting(&[t]).unwrap();
    assert_eq!(
        (
            lines.pitchers[0]["ap"].runs_allowed,
            lines.pitchers[0]["ap"].batters_faced
        ),
        (2, 1)
    );
    assert_eq!(
        (
            lines.pitchers[0]["oldp"].runs_allowed,
            lines.pitchers[0]["oldp"].batters_faced
        ),
        (2, 0)
    );
}

#[test]
fn sole_hr_rbi_sums_but_never_repairs_an_actors_unknown_pa() {
    for (first, tenth) in [(true, true), (false, true), (true, false)] {
        let mut before = state(0, 0, Half::Top);
        let mut stream = Vec::new();
        for n in 0..10 {
            let cause = if n == 0 {
                first
            } else if n == 9 {
                tenth
            } else {
                true
            };
            let event = hr(&before, cause, FieldingReview::Unknown);
            let t = checked(&before, event);
            before = t.after().clone();
            stream.push(t);
        }
        let lines = derive_accounting(&stream).unwrap();
        assert_eq!(
            lines.batters[0]["a0"].classified_rbi,
            (first && tenth).then_some(2)
        );
        assert_eq!(lines.batters[0]["a0"].credited_rbi, 2);
        for n in 1..9 {
            assert_eq!(lines.batters[0][&format!("a{n}")].classified_rbi, Some(1));
        }
    }
}

#[test]
fn reviewed_sole_hr_has_zero_po_a_in_192_independent_order_frames() {
    let mut frames = 0;
    for mask in 0..8 {
        for outs in 0..3 {
            for half in [Half::Top, Half::Bottom] {
                let before = state(mask, outs, half);
                for reverse_moves in [false, true] {
                    for reverse_credits in [false, true] {
                        let mut event = hr(&before, true, FieldingReview::NoMisplay);
                        if reverse_moves {
                            event.delta.moves.reverse();
                        }
                        if reverse_credits {
                            event.delta.credits.reverse();
                        }
                        let t = checked(&before, event);
                        assert_eq!(
                            t.sole_home_run_fielding(),
                            Some(bbsim_engine::accounting::FieldingContribution {
                                putouts: std::collections::BTreeMap::new(),
                                assists: std::collections::BTreeMap::new(),
                            })
                        );
                        assert_eq!(t.sole_safe_hit_fielding(), None);
                        assert_eq!(t.protected_award_fielding(), None);
                        assert_eq!(t.before(), &before);
                        assert_eq!(t.event().delta.outs, 0);
                        frames += 1;
                    }
                }
            }
        }
    }
    assert_eq!(frames, 192);
}

#[test]
fn hr_zero_fielding_requires_cause_and_review_and_keeps_prior_chances_partial() {
    let before = state(7, 0, Half::Top);
    let mut bare = hr(&before, false, FieldingReview::NoMisplay);
    bare.delta.credits.push(credit("h3", CreditKind::Assist));
    let t = checked(&before, bare);
    assert_eq!(t.sole_home_run_fielding(), None);
    assert_eq!(
        derive_accounting(&[t]).unwrap().fielders[1]["h3"].credited_assists,
        1
    );
    for earlier_chance in [false, true] {
        let mut event = hr(&before, true, FieldingReview::Unknown);
        if earlier_chance {
            event.delta.credits.extend([
                credit("h3", CreditKind::Assist),
                credit("h4", CreditKind::Error),
            ]);
        }
        let t = checked(&before, event);
        assert_eq!(t.sole_home_run_fielding(), None);
        assert_eq!(t.sole_home_run_rbi(), named("a0", 4));
        let lines = derive_accounting(&[t]).unwrap();
        assert_eq!(lines.batters[0]["a0"].credited_rbi, 4);
        assert_eq!(lines.pitchers[1]["oldp"].earned_runs, None);
    }
}

#[test]
fn reviewed_bound_hr_assist_is_known_illegal_before_compound_uncertainty() {
    let before = state(7, 1, Half::Top);
    let valid = hr(&before, true, FieldingReview::NoMisplay);
    for compound in [true, false] {
        let mut bad = valid.clone();
        bad.delta.credits.push(credit("h3", CreditKind::Assist));
        if compound {
            bad.balk = Some(BalkFacts {
                cause: BalkCause::PitcherAction602a,
                disposition: BalkDisposition::PlayProceedsWithoutReference,
                resolution: BalkResolution::Unknown,
            });
        }
        reject(&before, &valid, bad, TransitionError::IllegalCredit);
    }
}

#[test]
fn reviewed_hr_walk_off_exception_retains_zero_fielding_and_all_rbi() {
    for inning in [9, 11] {
        let mut before = state(7, 2, Half::Bottom);
        before.inning = inning;
        let t = checked(&before, hr(&before, true, FieldingReview::NoMisplay));
        assert!(t.after().over);
        assert_eq!(t.sole_home_run_rbi(), named("h0", 4));
        assert_eq!(
            t.sole_home_run_fielding(),
            Some(bbsim_engine::accounting::FieldingContribution {
                putouts: std::collections::BTreeMap::new(),
                assists: std::collections::BTreeMap::new(),
            })
        );
        let lines = derive_accounting(&[t]).unwrap();
        assert!(!lines.complete_game);
        assert_eq!(lines.batters[1]["h0"].classified_rbi, Some(4));
    }
}
