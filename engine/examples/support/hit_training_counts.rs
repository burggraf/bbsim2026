//! Standalone training-count loader prerequisite, not source authentication.
use super::joint_hit::{legal_vectors, Context, TrainingCell};

pub struct Artifact {
    pub qualification_sha256: String,
    pub training_events: u64,
    pub cells: Vec<TrainingCell>,
}

fn decimal(token: &str) -> Result<u64, String> {
    if token.is_empty()
        || !token.bytes().all(|b| b.is_ascii_digit())
        || (token.len() > 1 && token.starts_with('0'))
    {
        return Err("invalid canonical u64".into());
    }
    token.parse().map_err(|_| "u64 parse overflow".into())
}

/// Strict bytes-to-cells conversion. Digest/proxy labels are not authentication.
pub fn parse(bytes: &[u8]) -> Result<Artifact, String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "invalid count UTF-8")?;
    if !text.is_ascii() || text.contains('\r') || !text.ends_with('\n') {
        return Err("invalid ASCII/LF count encoding".into());
    }
    let lines: Vec<_> = text[..text.len() - 1].split('\n').take(54).collect();
    if lines.len() != 53
        || lines[0] != "joint-hit-training-counts-v1"
        || lines[1] != "proxy_endpoints_not_certified_causes\ttrue"
        || lines[4] != "value\tbase_mask\touts_before\tcounts"
    {
        return Err("invalid count headers or line count".into());
    }
    let digest = lines[2]
        .strip_prefix("qualification_sha256\t")
        .ok_or("invalid qualification digest label")?;
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("invalid qualification digest".into());
    }
    let declared = decimal(
        lines[3]
            .strip_prefix("training_events\t")
            .ok_or("invalid training total label")?,
    )?;
    let mut total = 0u64;
    let mut cells = Vec::with_capacity(48);
    for (i, line) in lines[5..].iter().enumerate() {
        let context = Context {
            value: (i / 24 + 1) as u8,
            mask: ((i % 24) / 3) as u8,
            outs: (i % 3) as u8,
        };
        let fields: Vec<_> = line.split('\t').take(5).collect();
        if fields.len() != 4
            || fields[0] != context.value.to_string()
            || fields[1] != context.mask.to_string()
            || fields[2] != context.outs.to_string()
        {
            return Err("invalid count context order or row shape".into());
        }
        if fields[3].split(',').count() != legal_vectors(context.value, context.mask)?.len() {
            return Err("invalid full count support shape".into());
        }
        let counts = fields[3]
            .split(',')
            .map(decimal)
            .collect::<Result<Vec<_>, _>>()?;
        for count in &counts {
            total = total
                .checked_add(*count)
                .ok_or("training aggregation overflow")?;
        }
        cells.push(TrainingCell { context, counts });
    }
    if total != declared {
        return Err("training total mismatch".into());
    }
    Ok(Artifact {
        qualification_sha256: digest.into(),
        training_events: total,
        cells,
    })
}
