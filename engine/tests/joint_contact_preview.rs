//! Experimental joint source-shape sampling; not historical contact certificates.
#[path = "../examples/support/joint_contact.rs"]
mod joint;
#[path = "../examples/support/preview.rs"]
mod preview;
use bbsim_engine::{apply, AirContact, CreditKind, FieldingReview, Half, Runner};
use joint::{ContactKind, JointModel, Sample};

fn baseline() -> preview::Baseline {
    preview::Baseline::from_counts([42823, 15142, 1984, 25947, 8531, 785, 6776, 1470, 82245])
        .unwrap()
}

#[test]
fn fixed_boundaries_preserve_legacy_categories_and_exactly_one_rng_word_per_pa() {
    let base = baseline();
    let model = JointModel::from_counts(&base, [17188, 19528, 5379, 536, 39614]).unwrap();
    let expected = [
        0x3fcd842faa8b150b,
        0x3fd3fa095bd4e642,
        0x3fd4a91e5c6cd28e,
        0x3fdd9a54a2f15f90,
        0x3fe0458294e445f7,
        0x3fe068290d556a82,
        0x3fe193168140df9b,
        0x3fe1d3f485c334cd,
        0x3fe4ca29925f5f41,
        0x3fe827970f269a30,
        0x3fe914e30924145f,
        0x3fe92c8d7590da2c,
        0x3ff0000000000000,
    ];
    assert_eq!(model.boundaries().map(f64::to_bits), expected);
    assert_eq!(model.marginals(), *base.probabilities());
    for seed in [1, 2, 3, 99] {
        let mut old = preview::Rng::new(seed);
        let mut new = preview::Rng::new(seed);
        for _ in 0..256 {
            let outcome = base.sample(&mut old);
            let selected = model.sample(&mut new);
            assert_eq!(selected.outcome, outcome);
            assert_eq!(
                selected.contact.is_some(),
                outcome == preview::Outcome::ContactOut
            );
            assert_eq!(old.next_u64(), new.next_u64());
        }
    }
    assert_eq!(
        joint::MODEL_VERSION,
        "pooled-jeffreys-joint-proxy-preview-v1"
    );
    assert_eq!(joint::POLICY_VERSION, "station-held-joint-proxy-preview-v1");
    assert_eq!(preview::MODEL_VERSION, "pooled-jeffreys-preview-v1");
    assert_eq!(preview::POLICY_VERSION, "station-ground-contact-v1");
    assert_eq!(preview::RNG_VERSION, "splitmix64-v1");
}

fn selected(index: usize) -> Sample {
    if index < 8 {
        Sample {
            outcome: preview::ALL[index],
            contact: None,
        }
    } else {
        Sample {
            outcome: preview::Outcome::ContactOut,
            contact: Some(
                [
                    ContactKind::Ground,
                    ContactKind::Fly,
                    ContactKind::LineDrive,
                    ContactKind::PopUp,
                    ContactKind::Residual,
                ][index - 8],
            ),
        }
    }
}

#[test]
fn exact_edges_go_to_the_next_interval_without_a_second_roll() {
    let model = JointModel::from_counts(&baseline(), [17188, 19528, 5379, 536, 39614]).unwrap();
    for (i, edge) in model.boundaries().iter().enumerate() {
        let bits = edge.to_bits();
        assert_eq!(
            model.sample_unit(f64::from_bits(bits - 1)).unwrap(),
            selected(i)
        );
        if i < 12 {
            assert_eq!(model.sample_unit(*edge).unwrap(), selected(i + 1));
            assert_eq!(
                model.sample_unit(f64::from_bits(bits + 1)).unwrap(),
                selected(i + 1)
            );
        } else {
            assert!(model.sample_unit(*edge).is_err());
            assert!(model.sample_unit(f64::from_bits(bits + 1)).is_err());
        }
    }
    assert_eq!(model.sample_unit(0.0).unwrap(), selected(0));
}

