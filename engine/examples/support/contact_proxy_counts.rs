//! Standalone integer count loading,not source authentication or a probability model.
const MAX_COUNT: u32 = 2_147_483_647;
const MAX_BYTES: usize = 16_384;

pub struct ContextCounts {
    pub mask: u8,
    pub outs: u8,
    pub parent_events: u32,
    pub response_events: [u32; 4],
}
pub struct Artifact {
    pub source_sha256: String,
    pub training_events: u32,
    pub contexts: Vec<ContextCounts>,
}

fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn decimal(token: &str) -> Result<u32, String> {
    if token.is_empty()
        || token.len() > 10
        || !token.bytes().all(|b| b.is_ascii_digit())
        || (token.len() > 1 && token.starts_with('0'))
    {
        return Err("invalid canonical count integer".into());
    }
    let value: u32 = token.parse().map_err(|_| "count integer overflow")?;
    if value > MAX_COUNT {
        return Err("count integer exceeds bound".into());
    }
    Ok(value)
}

/// Caller supplies expected source digest;matching metadata alone is not authentication.
pub fn parse(bytes: &[u8], expected_source_sha256: &str) -> Result<Artifact, String> {
    if !digest(expected_source_sha256) {
        return Err("invalid expected source digest".into());
    }
    if bytes.len() > MAX_BYTES || !bytes.ends_with(b"\n") {
        return Err("invalid count length or termination".into());
    }
    let text = std::str::from_utf8(bytes).map_err(|_| "invalid count UTF-8")?;
    if !text.is_ascii() || text.contains('\r') {
        return Err("invalid count ASCII/LF encoding".into());
    }
    let lines: Vec<_> = text[..text.len() - 1].split('\n').take(33).collect();
    let source_header = format!("source_sha256\t{expected_source_sha256}");
    let headers = [
        "contact-proxy-context-counts-v1",
        "model\tgeometry-proxy-context-jeffreys-v1",
        "season\t2019",
        "split\ttraining",
        &source_header,
        "prior\t1\t2",
        "classes\tSF\tSH\tGIDP\tremaining",
        "contexts\t24",
    ];
    if lines.len() != 32 || lines[..8] != headers {
        return Err("invalid count headers,source digest or line count".into());
    }
    let mut contexts = Vec::with_capacity(24);
    let mut total = 0u64;
    for (i, line) in lines[8..].iter().enumerate() {
        let mask = (i / 3) as u8;
        let outs = (i % 3) as u8;
        let fields: Vec<_> = line.split('\t').take(9).collect();
        if fields.len() != 8
            || fields[0] != "C"
            || fields[1] != mask.to_string()
            || fields[2] != outs.to_string()
        {
            return Err("invalid count context order or row shape".into());
        }
        let parent_events = decimal(fields[3])?;
        let response_events = [
            decimal(fields[4])?,
            decimal(fields[5])?,
            decimal(fields[6])?,
            decimal(fields[7])?,
        ];
        let supported = [
            mask >= 4 && outs < 2,
            (mask == 1 || mask == 5) && outs < 2,
            (mask == 1 || mask == 5) && outs < 2,
            true,
        ];
        if response_events.iter().map(|c| u64::from(*c)).sum::<u64>() != u64::from(parent_events)
            || response_events
                .iter()
                .zip(supported)
                .any(|(c, legal)| *c > 0 && !legal)
        {
            return Err("invalid count conservation or structural support".into());
        }
        total += u64::from(parent_events);
        if total > u64::from(MAX_COUNT) {
            return Err("training aggregate exceeds count bound".into());
        }
        contexts.push(ContextCounts {
            mask,
            outs,
            parent_events,
            response_events,
        });
    }
    Ok(Artifact {
        source_sha256: expected_source_sha256.into(),
        training_events: total as u32,
        contexts,
    })
}
