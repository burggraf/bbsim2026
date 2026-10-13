//! Characterize current support honestly; green here does not mean realistic generation.
#[path = "../examples/support/joint_contact.rs"]
mod joint;
#[path = "../examples/support/preview.rs"]
mod preview;
#[path = "../examples/support/player_ratings.rs"]
mod ratings;
#[path = "../examples/support/real_lineup.rs"]
mod real;
use bbsim_engine::{
    apply, Base, Credit, CreditKind, Destination, EventKind, GameState, Half, Runner, RunnerMove,
    TeamState,
};
const COUNTS: [u64; 9] = [42823, 15142, 1984, 25947, 8531, 785, 6776, 1470, 82245];
fn initial() -> GameState {
    let team = |side: &str| TeamState {
        batting_order: std::array::from_fn(|i| format!("{side}-{}", i + 1)),
        next_batter: 0,
        pitcher: format!("{side}-pitcher"),
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
fn occupied(mut state: GameState, mask: usize) -> GameState {
    for i in 0..3 {
        state.bases[i] = (mask & (1 << i) != 0).then(|| Runner {
            player_id: state.teams[state.half.offense()].batting_order[i + 4].clone(),
            responsible_pitcher: state.teams[1 - state.half.offense()].pitcher.clone(),
        });
    }
    state
}
fn sample(index: usize) -> joint::Sample {
    if index < 8 {
        joint::Sample {
            outcome: preview::ALL[index],
            contact: None,
        }
    } else {
        joint::Sample {
            outcome: preview::Outcome::ContactOut,
            contact: Some(
                [
                    joint::ContactKind::Ground,
                    joint::ContactKind::Fly,
                    joint::ContactKind::LineDrive,
                    joint::ContactKind::PopUp,
                    joint::ContactKind::Residual,
                ][index - 8],
            ),
        }
    }
}
#[test]
fn finite_taxonomy_is_legal_but_has_no_runner_only_sacrifices_or_multi_out_generation() {
    let mut frames = 0;
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            for mask in 0..8 {
                let mut before = initial();
                before.half = half;
                before = occupied(before, mask);
                before.outs = outs;
                for index in 0..13 {
                    let event = joint::event_for(&before, sample(index)).unwrap();
                    assert!(matches!(
                        event.kind,
                        EventKind::PlateAppearance | EventKind::OutOfParkHomeRun
                    ));
                    assert!(event.delta.outs <= 1);
                    assert_eq!(
                        event
                            .delta
                            .credits
                            .iter()
                            .filter(|c| c.kind == CreditKind::PlateAppearance)
                            .count(),
                        1
                    );
                    for kind in [
                        CreditKind::SacrificeFly,
                        CreditKind::SacrificeBunt,
                        CreditKind::GroundedIntoDoublePlay,
                        CreditKind::StolenBase,
                        CreditKind::CaughtStealing,
                    ] {
                        assert!(!event.delta.credits.iter().any(|c| c.kind == kind));
                    }
                    if index >= 8 {
                        assert_eq!(event.delta.outs, 1);
                        assert_eq!(event.delta.runs, 0);
                        assert_eq!(
                            event.delta.moves.len(),
                            1,
                            "contact outs hold every prior runner"
                        );
                    }
                    let transition = apply(&before, event).unwrap();
                    assert_eq!(transition.before(), &before);
                    assert_eq!(transition.after().teams[half.offense()].next_batter, 1);
                    assert_eq!(
                        transition.after().score[half.offense()] - before.score[half.offense()],
                        u16::from(transition.event().delta.runs)
                    );
                    assert_eq!(
                        apply(transition.before(), transition.event().clone()).unwrap(),
                        transition
                    );
                    let decoded =
                        bbsim_engine::EventTransition::from_bytes(&transition.to_bytes()).unwrap();
                    assert_eq!(decoded, transition);
                    frames += 1;
                }
            }
        }
    }
    assert_eq!(frames, 624);
}
#[test]
fn legal_extra_base_advance_is_supported_but_station_single_does_not_generate_it() {
    let before = occupied(initial(), 2);
    let original = preview::event_for(&before, preview::Outcome::Single).unwrap();
    let runner = before.bases[1].as_ref().unwrap();
    assert_eq!(
        original
            .delta
            .moves
            .iter()
            .find(|m| m.from == Base::Second)
            .unwrap()
            .to,
        Destination::Third
    );
    let mut scoring = original;
    scoring
        .delta
        .moves
        .iter_mut()
        .find(|m| m.from == Base::Second)
        .unwrap()
        .to = Destination::Home;
    scoring.delta.runs = 1;
    scoring.delta.credits.extend([
        Credit {
            player_id: runner.player_id.clone(),
            kind: CreditKind::Run,
        },
        Credit {
            player_id: before.teams[0].batting_order[0].clone(),
            kind: CreditKind::Rbi,
        },
    ]);
    let transition = apply(&before, scoring).unwrap();
    assert_eq!(transition.after().score, [1, 0]);
    assert_eq!(
        transition.after().bases[0].as_ref().unwrap().player_id,
        "Away-1"
    );
    assert!(transition.after().bases[1].is_none());
    assert_eq!(
        transition.event().delta.moves[0].runner.responsible_pitcher,
        runner.responsible_pitcher
    );
}
#[test]
fn supplied_runner_only_steal_does_not_imply_a_generated_attempt_policy() {
    let before = occupied(initial(), 1);
    let runner = before.bases[0].clone().unwrap();
    let mut steal = preview::event_for(&before, preview::Outcome::Walk).unwrap();
    steal.kind = EventKind::RunnerOnly(bbsim_engine::RunnerEvent::StealAttempt);
    steal.protected_award = None;
    steal.delta.moves = vec![RunnerMove {
        runner: runner.clone(),
        from: Base::First,
        to: Destination::Second,
    }];
    steal.delta.credits = vec![Credit {
        player_id: runner.player_id.clone(),
        kind: CreditKind::StolenBase,
    }];
    let transition = apply(&before, steal).unwrap();
    assert_eq!(transition.after().teams, before.teams);
    assert_eq!(transition.after().outs, before.outs);
    assert_eq!(transition.after().score, before.score);
    assert_eq!(transition.after().bases[1], Some(runner));
    assert!(transition.after().bases[0].is_none());
    assert!(!transition
        .event()
        .delta
        .credits
        .iter()
        .any(|c| c.kind == CreditKind::PlateAppearance));
}
#[test]
fn residue_is_partial_accounting_even_when_other_shape_controls_are_legal() {
    let before = initial();
    let transition = apply(&before, joint::event_for(&before, sample(12)).unwrap()).unwrap();
    assert!(transition.ordinary_air_fielding().is_none());
    assert!(transition.ordinary_ground_fielding().is_none());
    let stats = bbsim_engine::accounting::derive_accounting(&[transition]).unwrap();
    assert!(!stats.complete_game);
    assert!(!stats.classified_fielding[1].is_empty());
    assert!(stats.classified_fielding[1]
        .values()
        .all(|line| line.putouts.is_none() && line.assists.is_none()));
}
#[test]
fn matchup_inputs_are_participant_proxies_not_contextual_mechanism_calibration() {
    let league = *preview::Baseline::from_counts(COUNTS)
        .unwrap()
        .probabilities();
    let batter = [3, 2, 0, 4, 1, 0, 1, 0, 9];
    let pitcher = [5, 1, 0, 3, 1, 0, 1, 0, 9];
    let p = ratings::matchup(batter, pitcher, league).unwrap();
    assert!(p.iter().all(|x| x.is_finite() && *x > 0.0));
    assert!((p.iter().sum::<f64>() - 1.0).abs() <= 32.0 * f64::EPSILON);
    // No base/out, handedness or park argument exists in this participant formula.
    let mut text=format!("key\tvalue\nformat\tclosed-starting-lineup-v1\ngame_id\tOAK201903200\ndate\t2019-03-20\npark_id\tTOK01\naway_team\tSeattle Mariners\nhome_team\tOakland Athletics\naudit_sha256\t{}\nsnapshot_sha256\t{}\nsupplement_sha256\t{}\nattribution\t{}\n",real::REPORT_SHA,real::SNAPSHOT_SHA,real::SUPPLEMENT_SHA,joint::ATTRIBUTION);
    let mut counts=format!("key\tvalue\nformat\tplayer-proxy-counts-v1\nmodel\t{}\nopportunity_report_sha256\t{}\nlineup_report_sha256\t{}\nsnapshot_sha256\t{}\nsupplement_sha256\t{}\nbatter_kappa\t200\npitcher_kappa\t200\nleague_counts\t42823,15142,1984,25947,8531,785,6776,1470,82245\nattribution\t{}\nopportunity_basis\t{}\n",ratings::MODEL_VERSION,ratings::REPORT_SHA,real::REPORT_SHA,real::SNAPSHOT_SHA,real::SUPPLEMENT_SHA,joint::ATTRIBUTION,ratings::BASIS);
    for (side, prefix) in [("Away", "a"), ("Home", "h")] {
        text.push_str(&format!("{side}:0:1:{prefix}p\tSynthetic pitcher\n"));
        counts.push_str(&format!("pitcher:{prefix}p\t5,1,0,3,1,0,1,0,9\n"));
        for (i, pos) in [7, 8, 3, 10, 9, 2, 5, 6, 4].iter().enumerate() {
            text.push_str(&format!(
                "{side}:{}:{pos}:{prefix}{}\tSynthetic player\n",
                i + 1,
                i + 1
            ));
            counts.push_str(&format!("batter:{prefix}{}\t3,2,0,4,1,0,1,0,9\n", i + 1));
        }
    }
    let lineup = real::Lineup::read_tsv(&text).unwrap();
    let baseline = preview::Baseline::from_counts(COUNTS).unwrap();
    let model = ratings::Ratings::read_tsv(&counts, &baseline, &lineup).unwrap();
    activate_existing_entries(&baseline, &lineup, &model);
    for mask in 0..8 {
        for outs in 0..3 {
            let mut state = occupied(lineup.initial(), mask);
            state.outs = outs;
            assert_eq!(model.probabilities(&state, league).unwrap(), p);
        }
    }
}