#[test]
fn checked_counts_and_collapsed_intervals_are_rejected_not_retuned() {
    assert!(JointModel::from_counts(&baseline(), [u64::MAX; 5]).is_err());
    let base = preview::Baseline::from_counts([1u64 << 52, 0, 0, 0, 0, 0, 0, 0, 0]).unwrap();
    assert!(JointModel::from_counts(&base, [0; 5]).is_err());
    for concentrated in 0..5 {
        let mut counts = [0; 5];
        counts[concentrated] = 100;
        let base = preview::Baseline::from_counts([1, 1, 1, 1, 1, 1, 1, 1, 100]).unwrap();
        let model = JointModel::from_counts(&base, counts).unwrap();
        assert!(model
            .conditionals()
            .iter()
            .all(|p| p.is_finite() && *p > 0.0));
        assert!((model.conditionals().iter().sum::<f64>() - 1.0).abs() <= 32.0 * f64::EPSILON);
        assert!(model.boundaries().windows(2).all(|pair| pair[1] > pair[0]));
    }
}

#[test]
fn inconsistent_sample_pairs_are_not_silently_reinterpreted() {
    let log = preview::game(1, &baseline()).unwrap();
    for sample in [
        Sample {
            outcome: preview::Outcome::Single,
            contact: Some(ContactKind::Fly),
        },
        Sample {
            outcome: preview::Outcome::ContactOut,
            contact: None,
        },
    ] {
        assert!(joint::event_for(log[0].before(), sample).is_err());
    }
}

