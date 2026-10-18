#[path = "../examples/support/diagnostics.rs"]
mod diagnostics;
#[path = "../examples/support/game_trace.rs"]
mod game_trace;
#[path = "../examples/support/presentation.rs"]
mod presentation;
#[path = "../examples/support/preview.rs"]
mod preview;
#[path = "../examples/support/preview_cli.rs"]
mod preview_cli;
use bbsim_engine::{
    accounting::{derive_accounting, FieldingCoverageLine},
    apply, Base, Destination, EventTransition, GameState, Half, TeamState,
};
use preview::{Baseline, Outcome, Rng, ALL};

fn initial() -> GameState {
    let team = |name: &str| TeamState {
        defense: bbsim_engine::DefenseState::LegacyUnassigned,
        batting_order: std::array::from_fn(|i| format!("{name}-{}", i + 1)),
        next_batter: 0,
        pitcher: (format!("{name}-pitcher")).into(),
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
fn trace_flag_is_opt_in_and_repeated_trace_is_rejected() {
    let args = |values: &[&str]| values.iter().map(|v| v.to_string()).collect::<Vec<_>>();
    assert!(preview_cli::parse(&args(&[
        "--baseline",
        "counts.csv",
        "--autoplay",
        "--trace"
    ]))
    .is_ok());
    assert!(
        preview_cli::parse(&args(&["--baseline", "counts.csv", "--trace", "--trace"])).is_err()
    );
}

#[test]
fn native_trace_preserves_every_full_before_event_after_including_boundaries() {
    let baseline = Baseline::read_csv(CSV).unwrap();
    let log = preview::game(1, &baseline).unwrap();
    assert!(log
        .iter()
        .any(|t| t.before().half != t.after().half && t.after().outs == 0));
    assert!(log.last().unwrap().after().over);
    let mut actual = Vec::new();
    game_trace::write(&log, &mut actual).unwrap();
    assert!(
        !actual.is_empty(),
        "Native trace must contain every transition"
    );
    let state_summary = |s: &GameState| {
        let bases = s.bases.each_ref().map(|r| {
            r.as_ref().map_or("empty".into(), |r| {
                format!("{} (charged to {})", r.player_id, r.responsible_pitcher)
            })
        });
        format!(
            "{:?} {} | Outs: {} | Away {} - Home {} | Bases: 1={} 2={} 3={} | Game over: {}",
            s.half, s.inning, s.outs, s.score[0], s.score[1], bases[0], bases[1], bases[2], s.over
        )
    };
    let mut expected = String::from("TRACE FORMAT: engine-debug-v1\n");
    for (i, t) in log.iter().enumerate() {
        expected.push_str(&format!(
            "TRACE PLAY {} BEGIN\nBefore state: {}\nAfter state: {}\nBefore:\n{:#?}\nEvent:\n{:#?}\nAfter:\n{:#?}\nTRACE PLAY {} END\n",
            i + 1, state_summary(t.before()), state_summary(t.after()), t.before(), t.event(), t.after(), i + 1
        ));
    }
    assert_eq!(String::from_utf8(actual).unwrap(), expected);
    let mut plain = Vec::new();
    presentation::present(&log, &mut &b""[..], &mut plain, true).unwrap();
    assert!(String::from_utf8(plain)
        .unwrap()
        .contains("FINAL: Away 3 - Home 11"));
}

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
            autoplay: true,
            trace: false
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
fn generated_terminal_shows_contact_outs_seeded_game_and_known_or_unknown_er() {
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
    // The unchanged seed's bottom-four clean grand slam precedes an empty-base
    // muff and three ground outs. The stranded error runner cannot unearn those
    // four runs, or Home-2's earlier clean scoring in bottom three.
    assert_eq!(log.len(), 75);
    assert_eq!(log.last().unwrap().after().score, [3, 11]);
    let stats = derive_accounting(&log).unwrap();
    assert_eq!(stats.pitchers[0]["Away-pitcher"].earned_runs, Some(11));
    assert_eq!(stats.pitchers[1]["Home-pitcher"].earned_runs, Some(3));
    let pitcher_row = |rendered: &str, name: &str| {
        rendered
            .lines()
            .find(|line| line.starts_with(name))
            .unwrap()
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        pitcher_row(&text, "Away-pitcher"),
        ["Away-pitcher", "40", "8.0", "11", "12", "1", "7", "11"]
    );
    assert_eq!(
        pitcher_row(&text, "Home-pitcher"),
        ["Home-pitcher", "35", "9.0", "3", "6", "1", "8", "3"]
    );
    assert!(!text.contains('?'));
    assert_eq!(text.matches("Play ").count(), log.len());

    // Missing review, not a different seed or invented physical play, still
    // makes ER unknown. Preserve the historical unknown-marker assertion here.
    let mut partial = log.clone();
    assert_eq!(log[36].before().inning, 4);
    assert_eq!(log[36].before().half, Half::Bottom);
    assert_eq!(log[36].before().outs, 0);
    assert!(log[36].before().bases.iter().all(Option::is_none));
    assert!(log[36].event().muffed_ground_ball.is_some());
    assert_eq!(
        log[36].event().fielding_review,
        bbsim_engine::FieldingReview::SoleGroundMuff
    );
    let mut missing_review = log[36].event().clone();
    missing_review.fielding_review = bbsim_engine::FieldingReview::Unknown;
    partial[36] = apply(log[36].before(), missing_review).unwrap();
    assert_eq!(
        EventTransition::from_bytes(&partial[36].to_bytes()).unwrap(),
        partial[36]
    );
    for (known, unknown) in log.iter().zip(&partial) {
        assert_eq!(known.before(), unknown.before());
        assert_eq!(known.after(), unknown.after());
        assert_eq!(known.event().delta, unknown.event().delta);
    }
    let partial_stats = derive_accounting(&partial).unwrap();
    assert_eq!(partial_stats.pitchers[0]["Away-pitcher"].earned_runs, None);
    assert_eq!(partial_stats.pitchers[1], stats.pitchers[1]);
    assert_eq!(partial_stats.runs, stats.runs);
    let mut output = Vec::new();
    presentation::present(&partial, &mut &b""[..], &mut output, true).unwrap();
    let text = String::from_utf8(output).unwrap();
    assert!(text.contains('?'));
    assert!(text.contains("ground/contact out"));
    assert!(text.contains("GAME OVER"));
    assert!(text.contains("ER"));
    assert_eq!(text.matches("Play ").count(), partial.len());
    assert_eq!(
        pitcher_row(&text, "Away-pitcher"),
        ["Away-pitcher", "40", "8.0", "11", "12", "1", "7", "?"]
    );
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
            let mut unbound = derive_accounting(&[old]).unwrap();
            let bound = derive_accounting(&[t]).unwrap();
            let actors: Vec<_> = (1..=8)
                .map(|i| format!("Home-{i}"))
                .chain(["Home-pitcher".into()])
                .collect();
            let known = actors
                .iter()
                .map(|id| {
                    (
                        id.clone(),
                        FieldingCoverageLine {
                            observed_events: 1,
                            putouts: Some(u32::from(id == "Home-2")),
                            assists: Some(u32::from(id == "Home-4")),
                            errors: Some(0),
                            passed_balls: Some(0),
                        },
                    )
                })
                .collect();
            let unknown = actors
                .into_iter()
                .map(|id| {
                    (
                        id,
                        FieldingCoverageLine {
                            observed_events: 1,
                            putouts: None,
                            assists: None,
                            errors: Some(0),
                            passed_balls: Some(0),
                        },
                    )
                })
                .collect();
            assert_eq!(bound.classified_fielding, [Default::default(), known]);
            assert_eq!(unbound.classified_fielding, [Default::default(), unknown]);
            // Assert the new evidence-sensitive component above, then retain
            // whole-object equality for every pre-existing accounting field.
            unbound.classified_fielding = bound.classified_fielding.clone();
            assert_eq!(unbound, bound);
        } else {
            assert!(event.ordinary_ground_throw.is_none());
            assert_eq!(t.ordinary_ground_fielding(), None);
        }
    }
}

