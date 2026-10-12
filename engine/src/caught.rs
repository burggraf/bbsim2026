//! Explicit ordinary direct caught third strike, not inferred from K/PO credits.
use super::*;

pub(crate) fn validate(before: &GameState, event: &Event) -> Result<(), TransitionError> {
    let Some(facts) = &event.caught_strikeout else {
        return Ok(());
    };
    let defense = &before.teams[1 - before.half.offense()];
    if event.participants.batter.as_ref() != Some(&facts.batter_id) {
        return Err(TransitionError::InvalidParticipants);
    }
    if !rules::fielding_eligible(defense, &facts.catcher_id) || facts.catcher_id == defense.pitcher
    {
        return Err(TransitionError::IllegalCredit);
    }
    if event.kind != EventKind::PlateAppearance {
        return Err(TransitionError::IllegalStateChange);
    }
    if event
        .delta
        .credits
        .iter()
        .filter(|c| c.kind == CreditKind::Strikeout)
        .count()
        != 1
        || event.ordinary_ground_throw.is_some()
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
            || (c.player_id == facts.batter_id
                && matches!(c.kind, CreditKind::StolenBase | CreditKind::CaughtStealing))
            || (c.kind == CreditKind::Putout && c.player_id != facts.catcher_id)
    }) {
        return Err(TransitionError::IllegalCredit);
    }
    if event.fielding_review == FieldingReview::NoMisplay {
        let mut putouts = event
            .delta
            .credits
            .iter()
            .filter(|c| c.kind == CreditKind::Putout);
        if putouts.next().is_none()
            || putouts.next().is_some()
            || event
                .delta
                .credits
                .iter()
                .any(|c| c.kind == CreditKind::Assist)
        {
            return Err(TransitionError::IllegalCredit);
        }
    }
    // Known contradictory terminal receipt or legal first touch cannot be
    // hidden by missing handling/compound chronology. Other family guards
    // remain authoritative for actors, classifications and actual retirements.
    if event
        .single_delivery
        .as_ref()
        .is_some_and(|f| f.context == DeliveryContext::UncaughtThirdStrike)
        || event.balk.as_ref().is_some_and(|balk| {
            matches!(&balk.resolution, BalkResolution::ActualMinimumProgress(progress)
                if progress.batter_first_touch.runner_id == facts.batter_id
                    && progress.batter_first_touch.base == Destination::First)
        })
    {
        return Err(TransitionError::IllegalCredit);
    }
    super::balk::preflight(before, event)?;
    super::pitch::preflight(before, event)?;
    super::contact::validate(before, event)?;
    if event.two_out_contact.is_some() || event.single_delivery.is_some() || event.balk.is_some() {
        return Err(TransitionError::UnsupportedRule);
    }
    Ok(())
}
