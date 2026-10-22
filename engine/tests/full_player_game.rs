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
#[path = "../examples/support/pitcher_entry_v2.rs"]
mod pitcher_entry_v2;
#[allow(dead_code)]
#[path = "../examples/support/pitcher_policy_v2.rs"]
mod pitcher_policy_v2;
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
#[path = "../examples/support/season_pool.rs"]
mod season_pool;
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
    let full_simulation::Simulation {
        steps,
        draws,
        manager_trace: _,
    } = full_simulation::simulate_with_probabilities(
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

fn managed_bench_text() -> String {
    bench_text().replace("assumed-game-bench-v1", "season-pool-bench-v1").replace("same-game participant proxies assumed available at start; not verified eligibility", "observed season club players assumed available; no daily injury/minor-league reconstruction") + "season_policy\tseason-pool-soft-usage-v1\nAway:aph:pace\t-1\nHome:hph:pace\t1\n"
}
fn v2_lineup_text() -> String {
    let mut text = format!(
        "key\tvalue\nformat\tseason-pool-lineup-v2\ngame_id\tOAK201903200\ndate\t2019-03-20\npark_id\tTOK01\naway_team\tSeattle Mariners\nhome_team\tOakland Athletics\naway_team_id\tSEA\nhome_team_id\tOAK\nuse_dh\ttrue\naudit_sha256\t{}\nsnapshot_sha256\t{}\nsupplement_sha256\t{}\nattribution\t{}\nseason_policy\tseason-pool-pitcher-usage-v2\navailability_basis\t{}\n",
        real::REPORT_SHA,
        real::SNAPSHOT_SHA,
        real::SUPPLEMENT_SHA,
        joint::ATTRIBUTION,
        season_pool::AVAILABILITY,
    );
    for (side, pitcher, name, prefix) in [
        ("Away", "gonzm005", "Marco Gonzales", "a"),
        ("Home", "fierm001", "Michael Fiers", "h"),
    ] {
        text += &format!("{side}:0:1:{pitcher}\t{name}\n");
        for (i, position) in [7, 8, 3, 10, 9, 2, 5, 6, 4].into_iter().enumerate() {
            text += &format!(
                "{side}:{}:{position}:{prefix}{}\tSynthetic {prefix}{}\n",
                i + 1,
                i + 1,
                i + 1
            );
        }
    }
    text
}

fn v2_cast_text() -> String {
    let old = pool_cast_text();
    let mut text = old
        .replace(
            "season-pool-pitching-cast-v1",
            "season-pool-pitching-cast-v2",
        )
        .replace("season-pool-soft-usage-v1", "season-pool-pitcher-usage-v2")
        .replace("relief_pace:", "relief_usage:");
    text = text
        .lines()
        .map(|line| {
            if let Some((key, _)) = line.split_once('\t') {
                if key.starts_with("relief_usage:") {
                    return format!("{key}\t0,?");
                }
            }
            line.to_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\nclub_clock:Away\t0,162\nclub_clock:Home\t0,162\n";
    text
}

fn v2_bench_text() -> String {
    managed_bench_text()
        .replace("season-pool-bench-v1", "season-pool-bench-v2")
        .replace("season-pool-soft-usage-v1", "season-pool-pitcher-usage-v2")
}

#[test]
fn pitcher_usage_v2_live_selector_trace_is_valid_and_rng_neutral() {
    let lineup = real::Lineup::read_tsv(&v2_lineup_text()).unwrap();
    let cast = full_manager::Cast::read_complete_tsv(&v2_cast_text()).unwrap();
    let bench = full_simulation::bench::Bench::read_tsv(&v2_bench_text(), "OAK201903200").unwrap();
    let initial = lineup.initial();
    let fielders = defense(&lineup);
    let model = model();
    let baseline = preview::Baseline::from_counts(COUNTS).unwrap();
    let mut saw_hook = false;
    let mut saw_probe = false;
    for seed in 1..=16 {
        let simulate = |capture_trace| {
            full_simulation::simulate_with_personnel_options(
                &initial,
                &fielders,
                &model,
                Some(&cast),
                Some(&bench),
                full_simulation::SimulationOptions {
                    seed,
                    capture_v2_trace: capture_trace,
                },
                |_state| Ok(Some(model.player_probabilities(*baseline.probabilities())?)),
            )
            .unwrap()
        };
        let captured = simulate(true);
        let trace = captured.manager_trace.as_deref().unwrap();
        let log: Vec<_> = captured
            .steps
            .iter()
            .map(|step| step.transition().clone())
            .collect();
        cast.validate_v2_trace(trace, "OAK201903200", seed, &log)
            .unwrap();
        saw_hook |= trace.contains("\thook\t");
        saw_probe |= trace.contains("\tph_probe\t");
        if seed == 1 {
            let untraced = simulate(false);
            assert!(untraced.manager_trace.is_none());
            assert_eq!(captured.steps, untraced.steps);
            assert_eq!(captured.draws, untraced.draws);
        }
    }
    assert!(saw_hook, "the seeded cohort must exercise v2 pitcher hooks");
    assert!(
        saw_probe,
        "the seeded cohort must exercise v2 PH reserve probes"
    );
}

#[test]
fn pitcher_usage_v2_non_dh_ph_trace_binds_probe_to_vacancy_and_restoration() {
    use bbsim_engine::{DefenseState, Half, PersonnelChange, PitcherRole, TeamSide, TeamState};
    let stable_cast_text = v2_cast_text()
        .lines()
        .map(|line| {
            line.split_once('\t').map_or_else(
                || line.to_owned(),
                |(key, _)| {
                    if key.starts_with("workload:") {
                        format!("{key}\t100,1")
                    } else if key.starts_with("usage:") {
                        format!("{key}\t100")
                    } else {
                        line.to_owned()
                    }
                },
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let cast = full_manager::Cast::read_complete_tsv(&stable_cast_text).unwrap();
    let bench = full_simulation::bench::Bench::read_tsv(
        &v2_bench_text().replace("2,3,4,5,6,7,8,9", "?"),
        "OAK201903200",
    )
    .unwrap();
    let team = |prefix: &str, pitcher: &str| TeamState {
        defense: DefenseState::LegacyUnassigned,
        batting_order: std::array::from_fn(|slot| match slot {
            0 => format!("{prefix}1"),
            1 => pitcher.to_owned(),
            _ => format!("{prefix}{slot}"),
        }),
        next_batter: 0,
        pitcher: pitcher.to_owned().into(),
        dh_slot: None,
        dh_must_bat: false,
        bench: vec![],
        removed: vec![],
    };
    let initial = GameState {
        inning: 1,
        half: Half::Top,
        outs: 0,
        score: [0, 0],
        teams: [team("a", "gonzm005"), team("h", "fierm001")],
        bases: [None, None, None],
        over: false,
    };
    let fielders: [[String; 9]; 2] = [
        ["gonzm005", "a1", "a2", "a3", "a4", "a5", "a6", "a7", "a8"],
        ["fierm001", "h1", "h2", "h3", "h4", "h5", "h6", "h7", "h8"],
    ]
    .map(|side| side.map(str::to_owned));
    let pitcher_ids = cast.names();
    let model = model();
    let mut restored = false;
    for seed in 1..=1 {
        let run = full_simulation::simulate_with_personnel(
            &initial,
            &fielders,
            &model,
            Some(&cast),
            Some(&bench),
            seed,
            |state| {
                let offense = state.half.offense();
                let batter = &state.teams[offense].batting_order
                    [usize::from(state.teams[offense].next_batter)];
                Ok(Some(if pitcher_ids.contains_key(batter.as_str()) {
                    [
                        0.95, 0.005, 0.001, 0.006, 0.012, 0.008, 0.003, 0.005, 0.008, 0.002,
                    ]
                } else if batter == "aph" || batter == "hph" {
                    [0.30, 0.10, 0.02, 0.10, 0.20, 0.10, 0.05, 0.05, 0.07, 0.01]
                } else {
                    [
                        0.85, 0.03, 0.005, 0.02, 0.04, 0.015, 0.005, 0.01, 0.02, 0.005,
                    ]
                }))
            },
        )
        .unwrap();
        let trace = run.manager_trace.as_deref().unwrap();
        let log: Vec<_> = run
            .steps
            .iter()
            .map(|step| step.transition().clone())
            .collect();
        let decisions = cast
            .validate_v2_trace(trace, "OAK201903200", seed, &log)
            .unwrap();
        for (index, step) in run.steps.iter().enumerate() {
            let Some(PersonnelChange::PinchHitter { outgoing, .. }) = step
                .transition()
                .event()
                .delta
                .changes
                .iter()
                .find(|change| matches!(change, PersonnelChange::PinchHitter { .. }))
            else {
                continue;
            };
            if pitcher_ids.contains_key(outgoing.as_str()) {
                let side = usize::from(
                    step.transition().before().teams[1]
                        .batting_order
                        .contains(outgoing),
                );
                assert!(matches!(
                    &step.transition().after().teams[side].pitcher,
                    PitcherRole::Vacant { .. }
                ));
                let team_side = if side == 0 {
                    TeamSide::Away
                } else {
                    TeamSide::Home
                };
                let restoration = run.steps[index + 1..]
                    .iter()
                    .map(|step| step.transition())
                    .find(|transition| transition.event().delta.changes.iter().any(|change| {
                        matches!(change, PersonnelChange::AssignPitcher { team, .. } if *team == team_side)
                    }))
                    .expect("pitcher vacancy is restored before that team next defends");
                assert!(matches!(
                    &restoration.before().teams[side].pitcher,
                    PitcherRole::Vacant { .. }
                ));
                assert_eq!(restoration.before().half.offense(), 1 - side);
                restored = true;
            }
        }
        if restored {
            assert!(decisions
                .iter()
                .any(|d| d.reason == pitcher_entry_v2::Reason::PhProbe));
            assert!(decisions
                .iter()
                .any(|d| d.reason == pitcher_entry_v2::Reason::PhRestore));
            break;
        }
    }
    assert!(
        restored,
        "v2 cohort did not exercise pitcher replacement/restoration"
    );
}

#[test]
fn pitcher_usage_v2_personnel_readers_accept_complete_v2_schemas() {
    let lineup = real::Lineup::read_tsv(&v2_lineup_text()).unwrap();
    let cast = full_manager::Cast::read_complete_tsv(&v2_cast_text()).unwrap();
    let bench = full_simulation::bench::Bench::read_tsv(&v2_bench_text(), "OAK201903200").unwrap();
    assert!(lineup.is_managed() && lineup.is_generic());
    assert!(cast.is_managed() && cast.is_v2() && bench.is_managed());
    assert_eq!(
        lineup.policy_version(),
        Some(season_pool::PITCHER_USAGE_POLICY_V2)
    );
    assert_eq!(
        cast.season_policy(),
        Some(season_pool::PITCHER_USAGE_POLICY_V2)
    );
    assert_eq!(
        bench.season_policy(),
        Some(season_pool::PITCHER_USAGE_POLICY_V2)
    );
    assert_eq!(lineup.game_id(), "OAK201903200");
    assert_eq!(cast.game_id(), Some("OAK201903200"));
    cast.validate_v2_clocks(lineup.team_labels()).unwrap();
    assert!(cast.validate_v2_clocks(["DET", "OAK"]).is_err());
    let state = cast.prepare(&lineup.initial()).unwrap();
    let prepared = bench.prepare(&state, &defense(&lineup), &cast).unwrap();
    assert!(prepared.teams[0].bench.contains(&"aph".to_owned()));
    let clocks = cast.clocks().unwrap();
    assert_eq!((clocks[0].completed(), clocks[0].scheduled()), (0, 162));
    assert_eq!(cast.relief_usage().len(), 12);
    assert_eq!(cast.relief_usage()["gonzm005"].used(), 0);
    assert_eq!(cast.relief_usage()["gonzm005"].target(), None);
}

#[test]
fn pitcher_usage_v2_clock_schedule_and_initial_actor_bindings_are_enforced() {
    let cast_text = v2_cast_text().replace("club_clock:Away\t0,162", "club_clock:Away\t0,161");
    let cast = full_manager::Cast::read_tsv(&cast_text).unwrap();
    assert!(cast.validate_v2_clocks(["CHA", "OAK"]).is_ok());
    assert!(cast.validate_v2_clocks(["SEA", "OAK"]).is_err());
    assert!(cast.validate_v2_clocks(["XXX", "OAK"]).is_err());

    let cast = full_manager::Cast::read_tsv(&v2_cast_text()).unwrap();
    let mismatched = v2_lineup_text().replace("Away:0:1:gonzm005", "Away:0:1:gonzm006");
    let lineup = real::Lineup::read_tsv(&mismatched).unwrap();
    assert!(cast.prepare(&lineup.initial()).is_err());

    let lineup = real::Lineup::read_tsv(&v2_lineup_text()).unwrap();
    let cast = full_manager::Cast::read_tsv(&v2_cast_text()).unwrap();
    let state = cast.prepare(&lineup.initial()).unwrap();
    let conflict = v2_bench_text().replace("Away:aph:", "Away:a1:");
    let bench = full_simulation::bench::Bench::read_tsv(&conflict, "OAK201903200").unwrap();
    assert!(bench.prepare(&state, &defense(&lineup), &cast).is_err());
}

#[test]
fn pitcher_usage_v2_initial_names_are_bound_to_the_lineup() {
    let lineup = real::Lineup::read_tsv(&v2_lineup_text()).unwrap();
    let text = v2_cast_text().replace(
        "cast:Away:0:gonzm005\tMarco Gonzales",
        "cast:Away:0:gonzm005\tDifferent Name",
    );
    let cast = full_manager::Cast::read_tsv(&text).unwrap();
    let mut names = lineup.names();
    assert_eq!(
        cast.merge_names(&mut names).unwrap_err(),
        "lineup/cast display-name mismatch"
    );
}

#[test]
fn pitcher_usage_v2_cast_requires_exact_clock_and_relief_usage_sets() {
    let text = v2_cast_text();
    let cast = full_manager::Cast::read_tsv(&text).unwrap();
    assert!(cast.is_managed());
    for bad in [
        text.replace("club_clock:Away\t0,162\n", ""),
        text.replace("relief_usage:gonzm005\t0,?\n", ""),
        text.replace("relief_usage:gonzm005\t0,?\n", "relief_pace:gonzm005\t0\n"),
        text.clone() + "relief_usage:unlisted\t0,?\n",
        text.clone() + "unknown\ttrue\n",
        text.replace("club_clock:Home\t0,162", "club_clock:Home\t0,0"),
        text.replace("club_clock:Away\t0,162", "club_clock:Away\t0,160"),
        text.replace("relief_usage:gonzm005\t0,?", "relief_usage:gonzm005\t00,?"),
        text.replace("season-pool-pitcher-usage-v2", "season-pool-soft-usage-v1"),
        text.clone() + "club_clock:Away\t0,162\n",
    ] {
        assert!(
            full_manager::Cast::read_tsv(&bad).is_err(),
            "accepted {bad}"
        );
    }
}

#[test]
fn pitcher_usage_v2_bench_rejects_a_managed_v1_cast() {
    let cast = full_manager::Cast::read_tsv(&pool_cast_text()).unwrap();
    let bench = full_simulation::bench::Bench::read_tsv(&v2_bench_text(), "OAK201903200").unwrap();
    let lineup = lineup();
    let state = cast.prepare(&lineup.initial()).unwrap();
    let error = bench.prepare(&state, &defense(&lineup), &cast).unwrap_err();
    assert_eq!(error, "bench/cast mode mismatch");
}

#[test]
fn pitcher_usage_v2_bench_rejects_v1_or_unbound_metadata() {
    let text = v2_bench_text();
    assert!(full_simulation::bench::Bench::read_tsv(&text, "OAK201903200").is_ok());
    for bad in [
        text.replace("season_policy\tseason-pool-pitcher-usage-v2\n", ""),
        text.replace("season-pool-pitcher-usage-v2", "season-pool-soft-usage-v1"),
        text.replace("OAK201903200", "SEA201903200"),
        text.replace("Away:aph:pace\t-1\n", ""),
        text.clone() + "unknown\ttrue\n",
    ] {
        assert!(
            full_simulation::bench::Bench::read_tsv(&bad, "OAK201903200").is_err(),
            "accepted {bad}"
        );
    }
}

#[test]
fn pitcher_usage_v2_personnel_readers_reject_changed_source_pins() {
    for text in [
        v2_lineup_text().replace(real::REPORT_SHA, "wrong"),
        v2_lineup_text().replace(real::SNAPSHOT_SHA, "wrong"),
        v2_lineup_text().replace(real::SUPPLEMENT_SHA, "wrong"),
    ] {
        assert!(real::Lineup::read_tsv(&text).is_err());
    }
    for text in [
        v2_cast_text().replace(real::REPORT_SHA, "wrong"),
        v2_cast_text().replace(ratings::REPORT_SHA, "wrong"),
        v2_cast_text().replace(real::SNAPSHOT_SHA, "wrong"),
        v2_cast_text().replace(real::SUPPLEMENT_SHA, "wrong"),
    ] {
        assert!(full_manager::Cast::read_tsv(&text).is_err());
    }
    for text in [
        v2_bench_text().replace(real::SNAPSHOT_SHA, "wrong"),
        v2_bench_text().replace(real::SUPPLEMENT_SHA, "wrong"),
        v2_bench_text().replace(ratings::REPORT_SHA, "wrong"),
    ] {
        assert!(full_simulation::bench::Bench::read_tsv(&text, "OAK201903200").is_err());
    }
}

#[test]
fn season_pool_bench_schema_requires_canonical_per_actor_pace() {
    use full_simulation::bench::Bench;
    let text = managed_bench_text();
    assert!(Bench::read_tsv(&text, "OAK201903200").is_ok());
    for bad in [
        text.replace("Away:aph:pace\t-1\n", ""),
        text.replace("Away:aph:pace\t-1", "Away:aph:pace\t-2"),
        text.replace("Home:hph:pace\t1", "Home:hph:pace\t01"),
        text.replace("season_policy\tseason-pool-soft-usage-v1\n", ""),
        text.clone() + "Away:aph:pace\t0\n",
        text.clone() + "unknown\ttrue\n",
        bench_text() + "Away:aph:pace\t0\n",
    ] {
        assert!(
            Bench::read_tsv(&bad, "OAK201903200").is_err(),
            "accepted {bad}"
        );
    }
}

fn pool_cast_text() -> String {
    let old = cast_text();
    let mut text = old
        .lines()
        .filter(|line| {
            !line.starts_with("bullpen_report_sha256\t")
                && !line.starts_with("workload_report_sha256\t")
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    text=text.replace("closed-pitching-cast-v1","season-pool-pitching-cast-v1").replace("pa-mean-closed-cast-v1","empirical-pa-closed-cast-v2").replace("recorded nonstarters assumed available at start; not verified eligibility",season_pool::AVAILABILITY).replace("literal terminal-PA participants per pitcher-game appearance; not pitch counts, official BF or fatigue",season_pool::WORKLOAD_BASIS);
    text+="game_id\tOAK201903200\ncast_sizes\t5,7\nworkload_recipe_sha256\t512f542cee70c5824c6fff92f1278b51aad2be6a5a066db18b306091a3fae48f\noutcome_events_sha256\tea3fd660ac87deb02c61cddf28977db59f85f9a88f77aaabc0c07ba0f1831d1a\nseason_policy\tseason-pool-soft-usage-v1\nstarter_scale:Away\t100\nstarter_scale:Home\t100\n";
    for row in old.lines().filter_map(|l| l.strip_prefix("workload:")) {
        let (key, _) = row.split_once('\t').unwrap();
        text += &format!("usage:{key}\t0\n");
    }
    for id in full_manager::Cast::read_tsv(&old).unwrap().names().keys() {
        text += &format!("relief_pace:{id}\t0\n");
    }
    text
}

#[test]
fn managed_bench_cast_modes_refuse_before_ratings_and_preferences_do_not_shift_outcome_rng() {
    let historical_cast = full_manager::Cast::read_tsv(&cast_text()).unwrap();
    let managed_cast = full_manager::Cast::read_tsv(&pool_cast_text()).unwrap();
    let historical_bench =
        full_simulation::bench::Bench::read_tsv(&bench_text(), "OAK201903200").unwrap();
    let managed_bench =
        full_simulation::bench::Bench::read_tsv(&managed_bench_text(), "OAK201903200").unwrap();
    assert!(managed_cast.is_managed() && managed_bench.is_managed());
    assert!(!historical_cast.is_managed() && !historical_bench.is_managed());
    let l = lineup();
    let initial = l.initial();
    let fielders = defense(&l);
    let model = model();
    for (cast, bench) in [
        (&managed_cast, &historical_bench),
        (&historical_cast, &managed_bench),
    ] {
        let error = full_simulation::simulate_with_personnel(
            &initial,
            &fielders,
            &model,
            Some(cast),
            Some(bench),
            1,
            |_| panic!("mixed mode rating query"),
        )
        .err()
        .unwrap();
        assert_eq!(error, "bench/cast mode mismatch");
    }
    let base = preview::Baseline::from_counts(COUNTS).unwrap();
    let p = model.player_probabilities(*base.probabilities()).unwrap();
    for seed in [1, 17, 53] {
        let old = full_simulation::simulate_with_personnel(
            &initial,
            &fielders,
            &model,
            Some(&historical_cast),
            Some(&historical_bench),
            seed,
            |_| Ok(Some(p)),
        )
        .unwrap();
        let run = || {
            full_simulation::simulate_with_personnel(
                &initial,
                &fielders,
                &model,
                Some(&managed_cast),
                Some(&managed_bench),
                seed,
                |_| Ok(Some(p)),
            )
            .unwrap()
        };
        let managed = run();
        let repeat = run();
        // Different cast selection order cannot consume an outcome word; no hitter qualifies.
        assert_eq!(managed.draws, old.draws);
        assert_eq!(managed.steps, repeat.steps);
        assert!(managed
            .steps
            .iter()
            .all(|step| !matches!(step, full_simulation::Step::PinchHit(_))));
        for side in 0..2 {
            let mut used = BTreeSet::new();
            used.insert(initial.teams[side].pitcher.active_id().unwrap().to_owned());
            for step in &managed.steps {
                let t = step.transition();
                let before = t.before().teams[side].pitcher.active_id();
                let after = t.after().teams[side].pitcher.active_id();
                if before != after {
                    assert!(used.insert(after.unwrap().to_owned()), "pitcher reentry");
                }
            }
        }
    }
}

#[test]
fn managed_ph_vacancy_restoration_uses_same_ranker_and_holds_relief_first_batter() {
    use bbsim_engine::{PitcherObligation, PitcherRole};
    let cast = full_manager::Cast::read_tsv(&pool_cast_text()).unwrap();
    let bench =
        full_simulation::bench::Bench::read_tsv(&managed_bench_text(), "OAK201903200").unwrap();
    let l = lineup();
    let fielders = defense(&l);
    let mut initial = l.initial();
    for t in &mut initial.teams {
        let slot = t.dh_slot.take().unwrap() as usize;
        t.dh_must_bat = false;
        t.batting_order[slot] = t.pitcher.active_id().unwrap().into();
    }
    let mut before = bench
        .prepare(&cast.prepare(&initial).unwrap(), &fielders, &cast)
        .unwrap();
    before.inning = 7;
    before.half = bbsim_engine::Half::Top;
    for t in &mut before.teams {
        t.pitcher = PitcherRole::Active {
            player_id: t.pitcher.active_id().unwrap().into(),
            obligation: PitcherObligation::Satisfied,
        };
    }
    before.teams[0].next_batter = 3;
    let expected = cast.next_available(&before, 0).unwrap().to_owned();
    let ph = full_simulation::pinch::decide_managed_states(
        &before,
        &bench.pools[0],
        Some(&expected),
        &bench.pace,
        |s| {
            let mut p = [0.0625; 9];
            p[8] = 0.5;
            if s.teams[0].batting_order[3] == "aph" {
                p[8] -= 0.1;
                p[6] += 0.1;
            }
            Ok(p)
        },
    )
    .unwrap()
    .unwrap();
    let mut vacant = ph.after().clone();
    assert!(matches!(
        vacant.teams[0].pitcher,
        PitcherRole::Vacant { .. }
    ));
    // The vacancy persists through its offensive half; restoration occurs on defense.
    vacant.half = bbsim_engine::Half::Bottom;
    let restore = cast.restore_pitcher(&vacant).unwrap().unwrap();
    assert_eq!(
        restore.after().teams[0].pitcher.active_id(),
        Some(expected.as_str())
    );
    assert!(matches!(
        restore.after().teams[0].pitcher,
        PitcherRole::Active {
            obligation: PitcherObligation::ReliefBatter,
            ..
        }
    ));
    assert!(cast.next_change(restore.after(), &[]).unwrap().is_none());
    assert!(restore.after().teams[0]
        .removed
        .contains(&"gonzm005".into()));
}
