//! Canonical supplied transitions; no automatic facts, sampled proxy classes or manager policy.
#[path = "../examples/support/contact_consequence.rs"]
mod contact_consequence;
#[path = "../examples/support/preview.rs"]
mod preview;
use bbsim_engine::{accounting::derive_accounting, *};
use contact_consequence::{project, ContactDecision};
use preview::{Baseline, Outcome, Rng};
use std::collections::VecDeque;

const CSV: &str = "key,value\nformat,preview-counts-v1\ncatalog_sha256,2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583\nquery_version,preview-baseline-v1\nselected_pa,103\nexcluded_ibb,2\nexcluded_interference,1\nstrikeout,20\nwalk,8\nhbp,1\nsingle,15\ndouble,4\ntriple,1\nhome_run,4\nroe,2\ncontact_out,45\n";
fn state(mask: u8, outs: u8) -> GameState {
    GameState {
        inning: 2,
        half: Half::Top,
        outs,
        score: [0, 0],
        over: false,
        teams: ["a", "h"].map(|p| TeamState {
            batting_order: std::array::from_fn(|i| format!("{p}{i}")),
            next_batter: 0,
            pitcher: format!("{p}p"),
            dh_slot: Some(8),
            dh_must_bat: false,
            bench: vec![format!("{p}bench")],
            removed: vec![format!("{p}removed")],
        }),
        bases: std::array::from_fn(|i| {
            (mask & (1 << i) != 0).then(|| Runner {
                player_id: format!("a{}", i + 3),
                responsible_pitcher: format!("inherited-{i}"),
            })
        }),
    }
}
fn ordinary(s: &GameState, outcome: Outcome) -> EventTransition {
    apply(s, preview::event_for(s, outcome).unwrap()).unwrap()
}
fn finish(s: &GameState) -> EventTransition {
    ordinary(
        s,
        if s.half == Half::Bottom && s.inning >= 9 && s.score[1] <= s.score[0] {
            Outcome::HomeRun
        } else {
            Outcome::Strikeout
        },
    )
}
fn supplied(s: &GameState, family: u8) -> EventTransition {
    assert_eq!(
        contact_consequence::POLICY_VERSION,
        "supplied-contact-consequence-projection-v1"
    );
    let offense = &s.teams[s.half.offense()];
    let defense = &s.teams[1 - s.half.offense()];
    let batter = offense.batting_order[usize::from(offense.next_batter)].clone();
    let choice = match family {
        0 => ContactDecision::CaughtSacrificeFly {
            catch: OrdinaryAirCatchFacts {
                batter_id: batter,
                fielder_id: defense.batting_order[5].clone(),
                contact: AirContact::Fly,
            },
            facts: CaughtSacrificeFlyFacts {
                scoring_runner_id: s.bases[2].as_ref().unwrap().player_id.clone(),
                handling: SacrificeFlyHandling::QualifyingOutfielderOrInfielderInOutfield,
                retouch: SacrificeFlyRetouch::Legal,
                timing: SacrificeFlyRunTiming::AfterCatch,
            },
        },
        1 => ContactDecision::SacrificeBunt(SacrificeBuntThrowFacts {
            batter_id: batter,
            advancing_runner_id: s.bases[0].as_ref().unwrap().player_id.clone(),
            thrower_id: defense.batting_order[3].clone(),
            receiver_id: defense.batting_order[1].clone(),
            judgment: SacrificeBuntJudgment::SacrificingOwnChance,
        }),
        _ => ContactDecision::GroundForceDoublePlay(GroundForceDoublePlayFacts {
            batter_id: batter,
            forced_runner_id: s.bases[0].as_ref().unwrap().player_id.clone(),
            initial_fielder_id: defense.batting_order[3].clone(),
            pivot_id: defense.batting_order[2].clone(),
            first_receiver_id: defense.batting_order[1].clone(),
        }),
    };
    project(s, &choice).unwrap()
}
fn run_script(initial: &GameState, script: Vec<EventTransition>) -> (Vec<EventTransition>, usize) {
    let mut queue = VecDeque::from(script);
    let mut calls = 0;
    let counter = &mut calls;
    let owned = String::from("owned non-Copy supplied script");
    let log = preview::game_from_transitions(71, initial.clone(), move |s, _| {
        *counter += 1;
        assert!(!owned.is_empty());
        Ok(queue.pop_front().unwrap_or_else(|| finish(s)))
    })
    .unwrap();
    (log, calls)
}
fn bytes(log: &[EventTransition]) -> Vec<Vec<u8>> {
    log.iter().map(EventTransition::to_bytes).collect()
}
fn pa(log: &[EventTransition]) -> u32 {
    derive_accounting(log)
        .unwrap()
        .batters
        .iter()
        .flat_map(|t| t.values())
        .map(|l| l.plate_appearances)
        .sum()
}
fn bf(log: &[EventTransition]) -> u32 {
    derive_accounting(log)
        .unwrap()
        .pitchers
        .iter()
        .flat_map(|t| t.values())
        .map(|l| l.batters_faced)
        .sum()
}

