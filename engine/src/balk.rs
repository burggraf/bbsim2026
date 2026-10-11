//! One ordinary 6.02(a) imposed occurrence with exact before-state awards.
//! Other causes/progress/protection histories remain unsupported, not illegal.
use crate::{
    BalkCause, BalkDisposition, BalkFacts, BalkResolution, Base, CreditKind, Destination, Event,
    EventKind, FieldingReview, GameState, RunnerEvent, TransitionError,
};

fn units(event: &Event) -> usize {
    event
        .delta
        .credits
        .iter()
        .filter(|c| c.kind == CreditKind::Balk)
        .count()
}
fn exact_awards(facts: &BalkFacts) -> bool {
    (facts.cause, facts.disposition, facts.resolution)
        == (
            BalkCause::PitcherAction602a,
            BalkDisposition::PenaltyApplied,
            BalkResolution::ExactProtectedAwards,
        )
}

pub(crate) fn preflight(before: &GameState, event: &Event) -> Result<(), TransitionError> {
    if event.balk.is_none() && units(event) == 0 {
        return Ok(());
    }
    crate::rules::validate_snapshot(before)?;
    if before.over {
        return Err(TransitionError::InvalidState);
    }
    crate::rules::validate_participants(before, event)?;
    crate::rules::validate_pa_credits(event)?;
    crate::rules::validate_credit_counts(event)?;
    crate::rules::validate_fielding_review(event)?;
    crate::rules::validate_strikeout_eligibility(before, event)?;
    if event.delta.credits.iter().any(|c| {
        !crate::rules::credit_role_eligible(before, event, c)
            || (matches!(
                c.kind,
                CreditKind::Run | CreditKind::StolenBase | CreditKind::CaughtStealing
            ) && !event
                .delta
                .moves
                .iter()
                .any(|m| m.runner.player_id == c.player_id))
    }) {
        return Err(TransitionError::IllegalCredit);
    }
    // A final Out alone leaves prior arrival/holding unknown. Only an
    // automatic K or explicit matching before-first retirement proves that
    // this actual batter could not have completed a steal in the PA.
    if event.delta.credits.iter().any(|c| {
        c.kind == CreditKind::StolenBase
            && event.participants.batter.as_deref() == Some(c.player_id.as_str())
            && event.delta.moves.iter().any(|m| {
                m.runner.player_id == c.player_id
                    && m.from == Base::Plate
                    && m.to == Destination::Out
            })
            && ((before.outs < 2
                && before.bases[0].is_some()
                && event
                    .delta
                    .credits
                    .iter()
                    .any(|k| k.kind == CreditKind::Strikeout && k.player_id == c.player_id))
                || event.delta.third_out.as_ref().is_some_and(|third| {
                    third.runner_id == c.player_id
                        && third.kind == crate::ThirdOutKind::BatterBeforeFirst
                }))
    }) {
        return Err(TransitionError::IllegalCredit);
    }
    let Some(facts) = &event.balk else {
        return Ok(());
    };
    if units(event) > 1
        || (facts.cause == BalkCause::SqueezeOrStealHomeInterference601g
            && event.fielding_review == FieldingReview::NoMisplay)
        || (facts.cause == BalkCause::PitcherAction602a
            && facts.disposition == BalkDisposition::PlayProceedsWithoutReference
            && units(event) != 0)
    {
        return Err(TransitionError::IllegalCredit);
    }
    if exact_awards(facts) {
        let occupied = before.bases.iter().flatten().count();
        if occupied == 0
            || event.kind != EventKind::RunnerOnly(RunnerEvent::Balk)
            || event.delta.outs != 0
            || event.delta.third_out.is_some()
            || !event.delta.changes.is_empty()
            || event.delta.moves.len() != occupied
            || !before.bases.iter().enumerate().all(|(i, r)| r.as_ref().is_none_or(|r| {
                event.delta.moves.iter().filter(|m| {
                    m.runner.player_id == r.player_id
                        && m.from == [Base::First, Base::Second, Base::Third][i]
                        && m.to == [Destination::Second, Destination::Third, Destination::Home][i]
                }).count() == 1
            }))
            || units(event) != 1
            // This witness binds every move solely to its balk award, not to
            // an independent completed steal. Richer histories are separate.
            || event.delta.credits.iter().any(|c| c.kind == CreditKind::StolenBase)
        {
            return Err(TransitionError::IllegalCredit);
        }
    }
    Ok(())
}

pub(crate) fn validate(event: &Event) -> Result<(), TransitionError> {
    let Some(facts) = &event.balk else {
        return if units(event) == 0 {
            Ok(())
        } else {
            Err(TransitionError::UnsupportedRule)
        };
    };
    if matches!(
        facts.cause,
        BalkCause::SqueezeOrStealHomeInterference601g | BalkCause::PitchingProhibition602d
    ) || (facts.cause == BalkCause::PitcherAction602a
        && facts.disposition == BalkDisposition::PlayProceedsWithoutReference)
        || (!exact_awards(facts) && units(event) != 0)
        || (exact_awards(facts) && event.two_out_contact.is_some())
        || event.single_delivery.is_some()
        || event.delta.credits.iter().any(|c| {
            matches!(
                c.kind,
                CreditKind::Error
                    | CreditKind::WildPitch
                    | CreditKind::PassedBall
                    | CreditKind::StolenBase
            )
        })
    {
        // Omitting Balk does not resolve an asserted occurrence's relation to
        // a misplay chance, legal delivery or independently completed steal.
        // Final endpoints alone cannot bind cause or intermediate chronology.
        return Err(TransitionError::UnsupportedRule);
    }
    Ok(())
}
