//! Ordinary 6.02(a): imposed exact awards and named all-safe minimum progress.
//! Richer causes/retirement/appeal/protection histories need separate facts.
use crate::{
    BalkCause, BalkDisposition, BalkFacts, BalkProgressFacts, BalkResolution, Base, CreditKind,
    Destination, Event, EventKind, FieldingReview, GameState, RunnerEvent, TransitionError,
};

fn units(event: &Event) -> usize {
    event
        .delta
        .credits
        .iter()
        .filter(|c| c.kind == CreditKind::Balk)
        .count()
}
pub(crate) fn exact_awards(facts: &BalkFacts) -> bool {
    facts.cause == BalkCause::PitcherAction602a
        && facts.disposition == BalkDisposition::PenaltyApplied
        && matches!(&facts.resolution, BalkResolution::ExactProtectedAwards)
}

fn physical_preflight(before: &GameState, event: &Event) -> Result<(), TransitionError> {
    crate::rules::validate_event_shape(event)?;
    // Shape rejects movements during personnel changes. Before bases are the
    // canonical origins; FC batter replacement charge may still be unknown.
    crate::rules::validate_move_origins(&before.bases, event, None)?;
    let total_outs = crate::rules::validate_outs(before, event)?;
    if event.delta.outs == 0 && event.delta.third_out.is_none() {
        // No retirement/appeal ambiguity: reuse canonical named run checks
        // before missing progress can mask a definite represented error.
        crate::rules::validate_run_credits(before, event, total_outs)?;
    }
    Ok(())
}

fn touch_preflight(
    before: &GameState,
    event: &Event,
    facts: &BalkProgressFacts,
) -> Result<(), TransitionError> {
    let team = &before.teams[before.half.offense()];
    let batter = &team.batting_order[team.next_batter as usize];
    if facts.batter_first_touch.runner_id != *batter
        || facts.batter_first_touch.base != Destination::First
        || (event.delta.moves.iter().any(|m| {
            m.from == Base::Plate && m.runner.player_id == *batter && m.to == Destination::Out
        }) && (crate::rules::strikeout_first_ineligible(before, event)
            || event.delta.third_out.as_ref().is_some_and(|out| {
                out.runner_id == *batter && out.kind == crate::ThirdOutKind::BatterBeforeFirst
            })))
    {
        return Err(TransitionError::IllegalCredit);
    }
    for (i, touch) in facts.runner_next_touches.iter().enumerate() {
        let origin = before
            .bases
            .iter()
            .position(|r| r.as_ref().is_some_and(|r| r.player_id == touch.runner_id))
            .ok_or(TransitionError::IllegalCredit)?;
        if touch.base != [Destination::Second, Destination::Third, Destination::Home][origin]
            || facts.runner_next_touches[..i]
                .iter()
                .any(|r| r.runner_id == touch.runner_id)
        {
            return Err(TransitionError::IllegalCredit);
        }
    }
    Ok(())
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
        return physical_preflight(before, event);
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
    if let BalkResolution::ActualMinimumProgress(progress) = &facts.resolution {
        touch_preflight(before, event, progress)?;
        if facts.cause == BalkCause::PitcherAction602a
            && facts.disposition == BalkDisposition::PlayProceedsWithoutReference
            && before.bases.iter().all(Option::is_none)
        {
            return Err(TransitionError::IllegalCredit);
        }
    }
    physical_preflight(before, event)
}

fn at_or_beyond(actual: Destination, minimum: Destination) -> bool {
    let rank = |base| match base {
        Destination::First => Some(0),
        Destination::Second => Some(1),
        Destination::Third => Some(2),
        Destination::Home => Some(3),
        Destination::Out => None,
    };
    rank(actual)
        .zip(rank(minimum))
        .is_some_and(|(actual, minimum)| actual >= minimum)
}

fn validate_progress(
    before: &GameState,
    event: &Event,
    facts: &BalkProgressFacts,
) -> Result<(), TransitionError> {
    let occupied = before.bases.iter().flatten().count();
    if !event.kind.is_pa()
        || event.delta.outs != 0
        || event.delta.third_out.is_some()
        || facts.runner_next_touches.len() != occupied
        || !event.delta.moves.iter().any(|m| {
            m.from == Base::Plate
                && m.runner.player_id == facts.batter_first_touch.runner_id
                && at_or_beyond(m.to, facts.batter_first_touch.base)
        })
        || !before.bases.iter().enumerate().all(|(i, runner)| {
            runner.as_ref().is_none_or(|runner| {
                event.delta.moves.iter().any(|m| {
                    m.from == [Base::First, Base::Second, Base::Third][i]
                        && m.runner.player_id == runner.player_id
                        && at_or_beyond(
                            m.to,
                            [Destination::Second, Destination::Third, Destination::Home][i],
                        )
                })
            })
        })
    {
        // A supplied touch can precede return/retirement. This first shape
        // neither forbids that lawful history nor infers missing progress.
        return Err(TransitionError::UnsupportedRule);
    }
    if !matches!(
        crate::rules::ending(before, event, before.outs)?,
        crate::rules::Ending::Continue
    ) {
        return Err(TransitionError::UnsupportedRule);
    }
    Ok(())
}

pub(crate) fn validate(before: &GameState, event: &Event) -> Result<(), TransitionError> {
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
    ) || (!exact_awards(facts) && units(event) != 0)
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
    if let BalkResolution::ActualMinimumProgress(progress) = &facts.resolution {
        if facts.cause != BalkCause::PitcherAction602a
            || facts.disposition != BalkDisposition::PlayProceedsWithoutReference
        {
            return Err(TransitionError::UnsupportedRule);
        }
        return validate_progress(before, event, progress);
    }
    if facts.cause == BalkCause::PitcherAction602a
        && facts.disposition == BalkDisposition::PlayProceedsWithoutReference
    {
        return Err(TransitionError::UnsupportedRule);
    }
    Ok(())
}
