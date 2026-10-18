//! Characterize current support honestly; green here does not mean realistic generation.
#[path = "../examples/support/joint_contact.rs"]
mod joint;
#[path = "../examples/support/preview.rs"]
mod preview;
#[path = "../examples/support/player_ratings.rs"]
mod ratings;
#[path = "../examples/support/real_lineup.rs"]
mod real;
#[allow(dead_code)]
#[path = "../examples/support/season_pool.rs"]
mod season_pool;
use bbsim_engine::{
    apply, Base, Credit, CreditKind, Destination, EventKind, GameState, Half, Runner, RunnerMove,
    TeamState,
};
const COUNTS: [u64; 9] = [42823, 15142, 1984, 25947, 8531, 785, 6776, 1470, 82245];
fn initial() -> GameState {
    let team = |side: &str| TeamState {
        defense: bbsim_engine::DefenseState::LegacyUnassigned,
        batting_order: std::array::from_fn(|i| format!("{side}-{}", i + 1)),
        next_batter: 0,
        pitcher: (format!("{side}-pitcher")).into(),
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
            responsible_pitcher: state.teams[1 - state.half.offense()]
                .pitcher
                .active_id()
                .expect("legacy fixture pitcher")
                .to_owned(),
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

// Separately preregistered standalone dispatch; all original controls above unchanged.
#[path = "../examples/support/conditional_hit.rs"]
mod conditional_hit;
#[path = "../examples/support/hit_training_counts.rs"]
mod hit_training_counts;
#[path = "../examples/support/joint_hit.rs"]
mod joint_hit;
use conditional_hit::{ConditionalHits, HitDraw};
use joint_hit::{Context, Kernel, Model, ModeledCause, TrainingCell};

fn dispatch_support(value: u8, mask: u8) -> Vec<Vec<u8>> {
    let mut result = Vec::new();
    for code in 0..64 {
        let targets = [code % 4 + 1, (code / 4) % 4 + 1, code / 16 + 1];
        let mut vector = vec![value];
        let mut valid = true;
        for (i, target) in targets.into_iter().enumerate() {
            if mask & (1 << i) == 0 {
                valid &= target == 1;
            } else {
                valid &= target > i as u8;
                vector.push(target);
            }
        }
        for i in 0..vector.len() {
            for j in i + 1..vector.len() {
                valid &= vector[i] < vector[j] || (vector[i] == 4 && vector[j] == 4);
            }
        }
        if valid {
            result.push(vector);
        }
    }
    result.sort();
    result
}

fn dispatch_cells() -> Vec<TrainingCell> {
    let mut text = format!("joint-hit-training-counts-v1\nproxy_endpoints_not_certified_causes\ttrue\nqualification_sha256\t{}\ntraining_events\t24\nvalue\tbase_mask\touts_before\tcounts\n", "0".repeat(64));
    for value in [1, 2] {
        for mask in 0..8 {
            for outs in 0..3 {
                let counts = match (value, mask, outs) {
                    (1, 2, 0) => vec![8, 0, 0],
                    (1, 2, 1) => vec![0, 16, 0],
                    _ => vec![0; dispatch_support(value, mask).len()],
                };
                text.push_str(&format!(
                    "{value}\t{mask}\t{outs}\t{}\n",
                    counts
                        .iter()
                        .map(u64::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                ));
            }
        }
    }
    let artifact = hit_training_counts::parse(text.as_bytes()).unwrap();
    assert_eq!(artifact.training_events, 24);
    assert_eq!(artifact.qualification_sha256, "0".repeat(64)); // metadata only
    artifact.cells
}

fn hit_draw(word: u64) -> Option<HitDraw> {
    Some(HitDraw {
        word,
        cause: ModeledCause::UnaidedNoMisplay,
    })
}

#[test]
fn conditional_dispatch_requires_an_explicit_model_and_preserves_literal_contextual_advances() {
    let cells = dispatch_cells();
    let mut before = occupied(initial(), 2);
    before.bases[1].as_mut().unwrap().responsible_pitcher = "prior-pitcher".into();
    let untouched = before.clone();
    for (model, destination, runs) in [
        (Model::UniformLegal, Destination::Home, 1),
        (Model::PooledMask, Destination::Third, 0),
        (Model::BaseOut, Destination::Second, 0),
    ] {
        let policy = ConditionalHits::new(&cells, model).unwrap();
        let result = policy.transition(&before, sample(3), hit_draw(3 << 62));
        assert!(
            result.is_ok(),
            "valid single dispatch refused: {:?}",
            result.err()
        );
        let transition = result.unwrap();
        assert_eq!(transition.event().delta.moves[1].to, destination);
        assert_eq!(transition.event().delta.runs, runs);
        assert_eq!(transition.event().delta.outs, 0);
        assert_eq!(transition.after().teams[0].next_batter, 1);
        assert_eq!(transition.after().score, [u16::from(runs), 0]);
        let stats = bbsim_engine::accounting::derive_accounting(&[transition]).unwrap();
        assert_eq!(stats.batters[0]["Away-1"].plate_appearances, 1);
        assert_eq!(stats.batters[0]["Away-1"].credited_hits, 1);
        assert_eq!(stats.batters[0]["Away-1"].credited_rbi, u32::from(runs));
        assert_eq!(stats.pitchers[1]["Home-pitcher"].batters_faced, 1);
        assert_eq!(stats.pitchers[1]["Home-pitcher"].runs_allowed, 0);
        assert_eq!(
            stats.pitchers[1]
                .get("prior-pitcher")
                .map_or(0, |p| p.runs_allowed),
            u32::from(runs)
        );
    }
    assert_eq!(before, untouched);
}

#[test]
fn conditional_dispatch_constructor_refuses_bad_or_latently_collapsed_contexts() {
    let good = dispatch_cells();
    let mut cases = vec![vec![], good[..47].to_vec()];
    let mut duplicate = good.clone();
    duplicate[47] = duplicate[0].clone();
    cases.push(duplicate);
    let mut shape = good.clone();
    shape[47].counts.push(0);
    cases.push(shape);
    let mut context = good.clone();
    context[47].context.outs = 3;
    cases.push(context);
    let mut overflow = good.clone();
    overflow[0].counts[0] = u64::MAX;
    cases.push(overflow);
    for cells in cases {
        for model in [Model::UniformLegal, Model::PooledMask, Model::BaseOut] {
            assert!(
                ConditionalHits::new(&cells, model).is_err(),
                "invalid training table accepted"
            );
        }
    }
    let mut collapsed = good;
    for cell in &mut collapsed {
        cell.counts.fill(0);
    }
    // A late, non-requested context still has to be representable at construction.
    let c = Context {
        value: 2,
        mask: 7,
        outs: 2,
    };
    let cell = collapsed.iter_mut().find(|cell| cell.context == c).unwrap();
    cell.counts[0] = u64::MAX;
    assert!(Kernel::fit(
        &collapsed,
        Context {
            value: 1,
            mask: 0,
            outs: 0
        },
        Model::BaseOut
    )
    .is_ok());
    assert!(ConditionalHits::new(&collapsed, Model::BaseOut).is_err());
    assert!(ConditionalHits::new(&collapsed, Model::UniformLegal).is_ok());
}

#[test]
fn conditional_dispatch_has_exact_finite_taxonomy_and_canonical_accounting() {
    let cells = dispatch_cells();
    let mut frames = 0;
    for model in [Model::UniformLegal, Model::PooledMask, Model::BaseOut] {
        let policy = ConditionalHits::new(&cells, model).unwrap();
        for half in [Half::Top, Half::Bottom] {
            for mask in 0..8 {
                for outs in 0..3 {
                    let mut before = initial();
                    before.half = half;
                    before = occupied(before, mask);
                    before.outs = outs;
                    let untouched = before.clone();
                    for index in 0..13 {
                        let is_hit = matches!(index, 3 | 4);
                        for word in if is_hit { vec![0, u64::MAX] } else { vec![0] } {
                            let result = policy.transition(
                                &before,
                                sample(index),
                                if is_hit { hit_draw(word) } else { None },
                            );
                            assert!(result.is_ok(), "dispatch refused model {model:?} shape {index} mask {mask} outs {outs}: {:?}", result.err());
                            let transition = result.unwrap();
                            if is_hit {
                                let context = Context {
                                    value: if index == 3 { 1 } else { 2 },
                                    mask: mask as u8,
                                    outs,
                                };
                                let kernel = Kernel::fit(&cells, context, model).unwrap();
                                assert_eq!(
                                    kernel.vectors(),
                                    dispatch_support(context.value, context.mask)
                                );
                                assert!(kernel
                                    .probabilities()
                                    .iter()
                                    .all(|q| q.numerator > 0 && q.denominator >= q.numerator));
                                assert_eq!(kernel.masses().iter().sum::<u64>(), 1 << 53);
                                assert_eq!(kernel.edges().last(), Some(&(1 << 53)));
                                assert_eq!(
                                    transition,
                                    kernel
                                        .project(&before, word, ModeledCause::UnaidedNoMisplay)
                                        .unwrap()
                                );
                                let targets = if word == 0 {
                                    dispatch_support(context.value, context.mask)[0].clone()
                                } else {
                                    dispatch_support(context.value, context.mask)
                                        .last()
                                        .unwrap()
                                        .clone()
                                };
                                let expected_runs =
                                    targets.iter().filter(|target| **target == 4).count() as u8;
                                assert_eq!(transition.event().delta.runs, expected_runs);
                                assert_eq!(transition.event().delta.outs, 0);
                                assert_eq!(
                                    transition.event().fielding_review,
                                    bbsim_engine::FieldingReview::NoMisplay
                                );
                                for (movement, target) in
                                    transition.event().delta.moves.iter().zip(targets)
                                {
                                    assert_eq!(
                                        movement.to,
                                        [
                                            Destination::First,
                                            Destination::Second,
                                            Destination::Third,
                                            Destination::Home
                                        ][usize::from(target - 1)]
                                    );
                                }
                                let stats = bbsim_engine::accounting::derive_accounting(
                                    std::slice::from_ref(&transition),
                                )
                                .unwrap();
                                let offense = half.offense();
                                let batter = &before.teams[offense].batting_order[0];
                                assert_eq!(
                                    stats.batters[offense][batter].credited_rbi,
                                    u32::from(expected_runs)
                                );
                                assert_eq!(stats.batters[offense][batter].plate_appearances, 1);
                            } else {
                                assert_eq!(
                                    transition.to_bytes(),
                                    apply(
                                        &before,
                                        joint::event_for(&before, sample(index)).unwrap()
                                    )
                                    .unwrap()
                                    .to_bytes()
                                );
                            }
                            assert_eq!(transition.before(), &before);
                            assert_eq!(transition.after().teams[half.offense()].next_batter, 1);
                            assert_eq!(
                                bbsim_engine::EventTransition::from_bytes(&transition.to_bytes())
                                    .unwrap(),
                                transition
                            );
                            assert_eq!(
                                apply(&before, transition.event().clone()).unwrap(),
                                transition
                            );
                            assert_eq!(before, untouched);
                            frames += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(frames, 2160);
}

// Independent indexed SplitMix64 oracle; no use of the mutable producer RNG.
fn dispatch_reference_word(seed: u64, ordinal: u64) -> u64 {
    let mut z = seed.wrapping_add(0x9e3779b97f4a7c15u64.wrapping_mul(ordinal));
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

fn dispatch_prefix(
    seed: u64,
    policy: &ConditionalHits,
    sampler: &joint::JointModel,
) -> (Vec<bbsim_engine::EventTransition>, u64) {
    let mut state = initial();
    let mut rng = preview::Rng::new(seed);
    let mut words = 0;
    let mut log = Vec::new();
    while !(state.over || state.half == Half::Bottom && state.inning >= 9) {
        assert!(
            log.len() < 1000,
            "seed {seed}: censored-prefix resource ceiling"
        );
        let sample = sampler.sample(&mut rng);
        words += 1;
        let unit = (dispatch_reference_word(seed, words) >> 11) as f64 / (1u64 << 53) as f64;
        assert_eq!(
            sample,
            sampler.sample_unit(unit).unwrap(),
            "seed {seed}: PA word drift"
        );
        let draw = if matches!(
            sample.outcome,
            preview::Outcome::Single | preview::Outcome::Double
        ) {
            let word = rng.next_u64();
            words += 1;
            assert_eq!(
                word,
                dispatch_reference_word(seed, words),
                "seed {seed}: hit word drift"
            );
            hit_draw(word)
        } else {
            None
        };
        let transition = policy
            .transition(&state, sample, draw)
            .unwrap_or_else(|error| panic!("seed {seed}, frame {}: {error}", log.len() + 1));
        assert_eq!(transition.before(), &state);
        assert_eq!(
            apply(&state, transition.event().clone()).unwrap(),
            transition
        );
        assert_eq!(
            bbsim_engine::EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
            transition
        );
        state = transition.after().clone();
        log.push(transition);
    }
    assert!(!log.is_empty());
    assert!(state.over || (state.half == Half::Bottom && state.inning >= 9));
    assert!(policy.transition(&state, sample(0), None).is_err());
    // This diagnostic check-word is outside the censored generation prefix.
    assert_eq!(rng.next_u64(), dispatch_reference_word(seed, words + 1));
    assert_eq!(
        words,
        log.len() as u64
            + log
                .iter()
                .filter(|frame| frame
                    .event()
                    .delta
                    .credits
                    .iter()
                    .any(|credit| matches!(credit.kind, CreditKind::Single | CreditKind::Double)))
                .count() as u64
    );
    let stats = bbsim_engine::accounting::derive_accounting(&log).unwrap();
    assert_eq!(stats.runs, state.score.map(u32::from));
    for side in 0..2 {
        let pas = log
            .iter()
            .filter(|frame| frame.before().half.offense() == side)
            .count() as u32;
        let outs: u32 = log
            .iter()
            .filter(|frame| frame.before().half.offense() == side)
            .map(|frame| u32::from(frame.event().delta.outs))
            .sum();
        assert_eq!(
            stats.batters[side]
                .values()
                .map(|line| line.plate_appearances)
                .sum::<u32>(),
            pas
        );
        assert_eq!(
            stats.pitchers[1 - side]
                .values()
                .map(|line| line.batters_faced)
                .sum::<u32>(),
            pas
        );
        assert_eq!(
            stats.pitchers[1 - side]
                .values()
                .map(|line| line.outs_recorded)
                .sum::<u32>(),
            outs
        );
        assert_eq!(
            stats.batters[side]
                .values()
                .map(|line| line.credited_runs)
                .sum::<u32>(),
            stats.runs[side]
        );
        assert_eq!(
            stats.pitchers[1 - side]
                .values()
                .map(|line| line.runs_allowed)
                .sum::<u32>(),
            stats.runs[side]
        );
    }
    (log, words)
}

#[test]
fn conditional_dispatch_fixed_seed_prefixes_preserve_words_continuity_and_available_stats() {
    let base = preview::Baseline::from_counts([4, 2, 1, 4, 2, 1, 1, 1, 9]).unwrap();
    let sampler = joint::JointModel::from_counts(&base, [2, 2, 2, 1, 2]).unwrap();
    let cells = dispatch_cells();
    let mut prefixes = 0;
    for model in [Model::UniformLegal, Model::PooledMask, Model::BaseOut] {
        let policy = ConditionalHits::new(&cells, model).unwrap();
        for seed in 1..=128 {
            let first = dispatch_prefix(seed, &policy, &sampler);
            let repeat = dispatch_prefix(seed, &policy, &sampler);
            assert_eq!(first, repeat, "seed {seed}, model {model:?}: prefix drift");
            prefixes += 1;
        }
    }
    assert_eq!(prefixes, 384);
}

#[test]
fn conditional_dispatch_refuses_bad_draws_samples_states_and_terminal_boundaries() {
    let policy = ConditionalHits::new(&dispatch_cells(), Model::BaseOut).unwrap();
    let before = initial();
    assert!(policy.transition(&before, sample(3), None).is_err());
    assert!(policy.transition(&before, sample(4), None).is_err());
    assert!(policy
        .transition(
            &before,
            sample(3),
            Some(HitDraw {
                word: 0,
                cause: ModeledCause::Unknown
            })
        )
        .is_err());
    for index in (0..13).filter(|index| !matches!(index, 3 | 4)) {
        assert!(policy
            .transition(&before, sample(index), hit_draw(0))
            .is_err());
    }
    for broken in [
        joint::Sample {
            outcome: preview::Outcome::Single,
            contact: Some(joint::ContactKind::Ground),
        },
        joint::Sample {
            outcome: preview::Outcome::ContactOut,
            contact: None,
        },
    ] {
        assert!(policy.transition(&before, broken, hit_draw(0)).is_err());
    }
    let mut states = vec![];
    let mut ended = before.clone();
    ended.over = true;
    states.push(ended);
    let mut inning = before.clone();
    inning.inning = 0;
    states.push(inning);
    let mut outs = before.clone();
    outs.outs = 3;
    states.push(outs);
    let mut cursor = before.clone();
    cursor.teams[0].next_batter = 9;
    states.push(cursor);
    let mut identity = occupied(before.clone(), 1);
    identity.bases[0].as_mut().unwrap().player_id = "not-active".into();
    states.push(identity);
    for state in states {
        let original = state.clone();
        assert!(policy.transition(&state, sample(3), hit_draw(0)).is_err());
        assert!(policy.transition(&state, sample(0), None).is_err());
        assert_eq!(state, original);
    }
    for inning in [9, 10, u16::MAX] {
        for outs in 0..3 {
            for score in [[0, 0], [1, 0], [0, 1]] {
                for cursor in [0, 8] {
                    let mut state = occupied(initial(), 7);
                    state.half = Half::Bottom;
                    state.inning = inning;
                    state.outs = outs;
                    state.score = score;
                    // Rebind occupied identities to the actual offense, without changing rules.
                    state = occupied(state, 7);
                    state.teams[1].next_batter = cursor;
                    let original = state.clone();
                    for index in 0..13 {
                        let result = policy.transition(
                            &state,
                            sample(index),
                            if matches!(index, 3 | 4) {
                                hit_draw(0)
                            } else {
                                None
                            },
                        );
                        assert!(result.is_err(), "Bottom>=9 accepted");
                    }
                    assert_eq!(state, original);
                }
            }
        }
    }
}

// Preregistered existing-behavior diagnostic; no terminal producer or policy.
fn terminal_fixture(mask: u8, outs: u8, inning: u16, deficit: u8) -> GameState {
    let mut state = initial();
    state.half = Half::Bottom;
    state = occupied(state, usize::from(mask));
    state.inning = inning;
    state.outs = outs;
    state.score = [u16::from(deficit), 0];
    for (i, runner) in state.bases.iter_mut().enumerate() {
        if let Some(runner) = runner {
            runner.responsible_pitcher = format!("prior-pitcher-{i}");
        }
    }
    state
}
fn terminal_candidate(before: &GameState, value: u8, vector: &[u8]) -> bbsim_engine::Event {
    use bbsim_engine::{Delta, Event, FieldingReview, Participants, SoleSafeHitFacts};
    let batter = before.teams[1].batting_order[0].clone();
    let pitcher = before.teams[0]
        .pitcher
        .active_id()
        .expect("legacy fixture pitcher")
        .to_owned();
    let credit = |player: &str, kind| Credit {
        player_id: player.to_owned(),
        kind,
    };
    let destinations = [
        Destination::First,
        Destination::Second,
        Destination::Third,
        Destination::Home,
    ];
    let mut moves = vec![RunnerMove {
        runner: Runner {
            player_id: batter.clone(),
            responsible_pitcher: pitcher.clone(),
        },
        from: Base::Plate,
        to: destinations[usize::from(vector[0] - 1)],
    }];
    let mut credits = vec![
        credit(&batter, CreditKind::PlateAppearance),
        credit(&batter, CreditKind::Hit),
        credit(
            &batter,
            if value == 1 {
                CreditKind::Single
            } else {
                CreditKind::Double
            },
        ),
    ];
    let mut runs = 0;
    let mut position = 1;
    for (i, origin) in [Base::First, Base::Second, Base::Third]
        .into_iter()
        .enumerate()
    {
        if let Some(runner) = &before.bases[i] {
            let target = destinations[usize::from(vector[position] - 1)];
            moves.push(RunnerMove {
                runner: runner.clone(),
                from: origin,
                to: target,
            });
            if target == Destination::Home {
                runs += 1;
                credits.extend([
                    credit(&runner.player_id, CreditKind::Run),
                    credit(&batter, CreditKind::Rbi),
                ]);
            }
            position += 1;
        }
    }
    Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(batter.clone()),
            pitcher: Some(pitcher),
        },
        delta: Delta {
            moves,
            outs: 0,
            runs,
            credits,
            changes: vec![],
            third_out: None,
        },
        fielding_review: FieldingReview::NoMisplay,
        sole_safe_hit: Some(SoleSafeHitFacts { batter_id: batter }),
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        ordinary_ground_throw: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        muffed_ground_ball: None,
        ordinary_air_catch: None,
    }
}
fn check_terminal_positive(transition: &bbsim_engine::EventTransition, walkoff: bool) {
    let before = transition.before();
    let event = transition.event();
    let after = transition.after();
    assert_eq!(after.over, walkoff);
    assert_eq!(after.inning, before.inning);
    assert_eq!(after.half, before.half);
    assert_eq!(after.outs, before.outs);
    assert_eq!(after.score, [before.score[0], u16::from(event.delta.runs)]);
    assert_eq!(after.teams[1].next_batter, 1);
    assert_eq!(apply(before, event.clone()).unwrap(), *transition);
    assert_eq!(
        bbsim_engine::EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
        *transition
    );
    let stats =
        bbsim_engine::accounting::derive_accounting(std::slice::from_ref(transition)).unwrap();
    let batter = &stats.batters[1]["Home-1"];
    assert_eq!(
        (
            batter.plate_appearances,
            batter.credited_hits,
            batter.credited_rbi
        ),
        (1, 1, u32::from(event.delta.runs))
    );
    assert_eq!(batter.classified_rbi, Some(u32::from(event.delta.runs)));
    assert_eq!(batter.credited_hit_breakdown.as_ref().unwrap().hits(), 1);
    let value = event
        .delta
        .moves
        .iter()
        .find(|movement| movement.from == Base::Plate)
        .unwrap()
        .to;
    assert_eq!(
        batter
            .credited_hit_breakdown
            .as_ref()
            .unwrap()
            .total_bases(),
        if value == Destination::First { 1 } else { 2 }
    );
    assert_eq!(stats.pitchers[0]["Away-pitcher"].batters_faced, 1);
    assert_eq!(stats.pitchers[0]["Away-pitcher"].runs_allowed, 0);
    for movement in &event.delta.moves {
        if movement.from != Base::Plate && movement.to == Destination::Home {
            assert_eq!(
                stats.batters[1][&movement.runner.player_id].credited_runs,
                1
            );
            assert_eq!(
                stats.pitchers[0][&movement.runner.responsible_pitcher].runs_allowed,
                1
            );
        }
        if let Some(index) = [Destination::First, Destination::Second, Destination::Third]
            .iter()
            .position(|target| *target == movement.to)
        {
            assert_eq!(after.bases[index].as_ref(), Some(&movement.runner));
        }
    }
}
#[test]
fn terminal_compatibility_full_vectors_have_exact_canonical_partition_and_order_invariance() {
    use bbsim_engine::TransitionError;
    let mut logical = 0;
    let mut attempts = 0;
    let mut rows = 0;
    for value in [1, 2] {
        for mask in 0..8 {
            let origins: Vec<u8> = (1..=3)
                .filter(|origin| mask & (1 << (origin - 1)) != 0)
                .collect();
            let vectors = dispatch_support(value, mask);
            for outs in 0..3 {
                for inning in [9, 10] {
                    for deficit in 0..4 {
                        let before = terminal_fixture(mask, outs, inning, deficit);
                        let original = before.clone();
                        // continue, walkoff, IllegalCredit, UnsupportedRule; overlapping reasons.
                        let mut counts = [0_u64; 7];
                        for vector in &vectors {
                            let event = terminal_candidate(&before, value, vector);
                            let runs = event.delta.runs;
                            let winning = runs > deficit;
                            let ceilings: Vec<u8> = origins
                                .iter()
                                .zip(&vector[1..])
                                .filter_map(|(origin, target)| (*target == 4).then_some(4 - origin))
                                .collect();
                            let definite =
                                winning && ceilings.iter().all(|ceiling| value > *ceiling);
                            let ambiguous = winning
                                && !definite
                                && ceilings.iter().any(|ceiling| value > *ceiling);
                            let excess = winning && runs != deficit + 1;
                            let expected = if !winning {
                                0
                            } else if definite {
                                2
                            } else if ambiguous || excess {
                                3
                            } else {
                                1
                            };
                            counts[expected] += 1;
                            counts[4] += u64::from(definite);
                            counts[5] += u64::from(ambiguous);
                            counts[6] += u64::from(excess);
                            for reverse in [false, true] {
                                let mut candidate = event.clone();
                                if reverse {
                                    candidate.delta.moves.reverse();
                                    candidate.delta.credits.reverse();
                                }
                                match (expected, apply(&before, candidate)) {
                                    (0 | 1, Ok(transition)) => check_terminal_positive(&transition, expected == 1),
                                    (2, Err(TransitionError::IllegalCredit)) | (3, Err(TransitionError::UnsupportedRule)) => {},
                                    (_, actual) => panic!("value={value} mask={mask} outs={outs} inning={inning} deficit={deficit} vector={vector:?} reverse={reverse} expected={expected} actual={actual:?}"),
                                }
                                assert_eq!(before, original);
                                attempts += 1;
                            }
                            logical += 1;
                        }
                        assert_eq!(counts[..4].iter().sum::<u64>() as usize, vectors.len());
                        println!("TERMINAL_COMPAT {value} {mask} {outs} {inning} {deficit} {} {} {} {} {} {} {}",
                            counts[0], counts[1], counts[2], counts[3], counts[4], counts[5], counts[6]);
                        rows += 1;
                    }
                }
            }
        }
    }
    assert_eq!((logical, attempts, rows), (960, 1920, 384));
    println!("TERMINAL_TOTAL {logical} {attempts} {rows}");
}
#[test]
fn terminal_compatibility_literal_stopping_and_hit_ceiling_counterexamples() {
    use bbsim_engine::TransitionError;
    for (mask, value, deficit, vector, error) in [
        (
            6,
            1,
            0,
            vec![1, 4, 4],
            Some(TransitionError::UnsupportedRule),
        ),
        (4, 2, 0, vec![2, 4], Some(TransitionError::IllegalCredit)),
        (2, 2, 0, vec![2, 4], None),
        // Exactly two needed runs, but supplied movement order cannot certify which wins.
        (
            6,
            2,
            1,
            vec![2, 4, 4],
            Some(TransitionError::UnsupportedRule),
        ),
    ] {
        for inning in [9, 10] {
            for outs in 0..3 {
                let before = terminal_fixture(mask, outs, inning, deficit);
                let event = terminal_candidate(&before, value, &vector);
                for reverse in [false, true] {
                    let mut candidate = event.clone();
                    if reverse {
                        candidate.delta.moves.reverse();
                        candidate.delta.credits.reverse();
                    }
                    match error {
                        Some(ref expected) => {
                            assert_eq!(apply(&before, candidate), Err(expected.clone()))
                        }
                        None => check_terminal_positive(&apply(&before, candidate).unwrap(), true),
                    }
                }
            }
        }
    }
}
#[test]
fn terminal_compatibility_retains_all_model_boundaries_and_top_extra_controls() {
    use joint_hit::{Context, Kernel, Model, ModeledCause};
    let cells = dispatch_cells();
    let mut boundaries = 0;
    let mut top_controls = 0;
    for model in [Model::UniformLegal, Model::PooledMask, Model::BaseOut] {
        let policy = conditional_hit::ConditionalHits::new(&cells, model).unwrap();
        for value in [1, 2] {
            for mask in 0..8 {
                for outs in 0..3 {
                    let kernel = Kernel::fit(&cells, Context { value, mask, outs }, model).unwrap();
                    for inning in [9, 10, u16::MAX] {
                        for deficit in 0..4 {
                            let state = terminal_fixture(mask, outs, inning, deficit);
                            let original = state.clone();
                            for word in [0, u64::MAX] {
                                assert!(kernel
                                    .project(&state, word, ModeledCause::UnaidedNoMisplay)
                                    .is_err());
                                assert!(policy
                                    .transition(
                                        &state,
                                        sample(if value == 1 { 3 } else { 4 }),
                                        hit_draw(word)
                                    )
                                    .is_err());
                                assert_eq!(state, original);
                                boundaries += 1;
                            }
                        }
                    }
                    for inning in [9, 10] {
                        let mut state = terminal_fixture(mask, outs, inning, 0);
                        state.half = Half::Top;
                        state = occupied(state, usize::from(mask));
                        for word in [0, u64::MAX] {
                            let expected = kernel
                                .project(&state, word, ModeledCause::UnaidedNoMisplay)
                                .unwrap();
                            assert!(!expected.after().over);
                            assert_eq!(
                                policy
                                    .transition(
                                        &state,
                                        sample(if value == 1 { 3 } else { 4 }),
                                        hit_draw(word)
                                    )
                                    .unwrap(),
                                expected
                            );
                            top_controls += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!((boundaries, top_controls), (3456, 576));
}