#[test]
fn all_five_policies_conserve_all_before_mask_out_half_frames_with_dh_or_no_dh() {
    let log = preview::game(1, &baseline()).unwrap();
    let initial = log[0].before();
    for no_dh in [false, true] {
        for half in [Half::Top, Half::Bottom] {
            for outs in 0..3 {
                for mask in 0..8 {
                    let mut before = initial.clone();
                    before.half = half;
                    before.outs = outs;
                    if no_dh {
                        for team in &mut before.teams {
                            team.dh_slot = None;
                            team.dh_must_bat = false;
                            team.batting_order[8] = team.pitcher.clone();
                        }
                    }
                    for (i, base) in before.bases.iter_mut().enumerate() {
                        *base = (mask & (1 << i) != 0).then(|| Runner {
                            // Keep the next batter off base in the after snapshot (as in legacy fixtures).
                            player_id: before.teams[half.offense()].batting_order[i + 5].clone(),
                            responsible_pitcher: before.teams[1 - half.offense()].pitcher.clone(),
                        });
                    }
                    let old = apply(
                        &before,
                        preview::event_for(&before, preview::Outcome::ContactOut).unwrap(),
                    )
                    .unwrap_or_else(|error| panic!("legacy fixture no_dh{no_dh} {half:?} outs{outs} mask{mask} cursor{}: {error:?}; bases={:?}", before.teams[half.offense()].next_batter, before.bases));
                    for index in 8..13 {
                        let event = joint::event_for(&before, selected(index)).unwrap();
                        let transition = apply(&before, event).unwrap();
                        assert_eq!(transition.after(), old.after());
                        assert_eq!(transition.event().delta.moves, old.event().delta.moves);
                        assert_eq!(
                            transition.event().delta.third_out,
                            old.event().delta.third_out
                        );
                        if index == 8 {
                            assert_eq!(transition.event(), old.event());
                        } else if index < 12 {
                            let facts = transition.event().ordinary_air_catch.as_ref().unwrap();
                            let defense = &before.teams[1 - half.offense()];
                            assert!(defense.batting_order.contains(&facts.fielder_id));
                            assert_ne!(
                                defense
                                    .dh_slot
                                    .map(|slot| &defense.batting_order[slot as usize]),
                                Some(&facts.fielder_id)
                            );
                            assert_eq!(
                                facts.fielder_id,
                                defense.batting_order[[6, 7, 2][index - 9]]
                            );
                            assert_eq!(
                                transition.ordinary_air_fielding().unwrap().putouts
                                    [&facts.fielder_id],
                                1
                            );
                        } else {
                            assert_eq!(transition.event().fielding_review, FieldingReview::Unknown);
                            assert!(transition.ordinary_ground_fielding().is_none());
                            assert!(transition.ordinary_air_fielding().is_none());
                        }
                    }
                    for index in 0..8 {
                        assert_eq!(
                            joint::event_for(&before, selected(index)).unwrap(),
                            preview::event_for(&before, preview::ALL[index]).unwrap()
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn mutated_legacy_counts_cannot_disagree_with_preserved_probabilities() {
    let mut base = baseline();
    base.counts[0] += 1;
    assert!(JointModel::from_counts(&base, [17188, 19528, 5379, 536, 39614]).is_err());
}

#[test]
fn zero_observed_conditional_counts_have_a_positive_uniform_prior_not_an_epsilon() {
    let base = preview::Baseline::from_counts([1, 1, 1, 1, 1, 1, 1, 1, 0]).unwrap();
    let model = JointModel::from_counts(&base, [0; 5]).unwrap();
    assert_eq!(model.conditionals(), [0.2; 5]);
    assert!(model.boundaries().windows(2).all(|pair| pair[1] > pair[0]));
    assert!(JointModel::from_counts(&base, [1, 0, 0, 0, 0]).is_err());
    for value in [f64::NAN, f64::INFINITY, -1.0, 1.0] {
        assert!(model.sample_unit(value).is_err());
    }
}

#[test]
fn explicit_air_and_residual_change_causal_coverage_not_held_retirement_physics() {
    // Exercise the unchanged legacy CSV parser/game as actual callers, not a duplicate state factory.
    let text = "key,value\nformat,preview-counts-v1\ncatalog_sha256,2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583\nquery_version,preview-baseline-v1\nselected_pa,9\nexcluded_ibb,0\nexcluded_interference,0\nstrikeout,1\nwalk,1\nhbp,1\nsingle,1\ndouble,1\ntriple,1\nhome_run,1\nroe,1\ncontact_out,1\n";
    let base = preview::Baseline::read_csv(text).unwrap();
    let log = preview::game(1, &base).unwrap();
    let before = log[0].before();
    let old = apply(
        before,
        preview::event_for(before, preview::Outcome::ContactOut).unwrap(),
    )
    .unwrap();
    for (kind, contact) in [
        (ContactKind::Fly, Some(AirContact::Fly)),
        (ContactKind::LineDrive, Some(AirContact::LineDrive)),
        (ContactKind::PopUp, Some(AirContact::PopUp)),
        (ContactKind::Residual, None),
    ] {
        let event = joint::event_for(
            before,
            Sample {
                outcome: preview::Outcome::ContactOut,
                contact: Some(kind),
            },
        )
        .unwrap();
        assert!(event.ordinary_ground_throw.is_none());
        assert!(!event
            .delta
            .credits
            .iter()
            .any(|c| c.kind == CreditKind::Assist));
        assert_eq!(
            event.ordinary_air_catch.as_ref().map(|facts| facts.contact),
            contact
        );
        if kind == ContactKind::Residual {
            assert_eq!(event.fielding_review, FieldingReview::Unknown);
            assert!(!event
                .delta
                .credits
                .iter()
                .any(|c| c.kind == CreditKind::Putout));
        } else {
            assert_eq!(event.fielding_review, FieldingReview::NoMisplay);
        }
        let transition = apply(before, event).unwrap();
        assert_eq!(transition.after(), old.after());
        if contact.is_some() {
            let fielding = transition.ordinary_air_fielding().unwrap();
            assert_eq!(fielding.putouts.values().sum::<u32>(), 1);
            assert_eq!(fielding.assists.values().sum::<u32>(), 0);
        } else {
            assert!(transition.ordinary_air_fielding().is_none());
        }
    }
}

#[test]
fn residual_actor_uncertainty_is_not_repaired_by_later_qualified_model_events() {
    use bbsim_engine::accounting::derive_accounting;
    let legacy = preview::game(1, &baseline()).unwrap();
    let mut before = legacy[0].before().clone();
    let mut log = Vec::new();
    for sample in [selected(12), selected(6), selected(9), selected(8)] {
        let transition = apply(&before, joint::event_for(&before, sample).unwrap()).unwrap();
        before = transition.after().clone();
        log.push(transition);
    }
    let stats = derive_accounting(&log).unwrap();
    assert_eq!(stats.runs, [1, 0]);
    assert_eq!(stats.pitchers[1]["Home-pitcher"].outs_recorded, 3);
    assert_eq!(stats.pitchers[1]["Home-pitcher"].earned_runs, None);
    assert_eq!(stats.fielders[1]["Home-7"].credited_putouts, 1);
    for line in stats.classified_fielding[1].values() {
        assert_eq!(
            (line.putouts, line.assists, line.errors, line.passed_balls),
            (None, None, None, None)
        );
    }
    assert!(log[2].ordinary_air_fielding().is_some());
    assert!(log[3].ordinary_ground_fielding().is_some());
}

#[test]
fn named_before_actor_selection_excludes_each_dh_slot_and_supports_pitcher_catches() {
    let legacy = preview::game(1, &baseline()).unwrap();
    for slot in 0..9 {
        let mut before = legacy[0].before().clone();
        before.teams[1].dh_slot = Some(slot);
        for (index, ordinal) in [(9, 6), (10, 7), (11, 2)] {
            let transition =
                apply(&before, joint::event_for(&before, selected(index)).unwrap()).unwrap();
            let facts = transition.event().ordinary_air_catch.as_ref().unwrap();
            let expected_index = ordinal + usize::from(usize::from(slot) <= ordinal);
            assert_eq!(
                facts.fielder_id,
                before.teams[1].batting_order[expected_index]
            );
            assert_ne!(
                facts.fielder_id,
                before.teams[1].batting_order[usize::from(slot)]
            );
        }
    }
    let mut before = legacy[0].before().clone();
    before.teams[1].dh_slot = None;
    before.teams[1].dh_must_bat = false;
    before.teams[1].batting_order[2] = before.teams[1].pitcher.clone();
    let transition = apply(&before, joint::event_for(&before, selected(11)).unwrap()).unwrap();
    assert_eq!(
        transition
            .event()
            .ordinary_air_catch
            .as_ref()
            .unwrap()
            .fielder_id,
        before.teams[1].pitcher
    );
    assert_eq!(
        transition.ordinary_air_fielding().unwrap().putouts[&before.teams[1].pitcher],
        1
    );
}

#[test]
fn terminal_or_malformed_before_state_cannot_publish_legal_joint_transition() {
    let legacy = preview::game(1, &baseline()).unwrap();
    let mut before = legacy[0].before().clone();
    before.over = true;
    for index in 0..13 {
        assert!(joint::event_for(&before, selected(index)).is_err());
    }
    before.over = false;
    before.teams[1].batting_order[6] = before.teams[1].batting_order[8].clone();
    for index in 8..13 {
        let frozen = before.clone();
        let event = joint::event_for(&before, selected(index)).unwrap();
        assert!(apply(&before, event).is_err());
        assert_eq!(before, frozen);
    }
}

#[test]
fn loader_accepts_only_the_pinned_audited_logical_vector_and_provenance() {
    let base = baseline();
    let loaded = JointModel::read_csv(JOINT_CSV, &base).unwrap();
    let direct = JointModel::from_counts(&base, [17188, 19528, 5379, 536, 39614]).unwrap();
    assert_eq!(loaded.boundaries(), direct.boundaries());
    let lines = JOINT_CSV.lines().skip(1).collect::<Vec<_>>();
    let reordered = format!(
        "key,value\n{}\n",
        lines.iter().rev().copied().collect::<Vec<_>>().join("\n")
    );
    assert_eq!(
        JointModel::read_csv(&reordered, &base)
            .unwrap()
            .boundaries(),
        loaded.boundaries()
    );
    for (i, line) in lines.iter().enumerate() {
        let (key, _) = line.split_once(',').unwrap();
        let changed = lines
            .iter()
            .enumerate()
            .map(|(j, row)| {
                if i == j {
                    format!("{key},changed")
                } else {
                    (*row).to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            JointModel::read_csv(&format!("key,value\n{changed}\n"), &base).is_err(),
            "changed {key}"
        );
        let missing = lines
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, row)| *row)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            JointModel::read_csv(&format!("key,value\n{missing}\n"), &base).is_err(),
            "missing {key}"
        );
        assert!(
            JointModel::read_csv(&format!("{JOINT_CSV}{line}\n"), &base).is_err(),
            "duplicate {key}"
        );
    }
    for bad in [
        JOINT_CSV.replacen("key,value", "key,value,extra", 1),
        format!("{JOINT_CSV}extra,1\n"),
        JOINT_CSV.replace("residual,39614", "residual,39613"),
        JOINT_CSV.replace("residual,39614", "residual,18446744073709551616"),
        JOINT_CSV
            .replace("fly_candidate,19528", "fly_candidate,19529")
            .replace("ground_candidate,17188", "ground_candidate,17187"),
    ] {
        assert!(JointModel::read_csv(&bad, &base).is_err());
    }
    for value in [
        "+39614", "-39614", " 39614", "39614 ", "39614.0", "3.9614e4", "", "39614,0",
    ] {
        assert!(JointModel::read_csv(
            &JOINT_CSV.replace("residual,39614", &format!("residual,{value}")),
            &base
        )
        .is_err());
    }
    assert!(
        JointModel::read_csv(JOINT_CSV, &preview::Baseline::from_counts([1; 9]).unwrap()).is_err()
    );
}

#[test]
fn wrapper_requires_explicit_flag_and_does_not_steal_legacy_option_values() {
    let args = |values: &[&str]| values.iter().map(|v| v.to_string()).collect::<Vec<_>>();
    for values in [
        &["--baseline", "counts.csv", "--seed", "99", "--trace"][..],
        &["--help"][..],
        &["--baseline", "--contact-counts"][..],
        &["--seed", "--contact-counts"][..],
    ] {
        let supplied = args(values);
        assert_eq!(joint::split_options(&supplied).unwrap(), (supplied, None));
    }
    assert_eq!(
        joint::split_options(&args(&[
            "--baseline",
            "old.csv",
            "--contact-counts",
            "joint.csv",
            "--autoplay"
        ]))
        .unwrap(),
        (
            args(&["--baseline", "old.csv", "--autoplay"]),
            Some("joint.csv".into())
        )
    );
    for values in [
        &["--contact-counts"][..],
        &["--contact-counts", ""][..],
        &["--contact-counts", "--trace"][..],
        &["--contact-counts", "a", "--contact-counts", "b"][..],
    ] {
        assert!(joint::split_options(&args(values)).is_err());
    }
}

#[test]
fn joint_sampler_100000_draws_match_preregistered_joint_mass_diagnostic() {
    let base = baseline();
    let model = JointModel::from_counts(&base, [17188, 19528, 5379, 536, 39614]).unwrap();
    let mut rng = preview::Rng::new(19);
    let mut draws = [0u32; 13];
    for _ in 0..100_000 {
        let sample = model.sample(&mut rng);
        let index = (0..13).find(|i| selected(*i) == sample).unwrap();
        draws[index] += 1;
    }
    for (i, observed) in draws.iter().enumerate() {
        let p = if i < 8 {
            base.probabilities()[i]
        } else {
            base.probabilities()[8] * model.conditionals()[i - 8]
        };
        let expected = 100_000.0 * p;
        assert!(
            (f64::from(*observed) - expected).abs()
                <= 6.0 * (100_000.0 * p * (1.0 - p)).sqrt() + 1.0,
            "bin{i}: {observed}, expected{expected}"
        );
    }
    println!("JOINT100000 seed19 draws={draws:?}; fixed six-sigma+one count, not calibration");
}

#[test]
fn paired_2000_games_repeat_roundtrip_conserve_available_stats_and_legacy_physics() {
    use bbsim_engine::{accounting::derive_accounting, EventTransition};
    let base = baseline();
    let model = JointModel::from_counts(&base, [17188, 19528, 5379, 536, 39614]).unwrap();
    let mut kinds = [0u64; 5];
    for seed in 0..2000 {
        let old = preview::game(seed, &base).unwrap();
        let log = joint::game(seed, &model).unwrap();
        assert_eq!(log, joint::game(seed, &model).unwrap(), "seed{seed}");
        assert_eq!(log.len(), old.len());
        assert!(log.last().unwrap().after().over);
        let mut residual_outs = [0u32; 2];
        let mut raw_putouts = [0u32; 2];
        let mut raw_assists = [0u32; 2];
        for (i, (transition, legacy)) in log.iter().zip(&old).enumerate() {
            assert_eq!(transition.before(), legacy.before(), "seed{seed}/play{i}");
            assert_eq!(transition.after(), legacy.after(), "seed{seed}/play{i}");
            assert_eq!(transition.event().delta.moves, legacy.event().delta.moves);
            assert_eq!(
                transition.event().delta.third_out,
                legacy.event().delta.third_out
            );
            assert_eq!(
                EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
                *transition
            );
            if i > 0 {
                transition.follows(&log[i - 1]).unwrap();
            }
            let event = transition.event();
            let defense = 1 - transition.before().half.offense();
            raw_putouts[defense] += event
                .delta
                .credits
                .iter()
                .filter(|c| c.kind == CreditKind::Putout)
                .count() as u32;
            raw_assists[defense] += event
                .delta
                .credits
                .iter()
                .filter(|c| c.kind == CreditKind::Assist)
                .count() as u32;
            if event.ordinary_ground_throw.is_some() {
                kinds[0] += 1;
            } else if let Some(facts) = &event.ordinary_air_catch {
                kinds[match facts.contact {
                    AirContact::Fly => 1,
                    AirContact::LineDrive => 2,
                    AirContact::PopUp => 3,
                }] += 1;
            } else if event.fielding_review == FieldingReview::Unknown {
                kinds[4] += 1;
                residual_outs[defense] += 1;
                assert_eq!(event.delta.outs, 1);
            }
        }
        let stats = derive_accounting(&log).unwrap();
        assert!(stats.complete_game);
        assert_eq!(
            stats.runs.map(|n| n as u16),
            log.last().unwrap().after().score
        );
        for side in 0..2 {
            let batting = &stats.batters[1 - side];
            let pitching = &stats.pitchers[side];
            assert_eq!(
                batting.values().map(|b| b.plate_appearances).sum::<u32>(),
                pitching.values().map(|p| p.batters_faced).sum::<u32>()
            );
            assert_eq!(
                batting.values().map(|b| b.credited_runs).sum::<u32>(),
                stats.runs[1 - side]
            );
            assert_eq!(
                pitching.values().map(|p| p.runs_allowed).sum::<u32>(),
                stats.runs[1 - side]
            );
            assert_eq!(
                pitching.values().map(|p| p.outs_recorded).sum::<u32>(),
                raw_putouts[side] + residual_outs[side]
            );
            assert_eq!(
                stats.fielders[side]
                    .values()
                    .map(|f| f.credited_putouts)
                    .sum::<u32>(),
                raw_putouts[side]
            );
            assert_eq!(
                stats.fielders[side]
                    .values()
                    .map(|f| f.credited_assists)
                    .sum::<u32>(),
                raw_assists[side]
            );
            assert!(batting
                .values()
                .all(|b| b.credited_hit_breakdown.as_ref().map(|h| h.hits())
                    == Some(b.credited_hits)
                    && b.at_bats
                        == Some(b.plate_appearances - b.credited_walks - b.credited_hit_by_pitch)));
        }
    }
    assert!(kinds.iter().all(|n| *n > 0));
    println!("JOINT2000 seeds0..1999 all types={kinds:?}; paired legacy physics/repeats/codec/available-stat conservation, not complete boxes");
}

#[test]
fn generated_cli_optin_uses_native_joint_events_and_preserves_default_mode() {
    use std::{
        fs,
        path::PathBuf,
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };
    struct Fixture(PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).expect("remove only this test's exclusive fixture");
        }
    }
    let path = std::env::temp_dir().join(format!(
        "bbsim-joint-cli-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&path).unwrap();
    let fixture = Fixture(path);
    let target = fixture.0.join("target");
    let build = Command::new(env!("CARGO"))
        .args(["build", "--offline", "--locked", "--manifest-path"])
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .args(["--example", "generated_game", "--target-dir"])
        .arg(&target)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert!(!String::from_utf8_lossy(&build.stderr).contains("warning:"));
    let csv = fixture.0.join("joint.csv");
    fs::write(&csv, JOINT_CSV).unwrap();
    let old_csv = fixture.0.join("old.csv");
    let mut old = String::from("key,value\nformat,preview-counts-v1\ncatalog_sha256,2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583\nquery_version,preview-baseline-v1\nselected_pa,186517\nexcluded_ibb,753\nexcluded_interference,61\n");
    for key in [
        "strikeout",
        "walk",
        "hbp",
        "single",
        "double",
        "triple",
        "home_run",
        "roe",
        "contact_out",
    ] {
        old.push_str(
            JOINT_CSV
                .lines()
                .find(|line| line.starts_with(&format!("{key},")))
                .unwrap(),
        );
        old.push('\n');
    }
    fs::write(&old_csv, old).unwrap();
    let executable = target.join("debug/examples/generated_game");
    let run = |extra: &[&str]| {
        Command::new(&executable)
            .arg("--baseline")
            .arg(&old_csv)
            .args(["--seed", "1", "--autoplay", "--trace"])
            .args(extra)
            .output()
            .unwrap()
    };
    let plain = run(&[]);
    assert!(plain.status.success());
    assert!(plain.stderr.is_empty());
    let plain_text = String::from_utf8(plain.stdout).unwrap();
    assert!(plain_text
        .contains("Model: pooled-jeffreys-preview-v1 | Policy: station-ground-contact-v1"));
    assert!(!plain_text.contains("ordinary_air_catch"));
    assert!(!plain_text.contains(joint::MODEL_VERSION));
    let extra = ["--contact-counts", csv.to_str().unwrap()];
    let opted = run(&extra);
    assert!(
        opted.status.success(),
        "{}",
        String::from_utf8_lossy(&opted.stderr)
    );
    assert!(opted.stderr.is_empty());
    assert_eq!(opted.stdout, run(&extra).stdout);
    let text = String::from_utf8(opted.stdout).unwrap();
    assert!(text.contains(joint::MODEL_VERSION));
    assert!(text.contains(joint::POLICY_VERSION));
    assert!(text.contains("source-shape mixture"));
    assert!(text.contains("synthetic held retirements"));
    assert!(text.contains("residual partial coverage"));
    assert!(text.contains("ordinary_air_catch"));
    assert!(text.contains("Retrosheet"));
    assert!(text.contains("GAME OVER"));
    fs::write(&csv, JOINT_CSV.replace("residual,39614", "residual,39613")).unwrap();
    let invalid = run(&extra);
    assert!(!invalid.status.success());
    assert!(
        invalid.stdout.is_empty(),
        "unvalidated data must not publish a game"
    );
    let repeated = run(&[
        "--contact-counts",
        csv.to_str().unwrap(),
        "--contact-counts",
        csv.to_str().unwrap(),
    ]);
    assert_eq!(repeated.status.code(), Some(2));
    assert!(repeated.stdout.is_empty());
}

const JOINT_CSV: &str = r#"key,value
format,joint-proxy-counts-v1
cohort_format,2019-joint-contact-proxies-v1
catalog_sha256,2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583
source_commit,b449f7bec49f852c27803e096303a4c78ac661b9
snapshot_sha256,d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528
supplement_sha256,554632467416eef202b7a55cadad2d09e5355ce98c83868b26b5267f98852b82
recipe_sha256,174e9b959f59359f9fa86c7881dbbf9aefd5981f9376d08b60b975126ff4afc3
artifact_sha256,1573fb845ac24d644ac722cc267b3729b2174fc114ff9f6748e2435502fe29ca
included_pa,185703
excluded_ibb,753
excluded_interference,61
attribution,The information used here was obtained free of charge from and is copyrighted by Retrosheet. Interested parties may contact Retrosheet at "www.retrosheet.org".
contact_out,82245
double,8531
hbp,1984
home_run,6776
roe,1470
single,25947
strikeout,42823
triple,785
walk,15142
fly_candidate,19528
ground_candidate,17188
line_candidate,5379
pop_candidate,536
residual,39614
"#;
