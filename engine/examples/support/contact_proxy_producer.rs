//! Opt-in supplied-parent composition;no parent sampler or inferred special-play facts.
use super::{adapter, joint, preview};
use bbsim_engine::{apply, EventTransition, GameState};

pub const POLICY_VERSION: &str = "supplied-contact-proxy-producer-v1";

// Both variants already own a large native transition;the 216-byte difference
// does not justify changing the contracted by-value API or boxing both variants.
#[expect(
    clippy::large_enum_variant,
    reason = "Keep the contracted native by-value transition/adapter ownership without extra allocation"
)]
pub enum Produced {
    Ordinary(EventTransition),
    ContactProxy(adapter::Adapted),
}
impl Produced {
    pub fn transition(&self) -> &EventTransition {
        match self {
            Self::Ordinary(transition) => transition,
            Self::ContactProxy(adapted) => &adapted.transition,
        }
    }
    pub fn into_transition(self) -> EventTransition {
        match self {
            Self::Ordinary(transition) => transition,
            Self::ContactProxy(adapted) => adapted.transition,
        }
    }
}

/// Ordinary calls intentionally do not validate unused proxy artifacts.
/// Residual calls consume one class word before the strict existing adapter.
pub fn produce<F>(
    before: &GameState,
    parent_sample: joint::Sample,
    rng: &mut preview::Rng,
    bytes: &[u8],
    expected_source_sha256: &str,
    provide: F,
) -> Result<Produced, String>
where
    F: FnOnce(
        &GameState,
        &adapter::sampler::Selection,
    ) -> Result<Option<adapter::consequence::ContactDecision>, String>,
{
    let event = joint::event_for(before, parent_sample)
        .map_err(|error| format!("ordinary contact-proxy parent refused: {error}"))?;
    let parent = apply(before, event)
        .map_err(|error| format!("ordinary contact-proxy parent refused: {error:?}"))?;
    if parent_sample.outcome != preview::Outcome::ContactOut
        || parent_sample.contact != Some(joint::ContactKind::Residual)
    {
        return Ok(Produced::Ordinary(parent));
    }
    let word = rng.next_u64();
    adapter::adapt_with_provider(
        before,
        parent_sample,
        &parent,
        bytes,
        expected_source_sha256,
        word,
        provide,
    )
    .map(Produced::ContactProxy)
}
