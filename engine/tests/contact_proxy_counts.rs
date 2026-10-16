//! Count-only toy format controls;digests are metadata,not source authentication.
#[path = "../examples/support/contact_proxy_counts.rs"]
mod counts;

const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn payload() -> Vec<u8> {
    let mut text = format!("contact-proxy-context-counts-v1\nmodel\tgeometry-proxy-context-jeffreys-v1\nseason\t2019\nsplit\ttraining\nsource_sha256\t{DIGEST}\nprior\t1\t2\nclasses\tSF\tSH\tGIDP\tremaining\ncontexts\t24\n");
    for mask in 0..8 {
        for outs in 0..3 {
            let sf = u32::from(mask >= 4 && outs < 2);
            let sh = u32::from((mask == 1 || mask == 5) && outs < 2) * 2;
            let gidp = u32::from((mask == 1 || mask == 5) && outs < 2) * 3;
            let remaining = mask * 3 + outs + 4;
            text.push_str(&format!(
                "C\t{mask}\t{outs}\t{}\t{sf}\t{sh}\t{gidp}\t{remaining}\n",
                sf + sh + gidp + remaining
            ));
        }
    }
    text.into_bytes()
}

#[test]
fn complete24_integer_contexts_and_total() {
    let artifact = counts::parse(&payload(), DIGEST).unwrap();
    assert_eq!(artifact.source_sha256, DIGEST);
    assert_eq!(artifact.contexts.len(), 24);
    let mut total = 0;
    for (i, row) in artifact.contexts.iter().enumerate() {
        assert_eq!(
            (usize::from(row.mask), usize::from(row.outs)),
            (i / 3, i % 3)
        );
        assert_eq!(row.parent_events, row.response_events.iter().sum());
        assert_eq!(row.response_events[3], i as u32 + 4);
        total += row.parent_events;
    }
    assert_eq!(artifact.training_events, total);
}

#[test]
fn required_external_digest_and_metadata_match() {
    for bad in [
        "",
        "a",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    ] {
        assert!(
            counts::parse(&payload(), bad).is_err(),
            "expected digest {bad}"
        );
    }
}

#[test]
fn encoding_headers_and_line_domain_refuse() {
    let text = String::from_utf8(payload()).unwrap();
    let bad = [
        text.trim_end().to_string(),
        format!("\u{feff}{text}"),
        text.replace('\n', "\r\n"),
        text.replace("2019", "2020"),
        text.replace("training", "heldout"),
        text.replace("prior\t1\t2", "prior\t1\t3"),
        text.replace("contexts\t24", "contexts\t23"),
        format!("{text}\n"),
        format!("{text}#comment\n"),
    ];
    for bytes in bad {
        assert!(counts::parse(bytes.as_bytes(), DIGEST).is_err());
    }
    assert!(counts::parse(&[0xff], DIGEST).is_err());
    assert!(counts::parse(&vec![b' '; 16385], DIGEST).is_err());
}

#[test]
fn systematic_noncanonical_context_tokens_refuse() {
    let text = String::from_utf8(payload()).unwrap();
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
                    counts::parse((changed.join("\n") + "\n").as_bytes(), DIGEST).is_err(),
                    "row{row} column{column} token{token}"
                );
            }
        }
    }
}

#[test]
fn order_row_shape_structural_zeros_and_conservation_refuse() {
    let text = String::from_utf8(payload()).unwrap();
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    let mut changed = lines.clone();
    changed.swap(8, 9);
    assert!(counts::parse((changed.join("\n") + "\n").as_bytes(), DIGEST).is_err());
    for line in [
        "C\t0\t0\t5\t1\t0\t0\t4",
        "C\t0\t0\t5\t0\t0\t0\t4",
        "C\t0\t0\t4\t0\t0\t0\t4\t0",
        "D\t0\t0\t4\t0\t0\t0\t4",
    ] {
        let mut changed = lines.clone();
        changed[8] = line.into();
        assert!(
            counts::parse((changed.join("\n") + "\n").as_bytes(), DIGEST).is_err(),
            "{line}"
        );
    }
    let missing = lines[..31].join("\n") + "\n";
    assert!(counts::parse(missing.as_bytes(), DIGEST).is_err());
}

#[test]
fn every_one_byte_high_bit_mutation_refuses() {
    let good = payload();
    for index in 0..good.len() {
        let mut bad = good.clone();
        bad[index] ^= 128;
        assert!(counts::parse(&bad, DIGEST).is_err(), "byte{index}");
    }
}

#[test]
fn inclusive_integer_cap_and_aggregate_overflow() {
    let text = String::from_utf8(payload()).unwrap();
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    for (i, line) in lines[8..].iter_mut().enumerate() {
        *line = format!("C\t{}\t{}\t0\t0\t0\t0\t0", i / 3, i % 3);
    }
    assert_eq!(
        counts::parse((lines.join("\n") + "\n").as_bytes(), DIGEST)
            .unwrap()
            .training_events,
        0
    );
    lines[8] = "C\t0\t0\t2147483647\t0\t0\t0\t2147483647".into();
    assert_eq!(
        counts::parse((lines.join("\n") + "\n").as_bytes(), DIGEST)
            .unwrap()
            .training_events,
        2147483647
    );
    lines[9] = "C\t0\t1\t1\t0\t0\t0\t1".into();
    assert!(counts::parse((lines.join("\n") + "\n").as_bytes(), DIGEST).is_err());
}
