#[path = "../examples/support/diagnostics.rs"]
mod diagnostics;
#[path = "../examples/support/presentation.rs"]
mod presentation;
#[path = "../examples/support/preview.rs"]
mod preview;
#[path = "../examples/support/preview_cli.rs"]
mod preview_cli;
use bbsim_engine::{
    accounting::derive_accounting, apply, Base, Destination, EventTransition, GameState, Half,
    TeamState,
};
use preview::{Baseline, Outcome, Rng, ALL};

fn initial() -> GameState {
    let team = |name: &str| TeamState {
        batting_order: std::array::from_fn(|i| format!("{name}-{}", i + 1)),
        next_batter: 0,
        pitcher: format!("{name}-pitcher"),
        dh_slot: Some(8),
        dh_must_bat: true,
        bench: vec![],
        removed: vec![],
    };
    GameState {
        inning: 1,
        half: Half::Top,
        outs: 0,
        score: [0, 0],
        teams: [team("Away"), team("Home")],
        bases: [None, None, None],
        over: false,
    }
}
const CSV: &str = "key,value\nformat,preview-counts-v1\ncatalog_sha256,2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583\nquery_version,preview-baseline-v1\nselected_pa,103\nexcluded_ibb,2\nexcluded_interference,1\nstrikeout,20\nwalk,8\nhbp,1\nsingle,15\ndouble,4\ntriple,1\nhome_run,4\nroe,2\ncontact_out,45\n";

#[test]
fn diagnostic_harness_reports_checks_and_rejects_empty_batch() {
    let baseline = Baseline::read_csv(CSV).unwrap();
    assert!(diagnostics::verify(&baseline, 0).is_err());
    let report = diagnostics::verify(&baseline, 10).unwrap();
    assert!(report.contains("Verified 10 seeds"));
    assert!(report.contains("not calibration or complete boxes"));
}

#[test]
fn cli_requires_a_baseline_and_validates_seed_and_flags() {
    let args = |values: &[&str]| values.iter().map(|v| v.to_string()).collect::<Vec<_>>();
    assert_eq!(
        preview_cli::parse(&args(&[
            "--baseline",
            "counts.csv",
            "--seed",
            "123",
            "--autoplay"
        ]))
        .unwrap(),
        Some(preview_cli::Options {
            baseline: "counts.csv".into(),
            seed: 123,
            autoplay: true
        })
    );
    assert_eq!(preview_cli::parse(&args(&["--help"])).unwrap(), None);
    assert_eq!(
        preview_cli::parse(&args(&["--baseline", "counts.csv"]))
            .unwrap()
            .unwrap()
            .seed,
        1
    );
    for values in [
        &[][..],
        &["--seed", "x"][..],
        &["--baseline"][..],
        &["--baseline", "a", "--seed", "-1"][..],
        &["--baseline", "a", "--unknown"][..],
        &["--baseline", "a", "--baseline", "b"][..],
        &["--baseline", "a", "--seed", "1", "--seed", "2"][..],
    ] {
        assert!(preview_cli::parse(&args(values)).is_err());
    }
}

#[test]
fn generated_terminal_shows_contact_outs_seeded_game_and_unknown_er() {
    assert_eq!(preview::MODEL_VERSION, "pooled-jeffreys-preview-v1");
    assert_eq!(preview::POLICY_VERSION, "station-ground-contact-v1");
    assert_eq!(preview::RNG_VERSION, "splitmix64-v1");
    let baseline = Baseline::read_csv(CSV).unwrap();
    let log = preview::game(1, &baseline).unwrap();
    let mut output = Vec::new();
    presentation::present(&log, &mut &b""[..], &mut output, true).unwrap();
    let text = String::from_utf8(output).unwrap();
    assert!(text.contains("ground/contact out"));
    assert!(text.contains("GAME OVER"));
    assert!(text.contains("ER"));
    assert!(text.contains('?'));
    assert_eq!(text.matches("Play ").count(), log.len());
}

