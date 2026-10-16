//! Synthetic-only adapter behavior;supplied legal assertions are not historical proof.
#[path = "../examples/support/contact_proxy_adapter.rs"]
mod adapter;
#[path = "../examples/support/joint_contact.rs"]
pub mod joint;
#[path = "../examples/support/preview.rs"]
pub mod preview;
use adapter::{adapt, consequence, sampler, Disposition};
use bbsim_engine::*;
use consequence::ContactDecision;
use sampler::ProxyClass;

const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const QUARTER: u64 = 1 << 51;
const CLASSES: [ProxyClass; 4] = [
    ProxyClass::Sf,
    ProxyClass::Sh,
    ProxyClass::Gidp,
    ProxyClass::Remaining,
];
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
fn word(class: ProxyClass, mask: u8) -> u64 {
    match (class, mask) {
        (ProxyClass::Sf, _) | (ProxyClass::Sh, 1) => 0,
        (ProxyClass::Sh, 5) => 1 << 62,
        (ProxyClass::Gidp, _) => 1 << 63,
        (ProxyClass::Remaining, _) => u64::MAX,
        _ => panic!("unsupported fixture class/context"),
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
fn error(result: Result<adapter::Adapted, String>) -> String {
    result.err().expect("expected hard refusal")
}
fn special(before: &GameState, class: ProxyClass, facts: &ContactDecision) -> adapter::Adapted {
    adapt(
        before,
        sample(),
        &parent(before),
        &payload(),
        DIGEST,
        word(class, mask(before)),
        Some(facts),
    )
    .unwrap()
}
fn mask(before: &GameState) -> u8 {
    before
        .bases
        .iter()
        .enumerate()
        .fold(0, |m, (i, r)| m | (u8::from(r.is_some()) << i))
}

#[test]
fn all24_contexts_none_preserves_exact_parent_and_complete_selection_metadata() {
    assert_eq!(adapter::POLICY_VERSION, "supplied-contact-proxy-adapter-v1");
    assert_eq!(
        consequence::POLICY_VERSION,
        "supplied-contact-consequence-projection-v1"
    );
    let bytes = payload();
    for mask in 0..8 {
        for outs in 0..3 {
            let before = state(mask, outs);
            let original = parent(&before);
            for supplied in [0, 1 << 62, 1 << 63, 3 << 62, u64::MAX] {
                let selected = sampler::select(&bytes, DIGEST, mask, outs, supplied).unwrap();
                let got =
                    adapt(&before, sample(), &original, &bytes, DIGEST, supplied, None).unwrap();
                assert_eq!(got.selection.class, selected.class);
                assert_eq!(got.selection.grid_word, selected.grid_word);
                assert_eq!(got.selection.source_sha256, DIGEST);
                assert_eq!(got.selection.model_id, selected.model_id);
                assert_eq!(got.selection.selection_version, selected.selection_version);
                assert_eq!(got.selection.training_events, selected.training_events);
                context_equal(&got.selection.context, &selected.context);
                assert_eq!(got.realized, ProxyClass::Remaining);
                assert_eq!(
                    got.disposition,
                    if selected.class == ProxyClass::Remaining {
                        Disposition::RemainingClass
                    } else {
                        Disposition::ModeledFactsUnavailable
                    }
                );
                assert_eq!(got.transition, original);
                assert_eq!(got.transition.to_bytes(), original.to_bytes());
                assert_eq!(
                    got.transition.event().fielding_review,
                    FieldingReview::Unknown
                );
            }
        }
    }
}

#[test]
fn supplied_sf_all_supported_contexts_and_air_types_retains_canonical_facts_and_one_pa() {
    for mask in 4..8 {
        for outs in 0..2 {
            for contact in [AirContact::Fly, AirContact::LineDrive, AirContact::PopUp] {
                let before = state(mask, outs);
                let facts = decision(ProxyClass::Sf, contact);
                let got = special(&before, ProxyClass::Sf, &facts);
                assert_eq!(got.disposition, Disposition::Projected);
                assert_eq!(got.realized, ProxyClass::Sf);
                assert_eq!(got.selection.class, ProxyClass::Sf);
                assert_eq!(
                    got.transition,
                    consequence::project(&before, &facts).unwrap()
                );
                assert!(got.transition.caught_sacrifice_fly_facts().is_some());
                let mut expected = before.clone();
                expected.outs += 1;
                expected.score[0] += 1;
                expected.bases[2] = None;
                expected.teams[0].next_batter = 1;
                assert_eq!(got.transition.after(), &expected);
                assert_eq!(
                    got.transition
                        .event()
                        .delta
                        .credits
                        .iter()
                        .filter(|c| c.kind == CreditKind::PlateAppearance)
                        .count(),
                    1
                );
                assert_eq!(
                    EventTransition::from_bytes(&got.transition.to_bytes()).unwrap(),
                    got.transition
                );
            }
        }
    }
}

#[test]
fn supplied_sh_all_supported_contexts_preserves_responsibility_and_canonical_facts() {
    for mask in [1, 5] {
        for outs in 0..2 {
            let before = state(mask, outs);
            let facts = decision(ProxyClass::Sh, AirContact::Fly);
            let got = special(&before, ProxyClass::Sh, &facts);
            assert_eq!(
                (got.disposition, got.realized, got.selection.class),
                (Disposition::Projected, ProxyClass::Sh, ProxyClass::Sh)
            );
            assert_eq!(
                got.transition,
                consequence::project(&before, &facts).unwrap()
            );
            assert!(got.transition.sacrifice_bunt_throw_facts().is_some());
            let mut expected = before.clone();
            expected.outs += 1;
            expected.bases[1] = expected.bases[0].take();
            expected.teams[0].next_batter = 1;
            assert_eq!(got.transition.after(), &expected);
            assert_eq!(got.transition.event().delta.runs, 0);
            assert_eq!(
                EventTransition::from_bytes(&got.transition.to_bytes()).unwrap(),
                got.transition
            );
        }
    }
}

#[test]
fn supplied_gidp_all_supported_contexts_and_initial_equal_receiver_retain_third_out() {
    for mask in [1, 5] {
        for outs in 0..2 {
            for same in [false, true] {
                let before = state(mask, outs);
                let mut facts = decision(ProxyClass::Gidp, AirContact::Fly);
                if same {
                    if let ContactDecision::GroundForceDoublePlay(f) = &mut facts {
                        f.initial_fielder_id = f.first_receiver_id.clone();
                    }
                }
                let got = special(&before, ProxyClass::Gidp, &facts);
                assert_eq!(
                    (got.disposition, got.realized),
                    (Disposition::Projected, ProxyClass::Gidp)
                );
                assert_eq!(
                    got.transition,
                    consequence::project(&before, &facts).unwrap()
                );
                assert!(got.transition.ground_force_double_play_facts().is_some());
                let mut expected = before.clone();
                expected.teams[0].next_batter = 1;
                if outs == 0 {
                    expected.outs = 2;
                    expected.bases[0] = None;
                    assert!(got.transition.event().delta.third_out.is_none());
                } else {
                    expected.outs = 0;
                    expected.bases = [None, None, None];
                    expected.half = Half::Bottom;
                    assert_eq!(
                        got.transition
                            .event()
                            .delta
                            .third_out
                            .as_ref()
                            .unwrap()
                            .kind,
                        ThirdOutKind::BatterBeforeFirst
                    );
                }
                assert_eq!(got.transition.after(), &expected);
                assert_eq!(got.transition.event().delta.outs, 2);
                assert_eq!(got.transition.event().delta.runs, 0);
                assert_eq!(
                    EventTransition::from_bytes(&got.transition.to_bytes()).unwrap(),
                    got.transition
                );
            }
        }
    }
}

#[test]
fn quarter_equalities_neighbors_and_all_low11_words_preserve_strict_selected_classes() {
    let before = state(5, 0);
    let original = parent(&before);
    let bytes = payload();
    let mut grids = vec![0, (1 << 53) - 1];
    for edge in [QUARTER, 2 * QUARTER, 3 * QUARTER] {
        grids.extend([edge - 1, edge, edge + 1]);
    }
    for g in grids {
        for low in 0..2048 {
            let got = adapt(
                &before,
                sample(),
                &original,
                &bytes,
                DIGEST,
                (g << 11) | low,
                None,
            )
            .unwrap();
            assert_eq!(got.selection.class, CLASSES[(g / QUARTER) as usize]);
            assert_eq!(got.selection.grid_word, g);
            assert_eq!(got.transition, original);
            assert_eq!(got.realized, ProxyClass::Remaining);
        }
    }
}

#[test]
fn every_stale_or_wrong_supplied_family_refuses_including_remaining() {
    let before = state(5, 0);
    let original = parent(&before);
    let bytes = payload();
    for selected in CLASSES {
        for supplied in &CLASSES[..3] {
            if selected != *supplied {
                let facts = decision(*supplied, AirContact::Fly);
                assert_eq!(
                    error(adapt(
                        &before,
                        sample(),
                        &original,
                        &bytes,
                        DIGEST,
                        word(selected, 5),
                        Some(&facts)
                    )),
                    "contact-proxy adapter decision/class mismatch"
                );
            }
        }
    }
}

#[test]
fn unknown_contradictory_and_ineligible_declared_facts_never_become_fallback() {
    let before = state(5, 0);
    let original = parent(&before);
    let bytes = payload();
    let mut variants = vec![];
    for n in 0..4 {
        let mut facts = decision(ProxyClass::Sf, AirContact::Fly);
        if let ContactDecision::CaughtSacrificeFly { catch, facts: f } = &mut facts {
            match n {
                0 => f.handling = SacrificeFlyHandling::Unknown,
                1 => f.retouch = SacrificeFlyRetouch::Unknown,
                2 => f.timing = SacrificeFlyRunTiming::BeforeCatch,
                _ => catch.fielder_id = "hbench".into(),
            }
        }
        variants.push((ProxyClass::Sf, facts));
    }
    for n in 0..4 {
        let mut facts = decision(ProxyClass::Sh, AirContact::Fly);
        if let ContactDecision::SacrificeBunt(f) = &mut facts {
            match n {
                0 => f.judgment = SacrificeBuntJudgment::Unknown,
                1 => f.judgment = SacrificeBuntJudgment::ExclusivelyForBaseHit,
                2 => f.receiver_id = f.thrower_id.clone(),
                _ => f.advancing_runner_id = "a4".into(),
            }
        }
        variants.push((ProxyClass::Sh, facts));
    }
    for n in 0..4 {
        let mut facts = decision(ProxyClass::Gidp, AirContact::Fly);
        if let ContactDecision::GroundForceDoublePlay(f) = &mut facts {
            match n {
                0 => f.pivot_id = f.initial_fielder_id.clone(),
                1 => f.pivot_id = f.first_receiver_id.clone(),
                2 => f.forced_runner_id = "a4".into(),
                _ => f.initial_fielder_id = "hremoved".into(),
            }
        }
        variants.push((ProxyClass::Gidp, facts));
    }
    for (class, facts) in variants {
        let expected =
            consequence::project(&before, &facts).expect_err("invalid canonical fixture");
        assert_eq!(
            error(adapt(
                &before,
                sample(),
                &original,
                &bytes,
                DIGEST,
                word(class, 5),
                Some(&facts)
            )),
            expected
        );
    }
}

#[test]
fn bottom9_none_is_unsupported_fallback_but_provided_facts_remain_hard_refusal() {
    let mut before = state(5, 0);
    before.inning = 9;
    before.half = Half::Bottom;
    // Use the same named offense for these supplied facts while testing the API domain.
    before.teams.swap(0, 1);
    let original = parent(&before);
    let bytes = payload();
    for class in &CLASSES[..3] {
        let got = adapt(
            &before,
            sample(),
            &original,
            &bytes,
            DIGEST,
            word(*class, 5),
            None,
        )
        .unwrap();
        assert_eq!(got.disposition, Disposition::TerminalApiUnsupported);
        assert_eq!(got.realized, ProxyClass::Remaining);
        assert_eq!(got.transition, original);
        let facts = decision(*class, AirContact::Fly);
        let expected =
            consequence::project(&before, &facts).expect_err("terminal fixture unsupported");
        assert_eq!(
            error(adapt(
                &before,
                sample(),
                &original,
                &bytes,
                DIGEST,
                word(*class, 5),
                Some(&facts)
            )),
            expected
        );
    }
}

#[test]
fn non_parent_samples_and_real_other_transitions_cannot_be_relabelled_residual() {
    let before = state(5, 0);
    let original = parent(&before);
    let bytes = payload();
    for sample in [
        joint::Sample {
            outcome: preview::Outcome::Single,
            contact: None,
        },
        joint::Sample {
            outcome: preview::Outcome::Roe,
            contact: None,
        },
        joint::Sample {
            outcome: preview::Outcome::ContactOut,
            contact: None,
        },
        joint::Sample {
            outcome: preview::Outcome::Walk,
            contact: Some(joint::ContactKind::Residual),
        },
        joint::Sample {
            outcome: preview::Outcome::ContactOut,
            contact: Some(joint::ContactKind::Ground),
        },
    ] {
        assert_eq!(
            error(adapt(&before, sample, &original, &bytes, DIGEST, 0, None)),
            "invalid contact-proxy adapter parent sample"
        );
    }
    for sample in [
        joint::Sample {
            outcome: preview::Outcome::ContactOut,
            contact: Some(joint::ContactKind::Ground),
        },
        joint::Sample {
            outcome: preview::Outcome::Single,
            contact: None,
        },
    ] {
        let other = apply(&before, joint::event_for(&before, sample).unwrap()).unwrap();
        assert_eq!(
            error(adapt(
                &before,
                self::sample(),
                &other,
                &bytes,
                DIGEST,
                0,
                None
            )),
            "contact-proxy adapter parent transition mismatch"
        );
    }
    let special =
        consequence::project(&before, &decision(ProxyClass::Sf, AirContact::Fly)).unwrap();
    assert_eq!(
        error(adapt(&before, sample(), &special, &bytes, DIGEST, 0, None)),
        "contact-proxy adapter parent transition mismatch"
    );
}

#[test]
fn full_before_mismatch_and_ended_or_invalid_input_refuse_before_later_projection() {
    let before = state(5, 0);
    let original = parent(&before);
    let bytes = payload();
    for n in 0..6 {
        let mut changed = before.clone();
        match n {
            0 => changed.score[0] = 1,
            1 => changed.bases[0].as_mut().unwrap().responsible_pitcher = "other-charge".into(),
            2 => changed.teams[1].bench.push("extra".into()),
            3 => changed.over = true,
            4 => changed.outs = 3,
            _ => changed.inning = 0,
        }
        assert_eq!(
            error(adapt(
                &changed,
                sample(),
                &original,
                &bytes,
                DIGEST,
                0,
                None
            )),
            "contact-proxy adapter before-state mismatch"
        );
    }
    // The old parent has a different before state;this does not certify a later validator branch.
}

#[test]
fn complete_loader_unselected_row_byte_and_digest_refusals_remain_exact() {
    let before = state(5, 0);
    let original = parent(&before);
    let bytes = payload();
    let text = String::from_utf8(bytes.clone()).unwrap();
    let variants = [
        text.replace("C\t0\t0\t0\t0\t0\t0\t0\n", "C\t0\t0\t1\t1\t0\t0\t0\n")
            .into_bytes(),
        [bytes.clone(), b"extra\n".to_vec()].concat(),
        text.replace("split\ttraining", "split\theldout")
            .into_bytes(),
    ];
    for invalid in variants {
        let expected = sampler::select(&invalid, DIGEST, 5, 0, 0)
            .err()
            .expect("bad whole loader fixture");
        assert_eq!(
            error(adapt(
                &before,
                sample(),
                &original,
                &invalid,
                DIGEST,
                0,
                None
            )),
            expected
        );
    }
    let wrong = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let expected = sampler::select(&bytes, wrong, 5, 0, 0).err().unwrap();
    assert_eq!(
        error(adapt(&before, sample(), &original, &bytes, wrong, 0, None)),
        expected
    );
}

#[test]
fn repeated_projection_does_not_mutate_supplied_state_parent_bytes_or_facts() {
    let before = state(5, 0);
    let original = parent(&before);
    let bytes = payload();
    let facts = decision(ProxyClass::Sh, AirContact::Fly);
    let saved = (
        before.clone(),
        original.to_bytes(),
        bytes.clone(),
        facts.clone(),
    );
    let expected = consequence::project(&before, &facts).unwrap();
    for _ in 0..4 {
        let got = adapt(
            &before,
            sample(),
            &original,
            &bytes,
            DIGEST,
            word(ProxyClass::Sh, 5),
            Some(&facts),
        )
        .unwrap();
        assert_eq!(got.transition, expected);
        assert_eq!(got.disposition, Disposition::Projected);
    }
    assert_eq!((before, original.to_bytes(), bytes, facts), saved);
}
