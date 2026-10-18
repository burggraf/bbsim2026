//! Synthetic strict wire loading; no real source files or production callers.
#[path = "../examples/support/hit_training_counts.rs"]
mod hit_training_counts;
#[path = "../examples/support/joint_hit.rs"]
mod joint_hit;

use bbsim_engine::{GameState, Half, TeamState};
use joint_hit::{Context, Kernel, Model, ModeledCause};

const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn parse(text: &str) -> Result<hit_training_counts::Artifact, String> {
    hit_training_counts::parse(text.as_bytes())
}

// Independent Cartesian/pairwise geometry, not a call to the production support.
fn support(value: u8, mask: u8) -> Vec<Vec<u8>> {
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

fn wire(overrides: &[(Context, Vec<u64>)]) -> String {
    let total: u64 = overrides
        .iter()
        .flat_map(|(_, counts)| counts)
        .copied()
        .sum();
    let mut text = format!("joint-hit-training-counts-v1\nproxy_endpoints_not_certified_causes\ttrue\nqualification_sha256\t{DIGEST}\ntraining_events\t{total}\nvalue\tbase_mask\touts_before\tcounts\n");
    for value in [1, 2] {
        for mask in 0..8 {
            for outs in 0..3 {
                let c = Context { value, mask, outs };
                let counts = overrides
                    .iter()
                    .find(|(context, _)| *context == c)
                    .map_or_else(
                        || vec![0; support(value, mask).len()],
                        |(_, counts)| counts.clone(),
                    );
                let numbers = counts
                    .iter()
                    .map(u64::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                text.push_str(&format!("{value}\t{mask}\t{outs}\t{numbers}\n"));
            }
        }
    }
    text
}

fn replace_line(text: &str, index: usize, line: &str) -> String {
    let mut lines: Vec<&str> = text.lines().collect();
    lines[index] = line;
    lines.join("\n") + "\n"
}

#[test]
fn zero_and_sparse_tables_load_all48_contexts_and_literal_counts_without_authentication_claim() {
    let c = Context {
        value: 1,
        mask: 1,
        outs: 0,
    };
    let text = wire(&[(c, vec![3, 0, 1])]);
    let artifact = parse(&text).unwrap_or_else(|e| panic!("valid sparse table refused: {e}"));
    assert_eq!(artifact.qualification_sha256, DIGEST);
    assert_eq!(artifact.training_events, 4);
    assert_eq!(artifact.cells.len(), 48);
    assert_eq!(artifact.cells[3].context, c);
    assert_eq!(artifact.cells[3].counts, vec![3, 0, 1]);
    for (i, cell) in artifact.cells.iter().enumerate() {
        assert_eq!(
            i,
            usize::from(cell.context.value - 1) * 24
                + usize::from(cell.context.mask) * 3
                + usize::from(cell.context.outs)
        );
        assert_eq!(
            cell.counts.len(),
            support(cell.context.value, cell.context.mask).len()
        );
    }
    let zero = parse(&wire(&[])).unwrap();
    assert_eq!(zero.training_events, 0);
    assert!(zero
        .cells
        .iter()
        .all(|cell| cell.counts.iter().all(|n| *n == 0)));
    // A well-formed synthetic digest is metadata, not a source certificate.
    assert!(parse(&wire(&[]).replace(DIGEST, &"0".repeat(64))).is_ok());
}

#[test]
fn sparse_loaded_half_priors_are_an_already_green_exact_kernel_characterization() {
    let c = Context {
        value: 1,
        mask: 1,
        outs: 0,
    };
    let other_outs = Context { outs: 2, ..c };
    let artifact = parse(&wire(&[(c, vec![3, 0, 1]), (other_outs, vec![0, 2, 0])])).unwrap();
    assert_eq!(artifact.training_events, 6);
    for (model, literal) in [
        (Model::UniformLegal, vec![(1, 3), (1, 3), (1, 3)]),
        (Model::PooledMask, vec![(7, 15), (1, 3), (1, 5)]),
        (Model::BaseOut, vec![(7, 11), (1, 11), (3, 11)]),
    ] {
        let kernel = Kernel::fit(&artifact.cells, c, model).unwrap();
        assert_eq!(
            kernel
                .probabilities()
                .iter()
                .map(|q| (q.numerator, q.denominator))
                .collect::<Vec<_>>(),
            literal
        );
    }
}

#[test]
fn every_fixed_model_keeps_exact_support_grid_edges_sampling_and_explicit_projection_boundary() {
    let artifact = parse(&wire(&[])).unwrap();
    for cell in &artifact.cells {
        for model in [Model::UniformLegal, Model::PooledMask, Model::BaseOut] {
            let kernel = Kernel::fit(&artifact.cells, cell.context, model).unwrap();
            let expected = support(cell.context.value, cell.context.mask);
            assert_eq!(kernel.vectors(), expected);
            assert_eq!(kernel.probabilities().len(), expected.len());
            assert!(kernel
                .probabilities()
                .iter()
                .all(|q| q.numerator == 1 && q.denominator == expected.len() as u128));
            assert_eq!(kernel.masses().iter().sum::<u64>(), 1 << 53);
            assert_eq!(kernel.edges().last(), Some(&(1 << 53)));
            assert_eq!(kernel.sample(0), expected[0]);
            assert_eq!(kernel.sample(u64::MAX), expected.last().unwrap());
        }
    }
    let teams = std::array::from_fn(|side| TeamState {
        defense: bbsim_engine::DefenseState::LegacyUnassigned,
        batting_order: std::array::from_fn(|i| format!("side{side}b{i}")),
        next_batter: 0,
        pitcher: (format!("side{side}p")).into(),
        dh_slot: Some(8),
        dh_must_bat: false,
        bench: vec![],
        removed: vec![],
    });
    let mut before = GameState {
        inning: 2,
        half: Half::Top,
        outs: 0,
        score: [0, 0],
        teams,
        bases: [None, None, None],
        over: false,
    };
    let kernel = Kernel::fit(
        &artifact.cells,
        Context {
            value: 1,
            mask: 0,
            outs: 0,
        },
        Model::BaseOut,
    )
    .unwrap();
    assert!(kernel.project(&before, 0, ModeledCause::Unknown).is_err());
    assert!(kernel
        .project(&before, 0, ModeledCause::UnaidedNoMisplay)
        .is_ok());
    before.half = Half::Bottom;
    before.inning = 9;
    assert!(kernel
        .project(&before, 0, ModeledCause::UnaidedNoMisplay)
        .is_err());
}

#[test]
fn text_encoding_headers_digest_and_line_count_refuse_without_normalization() {
    let good = wire(&[]);
    let mut cases = vec![
        String::new(),
        good.replace('\n', "\r\n"),
        good.trim_end().into(),
        format!("{good}\n"),
        format!("{good}1\t0\t0\t0\n"),
        good.replace(DIGEST, &"A".repeat(64)),
        good.replace(DIGEST, &"g".repeat(64)),
        good.replace(DIGEST, &"0".repeat(63)),
        good.replace(DIGEST, &"0".repeat(65)),
        good.replace(DIGEST, &format!("{}é", "0".repeat(63))),
    ];
    for (index, bad) in [
        (0, "joint-hit-training-counts-v2"),
        (1, "proxy_endpoints_not_certified_causes\tfalse"),
        (2, "qualification_sha256\t"),
        (3, "training_events\t1"),
        (4, "mask\tvalue\touts\tcounts"),
    ] {
        cases.push(replace_line(&good, index, bad));
    }
    for text in cases {
        assert!(parse(&text).is_err(), "accepted malformed text: {text:?}");
    }
    assert!(hit_training_counts::parse(&[0xff, 0xfe]).is_err());
}

#[test]
fn canonical_integer_context_order_shape_and_total_refusals_are_explicit() {
    let good = wire(&[]);
    for bad in [
        "0\t0\t0\t0",
        "3\t0\t0\t0",
        "1\t8\t0\t0",
        "1\t0\t3\t0",
        "1\t0\t1\t0",
        "1\t0\t0\t",
        "1\t0\t0\t0,0",
        "1\t0\t0\t0\textra",
        "1\t0\t0\t-1",
        "1\t0\t0\t+0",
        "1\t0\t0\t00",
        "01\t0\t0\t0",
        "1\t 0\t0\t0",
        "1\t0\t0\tfalse",
        "1\t0\t0\t1e0",
        "1\t0\t0\t18446744073709551616",
        "1\t0\t0\t1",
    ] {
        assert!(
            parse(&replace_line(&good, 5, bad)).is_err(),
            "accepted {bad}"
        );
    }
    for bad in [
        "training_events\t+0",
        "training_events\t00",
        "training_events\t18446744073709551616",
    ] {
        assert!(parse(&replace_line(&good, 3, bad)).is_err());
    }
    assert!(parse(&replace_line(&good, 6, "1\t0\t0\t0")).is_err());
}

#[test]
fn checked_u64_total_and_unrepresentable_sampler_counts_are_not_clipped() {
    let c = Context {
        value: 1,
        mask: 0,
        outs: 0,
    };
    let maximum = parse(&wire(&[(c, vec![u64::MAX])])).unwrap();
    assert_eq!(maximum.training_events, u64::MAX);
    let overflow = replace_line(&wire(&[(c, vec![u64::MAX])]), 6, "1\t0\t1\t1");
    assert!(parse(&overflow).is_err());
    let c = Context {
        value: 1,
        mask: 1,
        outs: 0,
    };
    let artifact = parse(&wire(&[(c, vec![u64::MAX, 0, 0])])).unwrap();
    assert_eq!(artifact.cells[3].counts, vec![u64::MAX, 0, 0]);
    assert!(Kernel::fit(&artifact.cells, c, Model::UniformLegal).is_ok());
    for model in [Model::PooledMask, Model::BaseOut] {
        assert!(Kernel::fit(&artifact.cells, c, model).is_err());
    }
}