#[test]
fn counts_parser_reconciles_and_smoothing_is_positive() {
    let baseline = Baseline::read_csv(CSV).unwrap();
    assert_eq!(baseline.counts, [20, 8, 1, 15, 4, 1, 4, 2, 45]);
    assert!((baseline.probabilities().iter().sum::<f64>() - 1.0).abs() < 1e-12);
    for (i, p) in baseline.probabilities().iter().enumerate() {
        assert!((*p - (baseline.counts[i] as f64 + 0.5) / 104.5).abs() < 1e-12);
    }
    assert!(Baseline::from_counts([0; 9]).is_err());
    assert!(Baseline::from_counts([0, 0, 0, 0, 0, 0, 0, 0, 1])
        .unwrap()
        .probabilities()
        .iter()
        .all(|p| p.is_finite() && *p > 0.0));
    for bad in [
        CSV.replace("single,15\n", ""),
        CSV.replace("selected_pa,103", "selected_pa,104"),
        format!("{CSV}single,15\n"),
        CSV.replace("walk,8", "walk,-8"),
        CSV.replace("preview-counts-v1", "other"),
        CSV.replace("preview-baseline-v1", "other"),
        CSV.replace("single,15", "single,NaN"),
    ] {
        assert!(Baseline::read_csv(&bad).is_err(), "accepted {bad}");
    }
}

#[test]
fn splitmix_vectors_and_sampler_frequencies_are_fixed() {
    let mut rng = Rng::new(0);
    assert_eq!(
        [rng.next_u64(), rng.next_u64(), rng.next_u64()],
        [0xe220a8397b1dcdaf, 0x6e789e6aa1b965f4, 0x06c45d188009454f]
    );
    let baseline = Baseline::read_csv(CSV).unwrap();
    let mut rng = Rng::new(19);
    let mut counts = [0u32; 9];
    for _ in 0..100_000 {
        let sample = baseline.sample(&mut rng);
        counts[ALL.iter().position(|v| *v == sample).unwrap()] += 1;
    }
    for (i, p) in baseline.probabilities().iter().enumerate() {
        let observed = f64::from(counts[i]) / 100_000.0;
        assert!(
            (observed - p).abs() <= 6.0 * (p * (1.0 - p) / 100_000.0).sqrt() + 1.0 / 100_000.0,
            "category {i}: {observed} vs {p}"
        );
    }
}

#[test]
fn only_chosen_contact_out_supplies_bound_ground_throw_evidence() {
    for outcome in ALL {
        let before = initial();
        let event = preview::event_for(&before, outcome).unwrap();
        let t = apply(&before, event.clone()).unwrap();
        if outcome == Outcome::ContactOut {
            let facts = event
                .ordinary_ground_throw
                .as_ref()
                .expect("chosen direct ground throw");
            assert_eq!(facts.batter_id, "Away-1");
            assert_eq!(facts.thrower_id, "Home-4");
            assert_eq!(facts.receiver_id, "Home-2");
            let contribution = t
                .ordinary_ground_fielding()
                .expect("qualified contact evidence");
            assert_eq!(contribution.putouts, [("Home-2".into(), 1)].into());
            assert_eq!(contribution.assists, [("Home-4".into(), 1)].into());
            let mut generic = event;
            generic.ordinary_ground_throw = None;
            let old = apply(&before, generic).unwrap();
            assert_eq!(old.before(), t.before());
            assert_eq!(old.after(), t.after());
            assert_eq!(old.event().delta, t.event().delta);
            assert_eq!(
                derive_accounting(&[old]).unwrap(),
                derive_accounting(&[t]).unwrap()
            );
        } else {
            assert!(event.ordinary_ground_throw.is_none());
            assert_eq!(t.ordinary_ground_fielding(), None);
        }
    }
}

#[test]
fn every_category_has_a_legal_engine_event() {
    for outcome in ALL {
        let before = initial();
        let event = preview::event_for(&before, outcome).unwrap();
        let transition = apply(&before, event).unwrap();
        assert_eq!(
            EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
            transition
        );
        assert_eq!(
            transition.after().outs,
            u8::from(matches!(outcome, Outcome::Strikeout | Outcome::ContactOut))
        );
    }
}

