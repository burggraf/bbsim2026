//! Explicit direct caught-air episode, never inferred from trajectory or PO credits.
use super::*;

pub(crate) fn validate(
    before: &GameState,
    event: &Event,
    caught_sacrifice_fly: Option<&CaughtSacrificeFlyFacts>,
) -> Result<(), TransitionError> {
    if let Some(facts) = caught_sacrifice_fly {
        return crate::sacrifice_fly::validate(before, event, facts);
    }
    let Some(facts) = &event.ordinary_air_catch else {
        return Ok(());
    };
    let defense = &before.teams[1 - before.half.offense()];
    if event.participants.batter.as_ref() != Some(&facts.batter_id) {
        return Err(TransitionError::InvalidParticipants);
    }
    if !rules::fielding_eligible(defense, &facts.fielder_id) {
        return Err(TransitionError::IllegalCredit);
    }
    if event.kind != EventKind::PlateAppearance {
        return Err(TransitionError::IllegalStateChange);
    }
    if event.ordinary_ground_throw.is_some()
        || event.caught_strikeout.is_some()
        || event.protected_award.is_some()
        || event.sole_home_run.is_some()
        || event.sole_safe_hit.is_some()
        || event.muffed_ground_ball.is_some()
        || event.delta.credits.iter().any(|c| {
            matches!(
                c.kind,
                CreditKind::Hit
                    | CreditKind::Single
                    | CreditKind::Double
                    | CreditKind::Triple
                    | CreditKind::Strikeout
                    | CreditKind::Walk
                    | CreditKind::OrdinaryWalk
                    | CreditKind::IntentionalWalk
                    | CreditKind::HitByPitch
                    | CreditKind::SacrificeBunt
                    | CreditKind::SacrificeFly
                    | CreditKind::InterferenceAward
                    | CreditKind::GroundedIntoDoublePlay
            )
        })
    {
        return Err(TransitionError::IllegalCredit);
    }
    rules::validate_held_batter_before_first(before, event, &facts.batter_id)?;
    rules::validate_fielding_review(event)?;
    rules::validate_pa_credits(event)?;
    rules::validate_credit_counts(event)?;
    rules::validate_no_hit(event)?;
    rules::validate_run_credits(before, event, before.outs + 1)?;
    if event.delta.credits.iter().any(|c| {
        !rules::credit_role_eligible(before, event, c)
            || (c.kind == CreditKind::Putout && c.player_id != facts.fielder_id)
            || (c.player_id == facts.batter_id
                && matches!(c.kind, CreditKind::StolenBase | CreditKind::CaughtStealing))
    }) {
        return Err(TransitionError::IllegalCredit);
    }
    if event.fielding_review == FieldingReview::NoMisplay {
        for (kind, id) in [
            (CreditKind::PlateAppearance, &facts.batter_id),
            (CreditKind::AtBat, &facts.batter_id),
            (CreditKind::NoHit, &facts.batter_id),
            (CreditKind::Putout, &facts.fielder_id),
        ] {
            let mut units = event.delta.credits.iter().filter(|c| c.kind == kind);
            if units.next().is_none_or(|c| &c.player_id != id) || units.next().is_some() {
                return Err(TransitionError::IllegalCredit);
            }
        }
        if event
            .delta
            .credits
            .iter()
            .any(|c| c.kind == CreditKind::Assist)
        {
            return Err(TransitionError::IllegalCredit);
        }
    }
    // Preserve known context/actor contradictions before unsupported compounds.
    super::balk::preflight(before, event)?;
    super::pitch::preflight(before, event)?;
    super::contact::validate(before, event)?;
    if event.balk.as_ref().is_some_and(|balk| {
        matches!(&balk.resolution, BalkResolution::ActualMinimumProgress(progress)
            if progress.batter_first_touch.runner_id == facts.batter_id
                && progress.batter_first_touch.base == Destination::First)
    }) {
        return Err(TransitionError::IllegalCredit);
    }
    if event.two_out_contact.is_some() || event.single_delivery.is_some() || event.balk.is_some() {
        return Err(TransitionError::UnsupportedRule);
    }
    Ok(())
}
