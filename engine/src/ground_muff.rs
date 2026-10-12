//! One ordinary mechanical ground-ball muff, not inferred from Error credits.
use super::*;

pub(crate) fn validate(before: &GameState, event: &Event) -> Result<(), TransitionError> {
    let reviewed = event.fielding_review == FieldingReview::SoleGroundMuff;
    if event.muffed_ground_ball.is_none() && !reviewed {
        return Ok(());
    }
    rules::validate_snapshot(before)?;
    rules::validate_participants(before, event)?;
    if let Some(facts) = &event.muffed_ground_ball {
        if event.participants.batter.as_ref() != Some(&facts.batter_id) {
            return Err(TransitionError::InvalidParticipants);
        }
        if !rules::fielding_eligible(&before.teams[1 - before.half.offense()], &facts.fielder_id) {
            return Err(TransitionError::IllegalCredit);
        }
        if event.kind != EventKind::PlateAppearance {
            return Err(TransitionError::IllegalStateChange);
        }
    }
    rules::validate_pa_credits(event)?;
    rules::validate_credit_counts(event)?;
    rules::validate_no_hit(event)?;
    rules::validate_fielding_review(event)?;
    if event.delta.credits.iter().any(|c| {
        !rules::credit_role_eligible(before, event, c)
            || (reviewed
                && (matches!(c.kind, CreditKind::PassedBall | CreditKind::InterferenceAward)
                    // PO/A zero requires the bound no-chain/no-retirement method,
                    // not this review label alone on an unbound error play.
                    || (event.muffed_ground_ball.is_some()
                        && matches!(c.kind, CreditKind::Putout | CreditKind::Assist))))
    }) {
        return Err(TransitionError::IllegalCredit);
    }
    rules::validate_event_shape(event)?;
    rules::validate_move_origins(
        &before.bases,
        event,
        event
            .muffed_ground_ball
            .as_ref()
            .map(|_| event.participants.pitcher.as_str()),
    )?;
    let total_outs = rules::validate_outs(before, event)?;
    rules::validate_run_credits(before, event, total_outs)?;
    // A missing method cannot invent a foreign runner. This is identity only:
    // held/retired present actors may have unresolved intermediate attempts.
    if event.delta.credits.iter().any(|c| {
        matches!(c.kind, CreditKind::StolenBase | CreditKind::CaughtStealing)
            && event.participants.batter.as_ref() != Some(&c.player_id)
            && !before
                .bases
                .iter()
                .flatten()
                .any(|r| r.player_id == c.player_id)
            && !event
                .delta
                .moves
                .iter()
                .any(|m| m.runner.player_id == c.player_id)
    }) {
        return Err(TransitionError::IllegalCredit);
    }
    if let Some(facts) = &event.muffed_ground_ball {
        for kind in [
            CreditKind::PlateAppearance,
            CreditKind::AtBat,
            CreditKind::NoHit,
        ] {
            let mut units = event.delta.credits.iter().filter(|c| c.kind == kind);
            if units.next().is_none_or(|c| c.player_id != facts.batter_id) || units.next().is_some()
            {
                return Err(TransitionError::IllegalCredit);
            }
        }
        let named_errors = event
            .delta
            .credits
            .iter()
            .filter(|c| c.kind == CreditKind::Error && c.player_id == facts.fielder_id)
            .count();
        // Rule9.04(a)(3)'s error-RBI exception is before two outs only.
        // This bound lost before-first retirement would end the inning; this
        // rejects a supplied positive unit, not a new RBI/ER coverage proof.
        if before.outs == 2
            && event
                .delta
                .credits
                .iter()
                .any(|c| c.kind == CreditKind::Rbi)
        {
            return Err(TransitionError::IllegalCredit);
        }
        if named_errors == 0
            || (reviewed
                && event
                    .delta
                    .credits
                    .iter()
                    .filter(|c| c.kind == CreditKind::Error)
                    .count()
                    != 1)
            || event.delta.credits.iter().any(|c| {
                matches!(
                    c.kind,
                    CreditKind::Hit
                        | CreditKind::Single
                        | CreditKind::Double
                        | CreditKind::Triple
                        | CreditKind::Walk
                        | CreditKind::OrdinaryWalk
                        | CreditKind::IntentionalWalk
                        | CreditKind::Strikeout
                        | CreditKind::HitByPitch
                        | CreditKind::SacrificeBunt
                        | CreditKind::SacrificeFly
                        | CreditKind::InterferenceAward
                        | CreditKind::GroundedIntoDoublePlay
                        | CreditKind::StolenBase
                        | CreditKind::CaughtStealing
                        | CreditKind::WildPitch
                        | CreditKind::Balk
                )
            })
        {
            return Err(TransitionError::IllegalCredit);
        }
        if event.delta.outs != 0
            || !event.delta.changes.is_empty()
            || event.delta.moves.len() != before.bases.iter().flatten().count() + 1
            || event.delta.moves.iter().any(|m| {
                !matches!(
                    (m.from, m.to),
                    (Base::Plate, Destination::First)
                        | (Base::First, Destination::Second)
                        | (Base::Second, Destination::Third)
                        | (Base::Third, Destination::Home)
                )
            })
        {
            return Err(TransitionError::DeltaMismatch);
        }
    }
    // Preserve other families' known contradictions before missing binding.
    super::sole_safe_hit::validate(before, event)?;
    super::sole_home_run::validate(before, event)?;
    super::protected_award::validate(before, event)?;
    super::caught::validate(before, event)?;
    super::ground::validate(before, event)?;
    super::balk::preflight(before, event)?;
    super::pitch::preflight(before, event)?;
    super::contact::validate(before, event)?;
    if event.muffed_ground_ball.is_none()
        || event.two_out_contact.is_some()
        || event.single_delivery.is_some()
        || event.balk.is_some()
    {
        return Err(TransitionError::UnsupportedRule);
    }
    Ok(())
}