// Independent test-only predecessor loop. This is not another production loop.
fn legacy<F>(seed: u64, mut s: GameState, mut produce: F) -> Result<Vec<EventTransition>, String>
where
    F: FnMut(&GameState, &mut Rng) -> Result<(Outcome, Event), String>,
{
    let mut rng = Rng::new(seed);
    let mut log = Vec::new();
    while !s.over {
        if log.len() >= 10_000 {
            return Err(format!(
                "seed {seed}: resource ceiling reached, no winner fabricated"
            ));
        }
        let (outcome, event) = produce(&s, &mut rng)?;
        let t = apply(&s, event)
            .map_err(|e| format!("seed {seed}, play {}, {outcome:?}: {e:?}", log.len() + 1))?;
        s = t.after().clone();
        log.push(t);
    }
    Ok(log)
}
#[test]
fn existing_versions_and_synthetic_baseline_are_unchanged() {
    assert_eq!(preview::MODEL_VERSION, "pooled-jeffreys-preview-v1");
    assert_eq!(preview::POLICY_VERSION, "station-ground-contact-v1");
    assert_eq!(preview::RNG_VERSION, "splitmix64-v1");
    let a = Baseline::read_csv(CSV).unwrap();
    let b = Baseline::from_counts([20, 8, 1, 15, 4, 1, 4, 2, 45]).unwrap();
    assert_eq!(a.counts, b.counts);
    assert_eq!(a.probabilities(), b.probabilities());
    assert_eq!(
        bytes(&preview::game(1, &a).unwrap()),
        bytes(&preview::game(1, &b).unwrap())
    );
}
#[test]
fn both_paths_match_independent_predecessor_bytes_and_callback_rng_words() {
    let baseline = Baseline::read_csv(CSV).unwrap();
    for seed in (0..128).chain([u64::MAX]) {
        let initial = state(0, 0);
        let mut expected_words = Vec::new();
        let expected = legacy(seed, initial.clone(), |s, r| {
            expected_words.push(r.next_u64());
            let o = baseline.sample(r);
            Ok((o, preview::event_for(s, o)?))
        })
        .unwrap();
        let mut ordinary_words = Vec::new();
        let old = preview::game_from(seed, initial.clone(), |s, r| {
            ordinary_words.push(r.next_u64());
            let o = baseline.sample(r);
            Ok((o, preview::event_for(s, o)?))
        })
        .unwrap();
        let mut typed_words = Vec::new();
        let typed = preview::game_from_transitions(seed, initial.clone(), |s, r| {
            typed_words.push(r.next_u64());
            let o = baseline.sample(r);
            apply(s, preview::event_for(s, o)?).map_err(|e| format!("{e:?}"))
        })
        .unwrap();
        assert_eq!(bytes(&old), bytes(&expected), "ordinary seed {seed}");
        assert_eq!(bytes(&typed), bytes(&expected), "typed seed {seed}");
        assert_eq!(ordinary_words, expected_words);
        assert_eq!(typed_words, expected_words);
        assert_eq!(initial, state(0, 0));
    }
}
#[test]
fn supplied_sf_bunt_and_force_relay_grids_retain_every_transition_fact_and_pa() {
    for family in 0..3 {
        let masks: &[u8] = if family == 0 { &[4, 5, 6, 7] } else { &[1, 5] };
        for &mask in masks {
            for outs in 0..2 {
                let initial = state(mask, outs);
                let saved = initial.clone();
                let first = supplied(&initial, family);
                let original = first.clone();
                let encoded = first.to_bytes();
                assert!(encoded
                    .starts_with([b"BBE28".as_slice(), b"BBE29", b"BBE30"][usize::from(family)]));
                let (log, calls) = run_script(&initial, vec![first]);
                assert!(!log.is_empty());
                assert_eq!(log[0], original);
                assert_eq!(log[0].to_bytes(), encoded);
                assert_eq!(
                    log[0].caught_sacrifice_fly_facts(),
                    original.caught_sacrifice_fly_facts()
                );
                assert_eq!(
                    log[0].sacrifice_bunt_throw_facts(),
                    original.sacrifice_bunt_throw_facts()
                );
                assert_eq!(
                    log[0].ground_force_double_play_facts(),
                    original.ground_force_double_play_facts()
                );
                assert_eq!(EventTransition::from_bytes(&encoded).unwrap(), log[0]);
                assert_eq!(pa(&log[..1]), 1);
                assert_eq!(bf(&log[..1]), 1);
                assert_eq!(calls, log.len());
                assert!(log.last().unwrap().after().over);
                assert_eq!(log[0].before(), &initial);
                for pair in log.windows(2) {
                    assert_eq!(pair[1].before(), pair[0].after());
                    assert!(pair[1].follows(&pair[0]).is_ok());
                }
                assert_eq!(initial, saved);
            }
        }
    }
}
fn explicit_walkoff() -> EventTransition {
    let mut s = state(4, 0);
    s.inning = 9;
    s.half = Half::Bottom;
    s.bases[2].as_mut().unwrap().player_id = "h5".into();
    s.bases[2].as_mut().unwrap().responsible_pitcher = "ap".into();
    let event = preview::event_for(&s, Outcome::Single).unwrap();
    apply_with_walkoff_hit_facts(
        &s,
        event,
        WalkoffHitFacts {
            winning_runner_id: "h5".into(),
            batter_touch: NamedBaseTouch {
                runner_id: "h0".into(),
                base: Destination::First,
            },
            timing: NaturalCourseTouchTiming::ByWinningRun,
        },
    )
    .unwrap()
}
#[test]
fn explicit_walkoff_is_retained_and_stops_without_another_callback() {
    let t = explicit_walkoff();
    let original = t.clone();
    let before = t.before().clone();
    assert!(t.to_bytes().starts_with(b"BBE27"));
    assert!(t.after().over);
    let (log, calls) = run_script(&before, vec![t]);
    assert_eq!(calls, 1);
    assert_eq!(log, vec![original.clone()]);
    assert_eq!(bytes(&log), vec![original.to_bytes()]);
    assert_eq!(log[0].walkoff_hit_facts(), original.walkoff_hit_facts());
    assert_eq!(pa(&log), 1);
    assert_eq!(bf(&log), 1);
}
fn runner_event(s: &GameState, steal: bool) -> EventTransition {
    let i = usize::from(steal);
    let runner = s.bases[i].as_ref().unwrap();
    let mut e = preview::event_for(s, Outcome::Strikeout).unwrap();
    e.kind = EventKind::RunnerOnly(if steal {
        RunnerEvent::StealAttempt
    } else {
        RunnerEvent::Pickoff
    });
    e.delta = Delta {
        moves: vec![RunnerMove {
            runner: runner.clone(),
            from: if steal { Base::Second } else { Base::First },
            to: if steal {
                Destination::Third
            } else {
                Destination::First
            },
        }],
        outs: 0,
        runs: 0,
        credits: if steal {
            vec![Credit {
                player_id: runner.player_id.clone(),
                kind: CreditKind::StolenBase,
            }]
        } else {
            vec![]
        },
        changes: vec![],
        third_out: None,
    };
    e.caught_strikeout = None;
    e.fielding_review = FieldingReview::NoMisplay;
    apply(s, e).unwrap()
}
fn substitution(s: &GameState) -> EventTransition {
    let mut e = preview::event_for(s, Outcome::Strikeout).unwrap();
    e.kind = EventKind::Substitution;
    e.delta = Delta {
        moves: vec![],
        outs: 0,
        runs: 0,
        credits: vec![],
        third_out: None,
        changes: vec![PersonnelChange::BattingSlot {
            team: TeamSide::Away,
            slot: 7,
            outgoing: "a7".into(),
            incoming: "abench".into(),
        }],
    };
    e.caught_strikeout = None;
    e.fielding_review = FieldingReview::Unknown;
    apply(s, e).unwrap()
}
#[test]
fn mixed_supplied_prefix_transports_non_pa_and_substitution_without_extra_pa_or_charge() {
    let initial = state(5, 0);
    let mut s = initial.clone();
    let mut script = Vec::new();
    for action in 0..8 {
        let t = match action {
            0 => supplied(&s, 0),
            1 => supplied(&s, 1),
            2 => runner_event(&s, true),
            4 => ordinary(&s, Outcome::Walk),
            5 => supplied(&s, 2),
            7 => substitution(&s),
            _ => ordinary(&s, Outcome::Strikeout),
        };
        s = t.after().clone();
        script.push(t);
    }
    let originals = script.clone();
    let (log, calls) = run_script(&initial, script);
    assert!(log.len() >= 8);
    assert_eq!(&log[..8], originals);
    assert_eq!(bytes(&log[..8]), bytes(&originals));
    for (i, expected) in [1, 2, 2, 3, 4, 5, 6, 6].into_iter().enumerate() {
        assert_eq!(pa(&log[..=i]), expected);
        assert_eq!(bf(&log[..=i]), expected);
    }
    assert_eq!(log[2].before().teams, log[2].after().teams);
    assert_eq!(
        log[2].before().bases[1]
            .as_ref()
            .unwrap()
            .responsible_pitcher,
        log[2].after().bases[2]
            .as_ref()
            .unwrap()
            .responsible_pitcher
    );
    assert_eq!(
        log[7].before().teams[0].next_batter,
        log[7].after().teams[0].next_batter
    );
    assert_eq!(log[7].after().teams[0].batting_order[7], "abench");
    assert!(log[7].after().teams[0]
        .removed
        .contains(&String::from("a7")));
    assert_eq!(calls, log.len());
    assert!(log.last().unwrap().after().over);
    for pair in log.windows(2) {
        assert!(pair[1].follows(&pair[0]).is_ok());
    }
}
#[test]
fn whole_before_discontinuities_refuse_first_or_later_without_retry() {
    for change in 0..7 {
        for fail_at in [1, 3] {
            let mut initial = state(1, 0);
            // Keep the supplied runner distinct from every next batter in this prefix,
            // even when the wrong outs deliberately prevent third-out cleanup.
            initial.bases[0].as_mut().unwrap().player_id = "a5".into();
            let original = initial.clone();
            let mut calls = 0;
            let result = preview::game_from_transitions(83, initial.clone(), |s, _| {
                calls += 1;
                if calls != fail_at {
                    return Ok(ordinary(s, Outcome::Strikeout));
                }
                let mut wrong = s.clone();
                match change {
                    0 => wrong.score[0] += 1,
                    1 => wrong.teams[0].next_batter = 1,
                    2 => wrong.teams[0].bench.push("another-bench".into()),
                    3 => wrong.teams[0].removed.push("another-removed".into()),
                    4 => {
                        wrong.bases[0].as_mut().unwrap().responsible_pitcher =
                            "different-charge".into()
                    }
                    5 => wrong.inning += 1,
                    _ => wrong.outs = if wrong.outs == 0 { 1 } else { 0 },
                }
                let canonical = apply(
                    &wrong,
                    preview::event_for(&wrong, Outcome::Strikeout).unwrap(),
                );
                assert!(
                    canonical.is_ok(),
                    "fixture must be canonical: change {change}, fail_at {fail_at}: {canonical:?}"
                );
                let t = canonical.unwrap();
                assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
                Ok(t)
            });
            assert_eq!(
                result,
                Err(format!(
                    "seed 83, play {fail_at}: supplied transition before-state mismatch"
                )),
                "change {change}"
            );
            assert_eq!(calls, fail_at);
            assert_eq!(initial, original);
        }
    }
}
#[test]
fn producer_errors_are_verbatim_including_empty_and_reason_like_strings() {
    for reason in [
        "",
        "refused",
        "ModeledFactsUnavailable",
        "seed 89: resource ceiling reached, no winner fabricated",
        "arbitrary supplied error",
    ] {
        for fail_at in [1, 4] {
            let mut calls = 0;
            let result = preview::game_from_transitions(89, state(0, 0), |s, r| {
                calls += 1;
                let _ = r.next_u64();
                if calls == fail_at {
                    Err(reason.to_owned())
                } else {
                    Ok(ordinary(s, Outcome::Strikeout))
                }
            });
            assert_eq!(result, Err(reason.to_owned()));
            assert_eq!(calls, fail_at);
        }
    }
}
#[test]
fn valid_ended_initial_state_has_zero_callbacks_on_both_paths() {
    let initial = explicit_walkoff().after().clone();
    let typed =
        preview::game_from_transitions(97, initial.clone(), |_, _| panic!("ended typed callback"));
    let old = preview::game_from(97, initial, |_, _| panic!("ended ordinary callback"));
    assert_eq!(typed, Ok(vec![]));
    assert_eq!(old, Ok(vec![]));
}
#[test]
fn held_runner_non_pa_attempts_count_toward_exact_resource_cap() {
    let initial = state(1, 0);
    let mut calls = 0;
    let result = preview::game_from_transitions(101, initial.clone(), |s, _| {
        calls += 1;
        assert_eq!(s, &initial);
        let t = runner_event(s, false);
        assert_eq!(t.before(), t.after());
        assert_eq!(pa(std::slice::from_ref(&t)), 0);
        assert_eq!(bf(std::slice::from_ref(&t)), 0);
        Ok(t)
    });
    assert_eq!(
        result,
        Err("seed 101: resource ceiling reached, no winner fabricated".into())
    );
    assert_eq!(calls, 10_000);
}
#[test]
fn all_strikeout_resource_cap_and_old_apply_error_context_remain_exact() {
    let mut calls = 0;
    let result = preview::game_from(103, state(0, 0), |s, _| {
        calls += 1;
        Ok((
            Outcome::Strikeout,
            preview::event_for(s, Outcome::Strikeout)?,
        ))
    });
    assert_eq!(
        result,
        Err("seed 103: resource ceiling reached, no winner fabricated".into())
    );
    assert_eq!(calls, 10_000);
    for fail_at in [1, 3] {
        let initial = state(0, 0);
        let mut target = initial.clone();
        for _ in 1..fail_at {
            target = ordinary(&target, Outcome::Strikeout).after().clone();
        }
        let mut invalid = preview::event_for(&target, Outcome::Strikeout).unwrap();
        invalid.delta.outs = 4;
        let engine_error = apply(&target, invalid).unwrap_err();
        let mut calls = 0;
        let result = preview::game_from(107, initial, |s, _| {
            calls += 1;
            let mut e = preview::event_for(s, Outcome::Strikeout)?;
            if calls == fail_at {
                e.delta.outs = 4;
            }
            Ok((Outcome::Strikeout, e))
        });
        assert_eq!(
            result,
            Err(format!(
                "seed 107, play {fail_at}, Strikeout: {engine_error:?}"
            ))
        );
        assert_eq!(calls, fail_at);
    }
}
#[test]
fn repeated_owned_scripts_preserve_original_bytes_and_inputs() {
    let initial = state(4, 0);
    let original = initial.clone();
    let first = supplied(&initial, 0);
    let saved = first.clone();
    let (a, ca) = run_script(&initial, vec![first.clone()]);
    let (b, cb) = run_script(&initial, vec![first.clone()]);
    assert_eq!(a.first(), Some(&saved));
    assert_eq!(b.first(), Some(&saved));
    assert_eq!(ca, a.len());
    assert!(a.last().unwrap().after().over);
    assert_eq!(a, b);
    assert_eq!(bytes(&a), bytes(&b));
    assert_eq!(ca, cb);
    assert_eq!(first, saved);
    assert_eq!(initial, original);
}
