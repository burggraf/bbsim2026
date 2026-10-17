//! Synthetic-only supplied-parent producer contracts;not observed actors/source authentication.
#[path = "../examples/support/contact_proxy_adapter.rs"]
mod adapter;
#[path = "../examples/support/joint_contact.rs"]
pub mod joint;
#[path = "../examples/support/preview.rs"]
pub mod preview;
#[path = "../examples/support/contact_proxy_producer.rs"]
mod producer;
use adapter::{consequence, sampler, Disposition};
use bbsim_engine::accounting::derive_accounting;
use bbsim_engine::*;
use consequence::ContactDecision;
use sampler::ProxyClass;
const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
fn state(mask: u8, outs: u8) -> GameState {
    let teams = ["a", "h"].map(|prefix| TeamState {
        batting_order: std::array::from_fn(|i| format!("{prefix}{i}")),
        next_batter: 0,
        pitcher: format!("{prefix}p"),
        dh_slot: Some(8),
        dh_must_bat: false,
        bench: vec![format!("{prefix}bench")],
        removed: vec![format!("{prefix}removed")],
    });
    GameState {
        inning: 2,
        half: Half::Top,
        outs,
        score: [0, 0],
        bases: std::array::from_fn(|i| {
            (mask & (1 << i) != 0).then(|| Runner {
                player_id: format!("a{}", i + 3),
                responsible_pitcher: format!("inherited-{i}"),
            })
        }),
        teams,
        over: false,
    }
}
fn sample() -> joint::Sample {
    joint::Sample {
        outcome: preview::Outcome::ContactOut,
        contact: Some(joint::ContactKind::Residual),
    }
}
fn parent(before: &GameState) -> EventTransition {
    apply(before, joint::event_for(before, sample()).unwrap()).unwrap()
}
fn payload() -> Vec<u8> {
    let mut text = format!("contact-proxy-context-counts-v1\nmodel\tgeometry-proxy-context-jeffreys-v1\nseason\t2019\nsplit\ttraining\nsource_sha256\t{DIGEST}\nprior\t1\t2\nclasses\tSF\tSH\tGIDP\tremaining\ncontexts\t24\n");
    for mask in 0..8 {
        for outs in 0..3 {
            text.push_str(&format!("C\t{mask}\t{outs}\t0\t0\t0\t0\t0\n"));
        }
    }
    text.into_bytes()
}
fn decision(class: ProxyClass, contact: AirContact) -> ContactDecision {
    match class {
        ProxyClass::Sf => ContactDecision::CaughtSacrificeFly {
            catch: OrdinaryAirCatchFacts {
                batter_id: "a0".into(),
                fielder_id: "h5".into(),
                contact,
            },
            facts: CaughtSacrificeFlyFacts {
                scoring_runner_id: "a5".into(),
                handling: SacrificeFlyHandling::QualifyingOutfielderOrInfielderInOutfield,
                retouch: SacrificeFlyRetouch::Legal,
                timing: SacrificeFlyRunTiming::AfterCatch,
            },
        },
        ProxyClass::Sh => ContactDecision::SacrificeBunt(SacrificeBuntThrowFacts {
            batter_id: "a0".into(),
            advancing_runner_id: "a3".into(),
            thrower_id: "h3".into(),
            receiver_id: "h1".into(),
            judgment: SacrificeBuntJudgment::SacrificingOwnChance,
        }),
        ProxyClass::Gidp => ContactDecision::GroundForceDoublePlay(GroundForceDoublePlayFacts {
            batter_id: "a0".into(),
            forced_runner_id: "a3".into(),
            initial_fielder_id: "h3".into(),
            pivot_id: "h6".into(),
            first_receiver_id: "h1".into(),
        }),
        ProxyClass::Remaining => panic!("no special decision for Remaining"),
    }
}
fn context_equal(a: &sampler::ContextProbabilities, b: &sampler::ContextProbabilities) {
    assert_eq!(
        (
            a.mask,
            a.outs,
            a.parent_events,
            a.response_events,
            a.supported,
            a.prior_only,
            a.numerators,
            a.denominator
        ),
        (
            b.mask,
            b.outs,
            b.parent_events,
            b.response_events,
            b.supported,
            b.prior_only,
            b.numerators,
            b.denominator
        )
    );
    assert_eq!(
        a.probabilities.map(f64::to_bits),
        b.probabilities.map(f64::to_bits)
    );
    assert_eq!(a.cdf.map(f64::to_bits), b.cdf.map(f64::to_bits));
}
const KINDS: [joint::ContactKind; 5] = [
    joint::ContactKind::Ground,
    joint::ContactKind::Fly,
    joint::ContactKind::LineDrive,
    joint::ContactKind::PopUp,
    joint::ContactKind::Residual,
];
fn refuse<T>(result: Result<T, String>) -> String {
    match result {
        Err(e) => e,
        Ok(_) => panic!("expected hard refusal"),
    }
}
fn seeds() -> impl Iterator<Item = u64> {
    (0..128).chain([u64::MAX])
}
fn equivalent(got: &producer::Produced, expected: &adapter::Adapted) {
    let producer::Produced::ContactProxy(actual) = got else {
        panic!("proxy result disguised as Ordinary")
    };
    assert!(std::ptr::eq(got.transition(), &actual.transition));
    assert_eq!(actual.transition, expected.transition);
    assert_eq!(actual.transition.to_bytes(), expected.transition.to_bytes());
    assert_eq!(actual.realized, expected.realized);
    assert_eq!(actual.disposition, expected.disposition);
    let a = &actual.selection;
    let b = &expected.selection;
    assert_eq!(
        (
            a.class,
            &a.source_sha256,
            a.model_id,
            a.selection_version,
            a.training_events,
            a.grid_word
        ),
        (
            b.class,
            &b.source_sha256,
            b.model_id,
            b.selection_version,
            b.training_events,
            b.grid_word
        )
    );
    context_equal(&a.context, &b.context);
}
#[test]
fn policy_version_and_existing_versions_remain_distinct() {
    assert_eq!(
        producer::POLICY_VERSION,
        "supplied-contact-proxy-producer-v1"
    );
    assert_eq!(adapter::POLICY_VERSION, "supplied-contact-proxy-adapter-v1");
    assert_eq!(preview::RNG_VERSION, "splitmix64-v1");
    assert_eq!(
        adapter::PROVIDER_BOUNDARY_VERSION,
        "supplied-contact-proxy-provider-boundary-v1"
    );
    assert_eq!(
        consequence::POLICY_VERSION,
        "supplied-contact-consequence-projection-v1"
    );
}
#[test]
fn all12_ordinary_categories_in24_contexts_preserve_native_bytes_and_zero_words_callbacks() {
    let mut samples: Vec<_> = preview::ALL
        .into_iter()
        .filter(|o| *o != preview::Outcome::ContactOut)
        .map(|outcome| joint::Sample {
            outcome,
            contact: None,
        })
        .collect();
    samples.extend(
        KINDS
            .into_iter()
            .filter(|k| *k != joint::ContactKind::Residual)
            .map(|contact| joint::Sample {
                outcome: preview::Outcome::ContactOut,
                contact: Some(contact),
            }),
    );
    assert_eq!(samples.len(), 12);
    for mask in 0..8 {
        for outs in 0..3 {
            for supplied in &samples {
                for seed in [0, 1, u64::MAX] {
                    let before = state(mask, outs);
                    let original = before.clone();
                    let expected =
                        apply(&before, joint::event_for(&before, *supplied).unwrap()).unwrap();
                    let mut rng = preview::Rng::new(seed);
                    let mut reference = preview::Rng::new(seed);
                    let got = producer::produce(
                        &before,
                        *supplied,
                        &mut rng,
                        b"unused invalid artifact",
                        "unused invalid tag",
                        |_, _| panic!("ordinary callback"),
                    )
                    .unwrap();
                    assert!(matches!(&got, producer::Produced::Ordinary(_)));
                    assert_eq!(got.transition(), &expected);
                    assert_eq!(got.into_transition().to_bytes(), expected.to_bytes());
                    assert_eq!(rng.next_u64(), reference.next_u64());
                    assert_eq!(before, original);
                }
            }
        }
    }
}
#[test]
fn all41_inconsistent_pairs_refuse_before_rng_and_provider() {
    let options = [
        None,
        Some(KINDS[0]),
        Some(KINDS[1]),
        Some(KINDS[2]),
        Some(KINDS[3]),
        Some(KINDS[4]),
    ];
    let mut pairs = Vec::new();
    for outcome in preview::ALL {
        for contact in options {
            if (outcome == preview::Outcome::ContactOut) != contact.is_some() {
                pairs.push(joint::Sample { outcome, contact });
            }
        }
    }
    assert_eq!(pairs.len(), 41);
    for mask in 0..8 {
        for outs in 0..3 {
            for supplied in &pairs {
                let before = state(mask, outs);
                let expected = format!(
                    "ordinary contact-proxy parent refused: {}",
                    refuse(joint::event_for(&before, *supplied))
                );
                let mut rng = preview::Rng::new(37);
                let mut reference = preview::Rng::new(37);
                assert_eq!(
                    refuse(producer::produce(
                        &before,
                        *supplied,
                        &mut rng,
                        b"bad",
                        DIGEST,
                        |_, _| panic!("invalid-pair callback")
                    )),
                    expected
                );
                assert_eq!(rng.next_u64(), reference.next_u64());
            }
        }
    }
}
#[test]
fn canonical_parent_event_refusal_precedes_class_word_and_callback() {
    let mut before = state(0, 0);
    before.inning = 9;
    before.half = Half::Bottom;
    before.score = [0, 1];
    before.over = true;
    let expected = format!(
        "ordinary contact-proxy parent refused: {}",
        refuse(joint::event_for(&before, sample()))
    );
    let mut rng = preview::Rng::new(41);
    let mut reference = preview::Rng::new(41);
    assert_eq!(
        refuse(producer::produce(
            &before,
            sample(),
            &mut rng,
            b"bad",
            DIGEST,
            |_, _| panic!("ended callback")
        )),
        expected
    );
    assert_eq!(rng.next_u64(), reference.next_u64());
}
#[test]
fn residual24_contexts_and129_seeds_match_complete_native_results_and_one_word() {
    let bytes = payload();
    let original_bytes = bytes.clone();
    for mask in 0..8 {
        for outs in 0..3 {
            for seed in seeds() {
                let before = state(mask, outs);
                let original = before.clone();
                let mut reference = preview::Rng::new(seed);
                let word = reference.next_u64();
                let mut rng = preview::Rng::new(seed);
                let calls = std::cell::Cell::new(0);
                let expected = adapter::adapt_with_provider(
                    &before,
                    sample(),
                    &parent(&before),
                    &bytes,
                    DIGEST,
                    word,
                    |_, _| Ok(None),
                )
                .unwrap();
                let got = producer::produce(
                    &before,
                    sample(),
                    &mut rng,
                    &bytes,
                    DIGEST,
                    |s, selection| {
                        assert_eq!(s, &before);
                        assert_eq!(selection.grid_word, word >> 11);
                        calls.set(calls.get() + 1);
                        Ok(None)
                    },
                )
                .unwrap();
                equivalent(&got, &expected);
                assert_eq!(calls.get(), 1);
                assert_eq!(got.into_transition(), expected.transition);
                assert_eq!(rng.next_u64(), reference.next_u64());
                assert_eq!(before, original);
            }
        }
    }
    assert_eq!(bytes, original_bytes);
}
#[test]
fn supplied_sf_bunt_and_gidp_keep_all_facts_accounting_and_owned_accessors() {
    let bytes = payload();
    let mut seen = [false; 3];
    let mut air_seen = [false; 3];
    for mask in 0..8 {
        for outs in 0..2 {
            for seed in seeds() {
                let before = state(mask, outs);
                let mut reference = preview::Rng::new(seed);
                let word = reference.next_u64();
                let selected = sampler::select(&bytes, DIGEST, mask, outs, word).unwrap();
                let index = match selected.class {
                    ProxyClass::Sf => 0,
                    ProxyClass::Sh => 1,
                    ProxyClass::Gidp => 2,
                    ProxyClass::Remaining => continue,
                };
                let descriptors = if index == 0 {
                    vec![AirContact::Fly, AirContact::LineDrive, AirContact::PopUp]
                } else {
                    vec![AirContact::Fly]
                };
                for (air_index, contact) in descriptors.into_iter().enumerate() {
                    let supplied = decision(selected.class, contact);
                    let expected = adapter::adapt_with_provider(
                        &before,
                        sample(),
                        &parent(&before),
                        &bytes,
                        DIGEST,
                        word,
                        |_, _| Ok(Some(supplied.clone())),
                    )
                    .unwrap();
                    assert_eq!(expected.disposition, Disposition::Projected);
                    let mut rng = preview::Rng::new(seed);
                    let calls = std::cell::Cell::new(0);
                    let got =
                        producer::produce(&before, sample(), &mut rng, &bytes, DIGEST, |_, _| {
                            calls.set(calls.get() + 1);
                            Ok(Some(supplied))
                        })
                        .unwrap();
                    equivalent(&got, &expected);
                    assert_eq!(
                        got.transition().caught_sacrifice_fly_facts(),
                        expected.transition.caught_sacrifice_fly_facts()
                    );
                    assert_eq!(
                        got.transition().sacrifice_bunt_throw_facts(),
                        expected.transition.sacrifice_bunt_throw_facts()
                    );
                    assert_eq!(
                        got.transition().ground_force_double_play_facts(),
                        expected.transition.ground_force_double_play_facts()
                    );
                    let actual = got.into_transition();
                    assert_eq!(actual, expected.transition);
                    assert_eq!(
                        derive_accounting(&[actual]).unwrap(),
                        derive_accounting(&[expected.transition]).unwrap()
                    );
                    let mut after = preview::Rng::new(seed);
                    after.next_u64();
                    assert_eq!(rng.next_u64(), after.next_u64());
                    assert_eq!(calls.get(), 1);
                    seen[index] = true;
                    if index == 0 {
                        air_seen[air_index] = true;
                    }
                }
            }
        }
    }
    assert_eq!(seen, [true; 3]);
    assert_eq!(air_seen, [true; 3]);
}
#[test]
fn terminal_none_and_remaining_retain_native_dispositions_and_callback() {
    let bytes = payload();
    let mut seen_terminal = false;
    let mut seen_remaining = false;
    for seed in seeds() {
        let mut before = state(4, 0);
        before.inning = 9;
        before.half = Half::Bottom;
        for r in before.bases.iter_mut().flatten() {
            r.player_id = r.player_id.replacen('a', "h", 1);
        }
        let mut reference = preview::Rng::new(seed);
        let word = reference.next_u64();
        let mut rng = preview::Rng::new(seed);
        let calls = std::cell::Cell::new(0);
        let expected = adapter::adapt_with_provider(
            &before,
            sample(),
            &parent(&before),
            &bytes,
            DIGEST,
            word,
            |_, _| Ok(None),
        )
        .unwrap();
        let got = producer::produce(&before, sample(), &mut rng, &bytes, DIGEST, |_, _| {
            calls.set(calls.get() + 1);
            Ok(None)
        })
        .unwrap();
        equivalent(&got, &expected);
        seen_terminal |= expected.disposition == Disposition::TerminalApiUnsupported;
        seen_remaining |= expected.disposition == Disposition::RemainingClass;
        assert_eq!(calls.get(), 1);
        assert_eq!(rng.next_u64(), reference.next_u64());
    }
    assert!(seen_terminal && seen_remaining);
}
#[test]
fn malformed_whole_artifacts_and_tags_refuse_after_one_word_without_provider() {
    let original = payload();
    let before = state(5, 0);
    let wrong = "b".repeat(64);
    let inputs = [
        (b"invalid".to_vec(), DIGEST),
        (original[..original.len() - 1].to_vec(), DIGEST),
        (original.clone(), wrong.as_str()),
    ];
    for (bytes, tag) in inputs {
        for seed in [0, 31, u64::MAX] {
            let mut reference = preview::Rng::new(seed);
            let word = reference.next_u64();
            let mut rng = preview::Rng::new(seed);
            let expected = refuse(adapter::adapt_with_provider(
                &before,
                sample(),
                &parent(&before),
                &bytes,
                tag,
                word,
                |_, _| panic!("native loader callback"),
            ));
            assert_eq!(
                refuse(producer::produce(
                    &before,
                    sample(),
                    &mut rng,
                    &bytes,
                    tag,
                    |_, _| panic!("producer loader callback")
                )),
                expected
            );
            assert_eq!(rng.next_u64(), reference.next_u64());
        }
    }
    assert_eq!(payload(), original);
}
#[test]
fn move_owned_provider_errors_are_verbatim_including_empty_and_not_retried() {
    let before = state(5, 0);
    let bytes = payload();
    for message in [
        "",
        "arbitrary supplied error",
        "ModeledFactsUnavailable",
        "seed 9: resource ceiling reached, no winner fabricated",
    ] {
        let owned = message.to_owned();
        let calls = std::cell::Cell::new(0);
        let mut rng = preview::Rng::new(53);
        let mut reference = preview::Rng::new(53);
        reference.next_u64();
        let got = producer::produce(&before, sample(), &mut rng, &bytes, DIGEST, |_, _| {
            calls.set(calls.get() + 1);
            Err(owned)
        });
        assert_eq!(refuse(got), message);
        assert_eq!(calls.get(), 1);
        assert_eq!(rng.next_u64(), reference.next_u64());
    }
}
#[test]
fn wrong_family_unknown_and_ineligible_supplied_facts_keep_native_refusals() {
    let before = state(5, 0);
    let bytes = payload();
    let mut seen = [false; 3];
    for seed in seeds() {
        let mut reference = preview::Rng::new(seed);
        let word = reference.next_u64();
        let selected = sampler::select(&bytes, DIGEST, 5, 0, word).unwrap();
        let wrong = if selected.class == ProxyClass::Sf {
            ProxyClass::Sh
        } else {
            ProxyClass::Sf
        };
        let mut variants = vec![decision(wrong, AirContact::Fly)];
        if selected.class != ProxyClass::Remaining {
            let mut bad = decision(selected.class, AirContact::Fly);
            match &mut bad {
                ContactDecision::CaughtSacrificeFly { catch, .. } => {
                    catch.batter_id = "outsider".into()
                }
                ContactDecision::SacrificeBunt(f) => f.batter_id = "outsider".into(),
                ContactDecision::GroundForceDoublePlay(f) => f.batter_id = "outsider".into(),
            };
            variants.push(bad);
            let mut unknown = decision(selected.class, AirContact::Fly);
            match &mut unknown {
                ContactDecision::CaughtSacrificeFly { facts, .. } => {
                    facts.retouch = SacrificeFlyRetouch::Unknown
                }
                ContactDecision::SacrificeBunt(f) => f.judgment = SacrificeBuntJudgment::Unknown,
                ContactDecision::GroundForceDoublePlay(f) => {
                    f.pivot_id = f.first_receiver_id.clone()
                }
            };
            variants.push(unknown);
        }
        for (i, facts) in variants.into_iter().enumerate() {
            let expected = refuse(adapter::adapt_with_provider(
                &before,
                sample(),
                &parent(&before),
                &bytes,
                DIGEST,
                word,
                |_, _| Ok(Some(facts.clone())),
            ));
            let mut rng = preview::Rng::new(seed);
            let calls = std::cell::Cell::new(0);
            assert_eq!(
                refuse(producer::produce(
                    &before,
                    sample(),
                    &mut rng,
                    &bytes,
                    DIGEST,
                    |_, _| {
                        calls.set(calls.get() + 1);
                        Ok(Some(facts))
                    }
                )),
                expected
            );
            assert_eq!(calls.get(), 1);
            let mut after = preview::Rng::new(seed);
            after.next_u64();
            assert_eq!(rng.next_u64(), after.next_u64());
            seen[i] = true;
        }
    }
    assert_eq!(seen, [true; 3]);
}
#[test]
fn typed_loop_mixes_runner_only_and_produced_pa_then_stops_without_extra_callback() {
    let initial = state(1, 0);
    let bytes = payload();
    let mut calls = 0;
    let mut provider_calls = 0;
    let mut final_word = None;
    let mut reference = preview::Rng::new(61);
    reference.next_u64();
    let expected_final_word = reference.next_u64();
    let log = preview::game_from_transitions(61, initial, |before, rng| {
        calls += 1;
        if calls == 1 {
            let runner = before.bases[0].as_ref().unwrap().clone();
            let mut event = preview::event_for(before, preview::Outcome::Strikeout).unwrap();
            event.kind = EventKind::RunnerOnly(RunnerEvent::Pickoff);
            event.caught_strikeout = None;
            event.fielding_review = FieldingReview::NoMisplay;
            event.delta = Delta {
                moves: vec![RunnerMove {
                    runner,
                    from: Base::First,
                    to: Destination::First,
                }],
                outs: 0,
                runs: 0,
                credits: vec![],
                changes: vec![],
                third_out: None,
            };
            return apply(before, event).map_err(|e| format!("{e:?}"));
        }
        let supplied = if calls == 2 {
            sample()
        } else {
            joint::Sample {
                outcome: if before.inning == 9 && before.half == Half::Bottom {
                    preview::Outcome::HomeRun
                } else {
                    preview::Outcome::Strikeout
                },
                contact: None,
            }
        };
        let got = producer::produce(before, supplied, rng, &bytes, DIGEST, |_, _| {
            provider_calls += 1;
            Ok(None)
        })?;
        if calls == 2 {
            assert!(matches!(&got, producer::Produced::ContactProxy(_)));
        }
        let transition = got.into_transition();
        if transition.after().over {
            final_word = Some(rng.next_u64());
        }
        Ok(transition)
    })
    .unwrap();
    assert_eq!(calls, log.len());
    assert_eq!(provider_calls, 1);
    assert_eq!(final_word, Some(expected_final_word));
    assert!(log.last().unwrap().after().over);
    assert_eq!(log[0].before().teams, log[0].after().teams);
    for pair in log.windows(2) {
        assert_eq!(pair[0].after(), pair[1].before());
    }
    let accounting = derive_accounting(&log).unwrap();
    let pa: u32 = accounting
        .batters
        .iter()
        .flat_map(|t| t.values())
        .map(|b| b.plate_appearances)
        .sum();
    let bf: u32 = accounting
        .pitchers
        .iter()
        .flat_map(|t| t.values())
        .map(|p| p.batters_faced)
        .sum();
    assert_eq!(pa, log.len() as u32 - 1);
    assert_eq!(bf, pa);
}
#[test]
fn native_reducer_rejects_explicit_illegal_snapshot_before_class_word_or_callback() {
    let before = state(0, 3); // Deliberately illegal negative fixture,not new validation policy.
    let event = joint::event_for(&before, sample()).unwrap();
    let expected = match apply(&before, event) {
        Err(e) => format!("ordinary contact-proxy parent refused: {e:?}"),
        Ok(_) => panic!("illegal snapshot accepted"),
    };
    let mut rng = preview::Rng::new(67);
    let mut reference = preview::Rng::new(67);
    assert_eq!(
        refuse(producer::produce(
            &before,
            sample(),
            &mut rng,
            b"bad",
            DIGEST,
            |_, _| panic!("illegal snapshot callback")
        )),
        expected
    );
    assert_eq!(rng.next_u64(), reference.next_u64());
}
fn owned_refined_sf() -> adapter::Adapted {
    let before = state(4, 0);
    let supplied = decision(ProxyClass::Sf, AirContact::Fly);
    adapter::adapt(
        &before,
        sample(),
        &parent(&before),
        &payload(),
        DIGEST,
        0,
        Some(&supplied),
    )
    .unwrap()
}
#[test]
fn ordinary_borrow_accessor_is_the_original_native_reference() {
    let got = producer::Produced::Ordinary(parent(&state(5, 0)));
    let producer::Produced::Ordinary(inner) = &got else {
        unreachable!()
    };
    assert!(std::ptr::eq(got.transition(), inner));
}
#[test]
fn proxy_borrow_accessor_is_the_original_supplemental_native_reference() {
    let got = producer::Produced::ContactProxy(owned_refined_sf());
    let producer::Produced::ContactProxy(inner) = &got else {
        unreachable!()
    };
    assert!(std::ptr::eq(got.transition(), &inner.transition));
    assert!(got.transition().caught_sacrifice_fly_facts().is_some());
}
#[test]
fn ordinary_owned_accessor_keeps_the_complete_native_transition() {
    let original = parent(&state(5, 0));
    let bytes = original.to_bytes();
    let got = producer::Produced::Ordinary(original.clone()).into_transition();
    assert_eq!(got, original);
    assert_eq!(got.to_bytes(), bytes);
}
#[test]
fn proxy_owned_accessor_keeps_all_supplemental_facts_and_bytes() {
    let original = owned_refined_sf();
    let expected = original.transition.clone();
    let got = producer::Produced::ContactProxy(original).into_transition();
    assert_eq!(got.to_bytes(), expected.to_bytes());
    assert_eq!(got, expected);
    assert!(got.caught_sacrifice_fly_facts().is_some());
}
