//! Synthetic shuffled positions; no real source roster in Git.
#[path = "../examples/support/game_trace.rs"]
mod game_trace;
#[path = "../examples/support/joint_contact.rs"]
mod joint;
#[path = "../examples/support/preview.rs"]
mod preview;
#[path = "../examples/support/real_lineup.rs"]
mod real;
#[allow(dead_code)]
#[path = "../examples/support/season_pool.rs"]
mod season_pool;
use bbsim_engine::{
    accounting::derive_accounting, apply, Base, CreditKind, EventTransition, FieldingReview, Half,
    Runner,
};
use joint::{ContactKind, Sample};

fn fixture() -> String {
    let mut text=format!("key\tvalue\nformat\tclosed-starting-lineup-v1\ngame_id\tOAK201903200\ndate\t2019-03-20\npark_id\tTOK01\naway_team\tSeattle Mariners\nhome_team\tOakland Athletics\naudit_sha256\t{}\nsnapshot_sha256\t{}\nsupplement_sha256\t{}\nattribution\t{}\n",real::REPORT_SHA,real::SNAPSHOT_SHA,real::SUPPLEMENT_SHA,joint::ATTRIBUTION);
    for (side, prefix) in [("Away", "a"), ("Home", "h")] {
        text.push_str(&format!("{side}:0:1:{prefix}p\tPitcher {prefix}\n"));
        for (i, pos) in [7, 8, 3, 10, 9, 2, 5, 6, 4].into_iter().enumerate() {
            text.push_str(&format!(
                "{side}:{}:{pos}:{prefix}{}\tPlayer {prefix}{}\n",
                i + 1,
                i + 1,
                i + 1
            ));
        }
    }
    text
}
fn model() -> joint::JointModel {
    let base =
        preview::Baseline::from_counts([42823, 15142, 1984, 25947, 8531, 785, 6776, 1470, 82245])
            .unwrap();
    joint::JointModel::from_counts(&base, [17188, 19528, 5379, 536, 39614]).unwrap()
}
fn sample(i: usize) -> Sample {
    if i < 8 {
        Sample {
            outcome: preview::ALL[i],
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
                ][i - 8],
            ),
        }
    }
}
fn generic_fixture(dh: bool) -> String {
    let mut text = fixture()
        .replace("closed-starting-lineup-v1", "closed-starting-lineup-v2")
        .replace("OAK201903200", "LAN201904010")
        .replace("2019-03-20", "2019-04-01")
        .replace("TOK01", "LOS03")
        .replace("Seattle Mariners", "San Francisco Giants")
        .replace("Oakland Athletics", "Los Angeles Dodgers");
    text.push_str(&format!(
        "use_dh\t{dh}\naway_team_id\tSFN\nhome_team_id\tLAN\n"
    ));
    if !dh {
        for (side, prefix) in [("Away", "a"), ("Home", "h")] {
            text = text
                .replace(&format!("{side}:0:1:{prefix}p\tPitcher {prefix}\n"), "")
                .replace(
                    &format!("{side}:4:10:{prefix}4\tPlayer {prefix}4\n"),
                    &format!("{side}:4:1:{prefix}p\tPitcher {prefix}\n"),
                );
        }
    }
    text
}
#[test]
fn generic_dh_and_non_dh_have_exact_starting_roles_and_metadata() {
    for dh in [false, true] {
        let text = generic_fixture(dh);
        let lineup = real::Lineup::read_tsv(&text).unwrap();
        assert_eq!(lineup.game_id(), "LAN201904010");
        assert_eq!(
            lineup.team_names(),
            ["San Francisco Giants", "Los Angeles Dodgers"]
        );
        assert_eq!(lineup.team_labels(), ["SFN", "LAN"]);
        let state = lineup.initial();
        for (side, prefix) in [(0, "a"), (1, "h")] {
            assert_eq!(state.teams[side].pitcher, format!("{prefix}p").into());
            assert_eq!(state.teams[side].dh_slot, if dh { Some(3) } else { None });
            assert_eq!(state.teams[side].dh_must_bat, dh);
            assert_eq!(
                state.teams[side].batting_order[3],
                format!("{prefix}{}", if dh { "4" } else { "p" })
            );
        }
        let mut out = Vec::new();
        lineup.write_preamble(&mut out).unwrap();
        let out = String::from_utf8(out).unwrap();
        assert!(out.contains("Matchup: LAN201904010 | 2019-04-01 | LOS03"));
        assert!(!out.contains("Seattle") && !out.contains("Oakland"));
        let log = lineup.game(7, &model()).unwrap();
        assert!(log.last().unwrap().after().over);
        assert!(derive_accounting(&log).is_ok());
    }
}
#[test]
fn generic_lineup_rejects_bad_metadata_or_mixed_roles() {
    let text = generic_fixture(false);
    for bad in [
        text.replace("false", "0"),
        text.replace("2019-04-01", "2019-02-29"),
        text.replace("2019-04-01", "2020-04-01"),
        text.replace("LAN201904010", "SFN201904010"),
        text.replace("away_team_id\tSFN", "away_team_id\tLAN"),
        text.replace("Away:4:1:ap", "Away:0:1:ap"),
        text.replace("Away:4:1:ap", "Away:4:10:ap"),
        text.replace("Away:4:1:ap\tPitcher a\n", ""),
        text.clone() + "Away:0:1:ap\tPitcher a\n",
        text.clone() + "unknown\tvalue\n",
        generic_fixture(true).replace("use_dh\ttrue", "use_dh\tfalse"),
    ] {
        assert!(real::Lineup::read_tsv(&bad).is_err(), "accepted {bad}");
    }
    let hyphen = text.replace(":a1\t", ":jay-j001\t");
    assert!(hyphen.contains(real::REPORT_SHA));
    real::Lineup::read_tsv(&hyphen).unwrap();
}
#[test]
fn starting_ids_dh_and_positions_are_not_batting_ordinals() {
    let lineup = real::Lineup::read_tsv(&fixture()).unwrap();
    let state = lineup.initial();
    for (side, prefix) in [(0, "a"), (1, "h")] {
        assert_eq!(
            state.teams[side].batting_order,
            std::array::from_fn(|i| format!("{prefix}{}", i + 1))
        );
        assert_eq!(state.teams[side].pitcher, format!("{prefix}p").into());
        assert_eq!(state.teams[side].dh_slot, Some(3));
        assert!(state.teams[side].dh_must_bat);
        assert!(state.teams[side].bench.is_empty() && state.teams[side].removed.is_empty());
    }
    assert_eq!(state.bases, [None, None, None]);
    assert_eq!(state.half, Half::Top);
    assert_eq!(
        real::POLICY_VERSION,
        "station-held-positional-starting-preview-v1"
    );
    let mut rows = fixture().lines().map(str::to_owned).collect::<Vec<_>>();
    rows[1..].reverse();
    assert_eq!(
        real::Lineup::read_tsv(&(rows.join("\n") + "\n"))
            .unwrap()
            .initial(),
        state
    );
}
#[test]
fn loader_rejects_bad_grains_provenance_and_cross_team_rosters() {
    let text = fixture();
    for bad in [
        text.replace("key\tvalue", "key,value"),
        text.replace("closed-starting-lineup-v1", "unknown"),
        text.replace(real::REPORT_SHA, "wrong"),
        text.replace("Away:0:1:ap", "Away:0:10:ap"),
        text.replace("Away:4:10:a4", "Away:4:9:a4"),
        text.replace("Away:1:7:a1", "Away:1:7:a2"),
        text.replace("Home:1:7:h1", "Home:1:7:a1"),
        text.replace("Away:1:7:a1\tPlayer a1\n", ""),
        text.clone() + "Away:1:7:a1\tPlayer a1\n",
        text.replace("Player a1", "Player\ta1"),
        text.replace("Away:1:7:a1", "Away:1:7:bad-id"),
        text.clone() + "unknown\tvalue\n",
    ] {
        assert!(real::Lineup::read_tsv(&bad).is_err(), "accepted {bad}");
    }
}
#[test]
fn every_outcome_binds_catcher_ground_muff_air_and_residual_position_policy() {
    let lineup = real::Lineup::read_tsv(&fixture()).unwrap();
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            for mask in 0..8 {
                let mut state = lineup.initial();
                state.half = half;
                state.outs = outs;
                let prefix = if half == Half::Top { "a" } else { "h" };
                let defense = if half == Half::Top { "h" } else { "a" };
                for (i, base) in state.bases.iter_mut().enumerate() {
                    if mask & (1 << i) != 0 {
                        *base = Some(Runner {
                            player_id: format!("{prefix}{}", i + 6),
                            responsible_pitcher: format!("{defense}p"),
                        });
                    }
                }
                for i in 0..13 {
                    let before = state.clone();
                    let e = lineup.event_for(&state, sample(i)).unwrap();
                    let credits = |kind| {
                        e.delta
                            .credits
                            .iter()
                            .filter(|c| c.kind == kind)
                            .map(|c| c.player_id.clone())
                            .collect::<Vec<_>>()
                    };
                    match i {
                        0 => {
                            assert_eq!(credits(CreditKind::Putout), vec![format!("{defense}6")]);
                            assert_eq!(
                                e.caught_strikeout.as_ref().unwrap().catcher_id,
                                format!("{defense}6")
                            );
                        }
                        7 => {
                            assert_eq!(credits(CreditKind::Error), vec![format!("{defense}8")]);
                            assert_eq!(
                                e.muffed_ground_ball.as_ref().unwrap().fielder_id,
                                format!("{defense}8")
                            );
                        }
                        8 => {
                            assert_eq!(credits(CreditKind::Assist), vec![format!("{defense}8")]);
                            assert_eq!(credits(CreditKind::Putout), vec![format!("{defense}3")]);
                        }
                        9..=11 => {
                            let id = format!("{defense}{}", [2, 5, 7][i - 9]);
                            assert_eq!(credits(CreditKind::Putout), vec![id.clone()]);
                            assert_eq!(e.ordinary_air_catch.as_ref().unwrap().fielder_id, id);
                            assert!(e.ordinary_ground_throw.is_none());
                        }
                        12 => {
                            assert!(credits(CreditKind::Putout).is_empty());
                            assert!(credits(CreditKind::Assist).is_empty());
                            assert!(
                                e.ordinary_air_catch.is_none() && e.ordinary_ground_throw.is_none()
                            );
                            assert_eq!(e.fielding_review, FieldingReview::Unknown);
                        }
                        _ => {}
                    }
                    for c in &e.delta.credits {
                        if matches!(
                            c.kind,
                            CreditKind::Putout | CreditKind::Assist | CreditKind::Error
                        ) {
                            assert_ne!(c.player_id, format!("{defense}4"));
                        }
                    }
                    let t = apply(&state, e).unwrap();
                    assert_eq!(state, before);
                    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
                }
            }
        }
    }
}
#[test]
fn closed_roster_drift_and_inconsistent_samples_fail_without_fallback() {
    let lineup = real::Lineup::read_tsv(&fixture()).unwrap();
    for mutation in 0..5 {
        let mut state = lineup.initial();
        match mutation {
            0 => state.teams[1].batting_order.swap(0, 1),
            1 => state.teams[1].pitcher = "other".into(),
            2 => state.teams[1].dh_slot = Some(0),
            3 => state.teams[1].bench.push("other".into()),
            _ => state.teams[1].removed.push("other".into()),
        }
        assert!(lineup.event_for(&state, sample(0)).is_err());
    }
    assert!(lineup
        .event_for(
            &lineup.initial(),
            Sample {
                outcome: preview::Outcome::Single,
                contact: Some(ContactKind::Fly)
            }
        )
        .is_err());
    let mut state = lineup.initial();
    state.over = true;
    assert!(lineup.event_for(&state, sample(0)).is_err());
}
#[test]
fn optin_preserves_flag_looking_legacy_values_and_requires_one_lineup_path() {
    let args = |s: &[&str]| s.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let (old, path) =
        real::split_options(&args(&["--baseline", "--lineup", "--seed", "1"])).unwrap();
    assert_eq!(old, args(&["--baseline", "--lineup", "--seed", "1"]));
    assert!(path.is_none());
    let (old, path) = real::split_options(&args(&[
        "--baseline",
        "base",
        "--lineup",
        "roster",
        "--contact-counts",
        "counts",
    ]))
    .unwrap();
    assert_eq!(path, Some("roster".into()));
    assert_eq!(
        old,
        args(&["--baseline", "base", "--contact-counts", "counts"])
    );
    for bad in [
        &["--lineup"][..],
        &["--lineup", "--trace"][..],
        &["--lineup", "a", "--lineup", "b"][..],
    ] {
        assert!(real::split_options(&args(bad)).is_err());
    }
}
#[test]
fn named_native_commentary_uses_exact_tokens_and_preserves_raw_trace() {
    let lineup = real::Lineup::read_tsv(&fixture()).unwrap();
    let log = lineup.game(1, &model()).unwrap();
    let mut calls = Vec::new();
    game_trace::write_commentary_named(&log, &mut calls, &lineup.names()).unwrap();
    let calls = String::from_utf8(calls).unwrap();
    assert!(calls.contains("Pitcher h to Player a1"));
    assert!(!calls.contains("[hp]") && !calls.contains("[a1]"));
    assert!(!calls.contains("Away-"));
    let mut trace = Vec::new();
    game_trace::write(&log, &mut trace).unwrap();
    let trace = String::from_utf8(trace).unwrap();
    assert!(trace.contains("\"a1\""));
    assert!(!trace.contains("Player a1"));
    let mut meta = Vec::new();
    lineup.write_preamble(&mut meta).unwrap();
    let meta = String::from_utf8(meta).unwrap();
    assert!(meta.contains("TEAM Away: Seattle Mariners"));
    assert!(meta.contains("PLAYER Away 4 10 a4: Player a4"));
    assert!(meta.contains(joint::ATTRIBUTION));
}
#[test]
fn position_commentary_uses_supplied_lineup_roles_not_batting_slots_or_player_names() {
    let lineup = real::Lineup::read_tsv(&fixture()).unwrap();
    let positions = lineup.positions();
    for (id, position) in [
        ("hp", "P"),
        ("h1", "LF"),
        ("h2", "CF"),
        ("h3", "1B"),
        ("h4", "DH"),
        ("h5", "RF"),
        ("h6", "C"),
        ("h7", "3B"),
        ("h8", "SS"),
        ("h9", "2B"),
    ] {
        assert_eq!(positions[id], position);
    }
    for index in [7, 8, 9, 10, 11] {
        let before = lineup.initial();
        let t = apply(&before, lineup.event_for(&before, sample(index)).unwrap()).unwrap();
        let frozen = t.to_bytes();
        let mut output = Vec::new();
        game_trace::write_commentary_with_positions(
            std::slice::from_ref(&t),
            &mut output,
            &lineup.names(),
            &positions,
        )
        .unwrap();
        let text = String::from_utf8(output).unwrap();
        let location = match index {
            7 | 8 => "ground ball to SS",
            9 => "fly ball to CF",
            10 => "line drive to RF",
            11 => "pop-up to 3B",
            _ => unreachable!(),
        };
        assert!(text.lines().nth(1).unwrap().contains(location), "{text}");
        assert_eq!(t.to_bytes(), frozen);
        if index == 9 {
            let mut plain = Vec::new();
            game_trace::write_commentary_named(
                std::slice::from_ref(&t),
                &mut plain,
                &lineup.names(),
            )
            .unwrap();
            assert!(String::from_utf8(plain)
                .unwrap()
                .contains("fly ball caught by Player h2"));
            for position in ["P", "C", "1B", "2B", "3B", "SS", "LF", "CF", "RF"] {
                let supplied = [("h2".to_string(), position)].into_iter().collect();
                let mut call = Vec::new();
                game_trace::write_commentary_with_positions(
                    std::slice::from_ref(&t),
                    &mut call,
                    &lineup.names(),
                    &supplied,
                )
                .unwrap();
                assert!(String::from_utf8(call)
                    .unwrap()
                    .contains(&format!("fly ball to {position}, caught by Player h2")));
            }
        }
    }
}