#[test]
fn only_chosen_roe_binds_muff_review_without_changing_physics_or_other_coverage() {
    use bbsim_engine::{accounting::FieldingContribution, FieldingReview, Runner};
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            for mask in 0u8..8 {
                let mut before = initial();
                before.half = half;
                before.outs = outs;
                let offense = half.offense();
                let side = 1 - offense;
                before.bases = std::array::from_fn(|i| {
                    (mask & (1 << i) != 0).then(|| Runner {
                        player_id: before.teams[offense].batting_order[i + 5].clone(),
                        responsible_pitcher: before.teams[side]
                            .pitcher
                            .active_id()
                            .expect("legacy fixture pitcher")
                            .to_owned(),
                    })
                });
                for outcome in ALL {
                    let e = preview::event_for(&before, outcome).unwrap();
                    let t = apply(&before, e.clone()).unwrap();
                    if outcome != Outcome::Roe {
                        assert!(e.muffed_ground_ball.is_none());
                        assert_ne!(e.fielding_review, FieldingReview::SoleGroundMuff);
                        assert_eq!(t.muffed_ground_fielding(), None);
                        assert_eq!(t.muffed_ground_misplays(), None);
                        continue;
                    }
                    let facts = e
                        .muffed_ground_ball
                        .as_ref()
                        .expect("chosen mechanical muff");
                    let fielder = before.teams[side].batting_order[3].clone();
                    assert_eq!(facts.batter_id, before.teams[offense].batting_order[0]);
                    assert_eq!(facts.fielder_id, fielder);
                    assert_eq!(e.fielding_review, FieldingReview::SoleGroundMuff);
                    assert_eq!(
                        t.muffed_ground_fielding(),
                        Some(FieldingContribution {
                            putouts: Default::default(),
                            assists: Default::default(),
                        })
                    );
                    assert_eq!(t.reviewed_misplay_zeros(), None);
                    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
                    let mut absent = e.clone();
                    absent.muffed_ground_ball = None;
                    assert_eq!(
                        apply(&before, absent.clone()),
                        Err(bbsim_engine::TransitionError::UnsupportedRule)
                    );
                    absent.fielding_review = FieldingReview::Unknown;
                    let old = apply(&before, absent).unwrap();
                    assert_eq!(old.before(), t.before());
                    assert_eq!(old.after(), t.after());
                    assert_eq!(old.event().delta, t.event().delta);
                    let bound = derive_accounting(&[t]).unwrap();
                    let mut partial = derive_accounting(&[old]).unwrap();
                    assert_eq!(bound.classified_fielding[side].len(), 9);
                    for (id, line) in &bound.classified_fielding[side] {
                        assert_eq!(
                            line,
                            &FieldingCoverageLine {
                                observed_events: 1,
                                putouts: Some(0),
                                assists: Some(0),
                                errors: Some(u32::from(id == &fielder)),
                                passed_balls: Some(0),
                            }
                        );
                        assert_eq!(
                            partial.classified_fielding[side][id],
                            FieldingCoverageLine {
                                observed_events: 1,
                                putouts: None,
                                assists: None,
                                errors: None,
                                passed_balls: None,
                            }
                        );
                    }
                    let batter = &facts.batter_id;
                    assert_eq!(
                        bound.batters[offense][batter].classified_rbi,
                        if mask & 4 == 0 || outs == 2 {
                            Some(0)
                        } else {
                            None
                        }
                    );
                    assert_eq!(
                        partial.batters[offense][batter].classified_rbi,
                        if mask & 4 == 0 { Some(0) } else { None }
                    );
                    // Normalize only independently asserted two-out causal RBI
                    // and fielding evidence; every other statistic stays exact.
                    if outs == 2 && mask & 4 != 0 {
                        partial.batters[offense]
                            .get_mut(batter)
                            .unwrap()
                            .classified_rbi = Some(0);
                    }
                    partial.classified_fielding = bound.classified_fielding.clone();
                    assert_eq!(partial, bound);
                }
            }
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
                                    responsible_pitcher: before.teams[1 - offense]
                                        .pitcher
                                        .active_id()
                                        .expect("legacy fixture pitcher")
                                        .to_owned(),
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
                        let rbi_known = protected
                            || home_run
                            || safe_hit
                            || t.event().delta.runs == 0
                            || (outcome == Outcome::Roe && outs == 2);
                        assert_eq!(
                            t.rbi_contribution().is_some(),
                            rbi_known,
                            "RBI cause {half:?} outs{outs} mask{mask} {outcome:?}"
                        );
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

fn radio(t: &EventTransition) -> String {
    let mut output = Vec::new();
    game_trace::write_commentary(std::slice::from_ref(t), &mut output).unwrap();
    String::from_utf8(output).unwrap()
}

#[test]
fn radio_calls_name_actual_batter_pitcher_and_bound_fielders_for_all_categories() {
    let calls = [
        "Home-pitcher to Away-1: strike three! Away-1 is out.",
        "Home-pitcher to Away-1: ball four! Away-1 takes first.",
        "Home-pitcher to Away-1: hit by pitch. Away-1 takes first.",
        "Home-pitcher to Away-1: a single!",
        "Home-pitcher to Away-1: a double!",
        "Home-pitcher to Away-1: a triple!",
        "Home-pitcher to Away-1: home run!",
        "Home-pitcher to Away-1: ground ball, mishandled by Home-4! Away-1 reaches first on the error.",
        "Home-pitcher to Away-1: ground ball to Home-4, the throw to Home-2 — Away-1 is out at first!",
    ];
    for (outcome, expected) in ALL.into_iter().zip(calls) {
        let before = initial();
        let t = apply(&before, preview::event_for(&before, outcome).unwrap()).unwrap();
        let text = radio(&t);
        assert!(
            text.contains(&format!("CALL PLAY 1: {expected}\n")),
            "{outcome:?}: {text}"
        );
        assert!(text.contains(
            "RECAP PLAY 1: Before: Top 1 · 0 outs · Away 0–Home 0 · bases empty → After:"
        ));
        if matches!(
            outcome,
            Outcome::Single
                | Outcome::Double
                | Outcome::Triple
                | Outcome::HomeRun
                | Outcome::Walk
                | Outcome::Hbp
        ) {
            assert!(!text.contains("Home-4") && !text.contains("Home-2"));
        }
        assert!(!text.contains("swinging") && !text.contains("left field"));
    }
}

#[test]
fn radio_covers_all_before_runners_moving_scoring_or_holding_without_changing_transition() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            for mask in 0u8..8 {
                for outcome in ALL {
                    let mut before = initial();
                    before.half = half;
                    before.outs = outs;
                    for i in 0..3 {
                        if mask & (1 << i) != 0 {
                            before.bases[i] = Some(bbsim_engine::Runner {
                                player_id: before.teams[half.offense()].batting_order[i + 5]
                                    .clone(),
                                responsible_pitcher: before.teams[1 - half.offense()]
                                    .pitcher
                                    .active_id()
                                    .expect("legacy fixture pitcher")
                                    .to_owned(),
                            });
                        }
                    }
                    let t = apply(&before, preview::event_for(&before, outcome).unwrap()).unwrap();
                    let frozen = t.to_bytes();
                    let text = radio(&t);
                    let call = text
                        .lines()
                        .find(|l| l.starts_with("CALL PLAY 1:"))
                        .unwrap();
                    for r in before.bases.iter().flatten() {
                        assert!(
                            call.contains(&r.player_id),
                            "{outcome:?}/{mask}/{outs}/{half:?}: {call}"
                        );
                    }
                    assert_eq!(t.to_bytes(), frozen);
                }
            }
        }
    }
}

