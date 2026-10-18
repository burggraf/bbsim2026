//! Synthetic standalone prerequisite; no real counts, source causes or game integration.
#[path = "../examples/support/joint_hit.rs"]
mod joint_hit;

/// Independent Cartesian destinations plus all pairwise order constraints.
fn reference_support(value: u8, mask: u8) -> Vec<Vec<u8>> {
    let mut result = Vec::new();
    for code in 0u8..64 {
        let targets = [code % 4 + 1, (code / 4) % 4 + 1, code / 16 + 1];
        if targets
            .iter()
            .enumerate()
            .any(|(i, t)| mask & (1 << i) == 0 && *t != 1)
        {
            continue;
        }
        let mut vector = vec![value];
        let mut valid = true;
        for (i, target) in targets.iter().enumerate() {
            if mask & (1 << i) != 0 {
                valid &= *target > i as u8;
                vector.push(*target);
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

#[test]
fn full_joint_support_matches_independent_cartesian_pairwise_geometry() {
    let mut total = 0;
    for value in [1, 2] {
        for mask in 0..8 {
            let actual = joint_hit::legal_vectors(value, mask).unwrap();
            let expected = reference_support(value, mask);
            assert_eq!(actual, expected, "value={value}, mask={mask}");
            assert!(!actual.is_empty());
            assert!(actual.windows(2).all(|pair| pair[0] < pair[1]));
            total += actual.len();
        }
    }
    assert_eq!(total, 40);
    assert_eq!(joint_hit::legal_vectors(1, 0).unwrap(), vec![vec![1]]);
    assert_eq!(
        joint_hit::legal_vectors(1, 2).unwrap(),
        vec![vec![1, 2], vec![1, 3], vec![1, 4]]
    );
    assert_eq!(
        joint_hit::legal_vectors(2, 1).unwrap(),
        vec![vec![2, 3], vec![2, 4]]
    );
}

#[test]
fn invalid_typed_hit_values_and_masks_are_rejected_not_coerced() {
    for (value, mask) in [(0, 0), (3, 0), (255, 0), (1, 8), (2, 255)] {
        assert!(joint_hit::legal_vectors(value, mask).is_err());
    }
}

use bbsim_engine::{
    accounting::derive_accounting, EventTransition, FieldingReview, GameState, Half, Runner,
    TeamState,
};
use joint_hit::{Context, Kernel, Model, ModeledCause, Ratio, TrainingCell};

fn context(value: u8, mask: u8, outs: u8) -> Context {
    Context { value, mask, outs }
}

fn training() -> Vec<TrainingCell> {
    let mut cells = Vec::new();
    for value in [1, 2] {
        for mask in 0..8 {
            for outs in 0..3 {
                cells.push(TrainingCell {
                    context: context(value, mask, outs),
                    counts: vec![0; reference_support(value, mask).len()],
                });
            }
        }
    }
    cells
}

fn set_counts(cells: &mut [TrainingCell], c: Context, counts: &[u64]) {
    cells
        .iter_mut()
        .find(|cell| cell.context == c)
        .unwrap()
        .counts = counts.to_vec();
}

fn ratio(numerator: u128, denominator: u128) -> Ratio {
    Ratio {
        numerator,
        denominator,
    }
}

#[test]
fn exact_literal_pooled_and_base_out_half_priors_include_unobserved_categories() {
    let mut cells = training();
    set_counts(&mut cells, context(1, 1, 0), &[3, 0, 0]);
    set_counts(&mut cells, context(1, 1, 1), &[0, 2, 0]);
    for (outs, expected) in [
        (0, vec![ratio(7, 9), ratio(1, 9), ratio(1, 9)]),
        (1, vec![ratio(1, 7), ratio(5, 7), ratio(1, 7)]),
        (2, vec![ratio(1, 3); 3]),
    ] {
        let c = context(1, 1, outs);
        let uniform = Kernel::fit(&cells, c, Model::UniformLegal).unwrap();
        let pooled = Kernel::fit(&cells, c, Model::PooledMask).unwrap();
        let exact = Kernel::fit(&cells, c, Model::BaseOut).unwrap();
        assert_eq!(uniform.probabilities(), &[ratio(1, 3); 3]);
        assert_eq!(
            pooled.probabilities(),
            &[ratio(7, 13), ratio(5, 13), ratio(1, 13)]
        );
        assert_eq!(exact.probabilities(), expected);
        assert_eq!(exact.vectors(), reference_support(1, 1));
    }
}

#[test]
fn all_zero_training_contexts_have_full_positive_uniform_support_for_every_variant() {
    let cells = training();
    for cell in &cells {
        let k = cell.counts.len();
        for model in [Model::UniformLegal, Model::PooledMask, Model::BaseOut] {
            let kernel = Kernel::fit(&cells, cell.context, model).unwrap();
            assert_eq!(kernel.probabilities(), vec![ratio(1, k as u128); k]);
            assert_eq!(
                kernel.vectors(),
                reference_support(cell.context.value, cell.context.mask)
            );
        }
    }
}

#[test]
fn count_table_shapes_duplicates_domains_and_u64_aggregation_refuse_without_coercion() {
    let good = training();
    let c = context(1, 1, 0);
    let mut cases = vec![Vec::new(), good[..47].to_vec()];
    let mut duplicate = good.clone();
    duplicate[1] = duplicate[0].clone();
    cases.push(duplicate);
    for invalid in [
        context(0, 0, 0),
        context(3, 0, 0),
        context(1, 8, 0),
        context(1, 0, 3),
    ] {
        let mut cells = good.clone();
        cells[0].context = invalid;
        cases.push(cells);
        assert!(Kernel::fit(&good, invalid, Model::BaseOut).is_err());
    }
    let mut empty = good.clone();
    empty[0].counts.clear();
    cases.push(empty);
    let mut extra = good.clone();
    extra[0].counts.push(0);
    cases.push(extra);
    let mut overflow = good.clone();
    set_counts(&mut overflow, c, &[u64::MAX, 1, 0]);
    cases.push(overflow);
    let mut pooled_overflow = good.clone();
    set_counts(&mut pooled_overflow, c, &[u64::MAX, 0, 0]);
    set_counts(&mut pooled_overflow, context(1, 1, 1), &[1, 0, 0]);
    cases.push(pooled_overflow);
    for cells in cases {
        for model in [Model::UniformLegal, Model::PooledMask, Model::BaseOut] {
            assert!(Kernel::fit(&cells, c, model).is_err());
        }
    }
    let mut maximum = good;
    set_counts(&mut maximum, context(1, 0, 0), &[u64::MAX]);
    assert_eq!(
        Kernel::fit(&maximum, context(1, 0, 0), Model::BaseOut)
            .unwrap()
            .probabilities(),
        &[ratio(1, 1)]
    );
}

#[test]
fn table_permutation_is_exact_and_other_value_mask_counts_do_not_change_a_context() {
    let mut cells = training();
    set_counts(&mut cells, context(1, 1, 0), &[4, 0, 1]);
    let original = cells.clone();
    for model in [Model::UniformLegal, Model::PooledMask, Model::BaseOut] {
        let expected = Kernel::fit(&cells, context(1, 1, 0), model).unwrap();
        let mut changed = cells.clone();
        changed.reverse();
        assert_eq!(
            Kernel::fit(&changed, context(1, 1, 0), model)
                .unwrap()
                .probabilities(),
            expected.probabilities()
        );
        set_counts(&mut changed, context(2, 7, 2), &[1, 2]);
        // Use independent support length, not a hardcoded sparse observed category set.
        let cell = changed
            .iter_mut()
            .find(|cell| cell.context == context(2, 7, 2))
            .unwrap();
        cell.counts = vec![9; reference_support(2, 7).len()];
        assert_eq!(
            Kernel::fit(&changed, context(1, 1, 0), model)
                .unwrap()
                .probabilities(),
            expected.probabilities()
        );
    }
    assert_eq!(cells, original);
}

const GRID: u64 = 1 << 53;

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// Independent reduced-fraction CDF, not the implementation's common count denominator.
fn reference_edges(probabilities: &[Ratio]) -> Vec<u64> {
    let (mut n, mut d) = (0u128, 1u128);
    probabilities
        .iter()
        .map(|q| {
            let g = gcd(d, q.denominator);
            n = n * (q.denominator / g) + q.numerator * (d / g);
            d *= q.denominator / g;
            let g = gcd(n, d);
            n /= g;
            d /= g;
            u64::try_from((n * u128::from(GRID)).div_ceil(d)).unwrap()
        })
        .collect()
}

fn varied_training() -> Vec<TrainingCell> {
    let mut cells = training();
    for cell in &mut cells {
        for (j, count) in cell.counts.iter_mut().enumerate() {
            *count = 7 * (j as u64 + 1) * u64::from(cell.context.outs + 1)
                + u64::from(cell.context.value * cell.context.mask);
        }
    }
    cells
}

/// Additional coverage characterization, not a newly claimed fitting defect.
#[test]
fn independently_calculated_training_ratios_match_every_model_context() {
    let cells = varied_training();
    for cell in &cells {
        for model in [Model::UniformLegal, Model::PooledMask, Model::BaseOut] {
            let counts: Vec<u64> = (0..cell.counts.len())
                .map(|j| match model {
                    Model::UniformLegal => 0,
                    Model::BaseOut => cell.counts[j],
                    Model::PooledMask => cells
                        .iter()
                        .filter(|peer| {
                            peer.context.value == cell.context.value
                                && peer.context.mask == cell.context.mask
                        })
                        .map(|peer| peer.counts[j])
                        .sum(),
                })
                .collect();
            let denominator = u128::from(counts.iter().sum::<u64>()) * 2 + counts.len() as u128;
            let expected: Vec<Ratio> = counts
                .iter()
                .map(|count| {
                    let n = u128::from(*count) * 2 + 1;
                    let g = gcd(n, denominator);
                    ratio(n / g, denominator / g)
                })
                .collect();
            assert_eq!(
                Kernel::fit(&cells, cell.context, model)
                    .unwrap()
                    .probabilities(),
                expected
            );
        }
    }
}

#[test]
fn every_model_context_has_independent_exact_positive_conserving_grid_masses() {
    let cells = varied_training();
    for cell in &cells {
        for model in [Model::UniformLegal, Model::PooledMask, Model::BaseOut] {
            let kernel = Kernel::fit(&cells, cell.context, model).unwrap();
            assert_eq!(kernel.edges(), reference_edges(kernel.probabilities()));
            assert_eq!(kernel.edges().last(), Some(&GRID));
            let masses = kernel.masses();
            assert_eq!(masses.len(), kernel.vectors().len());
            assert_eq!(masses.iter().sum::<u64>(), GRID);
            assert!(masses.iter().all(|mass| *mass > 0));
            for (mass, q) in masses.iter().zip(kernel.probabilities()) {
                let quantized = u128::from(*mass) * q.denominator;
                let exact = u128::from(GRID) * q.numerator;
                assert!(quantized.abs_diff(exact) < q.denominator);
            }
        }
    }
}

#[test]
fn strict_integer_edges_high53_bits_and_literal_thirds_have_no_hidden_rng() {
    let cells = training();
    let thirds = Kernel::fit(&cells, context(1, 1, 0), Model::UniformLegal).unwrap();
    assert_eq!(
        thirds.edges(),
        &[3_002_399_751_580_331, 6_004_799_503_160_662, GRID]
    );
    assert_eq!(
        thirds.masses(),
        vec![
            3_002_399_751_580_331,
            3_002_399_751_580_331,
            3_002_399_751_580_330
        ]
    );
    for cell in &varied_training() {
        for model in [Model::UniformLegal, Model::PooledMask, Model::BaseOut] {
            let kernel = Kernel::fit(&varied_training(), cell.context, model).unwrap();
            assert_eq!(kernel.sample(0), kernel.vectors()[0]);
            assert_eq!(kernel.sample(u64::MAX), kernel.vectors().last().unwrap());
            for (j, edge) in kernel.edges().iter().enumerate() {
                let word = (edge - 1) << 11;
                assert_eq!(kernel.sample(word), kernel.vectors()[j]);
                assert_eq!(kernel.sample(word | 2047), kernel.sample(word));
                assert_eq!(kernel.sample(word), kernel.sample(word));
                if *edge < GRID {
                    assert_eq!(kernel.sample(edge << 11), kernel.vectors()[j + 1]);
                }
            }
        }
    }
}

#[test]
fn collapsed_intervals_refuse_without_clipping_or_removing_legal_categories() {
    for counts in [[u64::MAX, 0, 0], [0, u64::MAX, 0], [0, 0, u64::MAX]] {
        let mut cells = training();
        set_counts(&mut cells, context(1, 1, 0), &counts);
        assert!(Kernel::fit(&cells, context(1, 1, 0), Model::UniformLegal).is_ok());
        for model in [Model::PooledMask, Model::BaseOut] {
            assert!(Kernel::fit(&cells, context(1, 1, 0), model).is_err());
        }
    }
}

fn game(c: Context, half: Half, dh: bool, cursor: u8, inherited: bool) -> GameState {
    let team = |prefix: &str| {
        let pitcher = format!("{prefix}p");
        let mut batting_order = std::array::from_fn(|i| format!("{prefix}{i}"));
        if !dh {
            batting_order[8] = pitcher.clone();
        }
        TeamState {
            defense: bbsim_engine::DefenseState::LegacyUnassigned,
            batting_order,
            next_batter: cursor,
            pitcher: (pitcher).into(),
            dh_slot: dh.then_some(8),
            dh_must_bat: false,
            bench: vec![],
            removed: vec![],
        }
    };
    let teams = [team("a"), team("h")];
    let offense = half.offense();
    let bases = std::array::from_fn(|i| {
        (c.mask & (1 << i) != 0).then(|| Runner {
            player_id: teams[offense].batting_order[(usize::from(cursor) + i + 3) % 9].clone(),
            responsible_pitcher: if inherited {
                format!("prior{i}")
            } else {
                teams[1 - offense]
                    .pitcher
                    .active_id()
                    .expect("legacy fixture pitcher")
                    .to_owned()
            },
        })
    });
    GameState {
        inning: 2,
        half,
        outs: c.outs,
        score: [0, 0],
        teams,
        bases,
        over: false,
    }
}

#[test]
fn all_joint_vectors_project_explicit_modeled_cause_actors_charges_codec_and_independent_accounting(
) {
    let cells = training();
    let mut frames = 0;
    for cell in &cells {
        let kernel = Kernel::fit(&cells, cell.context, Model::UniformLegal).unwrap();
        for half in [Half::Top, Half::Bottom] {
            for dh in [false, true] {
                for cursor in 0..9 {
                    for inherited in [false, true] {
                        let before = game(cell.context, half, dh, cursor, inherited);
                        for (j, vector) in reference_support(cell.context.value, cell.context.mask)
                            .iter()
                            .enumerate()
                        {
                            let draw = if j == 0 { 0 } else { kernel.edges()[j - 1] } << 11;
                            let transition = kernel
                                .project(&before, draw, ModeledCause::UnaidedNoMisplay)
                                .unwrap_or_else(|error| panic!("{error}\ncontext {:?}\nhalf {half:?}, dh {dh}, cursor {cursor}, inherited {inherited}\nvector {vector:?}\nnext batter {}\nbefore bases {:?}", cell.context, before.teams[half.offense()].batting_order[usize::from((cursor + 1) % 9)], before.bases));
                            assert_eq!(
                                transition,
                                kernel
                                    .project(&before, draw, ModeledCause::UnaidedNoMisplay)
                                    .unwrap()
                            );
                            assert_eq!(transition.before(), &before);
                            let event = transition.event();
                            let offense = half.offense();
                            let defense = 1 - offense;
                            let batter = &before.teams[offense].batting_order[usize::from(cursor)];
                            let pitcher = before.teams[defense].pitcher.active_id().unwrap();
                            assert_eq!(event.participants.batter.as_ref(), Some(batter));
                            assert_eq!(event.participants.pitcher.as_deref().unwrap(), pitcher);
                            assert_eq!(event.fielding_review, FieldingReview::NoMisplay);
                            assert_eq!(&event.sole_safe_hit.as_ref().unwrap().batter_id, batter);
                            assert_eq!(event.delta.outs, 0);
                            assert!(event.delta.changes.is_empty());
                            let targets: Vec<u8> = event
                                .delta
                                .moves
                                .iter()
                                .map(|m| match m.to {
                                    bbsim_engine::Destination::First => 1,
                                    bbsim_engine::Destination::Second => 2,
                                    bbsim_engine::Destination::Third => 3,
                                    bbsim_engine::Destination::Home => 4,
                                    bbsim_engine::Destination::Out => {
                                        panic!("safe kernel produced an out")
                                    }
                                })
                                .collect();
                            assert_eq!(&targets, vector);
                            assert_eq!(&event.delta.moves[0].runner.player_id, batter);
                            assert_eq!(&event.delta.moves[0].runner.responsible_pitcher, pitcher);
                            let runs = vector.iter().filter(|t| **t == 4).count() as u8;
                            assert_eq!(event.delta.runs, runs);
                            assert_eq!(transition.after().score[offense], u16::from(runs));
                            assert_eq!(transition.after().outs, cell.context.outs);
                            assert_eq!(
                                transition.after().teams[offense].next_batter,
                                (cursor + 1) % 9
                            );
                            assert_eq!(transition.after().teams[defense], before.teams[defense]);
                            assert!(!transition.after().over);
                            assert_eq!(
                                EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
                                transition
                            );
                            let lines =
                                derive_accounting(std::slice::from_ref(&transition)).unwrap();
                            let line = &lines.batters[offense][batter];
                            assert_eq!(line.plate_appearances, 1);
                            assert_eq!(line.credited_hits, 1);
                            assert_eq!(line.classified_hits, Some(1));
                            assert_eq!(line.credited_rbi, u32::from(runs));
                            assert_eq!(line.classified_rbi, Some(u32::from(runs)));
                            let breakdown = line.credited_hit_breakdown.as_ref().unwrap();
                            assert_eq!(breakdown.singles, u32::from(cell.context.value == 1));
                            assert_eq!(breakdown.doubles, u32::from(cell.context.value == 2));
                            assert_eq!(lines.pitchers[defense][pitcher].batters_faced, 1);
                            let mut move_index = 1;
                            let mut current_runs = 0;
                            for (i, runner) in before.bases.iter().enumerate() {
                                if let Some(runner) = runner {
                                    assert_eq!(&event.delta.moves[move_index].runner, runner);
                                    let scored = vector[move_index] == 4;
                                    if scored {
                                        assert_eq!(
                                            lines.batters[offense][&runner.player_id].credited_runs,
                                            1
                                        );
                                        if runner.responsible_pitcher == *pitcher {
                                            current_runs += 1;
                                        }
                                    } else {
                                        let target = usize::from(vector[move_index] - 1);
                                        assert_eq!(
                                            transition.after().bases[target].as_ref(),
                                            Some(runner)
                                        );
                                    }
                                    if inherited {
                                        let charged = lines.pitchers[defense]
                                            .get(&format!("prior{i}"))
                                            .map_or(0, |p| p.runs_allowed);
                                        assert_eq!(charged, u32::from(scored));
                                    }
                                    move_index += 1;
                                }
                            }
                            assert_eq!(lines.pitchers[defense][pitcher].runs_allowed, current_runs);
                            frames += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(frames, 8_640);
}

#[test]
fn every_bottom_inning_ge9_refuses_even_nonwinning_empty_base_projection_and_top_extras_work() {
    let cells = training();
    let kernel = Kernel::fit(&cells, context(2, 0, 0), Model::UniformLegal).unwrap();
    for inning in 9..=u16::MAX {
        let mut before = game(context(2, 0, 0), Half::Bottom, true, 0, false);
        before.inning = inning;
        before.score = [20, 0];
        let original = before.clone();
        assert!(kernel
            .project(&before, u64::MAX, ModeledCause::UnaidedNoMisplay)
            .is_err());
        assert_eq!(before, original);
    }
    for inning in [1, 8, 9, 10, u16::MAX] {
        let mut before = game(context(2, 0, 0), Half::Top, false, 8, false);
        before.inning = inning;
        let transition = kernel
            .project(&before, 0, ModeledCause::UnaidedNoMisplay)
            .unwrap();
        assert_eq!(transition.after().inning, inning);
        assert!(!transition.after().over);
    }
    let mut before = game(context(2, 0, 0), Half::Bottom, true, 0, false);
    before.inning = 8;
    assert!(kernel
        .project(&before, 0, ModeledCause::UnaidedNoMisplay)
        .is_ok());
}

#[test]
fn vacant_defense_returns_an_error_not_a_panicking_string_assumption() {
    let c = context(1, 1, 0);
    let kernel = Kernel::fit(&training(), c, Model::BaseOut).unwrap();
    let mut before = game(c, Half::Top, true, 0, false);
    before.teams[1].pitcher = bbsim_engine::PitcherRole::Vacant { batting_slot: 8 };
    assert!(kernel
        .project(&before, 0, ModeledCause::UnaidedNoMisplay)
        .is_err());
}
#[test]
fn unknown_cause_context_over_invalid_actor_charge_and_score_overflow_fail_without_mutation() {
    let cells = training();
    let c = context(1, 1, 0);
    let kernel = Kernel::fit(&cells, c, Model::BaseOut).unwrap();
    let before = game(c, Half::Top, true, 0, false);
    assert!(kernel.project(&before, 0, ModeledCause::Unknown).is_err());
    let mut cases = Vec::new();
    // Canonical after-snapshot forbids the next batter still occupying a base.
    // This is an existing refusal, not a kernel or canonical-engine defect.
    let mut queued = before.clone();
    queued.bases[0].as_mut().unwrap().player_id = queued.teams[0].batting_order[1].clone();
    let queued_original = queued.clone();
    assert!(kernel
        .project(&queued, 0, ModeledCause::UnaidedNoMisplay)
        .is_err());
    assert_eq!(queued, queued_original);
    let mut over = before.clone();
    over.over = true;
    cases.push(over);
    let mut mask = before.clone();
    mask.bases[1] = mask.bases[0].take();
    cases.push(mask);
    let mut outs = before.clone();
    outs.outs = 1;
    cases.push(outs);
    let mut inning = before.clone();
    inning.inning = 0;
    cases.push(inning);
    let mut cursor = before.clone();
    cursor.teams[0].next_batter = 255;
    cases.push(cursor);
    let mut pitcher = before.clone();
    pitcher.teams[1].pitcher = String::new().into();
    cases.push(pitcher);
    let mut actor = before.clone();
    actor.bases[0].as_mut().unwrap().player_id = "alien".into();
    cases.push(actor);
    let mut charge = before.clone();
    charge.bases[0]
        .as_mut()
        .unwrap()
        .responsible_pitcher
        .clear();
    cases.push(charge);
    let mut duplicate = before.clone();
    duplicate.bases[0].as_mut().unwrap().player_id = duplicate.teams[0].batting_order[0].clone();
    cases.push(duplicate);
    let mut score = before.clone();
    score.score[0] = u16::MAX;
    cases.push(score);
    for state in cases {
        let original = state.clone();
        assert!(kernel
            .project(&state, u64::MAX, ModeledCause::UnaidedNoMisplay)
            .is_err());
        assert_eq!(state, original);
    }
}

fn word(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

#[test]
fn frequency_diagnostic_preregistered_seed19_100000_words_every_model_context() {
    let cells = varied_training();
    for cell in &cells {
        for model in [Model::UniformLegal, Model::PooledMask, Model::BaseOut] {
            let kernel = Kernel::fit(&cells, cell.context, model).unwrap();
            let mut seed = 19;
            let mut counts = vec![0u64; kernel.vectors().len()];
            for _ in 0..100_000 {
                let vector = kernel.sample(word(&mut seed));
                let j = kernel.vectors().iter().position(|v| v == vector).unwrap();
                counts[j] += 1;
            }
            for (j, (observed, mass)) in counts.iter().zip(kernel.masses()).enumerate() {
                let p = mass as f64 / GRID as f64;
                let expected = 100_000.0 * p;
                let tolerance = 6.0 * (100_000.0 * p * (1.0 - p)).sqrt() + 1.0;
                assert!((*observed as f64 - expected).abs() <= tolerance,
                    "{model:?}/{:?}/category{j}: observed{observed}, expected{expected}, tolerance{tolerance}", cell.context);
            }
        }
    }
}
