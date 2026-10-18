#[path = "../examples/support/season_accounting.rs"]
mod season_accounting;
// Isolate serialization; Python CLI tests exercise the real model/version modules.
mod full_simulation {
    pub const MODEL_VERSION: &str = "mechanics-test";
}
mod ratings {
    pub const MODEL_VERSION: &str = "ratings-test";
}
mod full_manager {
    pub const POLICY_VERSION: &str = "manager-test";
}
mod preview {
    pub const RNG_VERSION: &str = "rng-test";
}
use bbsim_engine::accounting::{BatterLine, FieldingCoverageLine, GameAccounting, PitcherLine};
use std::collections::BTreeMap;

fn fixture(complete_game: bool) -> GameAccounting {
    GameAccounting {
        innings: BTreeMap::new(),
        runs: [1, 0],
        batters: std::array::from_fn(|_| BTreeMap::new()),
        pitchers: std::array::from_fn(|_| BTreeMap::new()),
        fielders: std::array::from_fn(|_| BTreeMap::new()),
        classified_fielding: std::array::from_fn(|_| BTreeMap::new()),
        complete_game,
    }
}

#[test]
fn incomplete_game_refuses_before_any_output() {
    let mut output = Vec::new();
    let error = season_accounting::write(
        &mut output,
        &fixture(false),
        "LAN201904010",
        7,
        &["SFN".into(), "LAN".into()],
    )
    .unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(output.is_empty());
}

#[test]
fn known_zero_unknown_coverage_and_integer_pitching_outs_are_distinct() {
    let mut stats = fixture(true);
    stats.batters[0].insert(
        "b-1".into(),
        BatterLine {
            plate_appearances: 3,
            at_bats: None,
            classified_rbi: None,
            classified_hits: None,
            credited_hit_breakdown: None,
            credited_intentional_walks: None,
            ..BatterLine::default()
        },
    );
    stats.pitchers[1].insert(
        "p-1".into(),
        PitcherLine {
            batters_faced: 3,
            outs_recorded: 4,
            at_bats_against: None,
            earned_runs: None,
            classified_hits_allowed: None,
            credited_hit_breakdown_allowed: None,
            credited_intentional_walks_allowed: None,
            ..PitcherLine::default()
        },
    );
    stats.classified_fielding[1].insert(
        "f-1".into(),
        FieldingCoverageLine {
            observed_events: 2,
            putouts: Some(4),
            assists: None,
            errors: None,
            passed_balls: Some(0),
        },
    );
    let mut output = Vec::new();
    season_accounting::write(
        &mut output,
        &stats,
        "LAN201904010",
        u64::MAX,
        &["SFN".into(), "LAN".into()],
    )
    .unwrap();
    let text = String::from_utf8(output).unwrap();
    assert!(text.starts_with(
        "format\tcontrolled-accounting-v1\ngame\tLAN201904010\nseed\t18446744073709551615\n"
    ));
    assert!(text.contains("team\tAway\tSFN\t1\n"));
    assert!(text.contains("team\tHome\tLAN\t0\n"));
    assert!(text
        .contains("batter\tAway\tb-1\t3\t?\t0\t?\t?\t0\t0\t?\t?\t?\t?\t?\t0\t0\t0\t0\t0\t0\t0\n"));
    assert!(
        text.contains("pitcher\tHome\tp-1\t3\t4\t?\t0\t?\t?\t0\t0\t?\t?\t?\t?\t?\t0\t0\t0\t0\t0\n")
    );
    assert!(text.contains("fielder\tHome\tf-1\t2\t4\t?\t?\t0\n"));
    assert!(!text.contains("1.1"));
}

#[test]
fn output_errors_are_not_swallowed() {
    struct Broken;
    impl std::io::Write for Broken {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::ErrorKind::BrokenPipe.into())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let error = season_accounting::write(
        &mut Broken,
        &fixture(true),
        "LAN201904010",
        1,
        &["SFN".into(), "LAN".into()],
    )
    .unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
}

#[test]
fn ph_metadata_identifies_the_selected_ratings_model() {
    for model in [ratings::MODEL_VERSION, "experimental-ratings-test"] {
        let mut output = Vec::new();
        season_accounting::write_with_personnel(
            &mut output,
            &fixture(true),
            "LAN201904010",
            1,
            &["SFN".into(), "LAN".into()],
            Some((
                "ph-policy-test",
                "assumed availability",
                "assumed defense",
                model,
            )),
        )
        .unwrap();
        let text = String::from_utf8(output).unwrap();
        assert!(text.starts_with("format\tcontrolled-accounting-ph-v1\n"));
        assert!(text.lines().any(|line| line == format!("ratings\t{model}")));
        assert!(text.contains("pinch_hitting\tph-policy-test\n"));
    }
}
