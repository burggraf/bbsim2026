//! Standalone synthetic refinement from supplied facts;no RNG or producer hook.
#[path = "contact_consequence.rs"]
pub(crate) mod consequence;
#[path = "contact_proxy_sampler.rs"]
pub(crate) mod sampler;
use super::{joint, preview};
use bbsim_engine::{apply, EventTransition, GameState, Half};

pub const POLICY_VERSION: &str = "supplied-contact-proxy-adapter-v1";
pub const PROVIDER_BOUNDARY_VERSION: &str = "supplied-contact-proxy-provider-boundary-v1";
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Disposition {
    RemainingClass,
    ModeledFactsUnavailable,
    TerminalApiUnsupported,
    Projected,
}
pub struct Adapted {
    pub selection: sampler::Selection,
    pub transition: EventTransition,
    pub realized: sampler::ProxyClass,
    pub disposition: Disposition,
}
/// Bind a real ordinary residual parent,then select once and validate supplied facts.
pub fn adapt(
    before: &GameState,
    parent_sample: joint::Sample,
    parent: &EventTransition,
    bytes: &[u8],
    expected_source_sha256: &str,
    word: u64,
    decision: Option<&consequence::ContactDecision>,
) -> Result<Adapted, String> {
    adapt_with_provider(
        before,
        parent_sample,
        parent,
        bytes,
        expected_source_sha256,
        word,
        |_, _| Ok(decision.cloned()),
    )
}

/// Ask once after validation/selection;provider errors are not unavailable facts.
/// A supplied callback's external effects cannot be rolled back on later refusal.
pub fn adapt_with_provider<F>(
    before: &GameState,
    parent_sample: joint::Sample,
    parent: &EventTransition,
    bytes: &[u8],
    expected_source_sha256: &str,
    word: u64,
    provide: F,
) -> Result<Adapted, String>
where
    F: FnOnce(
        &GameState,
        &sampler::Selection,
    ) -> Result<Option<consequence::ContactDecision>, String>,
{
    if parent_sample.outcome != preview::Outcome::ContactOut
        || parent_sample.contact != Some(joint::ContactKind::Residual)
    {
        return Err("invalid contact-proxy adapter parent sample".into());
    }
    if parent.before() != before {
        return Err("contact-proxy adapter before-state mismatch".into());
    }
    let event = joint::event_for(before, parent_sample)
        .map_err(|error| format!("ordinary contact-proxy parent refused: {error}"))?;
    let expected = apply(before, event)
        .map_err(|error| format!("ordinary contact-proxy parent refused: {error:?}"))?;
    if &expected != parent {
        return Err("contact-proxy adapter parent transition mismatch".into());
    }
    let mask = before
        .bases
        .iter()
        .enumerate()
        .fold(0, |m, (i, runner)| m | (u8::from(runner.is_some()) << i));
    let selection = sampler::select(bytes, expected_source_sha256, mask, before.outs, word)?;
    let decision = provide(before, &selection)?;
    let (transition, realized, disposition) = if let Some(facts) = decision.as_ref() {
        let matching = matches!(
            (selection.class, facts),
            (
                sampler::ProxyClass::Sf,
                consequence::ContactDecision::CaughtSacrificeFly { .. }
            ) | (
                sampler::ProxyClass::Sh,
                consequence::ContactDecision::SacrificeBunt(_)
            ) | (
                sampler::ProxyClass::Gidp,
                consequence::ContactDecision::GroundForceDoublePlay(_)
            )
        );
        if !matching {
            return Err("contact-proxy adapter decision/class mismatch".into());
        }
        (
            consequence::project(before, facts)?,
            selection.class,
            Disposition::Projected,
        )
    } else {
        let reason = if selection.class == sampler::ProxyClass::Remaining {
            Disposition::RemainingClass
        } else if before.half == Half::Bottom && before.inning >= 9 {
            Disposition::TerminalApiUnsupported
        } else {
            Disposition::ModeledFactsUnavailable
        };
        (parent.clone(), sampler::ProxyClass::Remaining, reason)
    };
    Ok(Adapted {
        selection,
        transition,
        realized,
        disposition,
    })
}
