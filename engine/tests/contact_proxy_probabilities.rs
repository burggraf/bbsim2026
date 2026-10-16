//! Synthetic-only computational checks;toy digest metadata is not source authentication.
#[path = "../examples/support/contact_proxy_probabilities.rs"]
mod probabilities;

const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const MODEL: &str = "geometry-proxy-context-jeffreys-v1";
const LIMIT: f64 = 32.0 * f64::EPSILON;
const MAX: u32 = 2_147_483_647;
type Rows = [[u32; 4]; 24];

fn support(i: usize) -> [bool; 4] {
    let mask = i / 3;
    let outs = i % 3;
    [
        outs < 2 && matches!(mask, 4..=7),
        outs < 2 && matches!(mask, 1 | 5),
        outs < 2 && matches!(mask, 1 | 5),
        true,
    ]
}
fn payload(rows: &Rows) -> Vec<u8> {
    let mut text = format!("contact-proxy-context-counts-v1\nmodel\t{MODEL}\nseason\t2019\nsplit\ttraining\nsource_sha256\t{DIGEST}\nprior\t1\t2\nclasses\tSF\tSH\tGIDP\tremaining\ncontexts\t24\n");
    for (i, counts) in rows.iter().enumerate() {
        let n: u64 = counts.iter().map(|c| u64::from(*c)).sum();
        text.push_str(&format!(
            "C\t{}\t{}\t{n}\t{}\t{}\t{}\t{}\n",
            i / 3,
            i % 3,
            counts[0],
            counts[1],
            counts[2],
            counts[3]
        ));
    }
    text.into_bytes()
}
fn check(rows: &Rows) {
    let bytes = payload(rows);
    let before = bytes.clone();
    let result = probabilities::calculate(&bytes, DIGEST).unwrap();
    assert_eq!(result.contexts.len(), 24);
    assert_eq!(result.source_sha256, DIGEST);
    assert_eq!(result.model_id, MODEL);
    let total: u64 = rows.iter().flatten().map(|c| u64::from(*c)).sum();
    assert_eq!(u64::from(result.training_events), total);
    for (i, row) in result.contexts.iter().enumerate() {
        let legal = support(i);
        let n: u64 = rows[i].iter().map(|c| u64::from(*c)).sum();
        let k = legal.iter().filter(|b| **b).count() as u64;
        let denominator = 2 * n + k;
        let numerators = std::array::from_fn(|j| {
            if legal[j] {
                2 * u64::from(rows[i][j]) + 1
            } else {
                0
            }
        });
        assert_eq!(
            (usize::from(row.mask), usize::from(row.outs)),
            (i / 3, i % 3)
        );
        assert_eq!(u64::from(row.parent_events), n);
        assert_eq!(row.response_events, rows[i]);
        assert_eq!(row.supported, legal);
        assert_eq!(row.prior_only, n == 0);
        assert_eq!(row.denominator, denominator);
        assert_eq!(row.numerators, numerators);
        assert_eq!(row.numerators.iter().sum::<u64>(), denominator);
        let mut cumulative = 0.0;
        for j in 0..4 {
            let target = numerators[j] as f64 / denominator as f64;
            let p = row.probabilities[j];
            assert!(p.is_finite() && (0.0..=1.0).contains(&p));
            assert!((p - target).abs() <= LIMIT);
            if legal[j] {
                assert!(p > 0.0);
            } else {
                assert_eq!(p.to_bits(), 0);
            }
            cumulative += target;
            assert!(row.cdf[j].is_finite() && (0.0..=1.0).contains(&row.cdf[j]));
            assert!((row.cdf[j] - cumulative).abs() <= LIMIT);
            if j > 0 {
                assert!(row.cdf[j] >= row.cdf[j - 1]);
            }
            if !legal[j] {
                assert_eq!(row.cdf[j], if j == 0 { 0.0 } else { row.cdf[j - 1] });
            }
        }
        assert!((row.probabilities.iter().sum::<f64>() - 1.0).abs() <= LIMIT);
        assert_eq!(row.cdf[3].to_bits(), 1.0f64.to_bits());
    }
    assert_eq!(bytes, before);
}
fn allocations(
    column: usize,
    left: u32,
    legal: [bool; 4],
    row: &mut [u32; 4],
    result: &mut Vec<[u32; 4]>,
) {
    if column == 4 {
        if left == 0 {
            result.push(*row);
        }
        return;
    }
    for n in 0..=if legal[column] { left } else { 0 } {
        row[column] = n;
        allocations(column + 1, left - n, legal, row, result);
    }
}

