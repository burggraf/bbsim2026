//! Explicit exclusive unaided safe-hit cause; never inferred from supplied RBI.
use super::*;

pub(crate) fn validate(before: &GameState, event: &Event) -> Result<(), TransitionError> {
    let Some(facts) = &event.sole_safe_hit else {
        return Ok(());
    };
    rules::validate_snapshot(before)?;
    rules::validate_participants(before, event)?;
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
    rules::hit_value_credit(event)?.ok_or(TransitionError::IllegalCredit)?;
    let _ = rules::walk_classification(before, event)?;
    if event.delta.credits.iter().any(|c| {
        !rules::credit_role_eligible(before, event, c)
            || matches!(
                c.kind,
                CreditKind::StolenBase
                    | CreditKind::CaughtStealing
                    | CreditKind::WildPitch
                    | CreditKind::PassedBall
                    | CreditKind::Balk
                    | CreditKind::Putout
                    | CreditKind::Assist
            )
    }) {
        return Err(TransitionError::IllegalCredit);
    }
    rules::validate_event_shape(event)?;
    rules::validate_move_origins(&before.bases, event, Some(&event.participants.pitcher))?;
    rules::validate_outs(before, event)?;
    if event.delta.outs != 0
        || !event.delta.changes.is_empty()
        || event.delta.moves.len() != before.bases.iter().flatten().count() + 1
        || event.delta.moves.iter().any(|m| {
            let origin = match m.from {
                Base::Plate => 0,
                Base::First => 1,
                Base::Second => 2,
                Base::Third => 3,
            };
            let target = match m.to {
                Destination::First => 1,
                Destination::Second => 2,
                Destination::Third => 3,
                Destination::Home => 4,
                Destination::Out => 0,
            };
            target == 0 || target < origin || (origin == 0 && target == 4)
        })
    {
        return Err(TransitionError::DeltaMismatch);
    }
    rules::validate_run_credits(before, event, before.outs)?;
    let runs = event
        .delta
        .moves
        .iter()
        .filter(|m| m.to == Destination::Home)
        .count();
    if event
        .delta
        .credits
        .iter()
        .filter(|c| c.kind == CreditKind::Rbi)
        .count()
        != runs
    {
        return Err(TransitionError::IllegalCredit);
    }
    // Other bound families' known contradictions precede unrelated review.
    super::sole_home_run::validate(before, event)?;
    super::protected_award::validate(before, event)?;
    super::caught::validate(before, event)?;
    super::ground::validate(before, event)?;
    super::balk::preflight(before, event)?;
    super::pitch::preflight(before, event)?;
    super::contact::validate(before, event)?;
    rules::validate_hit_value(before, event)?;
    if event.two_out_contact.is_some() || event.single_delivery.is_some() || event.balk.is_some() {
        return Err(TransitionError::UnsupportedRule);
    }
    Ok(())
}
