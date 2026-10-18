//! Synthetic abilities on a pinned cast; no source player counts in this fixture.
#[path = "../examples/support/calibrated_ratings.rs"]
mod calibrated;
#[allow(dead_code)]
#[path = "../examples/support/player_calibration.rs"]
mod calibration;
#[allow(dead_code)]
#[path = "../examples/support/full_game.rs"]
mod full_game;
#[allow(dead_code)]
#[path = "../examples/support/full_manager.rs"]
mod full_manager;
#[allow(dead_code)]
#[path = "../examples/support/full_simulation.rs"]
mod full_simulation;
#[allow(dead_code)]
#[path = "../examples/support/full_transcript.rs"]
mod full_transcript;
#[allow(dead_code)]
#[path = "../examples/support/joint_contact.rs"]
mod joint;
#[allow(dead_code)]
#[path = "../examples/support/preview.rs"]
mod preview;
#[allow(dead_code)]
#[path = "../examples/support/player_ratings.rs"]
mod ratings;
#[allow(dead_code)]
#[path = "../examples/support/real_lineup.rs"]
mod real;
#[allow(dead_code)]
#[path = "../examples/support/transcript.rs"]
mod transcript;
#[allow(dead_code)]
#[path = "../examples/support/player_calibration_tree.rs"]
mod tree;
use bbsim_engine::{CreditKind, GameState};
use std::collections::{BTreeMap, BTreeSet};
const COUNTS: [u64; 9] = [42823, 15142, 1984, 25947, 8531, 785, 6776, 1470, 82245];
fn lineup() -> real::Lineup {
    let mut text = format!("key\tvalue\nformat\tclosed-starting-lineup-v1\ngame_id\tOAK201903200\ndate\t2019-03-20\npark_id\tTOK01\naway_team\tSeattle Mariners\nhome_team\tOakland Athletics\naudit_sha256\t{}\nsnapshot_sha256\t{}\nsupplement_sha256\t{}\nattribution\t{}\n",real::REPORT_SHA,real::SNAPSHOT_SHA,real::SUPPLEMENT_SHA,joint::ATTRIBUTION);
    for (side, prefix, pitcher, name) in [
        ("Away", "a", "gonzm005", "Marco Gonzales"),
        ("Home", "h", "fierm001", "Michael Fiers"),
    ] {
        text.push_str(&format!("{side}:0:1:{pitcher}\t{name}\n"));
        for (i, pos) in [7, 8, 3, 10, 9, 2, 5, 6, 4].into_iter().enumerate() {
            text.push_str(&format!(
                "{side}:{}:{pos}:{prefix}{}\tSynthetic {prefix}{}\n",
                i + 1,
                i + 1,
                i + 1
            ));
        }
    }
    real::Lineup::read_tsv(&text).unwrap()
}
fn cast_text() -> String {
    let mut fields: BTreeMap<String,String> = [
        ("attribution",joint::ATTRIBUTION),
        ("availability_basis","recorded nonstarters assumed available at start; not verified eligibility"),
        ("bullpen_report_sha256","758186cceb67815573e7e109763f7be7d5e57b9ec9583bc8a3cca299d39edc88"),
        ("exhaustion","last_pitcher_stays"),("format","closed-pitching-cast-v1"),
        ("league_counts","42823,15142,1984,25947,8531,785,6776,1470,82245"),
        ("lineup_report_sha256",real::REPORT_SHA),("model",ratings::MODEL_VERSION),
        ("opportunity_basis",ratings::BASIS),("opportunity_report_sha256",ratings::REPORT_SHA),
        ("policy","pa-mean-closed-cast-v1"),("snapshot_sha256",real::SNAPSHOT_SHA),
        ("supplement_sha256",real::SUPPLEMENT_SHA),
        ("workload_basis","literal terminal-PA participants per pitcher-game appearance; not pitch counts, official BF or fatigue"),
        ("workload_report_sha256","125768bd82ce54ead277d92301fbb7434e823a8fc4432777087c15b8e46ad2f9"),
        ("batter_kappa","200"),("pitcher_kappa","200")
    ].into_iter().map(|(k,v)|(k.into(),v.into())).collect();
    for (side, players) in [
        (
            "Away",
            &[
                ("gonzm005", "Marco Gonzales"),
                ("strih001", "Hunter Strickland"),
                ("gearc001", "Cory Gearrin"),
                ("rossz001", "Zac Rosscup"),
                ("rumbn001", "Nick Rumbelow"),
            ][..],
        ),
        (
            "Home",
            &[
                ("fierm001", "Michael Fiers"),
                ("dullr001", "Ryan Dull"),
                ("wendj001", "J.B. Wendelken"),
                ("hendl001", "Liam Hendriks"),
                ("trivl001", "Lou Trivino"),
                ("rodnf001", "Fernando Rodney"),
                ("buchr001", "Ryan Buchter"),
            ][..],
        ),
    ] {
        for (rank, &(id, name)) in players.iter().enumerate() {
            fields.insert(format!("cast:{side}:{rank}:{id}"), name.into());
            fields.insert(format!("pitcher:{id}"), "0,0,0,0,0,0,0,0,0".into());
            for role in ["start", "relief"] {
                fields.insert(format!("workload:{role}:{id}"), "0,1".into());
            }
        }
    }
    for prefix in ["a", "h"] {
        for i in 1..=9 {
            fields.insert(format!("batter:{prefix}{i}"), "0,0,0,0,0,0,0,0,0".into());
        }
    }
    "key\tvalue\n".to_owned()
        + &fields
            .into_iter()
            .map(|(k, v)| format!("{k}\t{v}\n"))
            .collect::<String>()
}
fn required(l: &real::Lineup, c: &full_manager::Cast) -> BTreeSet<String> {
    l.initial()
        .teams
        .iter()
        .flat_map(|t| {
            t.batting_order
                .iter()
                .map(|id| format!("batter:{id}"))
                .collect::<Vec<_>>()
        })
        .chain(c.names().keys().map(|id| format!("pitcher:{id}")))
        .collect()
}
fn model() -> full_simulation::Model {
    full_simulation::Model::new(
        [42823, 15142, 1984, 25947, 8531, 785, 6776, 1470, 82245, 61],
        [17188, 19528, 5379, 536, 39614],
    )
    .unwrap()
}
fn defense(l: &real::Lineup) -> [[String; 9]; 2] {
    let positions = l.positions();
    l.initial().teams.each_ref().map(|t| {
        std::array::from_fn(|i| {
            t.batting_order
                .iter()
                .map(String::as_str)
                .chain(t.pitcher.active_id())
                .find(|id| {
                    positions.get(*id)
                        == Some(&["P", "C", "1B", "2B", "3B", "SS", "LF", "CF", "RF"][i])
                })
                .unwrap()
                .to_owned()
        })
    })
}
fn calibrated_text(c: &full_manager::Cast, b: &preview::Baseline) -> String {
    calibrated_counts_text(c.ability_counts(), b)
}
fn calibrated_counts_text(abilities: &BTreeMap<String, [u64; 9]>, b: &preview::Baseline) -> String {
    let rates = tree::league_nodes(*b.probabilities()).unwrap();
    let mut text=format!("key\tvalue\nformat\tclosed-opponent-tree-ratings-v2\nmodel\t{}\nopportunity_report_sha256\t{}\nsnapshot_sha256\t{}\nsupplement_sha256\t{}\ncast_sha256\t{}\nreused_test_benchmark\ttrue\nstrengths\t{}\nattribution\t{}\naudit_sha256\t{}\nselection_sha256\t{}\nleague_nodes\t{}\n",calibrated::MODEL_VERSION,ratings::REPORT_SHA,real::SNAPSHOT_SHA,real::SUPPLEMENT_SHA,calibrated::CAST_SHA,calibrated::STRENGTHS,joint::ATTRIBUTION,"a".repeat(64),"b".repeat(64),rates.iter().map(f64::to_string).collect::<Vec<_>>().join(","));
    for (key, counts) in abilities {
        let actor_rates = if ["batter:aph", "batter:hph"].contains(&key.as_str()) {
            [0.01, 0.01, 0.01, 0.99, 0.99, 0.01, 0.01, 0.01]
        } else {
            rates
        };
        text += &format!(
            "counts:{key}\t{}\n{key}\t{}\n",
            counts
                .iter()
                .map(u64::to_string)
                .collect::<Vec<_>>()
                .join(","),
            actor_rates
                .iter()
                .map(f64::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );
    }
    text
}
#[test]
fn calibrated_tree_artifact_requires_exact_roles_counts_versions_rates_and_source_bindings() {
    let l = lineup();
    let c = full_manager::Cast::read_complete_tsv(&cast_text()).unwrap();
    let b = preview::Baseline::from_counts(COUNTS).unwrap();
    let wanted = required(&l, &c);
    let text = calibrated_text(&c, &b);
    let r = calibrated::Ratings::read_tsv(&text, c.ability_counts(), &b, &wanted).unwrap();
    let q = r.probabilities(&l.initial()).unwrap();
    for (a, z) in q.into_iter().zip(*b.probabilities()) {
        assert!((a - z).abs() < 1e-14);
    }
    for bad in [
        text.replace(
            "closed-opponent-tree-ratings-v2",
            "closed-opponent-tree-ratings-v9",
        ),
        text.replace(
            "reused_test_benchmark\ttrue",
            "reused_test_benchmark\tfalse",
        ),
        text.replace(calibrated::CAST_SHA, &"0".repeat(64)),
        text.replace("audit_sha256\t", "audit_sha256\tx"),
        text.lines()
            .filter(|row| !row.starts_with("pitcher:buchr001\t"))
            .collect::<Vec<_>>()
            .join("\n"),
        text.replace(
            "counts:batter:a1\t0,0,0,0,0,0,0,0,0",
            "counts:batter:a1\t1,0,0,0,0,0,0,0,0",
        ),
        text.clone() + "extra\t1\n",
        text.clone() + "model\tduplicate\n",
        text.replacen("league_nodes\t", "league_nodes\tnan,", 1),
    ] {
        assert!(calibrated::Ratings::read_tsv(&bad, c.ability_counts(), &b, &wanted).is_err());
    }
    for role in ["pitcher", "batter"] {
        let mut s = l.initial();
        if role == "pitcher" {
            s.teams[1].pitcher = "missing".into();
        } else {
            s.teams[0].batting_order[0] = "missing".into();
        }
        assert!(r.probabilities(&s).is_err());
    }
}
#[test]
fn calibrated_tree_consumes_one_parent_and_a_relief_profile_only_after_native_entry() {
    let l = lineup();
    let c = full_manager::Cast::read_complete_tsv(&cast_text()).unwrap();
    let b = preview::Baseline::from_counts(COUNTS).unwrap();
    let wanted = required(&l, &c);
    let m = model();
    let text = calibrated_text(&c, &b);
    let rates = tree::league_nodes(*b.probabilities()).unwrap();
    let neutral = rates
        .iter()
        .map(f64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let mut altered = rates;
    altered[0] = 1.0 - 1e-12;
    let altered = altered
        .iter()
        .map(f64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let normal = calibrated::Ratings::read_tsv(&text, c.ability_counts(), &b, &wanted).unwrap();
    let hitter = calibrated::Ratings::read_tsv(
        &text.replace(
            &format!("\nbatter:a1\t{neutral}\n"),
            &format!("\nbatter:a1\t{altered}\n"),
        ),
        c.ability_counts(),
        &b,
        &wanted,
    )
    .unwrap();
    let relief = calibrated::Ratings::read_tsv(
        &text.replace(
            &format!("\npitcher:dullr001\t{neutral}\n"),
            &format!("\npitcher:dullr001\t{altered}\n"),
        ),
        c.ability_counts(),
        &b,
        &wanted,
    )
    .unwrap();
    let run = |r: &calibrated::Ratings| {
        full_simulation::simulate_with_probabilities(
            &l.initial(),
            &defense(&l),
            &m,
            Some(&c),
            1,
            |s| Ok(Some(m.player_probabilities(r.probabilities(s)?)?)),
        )
        .unwrap()
    };
    let a = run(&normal);
    let h = run(&hitter);
    let z = run(&relief);
    assert_eq!(a.draws[0].1.word, h.draws[0].1.word);
    assert_eq!(h.draws[0].1.parent, 0);
    assert_ne!(a.draws[0].1.parent, h.draws[0].1.parent);
    let entry = a
        .steps
        .iter()
        .position(|s| s.transition().after().teams[1].pitcher.active_id() == Some("dullr001"))
        .unwrap();
    assert_eq!(a.steps[..=entry], z.steps[..=entry]);
    assert_ne!(a.steps, z.steps);
    assert_eq!(z.steps, run(&relief).steps);
    for game in [&a, &h, &z] {
        assert!(game.steps.last().unwrap().transition().after().over);
        full_simulation::verify_accounting(
            &game
                .steps
                .iter()
                .map(|s| s.transition().clone())
                .collect::<Vec<_>>(),
        )
        .unwrap();
        for (_, d) in &game.draws {
            assert_eq!(
                full_simulation::sample_pa(d.probabilities, d.word).unwrap(),
                d.parent
            );
        }
    }
}
#[test]
fn exact_cast_role_counts_are_consumed_and_missing_extra_or_unbounded_counts_refuse() {
    let l = lineup();
    let c = full_manager::Cast::read_tsv(&cast_text()).unwrap();
    let b = preview::Baseline::from_counts(COUNTS).unwrap();
    let expected = required(&l, &c);
    assert_eq!(c.ability_counts().len(), 30);
    let r = ratings::Ratings::from_counts(c.ability_counts().clone(), &b, &expected).unwrap();
    let p = r.probabilities(&l.initial(), *b.probabilities()).unwrap();
    for (x, y) in p.into_iter().zip(*b.probabilities()) {
        assert!((x - y).abs() < 1e-14);
    }
    for bad in ["missing", "extra", "above", "overflow"] {
        let mut counts = c.ability_counts().clone();
        match bad {
            "missing" => {
                counts.remove("pitcher:dullr001");
            }
            "extra" => {
                counts.insert("pitcher:unknown".into(), [0; 9]);
            }
            "above" => {
                counts.get_mut("batter:a1").unwrap()[6] = 6777;
            }
            _ => {
                counts.insert("batter:a1".into(), [u64::MAX; 9]);
            }
        }
        assert!(
            ratings::Ratings::from_counts(counts, &b, &expected).is_err(),
            "{bad}"
        );
    }
    let mut before = l.initial();
    before.teams[1].pitcher = "unknown".into();
    assert!(r.probabilities(&before, *b.probabilities()).is_err());
    before = l.initial();
    before.teams[0].batting_order[0] = "unknown".into();
    assert!(r.probabilities(&before, *b.probabilities()).is_err());
    before = l.initial();
    before.teams[0].next_batter = 9;
    assert!(r.probabilities(&before, *b.probabilities()).is_err());
}
#[test]
fn complete_cast_loader_refuses_a_terminal_member_removed_with_all_its_profiles() {
    let full = cast_text();
    let c = full_manager::Cast::read_complete_tsv(&full).unwrap();
    assert_eq!(c.names().len(), 12);
    for terminal in ["rumbn001", "buchr001"] {
        let shortened = full
            .lines()
            .filter(|row| !row.contains(terminal))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            full_manager::Cast::read_tsv(&shortened)
                .unwrap()
                .names()
                .len(),
            11
        );
        assert_eq!(
            full_manager::Cast::read_complete_tsv(&shortened).unwrap_err(),
            "complete text game requires all 12 pinned cast pitchers"
        );
    }
}
#[test]
fn ten_category_mass_and_fixed_word_cdf_boundaries_are_exact_without_second_draw() {
    let m = model();
    let b = preview::Baseline::from_counts(COUNTS).unwrap();
    let p = m.player_probabilities(*b.probabilities()).unwrap();
    assert!((p.iter().sum::<f64>() - 1.0).abs() < 1e-14);
    assert!((p[9] - 61.5 / 185769.0).abs() < 1e-16);
    for k in 0..9 {
        assert!((p[k] / (1.0 - p[9]) - b.probabilities()[k]).abs() < 1e-14);
    }
    let equal = [
        0.125, 0.125, 0.125, 0.125, 0.0625, 0.0625, 0.0625, 0.0625, 0.0625, 0.1875,
    ];
    let mut edge = 0.0;
    for k in 0..9 {
        edge += equal[k];
        let word = ((edge * (1u64 << 53) as f64) as u64) << 11;
        assert_eq!(full_simulation::sample_pa(equal, word - 2048).unwrap(), k);
        assert_eq!(full_simulation::sample_pa(equal, word).unwrap(), k + 1);
    }
    assert_eq!(full_simulation::sample_pa(equal, u64::MAX).unwrap(), 9);
    for bad in [f64::NAN, -1.0, 0.0, f64::INFINITY] {
        let mut q = p;
        q[0] = bad;
        assert!(full_simulation::sample_pa(q, 0).is_err());
    }
    let mut q = p;
    q[0] += 0.1;
    assert!(full_simulation::sample_pa(q, 0).is_err());
}
#[test]
fn reliever_only_rating_changes_only_after_native_entry_and_actual_draws_respond() {
    let l = lineup();
    let c = full_manager::Cast::read_tsv(&cast_text()).unwrap();
    let b = preview::Baseline::from_counts(COUNTS).unwrap();
    let m = model();
    let expected = required(&l, &c);
    let neutral = ratings::Ratings::from_counts(c.ability_counts().clone(), &b, &expected).unwrap();
    let mut changed = c.ability_counts().clone();
    changed.get_mut("pitcher:dullr001").unwrap()[0] = 40_000;
    let strong = ratings::Ratings::from_counts(changed, &b, &expected).unwrap();
    let run = |r: &ratings::Ratings| {
        full_simulation::simulate_with_probabilities(
            &l.initial(),
            &defense(&l),
            &m,
            Some(&c),
            1,
            |s| {
                Ok(Some(m.player_probabilities(
                    r.probabilities(s, *b.probabilities())?,
                )?))
            },
        )
        .unwrap()
        .steps
    };
    let a = run(&neutral);
    let z = run(&strong);
    let entry = a
        .iter()
        .position(|s| s.transition().after().teams[1].pitcher.active_id() == Some("dullr001"))
        .unwrap();
    assert_eq!(a[..=entry], z[..=entry]);
    assert_ne!(a, z);
    assert_eq!(run(&strong), z);
    for steps in [&a, &z] {
        assert!(steps.last().unwrap().transition().after().over);
        full_simulation::verify_accounting(
            &steps
                .iter()
                .map(|s| s.transition().clone())
                .collect::<Vec<_>>(),
        )
        .unwrap();
    }
    let mut state: GameState = a[entry].transition().after().clone();
    state.half = bbsim_engine::Half::Top;
    assert!(
        strong.probabilities(&state, *b.probabilities()).unwrap()[0]
            > neutral.probabilities(&state, *b.probabilities()).unwrap()[0] + 0.5
    );
    let mut audit = 0;
    let full_simulation::Simulation { steps, draws } =
        full_simulation::simulate_with_probabilities(
            &l.initial(),
            &defense(&l),
            &m,
            Some(&c),
            2,
            |s| {
                audit += 1;
                Ok(Some(m.player_probabilities(
                    neutral.probabilities(s, *b.probabilities())?,
                )?))
            },
        )
        .unwrap();
    assert_eq!(draws.len(), audit);
    assert_eq!(draws[0].1.word, preview::Rng::new(2).next_u64());
    for (index, draw) in draws {
        assert_eq!(
            full_simulation::sample_pa(draw.probabilities, draw.word).unwrap(),
            draw.parent
        );
        assert!(steps[index - 1]
            .transition()
            .event()
            .delta
            .credits
            .iter()
            .any(|c| c.kind == CreditKind::PlateAppearance));
    }
    let ordinary = steps
        .iter()
        .filter(|s| {
            let cs = &s.transition().event().delta.credits;
            cs.iter().any(|c| c.kind == CreditKind::PlateAppearance)
                && !cs.iter().any(|c| c.kind == CreditKind::IntentionalWalk)
        })
        .count();
    assert_eq!(audit,ordinary,"one vector consumed per ordinary PA, never a pitching change, tactical IBB or runner-only event");
}

#[test]
fn hitter_profile_changes_the_consumed_first_parent_and_missing_model_never_falls_back() {
    let l = lineup();
    let c = full_manager::Cast::read_tsv(&cast_text()).unwrap();
    let b = preview::Baseline::from_counts(COUNTS).unwrap();
    let m = model();
    let mut strong = c.ability_counts().clone();
    strong.get_mut("batter:a1").unwrap()[6] = 6000;
    let expected = required(&l, &c);
    let a = ratings::Ratings::from_counts(c.ability_counts().clone(), &b, &expected).unwrap();
    let z = ratings::Ratings::from_counts(strong, &b, &expected).unwrap();
    let run = |r: &ratings::Ratings| {
        full_simulation::simulate_with_probabilities(
            &l.initial(),
            &defense(&l),
            &m,
            Some(&c),
            1,
            |s| {
                Ok(Some(m.player_probabilities(
                    r.probabilities(s, *b.probabilities())?,
                )?))
            },
        )
        .unwrap()
    };
    let a = run(&a);
    let z = run(&z);
    assert_eq!(a.draws[0].1.word, z.draws[0].1.word);
    assert_eq!(z.draws[0].1.parent, 6);
    assert_ne!(a.draws[0].1.parent, z.draws[0].1.parent);
    assert_ne!(a.steps[0], z.steps[0]);
    assert!(z.steps[0]
        .transition()
        .event()
        .delta
        .credits
        .iter()
        .any(|c| c.kind == CreditKind::Hit));
    assert_eq!(
        z.steps[0].transition().event().kind,
        bbsim_engine::EventKind::OutOfParkHomeRun
    );
    let error = full_simulation::simulate_with_probabilities(
        &l.initial(),
        &defense(&l),
        &m,
        Some(&c),
        1,
        |_| Err("missing current pitcher rating".into()),
    )
    .err()
    .unwrap();
    assert!(error.contains("missing current pitcher rating"));
}

// Controlled synthetic abilities exercise management, not historical roster eligibility.
fn bench_text() -> String {
    let mut text = format!("key\tvalue\nformat\tassumed-game-bench-v1\ngame_id\tOAK201903200\npolicy\tlate-close-expected-bases-ph-v1\navailability_basis\tsame-game participant proxies assumed available at start; not verified eligibility\ndefense_basis\tsupplied observed positions assumed usable; not verified defensive eligibility\nsnapshot_sha256\t{}\nsupplement_sha256\t{}\nopportunity_report_sha256\t{}\nattribution\t{}\n", real::SNAPSHOT_SHA, real::SUPPLEMENT_SHA, ratings::REPORT_SHA, joint::ATTRIBUTION);
    for (side, id) in [("Away", "aph"), ("Home", "hph")] {
        text += &format!("{side}:{id}:name\tSynthetic {id}\n{side}:{id}:positions\t2,3,4,5,6,7,8,9\n{side}:{id}:counts\t0,0,0,0,0,0,4000,0,0\n");
    }
    text
}

#[test]
fn automatic_ph_reaches_actual_games_with_current_ratings_defense_and_named_output() {
    use bbsim_engine::{DefenseState, PersonnelChange, PitcherRole};
    let baseline = preview::Baseline::from_counts(COUNTS).unwrap();
    let cast = full_manager::Cast::read_complete_tsv(&cast_text().replace("0,1", "100,1")).unwrap();
    let bench = full_simulation::bench::Bench::read_tsv(&bench_text(), "OAK201903200").unwrap();
    let l = lineup();
    let model = model();
    let fielders = defense(&l);
    for dh in [false, true] {
        let mut initial = l.initial();
        if !dh {
            for t in &mut initial.teams {
                let slot = t.dh_slot.take().unwrap() as usize;
                t.dh_must_bat = false;
                t.batting_order[slot] = t.pitcher.active_id().unwrap().into();
            }
        }
        let mut counts = cast.ability_counts().clone();
        let mut required: BTreeSet<_> = initial
            .teams
            .iter()
            .flat_map(|t| t.batting_order.iter().map(|id| format!("batter:{id}")))
            .chain(cast.names().keys().map(|id| format!("pitcher:{id}")))
            .collect();
        if !dh {
            for id in cast.names().keys() {
                required.insert(format!("batter:{id}"));
                counts.insert(format!("batter:{id}"), [1000, 0, 0, 0, 0, 0, 0, 0, 0]);
            }
        }
        counts.retain(|id, _| required.contains(id));
        counts.extend(bench.counts.clone());
        required.extend(bench.counts.keys().cloned());
        let calibrated = calibrated::Ratings::read_tsv(
            &calibrated_counts_text(&counts, &baseline),
            &counts,
            &baseline,
            &required,
        )
        .unwrap();
        let ratings = ratings::Ratings::from_counts(counts, &baseline, &required).unwrap();
        for experimental in [false, true] {
            let probabilities = |s: &GameState| {
                if experimental {
                    calibrated.probabilities(s)
                } else {
                    ratings.probabilities(s, *baseline.probabilities())
                }
            };
            let mut ph_count = 0;
            let mut restored = 0;
            let mut ph_sides = [0; 2];
            let mut reaches = 0;
            let mut runs = 0;
            let mut again = 0;
            let mut names = l.names();
            names.extend(cast.names());
            names.extend(bench.names.clone());
            for seed in 1..=16 {
                let run = || {
                    full_simulation::simulate_with_personnel(
                        &initial,
                        &fielders,
                        &model,
                        Some(&cast),
                        Some(&bench),
                        seed,
                        |s| Ok(Some(model.player_probabilities(probabilities(s)?)?)),
                    )
                    .unwrap()
                };
                let a = run();
                let b = run();
                assert_eq!(a.steps, b.steps);
                assert_eq!(a.draws, b.draws);
                assert!(a.steps.last().unwrap().transition().after().over);
                let log: Vec<_> = a.steps.iter().map(|s| s.transition().clone()).collect();
                for pair in log.windows(2) {
                    pair[1].follows(&pair[0]).unwrap();
                }
                for (index, step) in a.steps.iter().enumerate() {
                    let t = step.transition();
                    for change in &t.event().delta.changes {
                        if let PersonnelChange::PinchHitter { incoming, .. } = change {
                            ph_count += 1;
                            ph_sides[t.before().half.offense()] += 1;
                            assert!(!a.draws.iter().any(|(n, _)| *n == index + 1));
                            assert!(log[index + 1..].iter().any(|p| p
                                .event()
                                .participants
                                .batter
                                .as_ref()
                                == Some(incoming)));
                        }
                        if matches!(change, PersonnelChange::AssignPitcher { .. }) {
                            restored += 1;
                            let side = 1 - t.before().half.offense();
                            assert!(matches!(
                                t.before().teams[side].pitcher,
                                PitcherRole::Vacant { .. }
                            ));
                            assert!(t.after().validate_play_ready().is_ok());
                        }
                    }
                    if t.event().kind != bbsim_engine::EventKind::Substitution {
                        t.before().validate_play_ready().unwrap();
                        assert!(matches!(
                            t.before().teams[1 - t.before().half.offense()].defense,
                            DefenseState::Assigned(_)
                        ));
                    }
                    let mut detailed = Vec::new();
                    let mut radio = Vec::new();
                    full_transcript::write_step(
                        &mut detailed,
                        index + 1,
                        step,
                        &["A".into(), "H".into()],
                        &names,
                    )
                    .unwrap();
                    full_transcript::write_radio_step(
                        &mut radio,
                        index + 1,
                        step,
                        &["A".into(), "H".into()],
                        &names,
                    )
                    .unwrap();
                    if t.event()
                        .delta
                        .changes
                        .iter()
                        .any(|c| matches!(c, PersonnelChange::PinchHitter { .. }))
                    {
                        assert!(String::from_utf8(radio).unwrap().contains("Pinch hitter:"));
                    }
                }
                for (n, draw) in &a.draws {
                    let state = a.steps[n - 1].transition().before();
                    assert_eq!(
                        draw.probabilities,
                        model
                            .player_probabilities(probabilities(state).unwrap())
                            .unwrap()
                    );
                }
                let stats = bbsim_engine::accounting::derive_accounting(&log).unwrap();
                for side in 0..2 {
                    assert!(!stats.pitchers[side].contains_key("aph"));
                    assert!(!stats.pitchers[side].contains_key("hph"));
                    if let Some(b) = stats.batters[side].get(["aph", "hph"][side]) {
                        reaches +=
                            b.classified_hits.unwrap() + b.credited_walks + b.credited_hit_by_pitch;
                        runs += b.credited_runs;
                        again += u32::from(b.plate_appearances > 1);
                    }
                }
                let mut boxes = Vec::new();
                full_transcript::write_box_score(
                    &mut boxes,
                    &a.steps,
                    &["Away".into(), "Home".into()],
                    &names,
                )
                .unwrap();
            }
            assert!(ph_count > 0, "no PH in complete {dh} games");
            assert!(ph_sides.iter().all(|n| *n > 0));
            assert!(reaches > 0 && runs > 0 && again > 0);
            println!("PH integration dh={dh} experimental={experimental}: selections={ph_count}, sides={ph_sides:?}, restorations={restored}, reaches={reaches}, runs={runs}, repeated_batters={again}");
            if !dh {
                assert!(
                    restored > 0,
                    "non-DH PH never followed by legal pitching restoration"
                );
            }
        }
    }
}

#[test]
fn bench_schema_and_membership_refuse_before_the_first_rating_callback() {
    use full_simulation::bench::Bench;
    let text = bench_text();
    for bad in [
        text.replace("assumed-game-bench-v1", "unknown"),
        text.replace("OAK201903200", "OAK201903210"),
        text.replace("2,3,4,5,6,7,8,9", "1,2"),
        text.replace("2,3,4,5,6,7,8,9", "2,2"),
        text.replace("0,0,0,0,0,0,4000,0,0", "0,0"),
        format!("{text}extra\tunknown\n"),
        format!("{text}Away:aph:name\tDuplicate\n"),
    ] {
        assert!(Bench::read_tsv(&bad, "OAK201903200").is_err());
    }
    let l = lineup();
    let m = model();
    let cast = full_manager::Cast::read_complete_tsv(&cast_text()).unwrap();
    for id in ["gonzm005", "a1", "h1"] {
        let b = Bench::read_tsv(
            &text.replace("Away:aph:", &format!("Away:{id}:")),
            "OAK201903200",
        )
        .unwrap();
        let mut calls = 0;
        let result = full_simulation::simulate_with_personnel(
            &l.initial(),
            &defense(&l),
            &m,
            Some(&cast),
            Some(&b),
            1,
            |_| {
                calls += 1;
                Ok(None)
            },
        );
        assert!(result.is_err());
        assert_eq!(calls, 0);
    }
    let b = Bench::read_tsv(&text, "OAK201903200").unwrap();
    for removed in [false, true] {
        let mut state = l.initial();
        if removed {
            state.teams[1].removed.push("retired".into());
        } else {
            state.teams[0].next_batter = 1;
        }
        let mut calls = 0;
        assert!(full_simulation::simulate_with_personnel(
            &state,
            &defense(&l),
            &m,
            Some(&cast),
            Some(&b),
            1,
            |_| {
                calls += 1;
                Ok(None)
            },
        )
        .is_err());
        assert_eq!(calls, 0);
    }
    let base = preview::Baseline::from_counts(COUNTS).unwrap();
    let mut expanded = cast.ability_counts().clone();
    expanded.extend(b.counts.clone());
    let roles = expanded.keys().cloned().collect();
    assert!(calibrated::Ratings::read_tsv(
        &calibrated_text(&cast, &base),
        &expanded,
        &base,
        &roles
    )
    .is_err());
    let legacy = full_simulation::simulate_with_probabilities(
        &l.initial(),
        &defense(&l),
        &m,
        Some(&cast),
        1,
        |_| Ok(None),
    )
    .unwrap();
    let untouched = full_simulation::simulate_with_personnel(
        &l.initial(),
        &defense(&l),
        &m,
        Some(&cast),
        None,
        1,
        |_| Ok(None),
    )
    .unwrap();
    assert_eq!(legacy.steps, untouched.steps);
    assert_eq!(legacy.draws, untouched.draws);
}