#[test]
fn full24_context_metadata_exact_rational_and_float_contract() {
    let rows = std::array::from_fn(|i| {
        std::array::from_fn(|j| if support(i)[j] { (i + j + 1) as u32 } else { 0 })
    });
    check(&rows);
}
#[test]
fn every_small_supported_partition_all24_contexts() {
    let mut cases = 0;
    for i in 0..24 {
        for n in 0..=8 {
            let mut vectors = Vec::new();
            allocations(0, n, support(i), &mut [0; 4], &mut vectors);
            for counts in vectors {
                let mut rows = [[0; 4]; 24];
                rows[i] = counts;
                check(&rows);
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 1716);
}
#[test]
fn observed_empty_contexts_are_prior_only_not_missing() {
    check(&[[0; 4]; 24]);
}
#[test]
fn maximum_four_class_denominator_exceeds_u32_without_overflow() {
    let mut rows = [[0; 4]; 24];
    rows[15][3] = MAX;
    let result = probabilities::calculate(&payload(&rows), DIGEST).unwrap();
    assert_eq!(result.contexts.len(), 24);
    assert_eq!(result.contexts[15].denominator, 4_294_967_298);
    assert_eq!(result.contexts[15].numerators, [1, 1, 1, 4_294_967_295]);
    check(&rows);
}
#[test]
fn boundary_counts_one_hot_near_one_hot_and_balanced_supported_rotations() {
    for i in 0..24 {
        let indices: Vec<_> = (0..4).filter(|j| support(i)[*j]).collect();
        for n in [
            0,
            1,
            2,
            3,
            7,
            8,
            9,
            32767,
            32768,
            65535,
            65536,
            1_073_741_823,
            1_073_741_824,
            MAX - 1,
            MAX,
        ] {
            for (position, j) in indices.iter().enumerate() {
                let mut rows = [[0; 4]; 24];
                rows[i][*j] = n;
                check(&rows);
                if n > 0 && indices.len() > 1 {
                    rows[i][*j] = n - 1;
                    rows[i][indices[(position + 1) % indices.len()]] = 1;
                    check(&rows);
                }
            }
            let mut rows = [[0; 4]; 24];
            for (position, j) in indices.iter().enumerate() {
                rows[i][*j] = n / (indices.len() as u32)
                    + u32::from((position as u32) < n % (indices.len() as u32));
            }
            check(&rows);
        }
    }
}
#[test]
fn public_byte_entry_propagates_complete_loader_refusals() {
    let good = payload(&[[0; 4]; 24]);
    let text = String::from_utf8(good.clone()).unwrap();
    let mut bad = vec![
        Vec::new(),
        good[..good.len() - 1].to_vec(),
        vec![b' '; 16_385],
        vec![255],
    ];
    for altered in [
        text.replace("2019", "2020"),
        text.replace("training", "heldout"),
        text.replace("prior\t1\t2", "prior\t1\t3"),
        text.replace("classes\tSF\tSH", "classes\tSH\tSF"),
        text.replace("contexts\t24", "contexts\t23"),
        text.replace('\n', "\r\n"),
        format!("\u{feff}{text}"),
        format!("{text}\n"),
        format!("{text}#comment\n"),
    ] {
        bad.push(altered.into_bytes());
    }
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    lines.swap(8, 9);
    bad.push((lines.join("\n") + "\n").into_bytes());
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    bad.push((lines[..31].join("\n") + "\n").into_bytes());
    for line in [
        "C\t0\t0\t1\t1\t0\t0\t0",
        "C\t0\t0\t1\t0\t0\t0\t0",
        "C\t0\t0\t0\t0\t0\t0\t0\t0",
        "D\t0\t0\t0\t0\t0\t0\t0",
    ] {
        let mut changed = lines.clone();
        changed[8] = line.into();
        bad.push((changed.join("\n") + "\n").into_bytes());
    }
    let mut rows = [[0; 4]; 24];
    rows[0][3] = MAX;
    rows[1][3] = 1;
    bad.push(payload(&rows));
    for bytes in bad {
        assert!(probabilities::calculate(&bytes, DIGEST).is_err());
    }
    for digest in [
        "",
        "a",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    ] {
        assert!(probabilities::calculate(&good, digest).is_err());
    }
}
#[test]
fn systematic_tokens_and_every_byte_high_bit_refuse_through_public_entry() {
    let good = payload(&[[0; 4]; 24]);
    let text = String::from_utf8(good.clone()).unwrap();
    let lines: Vec<_> = text.lines().collect();
    for row in 8..32 {
        for column in 1..8 {
            for token in [
                "-1",
                "+1",
                "01",
                "1.0",
                "True",
                "NaN",
                "",
                "2147483648",
                "999999999999999999999999999999999999999999999999",
            ] {
                let mut fields: Vec<_> = lines[row].split('\t').collect();
                fields[column] = token;
                let mut changed: Vec<String> = lines.iter().map(|s| (*s).into()).collect();
                changed[row] = fields.join("\t");
                assert!(
                    probabilities::calculate((changed.join("\n") + "\n").as_bytes(), DIGEST)
                        .is_err(),
                    "row{row} column{column} token{token}"
                );
            }
        }
    }
    for index in 0..good.len() {
        let mut bytes = good.clone();
        bytes[index] ^= 128;
        assert!(
            probabilities::calculate(&bytes, DIGEST).is_err(),
            "byte{index}"
        );
    }
}
#[test]
fn wrong_model_duplicate_context_and_every_structural_positive_remain_refusals() {
    let rows = [[0; 4]; 24];
    let good = String::from_utf8(payload(&rows)).unwrap();
    assert!(
        probabilities::calculate(good.replace(MODEL, "unselected-model").as_bytes(), DIGEST)
            .is_err()
    );
    let mut lines: Vec<_> = good.lines().collect();
    lines[9] = lines[8];
    assert!(probabilities::calculate((lines.join("\n") + "\n").as_bytes(), DIGEST).is_err());
    for i in 0..24 {
        for j in 0..4 {
            if !support(i)[j] {
                let mut rows = [[0; 4]; 24];
                rows[i][j] = 1;
                assert!(
                    probabilities::calculate(&payload(&rows), DIGEST).is_err(),
                    "context{i} class{j}"
                );
            }
        }
    }
}

#[test]
fn same_bytes_repeat_exact_probability_and_cdf_bits_without_a_word() {
    let mut rows = [[0; 4]; 24];
    rows[15] = [1, 2, 3, 4];
    let bytes = payload(&rows);
    let a = probabilities::calculate(&bytes, DIGEST).unwrap();
    let b = probabilities::calculate(&bytes, DIGEST).unwrap();
    assert_eq!(a.contexts.len(), 24);
    assert_eq!(b.contexts.len(), 24);
    for (a, b) in a.contexts.iter().zip(&b.contexts) {
        assert_eq!(a.numerators, b.numerators);
        assert_eq!(a.denominator, b.denominator);
        assert_eq!(
            a.probabilities.map(f64::to_bits),
            b.probabilities.map(f64::to_bits)
        );
        assert_eq!(a.cdf.map(f64::to_bits), b.cdf.map(f64::to_bits));
    }
    check(&rows);
}
