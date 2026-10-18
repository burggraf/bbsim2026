//! Explicit sole unaided out-of-park HR cause, not inferred from supplied RBI.
use super::*;

pub(crate) fn validate(before: &GameState, event: &Event) -> Result<(), TransitionError> {
    let Some(facts) = &event.sole_home_run else {
        return Ok(());
    };
    rules::validate_snapshot(before)?;
    rules::validate_participants(before, event)?;
    if event.participants.batter.as_ref() != Some(&facts.batter_id) {
        return Err(TransitionError::InvalidParticipants);
    }
    if event.kind != EventKind::OutOfParkHomeRun {
        return Err(TransitionError::IllegalStateChange);
    }
    rules::validate_pa_credits(event)?;
    rules::validate_credit_counts(event)?;
    rules::validate_no_hit(event)?;
    rules::validate_fielding_review(event)?;
    rules::validate_hit_value(before, event)?;
    let _ = rules::walk_classification(before, event)?;
    if event.delta.credits.iter().any(|c| {
        !rules::credit_role_eligible(before, event, c)
            // This bound zero-retirement cause plus full review excludes every
            // Rule 9.10 assist; Unknown earlier chances remain partial.
            || (event.fielding_review == FieldingReview::NoMisplay && c.kind == CreditKind::Assist)
            || matches!(
                c.kind,
                CreditKind::StolenBase
                    | CreditKind::CaughtStealing
                    | CreditKind::WildPitch
                    | CreditKind::PassedBall
                    | CreditKind::Balk
            )
    }) {
        return Err(TransitionError::IllegalCredit);
    }
    rules::validate_event_shape(event)?;
    rules::validate_move_origins(&before.bases, event, event.participants.pitcher.as_deref())?;
    rules::validate_outs(before, event)?;
    rules::validate_home_run(before, event)?;
    if !event.delta.changes.is_empty() {
        return Err(TransitionError::DeltaMismatch);
    }
    rules::validate_run_credits(before, event, before.outs)?;
    let runs = 1 + before.bases.iter().flatten().count();
    if usize::from(event.delta.runs) != runs
        || event
            .delta
            .credits
            .iter()
            .filter(|c| c.kind == CreditKind::Rbi)
            .count()
            != runs
    {
        return Err(TransitionError::IllegalCredit);
    }
    // Preserve other families' known contradictions before excluding richer
    // episodes from this proof. None leaves their existing semantics unchanged.
    super::protected_award::validate(before, event)?;
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