#[test]
fn radio_explains_held_runners_normalized_third_out_and_terminal_state() {
    let mut before = initial();
    before.outs = 2;
    before.bases[0] = Some(bbsim_engine::Runner {
        player_id: "Away-6".into(),
        responsible_pitcher: "Home-pitcher".into(),
    });
    let t = apply(
        &before,
        preview::event_for(&before, Outcome::ContactOut).unwrap(),
    )
    .unwrap();
    let text = radio(&t);
    assert!(
        text.contains("Away-6 is stranded at first. Side retired."),
        "{text}"
    );
    assert!(text.contains("After: Bottom 1 · 0 outs · Away 0–Home 0 · bases empty"));
    before = initial();
    before.inning = 9;
    before.half = Half::Bottom;
    let t = apply(
        &before,
        preview::event_for(&before, Outcome::HomeRun).unwrap(),
    )
    .unwrap();
    let text = radio(&t);
    assert!(text.contains("The game is over."));
    assert!(text.contains("After: Final · Away 0–Home 1 · bases empty"));
}

#[test]
fn inning_ending_strikeout_strands_runner_without_catcher_bookkeeping_or_display_ids() {
    let mut before = initial();
    before.outs = 2;
    before.bases[0] = Some(bbsim_engine::Runner {
        player_id: "Away-6".into(),
        responsible_pitcher: "Home-pitcher".into(),
    });
    let t = apply(
        &before,
        preview::event_for(&before, Outcome::Strikeout).unwrap(),
    )
    .unwrap();
    let frozen = t.to_bytes();
    let names = [
        ("Home-pitcher", "Michael Fiers"),
        ("Away-1", "Ryon Healy"),
        ("Home-1", "Nick Hundley"),
        ("Away-6", "Domingo Santana"),
    ]
    .into_iter()
    .map(|(id, name)| (id.into(), name.into()))
    .collect();
    let mut output = Vec::new();
    game_trace::write_commentary_named(std::slice::from_ref(&t), &mut output, &names).unwrap();
    let text = String::from_utf8(output).unwrap();
    assert_eq!(text.lines().nth(1).unwrap(), "CALL PLAY 1: Michael Fiers to Ryon Healy: strike three! Ryon Healy is out. Domingo Santana is stranded at first. Side retired.");
    assert!(!text.contains("receives the strike") && !text.contains("holds at"));
    assert!(!text.contains('[') && !text.contains("Home-pitcher") && !text.contains("Away-6"));
    assert_eq!(t.to_bytes(), frozen);
    assert!(t.event().caught_strikeout.is_some());
}

