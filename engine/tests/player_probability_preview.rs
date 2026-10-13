#[path = "../examples/support/joint_contact.rs"]
mod joint;
#[path = "../examples/support/preview.rs"]
mod preview;
#[path = "../examples/support/player_ratings.rs"]
mod ratings;
#[path = "../examples/support/real_lineup.rs"]
mod real;
use bbsim_engine::{accounting::derive_accounting, EventTransition};
const COUNTS: [u64; 9] = [42823, 15142, 1984, 25947, 8531, 785, 6776, 1470, 82245];
fn lineup() -> real::Lineup {
    let mut s=format!("key\tvalue\nformat\tclosed-starting-lineup-v1\ngame_id\tOAK201903200\ndate\t2019-03-20\npark_id\tTOK01\naway_team\tSeattle Mariners\nhome_team\tOakland Athletics\naudit_sha256\t{}\nsnapshot_sha256\t{}\nsupplement_sha256\t{}\nattribution\t{}\n",real::REPORT_SHA,real::SNAPSHOT_SHA,real::SUPPLEMENT_SHA,joint::ATTRIBUTION);
    for (side, p) in [("Away", "a"), ("Home", "h")] {
        s.push_str(&format!("{side}:0:1:{p}p\tPitcher {p}\n"));
        for (i, pos) in [7, 8, 3, 10, 9, 2, 5, 6, 4].iter().enumerate() {
            s.push_str(&format!(
                "{side}:{}:{pos}:{p}{}\tPlayer {p}{}\n",
                i + 1,
                i + 1,
                i + 1
            ));
        }
    }
    real::Lineup::read_tsv(&s).unwrap()
}
fn fixture() -> String {
    let mut s=format!("key\tvalue\nformat\tplayer-proxy-counts-v1\nmodel\t{}\nopportunity_report_sha256\t{}\nlineup_report_sha256\t{}\nsnapshot_sha256\t{}\nsupplement_sha256\t{}\nbatter_kappa\t200\npitcher_kappa\t200\nleague_counts\t42823,15142,1984,25947,8531,785,6776,1470,82245\nattribution\t{}\nopportunity_basis\t{}\n",ratings::MODEL_VERSION,ratings::REPORT_SHA,real::REPORT_SHA,real::SNAPSHOT_SHA,real::SUPPLEMENT_SHA,joint::ATTRIBUTION,ratings::BASIS);
    for p in ["a", "h"] {
        for i in 1..=9 {
            s.push_str(&format!("batter:{p}{i}\t3,2,0,4,1,0,1,0,9\n"));
        }
        s.push_str(&format!("pitcher:{p}p\t5,1,0,3,1,0,1,0,9\n"));
    }
    s
}
fn base() -> preview::Baseline {
    let mut text=String::from("key,value\nformat,preview-counts-v1\ncatalog_sha256,2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583\nquery_version,preview-baseline-v1\nselected_pa,186517\nexcluded_ibb,753\nexcluded_interference,61\n");
    for (key, n) in [
        "strikeout",
        "walk",
        "hbp",
        "single",
        "double",
        "triple",
        "home_run",
        "roe",
        "contact_out",
    ]
    .iter()
    .zip(COUNTS)
    {
        text.push_str(&format!("{key},{n}\n"));
    }
    let b = preview::Baseline::read_csv(&text).unwrap();
    assert_eq!(b.counts, COUNTS);
    b
}
fn model() -> joint::JointModel {
    let source = include_str!("joint_contact_preview.rs");
    let csv = source
        .split("const JOINT_CSV: &str = r#\"")
        .nth(1)
        .unwrap()
        .split("\"#;")
        .next()
        .unwrap();
    let m = joint::JointModel::read_csv(csv, &base()).unwrap();
    assert_eq!(m.marginals(), *base().probabilities());
    m
}
fn close(a: f64, b: f64) {
    assert!((a - b).abs() <= 32.0 * f64::EPSILON, "{a} != {b}");
}
#[test]
fn posterior_is_league_centered_positive_with_independent_mean_sd_and_neutral_controls() {
    let league = *base().probabilities();
    let counts = [3, 2, 0, 4, 1, 0, 1, 0, 9];
    let p = ratings::posterior(counts, league).unwrap();
    for k in 0..9 {
        close(p[k], (counts[k] as f64 + 200.0 * league[k]) / 220.0);
        assert!(p[k] > 0.0);
    }
    close(p.iter().sum(), 1.0);
    assert_eq!(ratings::posterior([0; 9], league).unwrap(), league);
    let m = ratings::matchup([0; 9], counts, league).unwrap();
    for k in 0..9 {
        close(m[k], p[k]);
    }
    let m = ratings::matchup([0; 9], [0; 9], league).unwrap();
    for k in 0..9 {
        close(m[k], league[k]);
    }
    assert!(ratings::posterior([u64::MAX; 9], league).is_err());
    let mut bad = league;
    bad[0] = f64::NAN;
    assert!(ratings::matchup(counts, counts, bad).is_err());
}
#[test]
fn normalized_log5_has_independent_literal_weights_and_directional_effects() {
    let league = *base().probabilities();
    let b = [2, 3, 0, 4, 1, 0, 8, 0, 7];
    let p = [9, 1, 0, 2, 0, 0, 1, 0, 8];
    let weights: Vec<_> = (0..9)
        .map(|k| {
            ((b[k] as f64 + 200.0 * league[k]) / 225.0)
                * ((p[k] as f64 + 200.0 * league[k]) / 221.0)
                / league[k]
        })
        .collect();
    let sum: f64 = weights.iter().sum();
    let m = ratings::matchup(b, p, league).unwrap();
    for k in 0..9 {
        close(m[k], weights[k] / sum);
        assert!(m[k].is_finite() && m[k] > 0.0);
    }
    close(m.iter().sum(), 1.0);
    let neutral = ratings::matchup([0; 9], [0; 9], league).unwrap();
    let mut strong = [0; 9];
    strong[6] = 100;
    assert!(ratings::matchup(strong, [0; 9], league).unwrap()[6] > neutral[6]);
    strong = [0; 9];
    strong[0] = 100;
    assert!(ratings::matchup([0; 9], strong, league).unwrap()[0] > neutral[0]);
}
#[test]
fn strict_loader_binds_role_set_and_rejects_changed_shape_provenance_and_counts() {
    let l = lineup();
    let text = fixture();
    let r = ratings::Ratings::read_tsv(&text, &base(), &l).unwrap();
    let m = r
        .probabilities(&l.initial(), *base().probabilities())
        .unwrap();
    close(m.iter().sum(), 1.0);
    for bad in [
        text.replace(ratings::REPORT_SHA, "wrong"),
        text.replace("batter_kappa\t200", "batter_kappa\t100"),
        text.replace("batter:a1\t3,2,0,4,1,0,1,0,9\n", ""),
        text.clone() + "batter:a1\t3,2,0,4,1,0,1,0,9\n",
        text.replace("batter:a1", "batter:other"),
        text.replace("pitcher:ap", "batter:ap"),
        text.replace("3,2,0,4,1,0,1,0,9", "-1,2,0,4,1,0,1,0,9"),
        text.replace("3,2,0,4,1,0,1,0,9", "999999,2,0,4,1,0,1,0,9"),
        text.replace("3,2,0,4,1,0,1,0,9", "03,2,0,4,1,0,1,0,9"),
    ] {
        assert!(
            ratings::Ratings::read_tsv(&bad, &base(), &l).is_err(),
            "{bad}"
        );
    }
    let mut state = l.initial();
    state.teams[1].pitcher = "missing".into();
    assert!(r.probabilities(&state, *base().probabilities()).is_err());
}
#[test]
fn dynamic_joint_cdf_preserves_conditionals_edges_and_exact_one_word() {
    let model = model();
    let league = *base().probabilities();
    let p = ratings::matchup([0; 9], [0; 9], league).unwrap();
    let updated = model.with_marginals(p).unwrap();
    assert_eq!(updated.conditionals(), model.conditionals());
    let edges = updated.boundaries();
    assert_eq!(edges[12], 1.0);
    assert!(edges.windows(2).all(|a| a[0] < a[1]));
    for (i, e) in edges[..12].iter().enumerate() {
        let below = f64::from_bits(e.to_bits() - 1);
        let a = updated.sample_unit(below).unwrap();
        let b = updated.sample_unit(*e).unwrap();
        assert_ne!(a, b, "edge {i}");
        assert_eq!(
            b,
            updated
                .sample_unit(f64::from_bits(e.to_bits() + 1))
                .unwrap()
        );
    }
    let mut x = preview::Rng::new(19);
    let mut y = preview::Rng::new(19);
    for _ in 0..1000 {
        updated.sample(&mut x);
        y.next_u64();
        assert_eq!(x.next_u64(), y.next_u64());
    }
    let mut bad = p;
    bad[0] = 0.0;
    assert!(model.with_marginals(bad).is_err());
    bad = p;
    bad[0] += 0.1;
    assert!(model.with_marginals(bad).is_err());
}
#[test]
fn optin_preserves_legacy_flag_looking_values_and_rejects_repeated_missing_path() {
    let args = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let a = args(&["--lineup", "--ratings", "--seed", "1"]);
    assert_eq!(ratings::split_options(&a).unwrap(), (a, None));
    let (old, path) =
        ratings::split_options(&args(&["--ratings", "path", "--lineup", "roster"])).unwrap();
    assert_eq!(old, args(&["--lineup", "roster"]));
    assert_eq!(path, Some("path".into()));
    for a in [
        &["--ratings"][..],
        &["--ratings", "--trace"][..],
        &["--ratings", "a", "--ratings", "b"][..],
    ] {
        assert!(ratings::split_options(&args(a)).is_err());
    }
}
#[test]
fn native_disclosures_and_existing_public_paths_remain_exercised() {
    let l = lineup();
    let b = base();
    let model = model();
    let r = ratings::Ratings::read_tsv(&fixture(), &b, &l).unwrap();
    assert_eq!(preview::MODEL_VERSION, "pooled-jeffreys-preview-v1");
    assert_eq!(preview::POLICY_VERSION, "station-ground-contact-v1");
    assert_eq!(preview::RNG_VERSION, "splitmix64-v1");
    assert_eq!(
        joint::MODEL_VERSION,
        "pooled-jeffreys-joint-proxy-preview-v1"
    );
    assert_eq!(joint::POLICY_VERSION, "station-held-joint-proxy-preview-v1");
    assert_eq!(
        real::POLICY_VERSION,
        "station-held-positional-starting-preview-v1"
    );
    let mut output = Vec::new();
    r.write_preamble(&mut output, &l, &b).unwrap();
    let text = String::from_utf8(output).unwrap();
    assert_eq!(
        text.lines().filter(|s| s.starts_with("RATING ")).count(),
        20
    );
    assert_eq!(
        text.lines().filter(|s| s.starts_with("MATCHUP ")).count(),
        18
    );
    assert!(text.contains(" | SD: "));
    assert!(text.contains(" | Counts: [3, 2, 0, 4, 1, 0, 1, 0, 9]"));
    assert!(text.contains("hindsight"));
    let mut roster = Vec::new();
    l.write_preamble(&mut roster).unwrap();
    assert_eq!(l.names().len(), 20);
    let mut rated_roster = Vec::new();
    l.write_preamble_with_rates(&mut rated_roster, true)
        .unwrap();
    let rated_roster = String::from_utf8(rated_roster).unwrap();
    assert!(!rated_roster.contains("no player fitting"));
    assert!(rated_roster.contains("participant-proxy"));
    assert!(preview::game(1, &b).unwrap().last().unwrap().after().over);
    assert!(joint::game(1, &model).unwrap().last().unwrap().after().over);
    assert!(l.game(1, &model).unwrap().last().unwrap().after().over);
    assert!(real::split_options(&[]).unwrap().1.is_none());
    assert!(joint::split_options(&[]).unwrap().1.is_none());
}
#[test]
fn two_thousand_rated_games_repeat_validate_codec_continuity_and_available_stats() {
    let l = lineup();
    let b = base();
    let model = model();
    let r = ratings::Ratings::read_tsv(&fixture(), &b, &l).unwrap();
    for seed in 0..2000 {
        let log = r.game(seed, &b, &model, &l).unwrap();
        assert_eq!(log, r.game(seed, &b, &model, &l).unwrap());
        for (i, t) in log.iter().enumerate() {
            assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
            if i > 0 {
                assert_eq!(log[i - 1].after(), t.before());
            }
            assert!(t.event().delta.changes.is_empty());
        }
        assert!(log.last().unwrap().after().over);
        let stats = derive_accounting(&log).unwrap();
        assert_eq!(stats.runs, log.last().unwrap().after().score.map(u32::from));
        assert_eq!(
            stats
                .batters
                .iter()
                .flat_map(|x| x.values())
                .map(|x| x.plate_appearances)
                .sum::<u32>(),
            log.len() as u32
        );
    }
}