#[test]
fn all_masks_outs_halves_and_winning_contexts_use_legal_policy() {
    for context in 0..3 {
        for half in [Half::Top, Half::Bottom] {
            for outs in 0..3 {
                for mask in 0u8..8 {
                    for outcome in ALL {
                        let mut before = initial();
                        before.half = half;
                        before.outs = outs;
                        if context > 0 {
                            before.inning = 9;
                            before.score = if context == 1 { [0, 0] } else { [1, 0] };
                        }
                        let offense = half.offense();
                        for base in 0..3 {
                            if mask & (1 << base) != 0 {
                                before.bases[base] = Some(bbsim_engine::Runner {
                                    player_id: before.teams[offense].batting_order[base + 5]
                                        .clone(),
                                    responsible_pitcher: before.teams[1 - offense].pitcher.clone(),
                                });
                            }
                        }
                        let event = preview::event_for(&before, outcome).unwrap();
                        let t=apply(&before,event).unwrap_or_else(|e|panic!("context{context} {half:?} outs{outs} mask{mask} {outcome:?}: {e:?}"));
                        assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
                        assert_eq!(
                            t.event().ordinary_ground_throw.is_some(),
                            outcome == Outcome::ContactOut
                        );
                        assert_eq!(
                            t.ordinary_ground_fielding().is_some(),
                            outcome == Outcome::ContactOut
                        );
                        assert_eq!(
                            t.event().caught_strikeout.is_some(),
                            outcome == Outcome::Strikeout
                        );
                        assert_eq!(
                            t.caught_strikeout_fielding().is_some(),
                            outcome == Outcome::Strikeout
                        );
                        if let Some(facts) = &t.event().caught_strikeout {
                            assert_eq!(
                                facts.batter_id,
                                before.teams[offense].batting_order
                                    [before.teams[offense].next_batter as usize]
                            );
                            assert_eq!(
                                facts.catcher_id,
                                before.teams[1 - offense].batting_order[0]
                            );
                            assert_eq!(
                                t.caught_strikeout_fielding().unwrap().putouts[&facts.catcher_id],
                                1
                            );
                            assert!(t.caught_strikeout_fielding().unwrap().assists.is_empty());
                        }
                        let protected = matches!(outcome, Outcome::Walk | Outcome::Hbp);
                        assert_eq!(t.event().protected_award.is_some(), protected);
                        assert_eq!(t.protected_award_rbi().is_some(), protected);
                        if let Some(facts) = &t.event().protected_award {
                            assert_eq!(
                                facts.batter_id,
                                before.teams[offense].batting_order
                                    [before.teams[offense].next_batter as usize]
                            );
                            assert_eq!(
                                t.protected_award_rbi().unwrap().runs_batted_in[&facts.batter_id],
                                u32::from(mask == 7)
                            );
                        }
                        let home_run = outcome == Outcome::HomeRun;
                        assert_eq!(t.event().sole_home_run.is_some(), home_run);
                        assert_eq!(t.sole_home_run_rbi().is_some(), home_run);
                        let batter = t.event().participants.batter.as_ref().unwrap();
                        if let Some(facts) = &t.event().sole_home_run {
                            assert_eq!(&facts.batter_id, batter);
                            assert_eq!(
                                t.sole_home_run_rbi().unwrap().runs_batted_in[batter],
                                1 + (mask as u32).count_ones()
                            );
                        }
                        let safe_hit =
                            matches!(outcome, Outcome::Single | Outcome::Double | Outcome::Triple);
                        assert_eq!(t.event().sole_safe_hit.is_some(), safe_hit);
                        assert_eq!(t.sole_safe_hit_rbi().is_some(), safe_hit);
                        if let Some(facts) = &t.event().sole_safe_hit {
                            assert_eq!(&facts.batter_id, batter);
                            let named_scorers = t
                                .event()
                                .delta
                                .moves
                                .iter()
                                .filter(|m| m.from != Base::Plate && m.to == Destination::Home)
                                .count() as u32;
                            assert_eq!(
                                t.sole_safe_hit_rbi().unwrap().runs_batted_in[batter],
                                named_scorers
                            );
                        }
                        let rbi_known =
                            protected || home_run || safe_hit || t.event().delta.runs == 0;
                        assert_eq!(t.rbi_contribution().is_some(), rbi_known);
                        let expected_rbi = rbi_known.then_some(if home_run {
                            1 + (mask as u32).count_ones()
                        } else if safe_hit {
                            t.event()
                                .delta
                                .moves
                                .iter()
                                .filter(|m| m.from != Base::Plate && m.to == Destination::Home)
                                .count() as u32
                        } else {
                            u32::from(protected && mask == 7)
                        });
                        assert_eq!(
                            t.rbi_contribution().map(|c| c.runs_batted_in[batter]),
                            expected_rbi
                        );
                        assert_eq!(
                            derive_accounting(std::slice::from_ref(&t)).unwrap().batters[offense]
                                [batter]
                                .classified_rbi,
                            expected_rbi
                        );
                        // Literal controls for the reproduced winning-order failure and
                        // the loaded walk-off; legality alone does not freeze v1 policy.
                        if half == Half::Bottom && outs == 0 && outcome == Outcome::Triple {
                            use bbsim_engine::{CreditKind, Destination};
                            let targets = t
                                .event()
                                .delta
                                .moves
                                .iter()
                                .map(|m| m.to)
                                .collect::<Vec<_>>();
                            if context == 2 && mask == 3 {
                                assert_eq!(
                                    targets,
                                    [Destination::Home, Destination::Home, Destination::Second]
                                );
                                assert_eq!(t.event().delta.runs, 2);
                                assert!(t
                                    .event()
                                    .delta
                                    .credits
                                    .iter()
                                    .any(|c| c.kind == CreditKind::Double));
                                assert!(!t
                                    .event()
                                    .delta
                                    .credits
                                    .iter()
                                    .any(|c| c.kind == CreditKind::Triple));
                            }
                            if context == 1 && mask == 7 {
                                assert_eq!(
                                    targets,
                                    [
                                        Destination::Second,
                                        Destination::Third,
                                        Destination::Home,
                                        Destination::First
                                    ]
                                );
                                assert_eq!(t.event().delta.runs, 1);
                                assert!(t
                                    .event()
                                    .delta
                                    .credits
                                    .iter()
                                    .any(|c| c.kind == CreditKind::Single));
                            }
                        }
                        assert_eq!(
                            t.event()
                                .delta
                                .credits
                                .iter()
                                .filter(|c| c.kind == bbsim_engine::CreditKind::PlateAppearance)
                                .count(),
                            1
                        );
                        assert_eq!(
                            t.event()
                                .delta
                                .credits
                                .iter()
                                .filter(|c| c.kind == bbsim_engine::CreditKind::NoHit)
                                .count(),
                            usize::from(matches!(outcome, Outcome::Roe | Outcome::ContactOut))
                        );
                        if t.after().over
                            && half == Half::Bottom
                            && t.event().delta.runs > 0
                            && outcome != Outcome::HomeRun
                        {
                            assert_eq!(t.after().score[1], t.after().score[0] + 1);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn two_thousand_games_are_repeatable_legal_and_conserve_available_statistics() {
    let baseline = Baseline::read_csv(CSV).unwrap();
    let first = preview::game(1, &baseline).unwrap();
    assert_eq!(first, preview::game(1, &baseline).unwrap());
    assert_ne!(first, preview::game(2, &baseline).unwrap());
    for seed in 0..2000 {
        let log = preview::game(seed, &baseline).unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        assert!(log.len() < 10_000, "seed {seed}");
        assert!(log.last().unwrap().after().over, "seed {seed}");
        for pair in log.windows(2) {
            pair[1].follows(&pair[0]).unwrap();
        }
        for t in &log {
            assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
        }
        let a = derive_accounting(&log).unwrap();
        assert!(a.complete_game);
        assert_eq!(a.runs.map(|v| v as u16), log.last().unwrap().after().score);
        for side in 0..2 {
            assert_eq!(
                a.batters[side]
                    .values()
                    .map(|b| b.plate_appearances)
                    .sum::<u32>(),
                a.pitchers[1 - side]
                    .values()
                    .map(|p| p.batters_faced)
                    .sum::<u32>()
            );
            assert_eq!(
                a.batters[side]
                    .values()
                    .map(|b| b.credited_runs)
                    .sum::<u32>(),
                a.runs[side]
            );
            assert_eq!(
                a.pitchers[1 - side]
                    .values()
                    .map(|p| p.runs_allowed)
                    .sum::<u32>(),
                a.runs[side]
            );
            assert_eq!(
                a.pitchers[1 - side]
                    .values()
                    .map(|p| p.outs_recorded)
                    .sum::<u32>(),
                a.fielders[1 - side]
                    .values()
                    .map(|f| f.credited_putouts)
                    .sum::<u32>()
            );
            for p in a.pitchers[1 - side].values() {
                assert_eq!(
                    p.classified_hits_allowed,
                    Some(p.credited_hits_allowed),
                    "seed {seed}"
                );
            }
            for b in a.batters[side].values() {
                assert_eq!(b.classified_hits, Some(b.credited_hits), "seed {seed}");
                assert_eq!(
                    b.credited_hits,
                    b.credited_hit_breakdown.as_ref().unwrap().hits()
                );
            }
        }
    }
}