#[test]
fn unchanged_loader_sampler_versions_and_empty_name_lookup_controls() {
    let counts = [42823, 15142, 1984, 25947, 8531, 785, 6776, 1470, 82245];
    let mut csv=String::from("key,value\nformat,preview-counts-v1\ncatalog_sha256,2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583\nquery_version,preview-baseline-v1\nselected_pa,186517\nexcluded_ibb,753\nexcluded_interference,61\n");
    for (key, count) in [
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
    .into_iter()
    .zip(counts)
    {
        csv.push_str(&format!("{key},{count}\n"));
    }
    let baseline = preview::Baseline::read_csv(&csv).unwrap();
    assert_eq!(baseline.counts, counts);
    let source = include_str!("joint_contact_preview.rs");
    let joint_csv = source
        .split("const JOINT_CSV: &str = r#\"")
        .nth(1)
        .unwrap()
        .split("\"#;")
        .next()
        .unwrap();
    let model = joint::JointModel::read_csv(joint_csv, &baseline).unwrap();
    assert_eq!(model.marginals(), *baseline.probabilities());
    assert_eq!(model.conditionals(), self::model().conditionals());
    assert_eq!(model.boundaries(), self::model().boundaries());
    assert_eq!(preview::MODEL_VERSION, "pooled-jeffreys-preview-v1");
    assert_eq!(preview::POLICY_VERSION, "station-ground-contact-v1");
    assert_eq!(preview::RNG_VERSION, "splitmix64-v1");
    assert_eq!(
        joint::MODEL_VERSION,
        "pooled-jeffreys-joint-proxy-preview-v1"
    );
    assert_eq!(joint::POLICY_VERSION, "station-held-joint-proxy-preview-v1");
    let args = vec!["--contact-counts".into(), "counts".into()];
    assert_eq!(
        joint::split_options(&args).unwrap(),
        (vec![], Some("counts".into()))
    );
    let mut old = preview::Rng::new(19);
    let mut new = preview::Rng::new(19);
    for _ in 0..256 {
        assert_eq!(baseline.sample(&mut old), model.sample(&mut new).outcome);
        assert_eq!(old.next_u64(), new.next_u64());
    }
    let ground_log = preview::game(1, &baseline).unwrap();
    assert!(ground_log.last().unwrap().after().over);
    let log = joint::game(1, &model).unwrap();
    let mut plain = Vec::new();
    let mut named = Vec::new();
    game_trace::write_commentary(&log, &mut plain).unwrap();
    game_trace::write_commentary_named(&log, &mut named, &std::collections::BTreeMap::new())
        .unwrap();
    assert_eq!(plain, named);
}

#[test]
fn two_thousand_repeated_games_keep_joint_physics_and_available_statistics() {
    let lineup = real::Lineup::read_tsv(&fixture()).unwrap();
    let model = model();
    let initial = lineup.initial();
    for seed in 0..2000 {
        let log = lineup.game(seed, &model).unwrap();
        assert_eq!(lineup.game(seed, &model).unwrap(), log);
        let old = joint::game(seed, &model).unwrap();
        assert_eq!(old.len(), log.len(), "seed {seed}");
        for (i, (t, control)) in log.iter().zip(&old).enumerate() {
            assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
            if i > 0 {
                assert_eq!(log[i - 1].after(), t.before());
            }
            assert_eq!(t.after().score, control.after().score);
            assert_eq!(t.after().outs, control.after().outs);
            assert_eq!(t.after().half, control.after().half);
            assert_eq!(t.after().inning, control.after().inning);
            assert_eq!(t.event().delta.outs, control.event().delta.outs);
            assert_eq!(t.event().delta.runs, control.event().delta.runs);
            assert_eq!(
                t.before().teams[0].batting_order,
                initial.teams[0].batting_order
            );
            assert_eq!(
                t.before().teams[1].batting_order,
                initial.teams[1].batting_order
            );
            assert_eq!(
                t.event().participants.pitcher.as_deref(),
                t.before().teams[1 - t.before().half.offense()]
                    .pitcher
                    .active_id()
            );
            assert!(t.event().delta.changes.is_empty());
            for from in [Base::First, Base::Second, Base::Third] {
                assert_eq!(
                    t.event()
                        .delta
                        .moves
                        .iter()
                        .filter(|m| m.from == from)
                        .map(|m| m.to)
                        .collect::<Vec<_>>(),
                    control
                        .event()
                        .delta
                        .moves
                        .iter()
                        .filter(|m| m.from == from)
                        .map(|m| m.to)
                        .collect::<Vec<_>>()
                );
            }
        }
        assert!(log.last().unwrap().after().over);
        let stats = derive_accounting(&log).unwrap();
        assert_eq!(stats.runs, log.last().unwrap().after().score.map(u32::from));
        assert_eq!(
            stats
                .batters
                .iter()
                .flat_map(|s| s.values())
                .map(|b| b.plate_appearances)
                .sum::<u32>(),
            log.len() as u32
        );
    }
}

#[test]
fn season_pool_lineup_reuses_legal_generic_roles_with_explicit_availability() {
    for dh in [false, true] {
        let text = generic_fixture(dh).replace("closed-starting-lineup-v2", "season-pool-lineup-v1") + "season_policy\tseason-pool-soft-usage-v1\navailability_basis\tobserved season club players assumed available; no daily injury/minor-league reconstruction\n";
        let managed = real::Lineup::read_tsv(&text).unwrap();
        assert!(managed.is_managed() && managed.is_generic());
        assert!(!real::Lineup::read_tsv(&generic_fixture(dh))
            .unwrap()
            .is_managed());
        for bad in [text.replace("season_policy\tseason-pool-soft-usage-v1\n", ""), text.clone()+"season_policy\tseason-pool-soft-usage-v1\n",text.replace("season-pool-soft-usage-v1", "wrong"),text.replace("availability_basis\tobserved season club players assumed available; no daily injury/minor-league reconstruction\n", ""),text.clone()+"unknown\ttrue\n", generic_fixture(dh)+"season_policy\tseason-pool-soft-usage-v1\n"] { assert!(real::Lineup::read_tsv(&bad).is_err(),"accepted {bad}"); }
    }
}
