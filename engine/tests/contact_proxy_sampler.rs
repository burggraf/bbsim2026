//! Synthetic-only selector checks;fictional digest is not source authentication.
#[path = "../examples/support/contact_proxy_sampler.rs"]
mod sampler;
use sampler::probabilities;

const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const MODEL: &str = "geometry-proxy-context-jeffreys-v1";
const VERSION: &str = "geometry-proxy-class-high53-cdf-v1";
const GRID: u64 = 1 << 53;
const MAX: u32 = 2_147_483_647;
type Rows = [[u32; 4]; 24];
const CLASSES: [sampler::ProxyClass; 4] = [
    sampler::ProxyClass::Sf,
    sampler::ProxyClass::Sh,
    sampler::ProxyClass::Gidp,
    sampler::ProxyClass::Remaining,
];
fn support(i: usize) -> [bool; 4] {
    let (mask, outs) = (i / 3, i % 3);
    [
        outs < 2 && mask >= 4,
        outs < 2 && matches!(mask, 1 | 5),
        outs < 2 && matches!(mask, 1 | 5),
        true,
    ]
}
fn payload(rows: &Rows) -> Vec<u8> {
    let mut s=format!("contact-proxy-context-counts-v1\nmodel\t{MODEL}\nseason\t2019\nsplit\ttraining\nsource_sha256\t{DIGEST}\nprior\t1\t2\nclasses\tSF\tSH\tGIDP\tremaining\ncontexts\t24\n");
    for (i, c) in rows.iter().enumerate() {
        let n: u64 = c.iter().map(|x| u64::from(*x)).sum();
        s.push_str(&format!(
            "C\t{}\t{}\t{n}\t{}\t{}\t{}\t{}\n",
            i / 3,
            i % 3,
            c[0],
            c[1],
            c[2],
            c[3]
        ));
    }
    s.into_bytes()
}
// Independent integer dyadic decoding;no production float comparison or selector.
fn edge(p: f64) -> u64 {
    assert!(p.is_finite() && (0.0..=1.0).contains(&p));
    if p == 0.0 {
        return 0;
    }
    let bits = p.to_bits();
    let exponent = ((bits >> 52) & 2047) as i32;
    assert!((990..=1023).contains(&exponent));
    let mantissa = (bits & ((1 << 52) - 1)) | (1 << 52);
    let shift = exponent - 1022;
    if shift >= 0 {
        mantissa << shift
    } else {
        let k = (-shift) as u32;
        (mantissa >> k) + u64::from(mantissa & ((1 << k) - 1) != 0)
    }
}
fn check(rows: &Rows, i: usize) {
    let bytes = payload(rows);
    let original = bytes.clone();
    let table = probabilities::calculate(&bytes, DIGEST).unwrap();
    let expected = &table.contexts[i];
    let edges = expected.cdf.map(edge);
    let mut previous = 0;
    let mut total = 0;
    assert_eq!(expected.supported, support(i));
    for (j, e) in edges.iter().enumerate() {
        assert!(*e >= previous);
        let mass = *e - previous;
        total += mass;
        assert_eq!(mass > 0, support(i)[j]);
        previous = *e;
    }
    assert_eq!((previous, total), (GRID, GRID));
    let mut grids = vec![0, GRID - 1];
    for e in edges {
        for g in [e.saturating_sub(1), e, e + 1] {
            if g < GRID {
                grids.push(g);
            }
        }
    }
    grids.sort_unstable();
    grids.dedup();
    for g in grids {
        let got = sampler::select(&bytes, DIGEST, (i / 3) as u8, (i % 3) as u8, g << 11).unwrap();
        let j = edges.iter().position(|e| g < *e).unwrap();
        assert_eq!(got.class, CLASSES[j], "context{i} grid{g}");
        assert_eq!(got.grid_word, g);
        assert_eq!(got.source_sha256, table.source_sha256);
        assert_eq!(got.model_id, table.model_id);
        assert_eq!(got.selection_version, VERSION);
        assert_eq!(got.training_events, table.training_events);
        let c = &got.context;
        assert_eq!(
            (c.mask, c.outs, c.parent_events, c.prior_only),
            (
                expected.mask,
                expected.outs,
                expected.parent_events,
                expected.prior_only
            )
        );
        assert_eq!(c.response_events, expected.response_events);
        assert_eq!(c.supported, expected.supported);
        assert_eq!(c.numerators, expected.numerators);
        assert_eq!(c.denominator, expected.denominator);
        assert_eq!(
            c.probabilities.map(f64::to_bits),
            expected.probabilities.map(f64::to_bits)
        );
        assert_eq!(c.cdf.map(f64::to_bits), expected.cdf.map(f64::to_bits));
    }
    assert_eq!(bytes, original);
}
fn allocations(
    column: usize,
    left: u32,
    legal: [bool; 4],
    row: &mut [u32; 4],
    out: &mut Vec<[u32; 4]>,
) {
    if column == 4 {
        if left == 0 {
            out.push(*row);
        }
        return;
    }
    for n in 0..=if legal[column] { left } else { 0 } {
        row[column] = n;
        allocations(column + 1, left - n, legal, row, out);
    }
}
#[test]
fn populated_and_prior_only_all24_contexts_preserve_metadata_and_exact_edges() {
    let populated = std::array::from_fn(|i| {
        std::array::from_fn(|j| if support(i)[j] { (i + j + 1) as u32 } else { 0 })
    });
    for i in 0..24 {
        check(&populated, i);
        check(&[[0; 4]; 24], i);
    }
}
#[test]
fn all1716_small_partitions_cover_exact_grid_boundaries_and_structural_mass() {
    let mut cases = 0;
    for i in 0..24 {
        for n in 0..=8 {
            let mut vectors = Vec::new();
            allocations(0, n, support(i), &mut [0; 4], &mut vectors);
            for c in vectors {
                let mut rows = [[0; 4]; 24];
                rows[i] = c;
                check(&rows, i);
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 1716);
}
#[test]
fn capped_one_hot_near_one_hot_balanced_rotations_and_max_denominator() {
    let mut cases = 0;
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
                check(&rows, i);
                cases += 1;
                if n > 0 && indices.len() > 1 {
                    rows[i][*j] = n - 1;
                    rows[i][indices[(position + 1) % indices.len()]] = 1;
                    check(&rows, i);
                    cases += 1;
                }
            }
            let mut rows = [[0; 4]; 24];
            for (position, j) in indices.iter().enumerate() {
                rows[i][*j] = n / indices.len() as u32
                    + u32::from((position as u32) < n % indices.len() as u32);
            }
            check(&rows, i);
            cases += 1;
        }
    }
    assert_eq!(cases, 1324);
    let mut rows = [[0; 4]; 24];
    rows[15][3] = MAX;
    assert_eq!(
        sampler::select(&payload(&rows), DIGEST, 5, 0, u64::MAX)
            .unwrap()
            .context
            .denominator,
        4_294_967_298
    );
}
#[test]
fn all2048_low_bits_are_inert_at_prior_only_boundaries_and_nonempty_examples() {
    for populated in [false, true] {
        let rows = std::array::from_fn(|i| {
            std::array::from_fn(|j| {
                if populated && support(i)[j] {
                    (j + 1) as u32
                } else {
                    0
                }
            })
        });
        let bytes = payload(&rows);
        let table = probabilities::calculate(&bytes, DIGEST).unwrap();
        for (i, c) in table.contexts.iter().enumerate() {
            let edges = c.cdf.map(edge);
            let mut grids = vec![0, GRID - 1];
            grids.extend(edges.into_iter().map(|e| e.min(GRID - 1)));
            grids.sort_unstable();
            grids.dedup();
            for g in grids {
                let expected = CLASSES[edges.iter().position(|e| g < *e).unwrap()];
                for low in 0..2048 {
                    let got = sampler::select(
                        &bytes,
                        DIGEST,
                        (i / 3) as u8,
                        (i % 3) as u8,
                        (g << 11) | low,
                    )
                    .unwrap();
                    assert_eq!(got.grid_word, g);
                    assert_eq!(got.class, expected);
                }
            }
        }
    }
}
#[test]
fn every_invalid_u8_context_refuses_without_alias_panic_or_fallback() {
    let bytes = payload(&[[0; 4]; 24]);
    for mask in 8..=u8::MAX {
        for outs in 0..3 {
            assert!(sampler::select(&bytes, DIGEST, mask, outs, u64::MAX).is_err());
        }
    }
    for mask in 0..8 {
        for outs in 3..=u8::MAX {
            assert!(sampler::select(&bytes, DIGEST, mask, outs, 0).is_err());
        }
    }
}
#[test]
fn complete_loading_refusals_include_unselected_context_and_every_high_byte() {
    let bytes = payload(&[[0; 4]; 24]);
    let text = String::from_utf8(bytes.clone()).unwrap();
    for bad in [
        Vec::new(),
        bytes[..bytes.len() - 1].to_vec(),
        vec![b' '; 16_385],
        vec![255],
        text.replace(MODEL, "wrong-model").into_bytes(),
        text.replace("2019", "2020").into_bytes(),
        text.replace("training", "heldout").into_bytes(),
        text.replace("prior\t1\t2", "prior\t1\t3").into_bytes(),
        text.replace('\n', "\r\n").into_bytes(),
        format!("\u{feff}{text}").into_bytes(),
        format!("{text}\n").into_bytes(),
    ] {
        assert!(sampler::select(&bad, DIGEST, 0, 0, 0).is_err());
    }
    for digest in [
        "",
        "a",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    ] {
        assert!(sampler::select(&bytes, digest, 0, 0, 0).is_err());
    }
    for index in 0..bytes.len() {
        let mut bad = bytes.clone();
        bad[index] ^= 128;
        assert!(
            sampler::select(&bad, DIGEST, 0, 0, 0).is_err(),
            "byte{index}"
        );
    }
    let lines: Vec<_> = text.lines().collect();
    for row in 8..32 {
        for column in 1..8 {
            for token in ["-1", "+1", "01", "1.0", "True", "NaN", "", "2147483648"] {
                let mut changed: Vec<String> = lines.iter().map(|s| (*s).into()).collect();
                let mut fields: Vec<_> = lines[row].split('\t').collect();
                fields[column] = token;
                changed[row] = fields.join("\t");
                assert!(
                    sampler::select((changed.join("\n") + "\n").as_bytes(), DIGEST, 0, 0, 0)
                        .is_err()
                );
            }
        }
    }
    let mut changed = lines.clone();
    changed.swap(8, 9);
    assert!(sampler::select((changed.join("\n") + "\n").as_bytes(), DIGEST, 0, 0, 0).is_err());
    changed = lines.clone();
    changed[31] = lines[30];
    assert!(sampler::select((changed.join("\n") + "\n").as_bytes(), DIGEST, 0, 0, 0).is_err());
    assert!(sampler::select((lines[..31].join("\n") + "\n").as_bytes(), DIGEST, 0, 0, 0).is_err());
    for i in 0..24 {
        for j in 0..4 {
            if !support(i)[j] {
                let mut rows = [[0; 4]; 24];
                rows[i][j] = 1;
                assert!(sampler::select(&payload(&rows), DIGEST, 0, 0, 0).is_err());
            }
        }
    }
    let mut rows = [[0; 4]; 24];
    rows[0][3] = MAX;
    rows[23][3] = 1;
    assert!(sampler::select(&payload(&rows), DIGEST, 0, 0, 0).is_err());
}
#[test]
fn exact_dyadic_reference_controls_and_repeat_immutability() {
    assert_eq!(edge(0.0), 0);
    assert_eq!(edge(1.0), GRID);
    assert_eq!(edge(0.5), GRID / 2);
    assert_eq!(edge(0.25), GRID / 4);
    assert_eq!(edge(f64::from_bits(0.25f64.to_bits() + 1)), GRID / 4 + 1);
    let mut rows = [[0; 4]; 24];
    rows[15] = [1, 2, 3, 4];
    let bytes = payload(&rows);
    let saved = bytes.clone();
    for word in [0, 1, 2047, 2048, 1 << 63, u64::MAX] {
        let a = sampler::select(&bytes, DIGEST, 5, 0, word).unwrap();
        let b = sampler::select(&bytes, DIGEST, 5, 0, word).unwrap();
        assert_eq!(a.grid_word, word >> 11);
        assert_eq!(a.grid_word, b.grid_word);
        assert_eq!(a.class, b.class);
        assert_eq!(
            a.context.cdf.map(f64::to_bits),
            b.context.cdf.map(f64::to_bits)
        );
        assert_eq!(bytes, saved);
    }
}

#[test]
fn zero_word_selects_first_positive_sf_not_a_constant_remaining() {
    let bytes = payload(&[[0; 4]; 24]);
    assert_eq!(
        sampler::select(&bytes, DIGEST, 5, 0, 0).unwrap().class,
        sampler::ProxyClass::Sf
    );
}
#[test]
fn exact_cdf_grid_equality_advances_to_next_class_with_strict_comparison() {
    let bytes = payload(&[[0; 4]; 24]);
    assert_eq!(
        sampler::select(&bytes, DIGEST, 5, 0, 1 << 62)
            .unwrap()
            .class,
        sampler::ProxyClass::Sh
    );
    assert_eq!(
        sampler::select(&bytes, DIGEST, 5, 0, 1 << 63)
            .unwrap()
            .class,
        sampler::ProxyClass::Gidp
    );
    assert_eq!(
        sampler::select(&bytes, DIGEST, 5, 0, 3 << 62)
            .unwrap()
            .class,
        sampler::ProxyClass::Remaining
    );
}
