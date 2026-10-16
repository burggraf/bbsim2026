//! Standalone proposal arithmetic;callers still own whole-source authentication.
#[path = "contact_proxy_counts.rs"]
mod counts;

const MODEL_ID: &str = "geometry-proxy-context-jeffreys-v1";
const LIMIT: f64 = 32.0 * f64::EPSILON;

pub struct ContextProbabilities {
    pub mask: u8,
    pub outs: u8,
    pub parent_events: u32,
    pub response_events: [u32; 4],
    pub supported: [bool; 4],
    pub prior_only: bool,
    pub numerators: [u64; 4],
    pub denominator: u64,
    pub probabilities: [f64; 4],
    pub cdf: [f64; 4],
}
pub struct ProbabilityTable {
    pub source_sha256: String,
    pub model_id: &'static str,
    pub training_events: u32,
    pub contexts: Vec<ContextProbabilities>,
}
/// Complete strict byte loading precedes arithmetic;there is no parsed-struct bypass.
pub fn calculate(bytes: &[u8], expected_source_sha256: &str) -> Result<ProbabilityTable, String> {
    let input = counts::parse(bytes, expected_source_sha256)?;
    let mut contexts = Vec::with_capacity(24);
    for row in input.contexts {
        let supported = [
            row.outs < 2 && row.mask >= 4,
            row.outs < 2 && matches!(row.mask, 1 | 5),
            row.outs < 2 && matches!(row.mask, 1 | 5),
            true,
        ];
        let k = supported.iter().filter(|legal| **legal).count() as u64;
        let denominator = 2 * u64::from(row.parent_events) + k;
        let numerators: [u64; 4] = std::array::from_fn(|j| {
            if supported[j] {
                2 * u64::from(row.response_events[j]) + 1
            } else {
                0
            }
        });
        if numerators.iter().sum::<u64>() != denominator {
            return Err("invalid computational rational closure".into());
        }
        let probabilities: [f64; 4] = std::array::from_fn(|j| {
            if supported[j] {
                numerators[j] as f64 / denominator as f64
            } else {
                0.0
            }
        });
        if probabilities
            .iter()
            .enumerate()
            .any(|(j, p)| !p.is_finite() || !(0.0..=1.0).contains(p) || (supported[j] && *p <= 0.0))
        {
            return Err("invalid computational probability".into());
        }
        let mut cumulative = 0.0;
        let mut cdf = [0.0; 4];
        for (j, p) in probabilities.iter().enumerate() {
            cumulative += p;
            cdf[j] = cumulative;
        }
        if !cumulative.is_finite()
            || (cumulative - 1.0).abs() > LIMIT
            || cdf[..3].iter().any(|p| !(0.0..=1.0).contains(p))
            || cdf.windows(2).any(|w| w[1] < w[0])
        {
            return Err("invalid computational CDF closure".into());
        }
        cdf[3] = 1.0;
        contexts.push(ContextProbabilities {
            mask: row.mask,
            outs: row.outs,
            parent_events: row.parent_events,
            response_events: row.response_events,
            supported,
            prior_only: row.parent_events == 0,
            numerators,
            denominator,
            probabilities,
            cdf,
        });
    }
    Ok(ProbabilityTable {
        source_sha256: input.source_sha256,
        model_id: MODEL_ID,
        training_events: input.training_events,
        contexts,
    })
}
