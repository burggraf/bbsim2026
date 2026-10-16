//! Synthetic-only supplied-word class selection;not source authentication or gameplay.
#[path = "contact_proxy_probabilities.rs"]
pub(crate) mod probabilities;
pub use probabilities::ContextProbabilities;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyClass {
    Sf,
    Sh,
    Gidp,
    Remaining,
}
pub struct Selection {
    pub class: ProxyClass,
    pub source_sha256: String,
    pub model_id: &'static str,
    pub selection_version: &'static str,
    pub training_events: u32,
    pub context: ContextProbabilities,
    pub grid_word: u64,
}
/// Complete strict loading/calculation precedes selection;no parsed-table bypass.
pub fn select(
    bytes: &[u8],
    expected_source_sha256: &str,
    mask: u8,
    outs: u8,
    word: u64,
) -> Result<Selection, String> {
    let table = probabilities::calculate(bytes, expected_source_sha256)?;
    if mask >= 8 || outs >= 3 {
        return Err("invalid contact-proxy selection context".into());
    }
    let context = table
        .contexts
        .into_iter()
        .nth(usize::from(mask) * 3 + usize::from(outs))
        .ok_or("missing contact-proxy selection context")?;
    let grid_word = word >> 11;
    let u = grid_word as f64 * (1.0 / 9_007_199_254_740_992.0);
    let classes = [
        ProxyClass::Sf,
        ProxyClass::Sh,
        ProxyClass::Gidp,
        ProxyClass::Remaining,
    ];
    let index = context
        .cdf
        .iter()
        .position(|edge| u < *edge)
        .ok_or("uncovered contact-proxy selection grid")?;
    Ok(Selection {
        class: classes[index],
        source_sha256: table.source_sha256,
        model_id: table.model_id,
        selection_version: "geometry-proxy-class-high53-cdf-v1",
        training_events: table.training_events,
        context,
        grid_word,
    })
}
