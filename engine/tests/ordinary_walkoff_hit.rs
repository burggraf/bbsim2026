use bbsim_engine::accounting::derive_accounting;
use bbsim_engine::{
    apply, apply_with_walkoff_hit_facts, Base, Credit, CreditKind, Delta, Destination, Event,
    EventKind, EventTransition, FieldingReview, GameState, Half, NamedBaseTouch,
    NaturalCourseTouchTiming, Participants, Runner, RunnerMove, SoleSafeHitFacts, TeamState,
    TransitionError, WalkoffHitFacts,
};

fn fixture(mask: u8, outs: u8) -> GameState {
    let teams = std::array::from_fn(|side| TeamState {
        batting_order: std::array::from_fn(|i| {
            // Before runners h1/h2/h3 must not be the queued next batter h8.
            let slot = match i {
                1 => 8,
                8 => 1,
                _ => i,
            };
            format!("{}{}", ["a", "h"][side], slot)
        }),
        next_batter: 0,
        pitcher: format!("p{side}"),
        dh_slot: Some(8),
        dh_must_bat: false,
        bench: vec![],
        removed: vec![],
    });
    GameState {
        inning: 9,
        half: Half::Bottom,
        outs,
        score: [10, 10],
        bases: std::array::from_fn(|i| {
            (mask & (1 << i) != 0).then(|| Runner {
                player_id: format!("h{}", i + 1),
                responsible_pitcher: "p0".into(),
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
fn event(before: &GameState, value: u8, targets: &[u8]) -> Event {
    let destinations = [
        Destination::First,
        Destination::Second,
        Destination::Third,
        Destination::Home,
    ];
    let mut moves = vec![RunnerMove {
        runner: Runner {
            player_id: "h0".into(),
            responsible_pitcher: "p0".into(),
        },
        from: Base::Plate,
        to: destinations[usize::from(targets[0] - 1)],
    }];
    let mut credits = vec![
        credit("h0", CreditKind::PlateAppearance),
        credit("h0", CreditKind::Hit),
        credit(
            "h0",
            [CreditKind::Single, CreditKind::Double][usize::from(value - 1)],
        ),
    ];
    let mut position = 1;
    let mut runs = 0;
    for (i, runner) in before.bases.iter().enumerate() {
        if let Some(runner) = runner {
            let to = destinations[usize::from(targets[position] - 1)];
            position += 1;
            moves.push(RunnerMove {
                runner: runner.clone(),
                from: [Base::First, Base::Second, Base::Third][i],
                to,
            });
            if to == Destination::Home {
                runs += 1;
                credits.extend([
                    credit(&runner.player_id, CreditKind::Run),
                    credit("h0", CreditKind::Rbi),
                ]);
            }
        }
    }
    Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some("h0".into()),
            pitcher: "p0".into(),
        },
        fielding_review: FieldingReview::NoMisplay,
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
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: Some(SoleSafeHitFacts {
            batter_id: "h0".into(),
        }),
        muffed_ground_ball: None,
        ordinary_air_catch: None,
    }
}
fn facts(winner: &str, base: Destination, timing: NaturalCourseTouchTiming) -> WalkoffHitFacts {
    WalkoffHitFacts {
        winning_runner_id: winner.into(),
        batter_touch: NamedBaseTouch {
            runner_id: "h0".into(),
            base,
        },
        timing,
    }
}
fn after(before: &GameState, event: &Event) -> GameState {
    let mut result = before.clone();
    let offense = before.half.offense();
    result.teams[offense].next_batter = 1;
    result.score[offense] += u16::from(event.delta.runs);
    result.bases = [None, None, None];
    for m in &event.delta.moves {
        let index = match m.to {
            Destination::First => Some(0),
            Destination::Second => Some(1),
            Destination::Third => Some(2),
            _ => None,
        };
        if let Some(index) = index {
            result.bases[index] = Some(m.runner.clone());
        }
    }
    result.over = true;
    result
}
fn checked(before: &GameState, event: &Event, facts: &WalkoffHitFacts) -> EventTransition {
    let original = before.clone();
    let result = apply_with_walkoff_hit_facts(before, event.clone(), facts.clone()).unwrap();
    assert_eq!(*before, original);
    assert_eq!(result.after(), &after(before, event));
    assert_eq!(result.walkoff_hit_facts(), Some(facts));
    assert_eq!(
        EventTransition::try_new_with_walkoff_hit_facts(
            before.clone(),
            event.clone(),
            result.after().clone(),
            facts.clone()
        ),
        Ok(result.clone())
    );
    assert_eq!(
        EventTransition::from_bytes(&result.to_bytes()),
        Ok(result.clone())
    );
    result
}
fn refused(before: &GameState, event: &Event, facts: &WalkoffHitFacts, error: TransitionError) {
    let original = before.clone();
    assert_eq!(
        apply_with_walkoff_hit_facts(before, event.clone(), facts.clone()),
        Err(error.clone())
    );
    assert_eq!(*before, original);
    assert_eq!(
        EventTransition::try_new_with_walkoff_hit_facts(
            before.clone(),
            event.clone(),
            after(before, event),
            facts.clone()
        ),
        Err(error)
    );
}

#[test]
fn named_winner_resolves_mixed_ceiling_without_vector_order_inference() {
    let mut before = fixture(5, 1);
    before.score = [11, 10];
    let mut event = event(&before, 2, &[2, 4, 4]);
    let facts = facts(
        "h1",
        Destination::Second,
        NaturalCourseTouchTiming::ByWinningRun,
    );
    assert_eq!(
        apply(&before, event.clone()),
        Err(TransitionError::UnsupportedRule)
    );
    let first = checked(&before, &event, &facts);
    event.delta.moves.reverse();
    event.delta.credits.reverse();
    let second = checked(&before, &event, &facts);
    assert_eq!(first.after(), second.after());
    assert_eq!(
        derive_accounting(&[first]).unwrap(),
        derive_accounting(&[second]).unwrap()
    );
}

#[test]
fn natural_course_second_after_winning_run_is_not_instant_clipped() {
    let before = fixture(2, 2);
    let event = event(&before, 2, &[2, 4]);
    let facts = facts(
        "h2",
        Destination::Second,
        NaturalCourseTouchTiming::AfterWinningRun,
    );
    let result = checked(&before, &event, &facts);
    assert!(result.after().over);
    assert_eq!(result.after().score, [10, 11]);
    assert_eq!(result.after().teams[1].next_batter, 1);
    assert_eq!(result.sole_safe_hit_rbi().unwrap().runs_batted_in["h0"], 1);
    let accounting = derive_accounting(std::slice::from_ref(&result)).unwrap();
    assert_eq!(accounting.runs, [0, 1]);
    // Legacy reducer first sees the stale batter; canonical construction also
    // independently refuses any transition whose before state is already over.
    assert_eq!(
        apply(result.after(), event.clone()),
        Err(TransitionError::InvalidParticipants)
    );
    assert_eq!(
        EventTransition::try_new(
            result.after().clone(),
            event.clone(),
            result.after().clone()
        ),
        Err(TransitionError::InvalidState)
    );
}

#[test]
fn incorrect_winner_and_batter_touch_identities_refuse() {
    let before = fixture(6, 0);
    let event = event(&before, 1, &[1, 3, 4]);
    for id in ["", "missing", "a1", "h0", "h2"] {
        refused(
            &before,
            &event,
            &facts(
                id,
                Destination::First,
                NaturalCourseTouchTiming::ByWinningRun,
            ),
            TransitionError::InvalidParticipants,
        );
    }
    let mut f = facts(
        "h3",
        Destination::First,
        NaturalCourseTouchTiming::ByWinningRun,
    );
    f.batter_touch.runner_id = "h3".into();
    refused(&before, &event, &f, TransitionError::InvalidParticipants);
}

#[test]
fn winner_ceiling_and_continuous_no_passing_constraint_are_not_bypassed() {
    let mut before = fixture(5, 0);
    before.score = [11, 10];
    let double = event(&before, 2, &[2, 4, 4]);
    refused(
        &before,
        &double,
        &facts(
            "h3",
            Destination::Second,
            NaturalCourseTouchTiming::ByWinningRun,
        ),
        TransitionError::IllegalCredit,
    );
    let single = event(&before, 1, &[1, 4, 4]);
    refused(
        &before,
        &single,
        &facts(
            "h3",
            Destination::First,
            NaturalCourseTouchTiming::ByWinningRun,
        ),
        TransitionError::UnsupportedRule,
    );
}

#[test]
fn passing_a_held_preceding_runner_cannot_be_certified_as_an_ordinary_safe_hit() {
    let b = fixture(3, 0);
    let e = event(&b, 1, &[1, 4, 3]); // First scores past Second held at Third.
    let f = facts(
        "h1",
        Destination::First,
        NaturalCourseTouchTiming::ByWinningRun,
    );
    refused(&b, &e, &f, TransitionError::UnsupportedRule);
    let b = fixture(5, 0);
    let e = event(&b, 1, &[2, 1, 4]); // Batter goes past First held at First.
    let f = facts(
        "h3",
        Destination::Second,
        NaturalCourseTouchTiming::ByWinningRun,
    );
    refused(&b, &e, &f, TransitionError::UnsupportedRule);
}

#[test]
fn unknown_timing_and_incompatible_physical_touch_refuse() {
    let before = fixture(2, 0);
    let event = event(&before, 2, &[2, 4]);
    refused(
        &before,
        &event,
        &facts("h2", Destination::Second, NaturalCourseTouchTiming::Unknown),
        TransitionError::UnsupportedRule,
    );
    for base in [
        Destination::First,
        Destination::Third,
        Destination::Home,
        Destination::Out,
    ] {
        refused(
            &before,
            &event,
            &facts("h2", base, NaturalCourseTouchTiming::ByWinningRun),
            TransitionError::UnsupportedRule,
        );
    }
}

#[test]
fn missing_cause_review_or_classification_does_not_become_a_certificate() {
    let before = fixture(4, 0);
    let original = event(&before, 1, &[1, 4]);
    let f = facts(
        "h3",
        Destination::First,
        NaturalCourseTouchTiming::ByWinningRun,
    );
    let mut e = original.clone();
    e.sole_safe_hit = None;
    refused(&before, &e, &f, TransitionError::UnsupportedRule);
    let mut e = original.clone();
    e.fielding_review = FieldingReview::Unknown;
    refused(&before, &e, &f, TransitionError::UnsupportedRule);
    let mut e = original;
    e.delta.credits.retain(|c| c.kind != CreditKind::Single);
    refused(&before, &e, &f, TransitionError::IllegalCredit);
}

#[test]
fn exact_winning_count_and_late_bottom_context_are_required() {
    let before = fixture(4, 0);
    let e = event(&before, 1, &[1, 4]);
    let f = facts(
        "h3",
        Destination::First,
        NaturalCourseTouchTiming::ByWinningRun,
    );
    for (inning, score) in [(8, [10, 10]), (9, [11, 10]), (9, [9, 10])] {
        let mut b = before.clone();
        b.inning = inning;
        b.score = score;
        refused(&b, &e, &f, TransitionError::UnsupportedRule);
    }
    let mut b = before.clone();
    b.half = Half::Top;
    b.teams.swap(0, 1); // Preserve valid literal participants in the other half.
    refused(&b, &e, &f, TransitionError::UnsupportedRule);
    let b = fixture(6, 0);
    let e = event(&b, 1, &[1, 4, 4]);
    refused(
        &b,
        &e,
        &facts(
            "h2",
            Destination::First,
            NaturalCourseTouchTiming::ByWinningRun,
        ),
        TransitionError::UnsupportedRule,
    );
}

#[test]
fn empty_bases_triples_and_retirements_remain_outside_this_family() {
    let b = fixture(0, 0);
    let e = event(&b, 1, &[1]);
    let f = facts(
        "h3",
        Destination::First,
        NaturalCourseTouchTiming::ByWinningRun,
    );
    refused(&b, &e, &f, TransitionError::UnsupportedRule);
    let b = fixture(1, 0);
    let mut e = event(&b, 2, &[3, 4]);
    for c in &mut e.delta.credits {
        if c.kind == CreditKind::Double {
            c.kind = CreditKind::Triple;
        }
    }
    let f = facts(
        "h1",
        Destination::Third,
        NaturalCourseTouchTiming::ByWinningRun,
    );
    refused(&b, &e, &f, TransitionError::UnsupportedRule);
    let b = fixture(4, 0);
    let mut e = event(&b, 1, &[1, 4]);
    e.delta.moves[1].to = Destination::Out;
    e.delta.outs = 1;
    e.delta.runs = 0;
    e.delta
        .credits
        .retain(|c| !matches!(c.kind, CreditKind::Run | CreditKind::Rbi));
    let f = facts(
        "h3",
        Destination::First,
        NaturalCourseTouchTiming::ByWinningRun,
    );
    // A retirement is a known contradiction of the existing exclusive safe cause.
    refused(&b, &e, &f, TransitionError::DeltaMismatch);
}

#[test]
fn maximum_score_gap_refuses_without_arithmetic_overflow() {
    let mut b = fixture(4, 0);
    b.score = [u16::MAX, 0];
    let e = event(&b, 1, &[1, 4]);
    let f = facts(
        "h3",
        Destination::First,
        NaturalCourseTouchTiming::ByWinningRun,
    );
    refused(&b, &e, &f, TransitionError::UnsupportedRule);
}

#[test]
fn canonical_bad_units_origins_and_after_state_remain_invalid() {
    let b = fixture(2, 0);
    let original = event(&b, 2, &[2, 4]);
    let f = facts(
        "h2",
        Destination::Second,
        NaturalCourseTouchTiming::ByWinningRun,
    );
    let mut e = original.clone();
    e.delta.runs = 2;
    assert!(apply_with_walkoff_hit_facts(&b, e, f.clone()).is_err());
    let mut e = original.clone();
    e.delta.credits.push(credit("h2", CreditKind::Run));
    assert!(apply_with_walkoff_hit_facts(&b, e, f.clone()).is_err());
    let mut e = original.clone();
    e.delta.moves[1].runner.responsible_pitcher = "wrong".into();
    refused(&b, &e, &f, TransitionError::DeltaMismatch);
    let mut wrong_after = after(&b, &original);
    wrong_after.score[1] += 1;
    assert_eq!(
        EventTransition::try_new_with_walkoff_hit_facts(b, original, wrong_after, f),
        Err(TransitionError::DeltaMismatch)
    );
}

#[test]
fn legacy_bytes_and_debug_are_exact_when_facts_are_absent() {
    let b = fixture(2, 0);
    let e = event(&b, 2, &[2, 4]);
    let old = apply(&b, e.clone()).unwrap();
    let raw = old.to_bytes();
    assert_eq!(&raw[..5], b"BBE25");
    assert_eq!(old.walkoff_hit_facts(), None);
    assert_eq!(
        format!("{old:?}"),
        format!(
            "EventTransition {{ before: {:?}, event: {:?}, after: {:?} }}",
            old.before(),
            old.event(),
            old.after()
        )
    );
    assert_eq!(EventTransition::from_bytes(&raw).unwrap().to_bytes(), raw);
    assert_eq!(EventTransition::try_new(b, e, old.after().clone()), Ok(old));
}

#[test]
fn new_codec_roundtrip_and_byte_tampering_revalidate_facts() {
    let b = fixture(2, 0);
    let e = event(&b, 2, &[2, 4]);
    let f = facts(
        "h2",
        Destination::Second,
        NaturalCourseTouchTiming::AfterWinningRun,
    );
    let t = checked(&b, &e, &f);
    let raw = t.to_bytes();
    assert_eq!(&raw[..5], b"BBE27");
    for end in 0..raw.len() {
        assert!(
            EventTransition::from_bytes(&raw[..end]).is_err(),
            "truncation {end}"
        );
    }
    let mut bad = raw.clone();
    bad.push(0);
    assert_eq!(
        EventTransition::from_bytes(&bad),
        Err(TransitionError::InvalidEncoding)
    );
    let mut bad = raw.clone();
    bad[4] = b'8';
    assert_eq!(
        EventTransition::from_bytes(&bad),
        Err(TransitionError::InvalidEncoding)
    );
    let mut bad = raw.clone();
    *bad.last_mut().unwrap() = 255;
    assert_eq!(
        EventTransition::from_bytes(&bad),
        Err(TransitionError::InvalidEncoding)
    );
    let mut bad = raw.clone();
    *bad.last_mut().unwrap() = 0;
    assert_eq!(
        EventTransition::from_bytes(&bad),
        Err(TransitionError::UnsupportedRule)
    );
    // Required tail: winner String + named batter String/base + timing.
    let tail = raw.len() - (8 + 2 + 8 + 2 + 1 + 1);
    let mut bad = raw;
    bad[tail + 9] = b'9';
    assert_eq!(
        EventTransition::from_bytes(&bad),
        Err(TransitionError::InvalidParticipants)
    );
}

fn vectors(origins: &[u8], position: usize, current: &mut Vec<u8>, out: &mut Vec<Vec<u8>>) {
    if position == origins.len() {
        out.push(current.clone());
        return;
    }
    for destination in origins[position].max(1)..=4 {
        if position == 0 && destination == 4 {
            continue;
        }
        if current
            .iter()
            .any(|earlier| *earlier > destination || *earlier == destination && destination != 4)
        {
            continue;
        }
        current.push(destination);
        vectors(origins, position + 1, current, out);
        current.pop();
    }
}

#[test]
fn cartesian_named_winner_matrix_is_order_invariant_and_conserving() {
    let mut counts = [0u64; 3]; // accepted, ceiling contradiction, impossible winner order
    for mask in 1..8 {
        let origins: Vec<u8> = std::iter::once(0)
            .chain((1..=3).filter(|i| mask & (1 << (i - 1)) != 0))
            .collect();
        let mut support = vec![];
        vectors(&origins, 0, &mut vec![], &mut support);
        for targets in support {
            let scorers: Vec<_> = origins
                .iter()
                .zip(&targets)
                .filter(|(_, target)| **target == 4)
                .map(|(origin, _)| *origin)
                .collect();
            if scorers.is_empty() {
                continue;
            }
            for value in 1..=2 {
                if value > targets[0] {
                    continue;
                }
                for inning in [9, 10] {
                    for outs in 0..3 {
                        for winner in &scorers {
                            for phase in [
                                NaturalCourseTouchTiming::ByWinningRun,
                                NaturalCourseTouchTiming::AfterWinningRun,
                            ] {
                                for reverse in [false, true] {
                                    let mut b = fixture(mask, outs);
                                    b.inning = inning;
                                    b.score = [10, 11 - scorers.len() as u16];
                                    let mut e = event(&b, value, &targets);
                                    if reverse {
                                        e.delta.moves.reverse();
                                        e.delta.credits.reverse();
                                    }
                                    let f = facts(
                                        &format!("h{winner}"),
                                        [
                                            Destination::First,
                                            Destination::Second,
                                            Destination::Third,
                                        ][usize::from(targets[0] - 1)],
                                        phase,
                                    );
                                    if value > 4 - winner {
                                        refused(&b, &e, &f, TransitionError::IllegalCredit);
                                        counts[1] += 1;
                                    } else if scorers.iter().any(|origin| origin < winner) {
                                        refused(&b, &e, &f, TransitionError::UnsupportedRule);
                                        counts[2] += 1;
                                    } else {
                                        let t = checked(&b, &e, &f);
                                        assert_eq!(t.after().score[1], 11);
                                        counts[0] += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(counts.iter().all(|count| *count > 0));
    println!("ordinary walkoff Cartesian accepted/ceiling/order: {counts:?}");
}
