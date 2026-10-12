//! Explicit exclusive protected BB/HBP advancement, not inferred from net bases.
use super::*;

pub(crate) fn validate(before: &GameState, event: &Event) -> Result<(), TransitionError> {
    let Some(facts) = &event.protected_award else {
        return Ok(());
    };
    if event.participants.batter.as_ref() != Some(&facts.batter_id) {
        return Err(TransitionError::InvalidParticipants);
    }
    if event.kind != EventKind::PlateAppearance {
        return Err(TransitionError::IllegalStateChange);
    }
    rules::validate_pa_credits(event)?;
    rules::validate_credit_counts(event)?;
    rules::validate_no_hit(event)?;
    rules::validate_fielding_review(event)?;
    if event
        .delta
        .credits
        .iter()
        .filter(|c| matches!(c.kind, CreditKind::Walk | CreditKind::HitByPitch))
        .count()
        != 1
        || event.delta.credits.iter().any(|c| {
            !rules::credit_role_eligible(before, event, c)
                // Full review of this bound zero-retirement award rules out
                // Rule 9.10's error-prevented-out assist exception.
                || (event.fielding_review == FieldingReview::NoMisplay && c.kind == CreditKind::Assist)
                || matches!(
                    c.kind,
                    CreditKind::StolenBase
                        | CreditKind::CaughtStealing
                        | CreditKind::WildPitch
                        | CreditKind::PassedBall
                        | CreditKind::Balk
                )
        })
    {
        return Err(TransitionError::IllegalCredit);
    }
    rules::validate_event_shape(event)?;
    rules::validate_move_origins(&before.bases, event, Some(&event.participants.pitcher))?;
    rules::validate_outs(before, event)?;
    if event.delta.outs != 0
        || !event.delta.changes.is_empty()
        || event.delta.third_out.is_some()
        || !rules::protected_award_moves(before, event)
    {
        return Err(TransitionError::DeltaMismatch);
    }
    rules::validate_run_credits(before, event, before.outs)?;
    let rbi = usize::from(before.bases.iter().all(Option::is_some));
    if usize::from(event.delta.runs) != rbi
        || event
            .delta
            .credits
            .iter()
            .filter(|c| c.kind == CreditKind::Rbi)
            .count()
            != rbi
    {
        return Err(TransitionError::IllegalCredit);
    }
    // Preserve other families' known contradictions before compound uncertainty.
    super::caught::validate(before, event)?;
    super::ground::validate(before, event)?;
    super::balk::preflight(before, event)?;
    super::pitch::preflight(before, event)?;
    super::contact::validate(before, event)?;
    if event.two_out_contact.is_some() || event.single_delivery.is_some() || event.balk.is_some() {
        return Err(TransitionError::UnsupportedRule);
    }
    Ok(())
}
