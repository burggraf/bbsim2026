//! One explicitly bound ordinary direct ground throw to first. Generic contact
//! and supplied PO/A are never promoted into these factual assertions.
use super::*;

pub(crate) fn validate(before: &GameState, event: &Event) -> Result<(), TransitionError> {
    let Some(facts) = &event.ordinary_ground_throw else {
        return Ok(());
    };
    let defense = &before.teams[1 - before.half.offense()];
    if event.participants.batter.as_ref() != Some(&facts.batter_id) {
        return Err(TransitionError::InvalidParticipants);
    }
    if !rules::fielding_eligible(defense, &facts.thrower_id)
        || !rules::fielding_eligible(defense, &facts.receiver_id)
    {
        return Err(TransitionError::IllegalCredit);
    }
    if facts.thrower_id == facts.receiver_id {
        return Err(TransitionError::DeltaMismatch);
    }
    if event.kind != EventKind::PlateAppearance {
        return Err(TransitionError::IllegalStateChange);
    }
    if event.delta.credits.iter().any(|c| {
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
    }) {
        return Err(TransitionError::IllegalCredit);
    }
    rules::validate_event_shape(event)?;
    rules::validate_move_origins(&before.bases, event, Some(&defense.pitcher))?;
    rules::validate_outs(before, event)?;
    if event.delta.outs != 1
        || event.delta.runs != 0
        || event.delta.moves.iter().any(|m| {
            !matches!(
                (m.from, m.to),
                (Base::Plate, Destination::Out)
                    | (Base::First, Destination::First)
                    | (Base::Second, Destination::Second)
                    | (Base::Third, Destination::Third)
            )
        })
    {
        return Err(TransitionError::DeltaMismatch);
    }
    match (&event.delta.third_out, before.outs) {
        (Some(out), 2)
            if out.runner_id == facts.batter_id
                && out.kind == ThirdOutKind::BatterBeforeFirst
                && out.home_before_out.is_empty() => {}
        (None, 0 | 1) => {}
        _ => return Err(TransitionError::DeltaMismatch),
    }
    rules::validate_fielding_review(event)?;
    rules::validate_pa_credits(event)?;
    rules::validate_credit_counts(event)?;
    rules::validate_no_hit(event)?;
    rules::validate_run_credits(before, event, before.outs + 1)?;
    if event.delta.credits.iter().any(|c| {
        !rules::credit_role_eligible(before, event, c)
                // This batter never legally acquired first; unrelated missing
                // history cannot supply his completed steal or CS retirement.
                || (c.player_id == facts.batter_id
                    && matches!(c.kind, CreditKind::StolenBase | CreditKind::CaughtStealing))
    }) {
        return Err(TransitionError::IllegalCredit);
    }
    // A definite PO for another actor cannot belong to this sole actual out.
    if event
        .delta
        .credits
        .iter()
        .any(|c| c.kind == CreditKind::Putout && c.player_id != facts.receiver_id)
    {
        return Err(TransitionError::IllegalCredit);
    }
    if event.fielding_review == FieldingReview::NoMisplay {
        // These facts plus whole-chance review bound all eligible PO/A units.
        // Unknown earlier missed chances cannot justify this exact-unit check.
        for (kind, id) in [
            (CreditKind::PlateAppearance, &facts.batter_id),
            (CreditKind::AtBat, &facts.batter_id),
            (CreditKind::NoHit, &facts.batter_id),
            (CreditKind::Putout, &facts.receiver_id),
            (CreditKind::Assist, &facts.thrower_id),
        ] {
            let mut units = event.delta.credits.iter().filter(|c| c.kind == kind);
            if units.next().is_none_or(|c| &c.player_id != id) || units.next().is_some() {
                return Err(TransitionError::IllegalCredit);
            }
        }
    }
    // A missing compound binding must not hide the existing families' known
    // context/actor/retirement contradictions. These guards do not recurse into
    // batter_charge or qualify an unrepresented compound episode.
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
        // Separate compound episodes need their own binding; not impossible.
        return Err(TransitionError::UnsupportedRule);
    }
    Ok(())
}