fn activate_existing_entries(
    base: &preview::Baseline,
    lineup: &real::Lineup,
    ratings: &ratings::Ratings,
) {
    let catalog = "2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583";
    let keys = [
        "strikeout",
        "walk",
        "hbp",
        "single",
        "double",
        "triple",
        "home_run",
        "roe",
        "contact_out",
    ];
    let mut basic=format!("key,value\nformat,preview-counts-v1\ncatalog_sha256,{catalog}\nquery_version,preview-baseline-v1\nselected_pa,186517\nexcluded_ibb,753\nexcluded_interference,61\n");
    let mut full=format!("key,value\nformat,joint-proxy-counts-v1\ncohort_format,2019-joint-contact-proxies-v1\ncatalog_sha256,{catalog}\nsource_commit,b449f7bec49f852c27803e096303a4c78ac661b9\nsnapshot_sha256,{}\nsupplement_sha256,{}\nrecipe_sha256,174e9b959f59359f9fa86c7881dbbf9aefd5981f9376d08b60b975126ff4afc3\nartifact_sha256,1573fb845ac24d644ac722cc267b3729b2174fc114ff9f6748e2435502fe29ca\nincluded_pa,185703\nexcluded_ibb,753\nexcluded_interference,61\nattribution,{}\n",real::SNAPSHOT_SHA,real::SUPPLEMENT_SHA,joint::ATTRIBUTION);
    for (key, count) in keys.into_iter().zip(COUNTS) {
        basic.push_str(&format!("{key},{count}\n"));
        full.push_str(&format!("{key},{count}\n"));
    }
    for (key, count) in [
        "ground_candidate",
        "fly_candidate",
        "line_candidate",
        "pop_candidate",
        "residual",
    ]
    .into_iter()
    .zip([17188, 19528, 5379, 536, 39614])
    {
        full.push_str(&format!("{key},{count}\n"));
    }
    let parsed = preview::Baseline::read_csv(&basic).unwrap();
    assert_eq!(parsed.probabilities(), base.probabilities());
    let joint = joint::JointModel::read_csv(&full, base).unwrap();
    assert_eq!(joint.marginals(), *base.probabilities());
    assert_eq!(joint.conditionals().len(), 5);
    assert_eq!(joint.boundaries()[12], 1.0);
    assert_eq!(
        joint.sample_unit(joint.boundaries()[0]).unwrap().outcome,
        preview::Outcome::Walk
    );
    let args = vec!["native".to_string()];
    assert_eq!(joint::split_options(&args).unwrap(), (args.clone(), None));
    assert_eq!(real::split_options(&args).unwrap(), (args.clone(), None));
    assert_eq!(ratings::split_options(&args).unwrap(), (args, None));
    assert_eq!(
        [
            preview::MODEL_VERSION,
            preview::POLICY_VERSION,
            preview::RNG_VERSION,
            joint::MODEL_VERSION,
            joint::POLICY_VERSION,
            real::POLICY_VERSION
        ],
        [
            "pooled-jeffreys-preview-v1",
            "station-ground-contact-v1",
            "splitmix64-v1",
            "pooled-jeffreys-joint-proxy-preview-v1",
            "station-held-joint-proxy-preview-v1",
            "station-held-positional-starting-preview-v1"
        ]
    );
    let runs = [
        preview::game(1, base).unwrap(),
        joint::game(1, &joint).unwrap(),
        lineup.game(1, &joint).unwrap(),
        ratings.game(1, base, &joint, lineup).unwrap(),
    ];
    let repeats = [
        preview::game(1, base).unwrap(),
        joint::game(1, &joint).unwrap(),
        lineup.game(1, &joint).unwrap(),
        ratings.game(1, base, &joint, lineup).unwrap(),
    ];
    assert_eq!(runs, repeats);
    for game in runs {
        assert!(game.last().unwrap().after().over);
        for frame in game {
            assert_eq!(
                bbsim_engine::EventTransition::from_bytes(&frame.to_bytes()).unwrap(),
                frame
            );
        }
    }
    assert_eq!(lineup.names().len(), 20);
    let mut output = vec![];
    lineup.write_preamble(&mut output).unwrap();
    lineup.write_preamble_with_rates(&mut output, true).unwrap();
    ratings.write_preamble(&mut output, lineup, base).unwrap();
    let text = String::from_utf8(output).unwrap();
    assert!(text.contains("RATINGS FORMAT: player-proxy-counts-v1"));
    assert!(text.contains("not calibrated or predictive"));
}