#[test]
fn radio_only_calls_credited_runs_and_does_not_invent_missing_fielding_facts() {
    let mut before = initial();
    before.bases[2] = Some(bbsim_engine::Runner {
        player_id: "Away-6".into(),
        responsible_pitcher: "Home-pitcher".into(),
    });
    let t = apply(
        &before,
        preview::event_for(&before, Outcome::Single).unwrap(),
    )
    .unwrap();
    assert!(radio(&t).contains("Away-6 scores from third."));
    let mut event = preview::event_for(&before, Outcome::ContactOut).unwrap();
    event.ordinary_ground_throw = None;
    let t = apply(&before, event).unwrap();
    let text = radio(&t);
    assert!(!text.contains("the throw") && !text.contains("ground ball to"));
    assert!(text.contains("Home-4 (assist)") && text.contains("Home-2 (putout)"));
    // A physical home crossing on a before-first third out is not a credited run.
    before.outs = 2;
    let mut event = preview::event_for(&before, Outcome::ContactOut).unwrap();
    event.ordinary_ground_throw = None;
    event.delta.moves.push(bbsim_engine::RunnerMove {
        runner: before.bases[2].clone().unwrap(),
        from: Base::Third,
        to: Destination::Home,
    });
    let t = apply(&before, event).unwrap();
    let text = radio(&t);
    assert!(text.contains("Away-6 crosses home from third; run not counted."));
    assert!(!text.contains("scores from third"));
}
