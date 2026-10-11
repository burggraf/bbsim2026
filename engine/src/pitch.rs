//! Bounded single-delivery scorer facts. Known contradictions precede missing history.
use crate::{
    Base, CreditKind, DeliveryContext, Destination, Event, EventKind, FieldingReview, GameState,
    HandlingAssessment, RecoveryReview, RunnerEvent, SingleDeliveryFacts, TransitionError,
};

fn units(event: &Event, kind: CreditKind) -> usize {
    event
        .delta
        .credits
        .iter()
        .filter(|c| c.kind == kind)
        .count()
}
fn supplied_units(event: &Event) -> usize {
    units(event, CreditKind::WildPitch) + units(event, CreditKind::PassedBall)
}
fn destination_rank(destination: Destination) -> Option<u8> {
    match destination {
        Destination::First => Some(1),
        Destination::Second => Some(2),
        Destination::Third => Some(3),
        Destination::Home => Some(4),
        Destination::Out => None,
    }
}
fn base_rank(base: Base) -> u8 {
    match base {
        Base::Plate => 0,
        Base::First => 1,
        Base::Second => 2,
        Base::Third => 3,
    }
}

fn award_floor(before: &GameState, context: DeliveryContext, from: Base) -> u8 {
    let origin = base_rank(from);
    if context != DeliveryContext::DeliveredFourthBall {
        origin
    } else if from == Base::Plate {
        1
    } else if before.bases[..usize::from(origin)]
        .iter()
        .all(Option::is_some)
    {
        origin + 1
    } else {
        origin
    }
}

/// Supplied SB must not contradict awards or explicit handling causality.
/// Return unresolved history separately so no unknown can mask a known error.
fn stolen_base_history(
    before: &GameState,
    event: &Event,
    facts: &SingleDeliveryFacts,
) -> Result<bool, TransitionError> {
    let mut unresolved = false;
    for credit in event
        .delta
        .credits
        .iter()
        .filter(|c| c.kind == CreditKind::StolenBase)
    {
        let movement = event
            .delta
            .moves
            .iter()
            .find(|m| m.runner.player_id == credit.player_id)
            .ok_or(TransitionError::IllegalCredit)?;
        let origin = base_rank(movement.from);
        let Some(reached) = destination_rank(movement.to) else {
            if movement.from == Base::Plate
                && ((facts.context == DeliveryContext::UncaughtThirdStrike
                    && before.outs < 2
                    && before.bases[0].is_some())
                    || event.delta.third_out.as_ref().is_some_and(|third| {
                        third.runner_id == credit.player_id
                            && third.kind == crate::ThirdOutKind::BatterBeforeFirst
                    }))
            {
                return Err(TransitionError::IllegalCredit);
            }
            // A completed intermediate steal followed by another-base retirement
            // is possible, but final Out does not bind arrival/holding chronology.
            unresolved = true;
            continue;
        };
        let award = award_floor(before, facts.context, movement.from);
        if movement.from == Base::Plate || reached <= award {
            return Err(TransitionError::IllegalCredit);
        }
        if award > origin {
            // An intended next base is this runner's protected award. Extra
            // advancement does not establish an independent completed steal.
            unresolved = true;
        }
        if facts
            .advances
            .iter()
            .any(|a| a.runner_id == credit.player_id)
        {
            match &facts.pre_delivery_steals {
                Some(starts) if starts.iter().any(|s| s.runner_id == credit.player_id) => {}
                Some(_)
                    if reached == origin + 1
                        && facts.advances.iter().any(|a| {
                            a.runner_id == credit.player_id && a.reached == movement.to
                        }) =>
                {
                    return Err(TransitionError::IllegalCredit)
                }
                _ => unresolved = true, // Longer/intermediate reaches need the steal/handling chronology.
            }
        }
    }
    Ok(unresolved)
}

/// Definite automatic retirement, not a subtraction of arbitrary K outs.
fn automatic_batter_out(before: &GameState, event: &Event, facts: &SingleDeliveryFacts) -> bool {
    facts.context == DeliveryContext::UncaughtThirdStrike
        && crate::rules::strikeout_first_ineligible(before, event)
        && event
            .delta
            .moves
            .iter()
            .any(|m| m.from == Base::Plate && m.to == Destination::Out)
}

fn automatic_out_only(before: &GameState, event: &Event, facts: &SingleDeliveryFacts) -> bool {
    automatic_batter_out(before, event, facts)
        && event.delta.outs == 1
        && event.delta.third_out.is_none()
        && event
            .delta
            .moves
            .iter()
            .all(|m| m.from == Base::Plate || m.to != Destination::Out)
}

/// The one additional out is bound to this delivery's named recovery actor.
fn automatic_recovery_out(before: &GameState, event: &Event, facts: &SingleDeliveryFacts) -> bool {
    if !automatic_batter_out(before, event, facts) || event.delta.outs != 2 {
        return false;
    }
    let id = match &facts.recovery {
        RecoveryReview::BeforeAnyAdvance { runner_id }
        | RecoveryReview::AfterAdvance { runner_id, .. } => runner_id,
        _ => return false,
    };
    let mut retired = event
        .delta
        .moves
        .iter()
        .filter(|m| m.from != Base::Plate && m.to == Destination::Out);
    retired.next().is_some_and(|m| &m.runner.player_id == id) && retired.next().is_none()
}

/// Supported final reaches, awards, steal starts and bounded recovery order.
fn qualification(before: &GameState, event: &Event, facts: &SingleDeliveryFacts) -> Option<bool> {
    if event.kind == EventKind::RunnerOnly(RunnerEvent::Balk) {
        return None;
    }
    let recovery_automatic = automatic_recovery_out(before, event, facts);
    let automatic = automatic_out_only(before, event, facts) || recovery_automatic;
    match &facts.recovery {
        RecoveryReview::BeforeAnyAdvance { .. }
        | RecoveryReview::RecoveredThirdStrikeBatterBeforeFirst => return Some(false),
        RecoveryReview::Unknown => return None,
        RecoveryReview::NoRecoveryPutout if event.delta.outs != 0 && !automatic => return None,
        RecoveryReview::AfterAdvance { .. } if event.delta.outs != 1 && !recovery_automatic => {
            return None
        }
        _ => {}
    }
    if facts.advances.iter().any(|a| {
        !event
            .delta
            .moves
            .iter()
            .any(|m| m.runner.player_id == a.runner_id && m.to == a.reached)
    }) || (facts.context == DeliveryContext::UncaughtThirdStrike
        && !automatic
        && !event
            .delta
            .moves
            .iter()
            .any(|m| m.from == Base::Plate && m.to == Destination::First))
    {
        return None;
    }
    // Supplied causality is not exhaustive review. In the automatic family,
    // an empty vector (even with no final advance), or an unbound final advance,
    // cannot certify suppression. An independent known qualifier may still win.
    let mut unresolved = automatic
        && (facts.advances.is_empty()
            || event.delta.moves.iter().any(|m| {
                m.from != Base::Plate
                    && destination_rank(m.to).is_some_and(|rank| rank > base_rank(m.from))
                    && !facts
                        .advances
                        .iter()
                        .any(|a| a.runner_id == m.runner.player_id && a.reached == m.to)
            }));
    for advance in &facts.advances {
        if let RecoveryReview::AfterAdvance { advancer_id, .. } = &facts.recovery {
            if &advance.runner_id != advancer_id {
                unresolved = true; // Only the named advance is ordered before recovery.
                continue;
            }
        }
        let movement = event
            .delta
            .moves
            .iter()
            .find(|m| m.runner.player_id == advance.runner_id)?;
        if movement.to != advance.reached {
            return None; // Intermediate reaches need more chronology, not endpoint inference.
        }
        let reached = destination_rank(advance.reached)?;
        let origin = base_rank(movement.from);
        let award = award_floor(before, facts.context, movement.from);
        if reached <= award {
            continue;
        }
        if movement.from == Base::Plate {
            return Some(true);
        }
        match &facts.pre_delivery_steals {
            Some(starts) => {
                let target = starts
                    .iter()
                    .find(|start| start.runner_id == advance.runner_id)
                    .map(|start| destination_rank(start.target))
                    .unwrap_or(Some(origin));
                if reached > target?.max(award) {
                    return Some(true);
                }
            }
            None if reached > (origin + 1).max(award) => return Some(true),
            None => unresolved = true,
        }
    }
    if unresolved {
        None
    } else {
        Some(false)
    }
}

fn recovery_facts(
    before: &GameState,
    event: &Event,
    facts: &SingleDeliveryFacts,
) -> Result<(), TransitionError> {
    let retired = match &facts.recovery {
        RecoveryReview::BeforeAnyAdvance { runner_id }
        | RecoveryReview::AfterAdvance { runner_id, .. } => Some(runner_id.as_str()),
        RecoveryReview::RecoveredThirdStrikeBatterBeforeFirst => {
            if facts.context != DeliveryContext::UncaughtThirdStrike {
                return Err(TransitionError::IllegalCredit);
            }
            event.participants.batter.as_deref()
        }
        _ => None,
    };
    if let Some(id) = retired {
        let movement = event
            .delta
            .moves
            .iter()
            .find(|m| m.runner.player_id == id && m.to == Destination::Out)
            .ok_or(TransitionError::IllegalCredit)?;
        if matches!(
            facts.recovery,
            RecoveryReview::BeforeAnyAdvance { .. }
                | RecoveryReview::RecoveredThirdStrikeBatterBeforeFirst
        ) && (facts.advances.iter().any(|a| a.runner_id == id)
            || event
                .delta
                .credits
                .iter()
                .any(|c| c.kind == CreditKind::StolenBase && c.player_id == id))
        {
            // These retirements explicitly precede this actor's safe causal reach.
            // Other actors may advance later; intermediate reaches with other
            // recovery reviews remain chronology questions, not contradictions.
            return Err(TransitionError::IllegalCredit);
        }
        if movement.from == Base::Plate
            && facts.context == DeliveryContext::UncaughtThirdStrike
            && before.outs < 2
            && before.bases[0].is_some()
        {
            return Err(TransitionError::IllegalCredit); // Automatic K is not a defensive recovery putout.
        }
        if facts.recovery == RecoveryReview::RecoveredThirdStrikeBatterBeforeFirst
            && (movement.from != Base::Plate
                || event.delta.third_out.as_ref().is_some_and(|third| {
                    third.runner_id == id
                        && matches!(third.kind, crate::ThirdOutKind::BatterAfterFirstTiming)
                }))
        {
            return Err(TransitionError::IllegalCredit);
        }
        if let Some(third) = &event.delta.third_out {
            if automatic_recovery_out(before, event, facts)
                && (third.runner_id != id
                    || matches!(
                        third.kind,
                        crate::ThirdOutKind::Force
                            | crate::ThirdOutKind::BatterBeforeFirst
                            | crate::ThirdOutKind::BatterAfterFirstTiming
                    ))
            {
                // The automatic K precedes this pitch's recovery out and
                // cannot leave a batter-created force on the before runner.
                return Err(TransitionError::DeltaMismatch);
            }
            if third.runner_id == id {
                match &facts.recovery {
                    RecoveryReview::BeforeAnyAdvance { .. }
                        if !third.home_before_out.is_empty() =>
                    {
                        return Err(TransitionError::IllegalCredit)
                    }
                    RecoveryReview::AfterAdvance { advancer_id, .. } => {
                        if facts
                            .advances
                            .iter()
                            .any(|a| &a.runner_id == advancer_id && a.reached == Destination::Home)
                            && !third.home_before_out.contains(advancer_id)
                        {
                            return Err(TransitionError::IllegalCredit);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    if let RecoveryReview::AfterAdvance { advancer_id, .. } = &facts.recovery {
        if !facts.advances.iter().any(|a| &a.runner_id == advancer_id) {
            return Err(TransitionError::IllegalCredit);
        }
    }
    Ok(())
}

pub(crate) fn preflight(before: &GameState, event: &Event) -> Result<(), TransitionError> {
    if event
        .single_delivery
        .as_ref()
        .is_some_and(|facts| facts.context == DeliveryContext::UncaughtThirdStrike)
    {
        crate::rules::validate_snapshot(before)?;
        if before.over {
            return Err(TransitionError::InvalidState);
        }
        crate::rules::validate_participants(before, event)?;
    }
    let defense = &before.teams[1 - before.half.offense()];
    if event
        .delta
        .credits
        .iter()
        .any(|c| c.kind == CreditKind::WildPitch && c.player_id != defense.pitcher)
    {
        return Err(TransitionError::IllegalCredit);
    }
    if units(event, CreditKind::WildPitch) != 0 || event.single_delivery.is_some() {
        crate::rules::validate_pa_credits(event)?;
        crate::rules::validate_credit_counts(event)?;
        crate::rules::validate_fielding_review(event)?;
        if event
            .delta
            .credits
            .iter()
            .any(|c| !crate::rules::credit_role_eligible(before, event, c))
        {
            return Err(TransitionError::IllegalCredit);
        }
        crate::rules::validate_strikeout_eligibility(before, event)?;
    }
    let Some(facts) = &event.single_delivery else {
        return Ok(());
    };
    if facts.catcher_id == defense.pitcher
        || !crate::rules::fielding_eligible(defense, &facts.catcher_id)
        || supplied_units(event) > 1
        || event
            .delta
            .credits
            .iter()
            .any(|c| c.kind == CreditKind::PassedBall && c.player_id != facts.catcher_id)
        || match facts.handling {
            HandlingAssessment::PitcherWild => units(event, CreditKind::PassedBall) != 0,
            HandlingAssessment::CatcherPassed => {
                units(event, CreditKind::WildPitch) != 0
                    || event.fielding_review == FieldingReview::NoMisplay
            }
        }
    {
        return Err(TransitionError::IllegalCredit);
    }
    let context_matches = match facts.context {
        DeliveryContext::RunnerOnly => matches!(event.kind, EventKind::RunnerOnly(_)),
        DeliveryContext::UncaughtThirdStrike => {
            event.kind == EventKind::PlateAppearance
                && units(event, CreditKind::Strikeout) == 1
                && event.delta.credits.iter().any(|c| {
                    c.kind == CreditKind::Strikeout
                        && event.participants.batter.as_deref() == Some(c.player_id.as_str())
                })
        }
        DeliveryContext::DeliveredFourthBall => {
            event.kind == EventKind::PlateAppearance
                && units(event, CreditKind::Walk) == 1
                && event.delta.credits.iter().any(|c| {
                    c.kind == CreditKind::Walk
                        && event.participants.batter.as_deref() == Some(c.player_id.as_str())
                })
        }
    };
    if !context_matches {
        return Err(TransitionError::IllegalCredit);
    }
    if facts.context == DeliveryContext::UncaughtThirdStrike {
        crate::rules::validate_event_shape(event)?;
        // A retired batter cannot be an ambiguous safe FC replacement. Its
        // canonical current-pitcher charge is known without recursive inference.
        let retired_batter = event
            .delta
            .moves
            .iter()
            .any(|m| m.from == Base::Plate && m.to == Destination::Out);
        crate::rules::validate_move_origins(
            &before.bases,
            event,
            retired_batter.then_some(defense.pitcher.as_str()),
        )?;
        let _ = crate::rules::move_destinations(before.bases.clone(), event)?;
        let total_outs = crate::rules::validate_outs(before, event)?;
        let known_runner_timing = event.delta.third_out.as_ref().is_some_and(|third| {
            third.kind == crate::ThirdOutKind::Timing
                && event.delta.moves.iter().any(|m| {
                    m.from != Base::Plate
                        && m.to == Destination::Out
                        && m.runner.player_id == third.runner_id
                })
        });
        if total_outs < 3 || known_runner_timing {
            crate::rules::validate_run_credits(before, event, total_outs)?;
        }
    }
    for advance in &facts.advances {
        let movement = event
            .delta
            .moves
            .iter()
            .find(|m| m.runner.player_id == advance.runner_id);
        if advance.reached == Destination::Home
            && automatic_batter_out(before, event, facts)
            && before.outs + event.delta.outs < 3
            && before
                .bases
                .iter()
                .flatten()
                .any(|runner| runner.player_id == advance.runner_id)
            && movement.is_none_or(|m| m.to != Destination::Home)
        {
            // A legal Home acquisition scores in this non-third-out frame;
            // returning cannot nullify it (5.08(a) Comment). Missing pitch
            // review cannot turn that known final-state conflict into unknown.
            return Err(TransitionError::DeltaMismatch);
        }
        let Some(movement) = movement else {
            let origin = before
                .bases
                .iter()
                .position(|runner| {
                    runner
                        .as_ref()
                        .is_some_and(|r| r.player_id == advance.runner_id)
                })
                .filter(|_| automatic_batter_out(before, event, facts))
                .ok_or(TransitionError::IllegalCredit)?;
            if destination_rank(advance.reached).is_none_or(|rank| rank <= origin as u8 + 1) {
                return Err(TransitionError::IllegalCredit);
            }
            // A known held before runner may have reached Second/Third and
            // returned with no net move. Qualification defers that chronology; it is
            // not the foreign-actor contradiction, nor a new positive qualifier.
            continue;
        };
        if destination_rank(advance.reached).is_none_or(|rank| rank <= base_rank(movement.from))
            || (movement.from == Base::Plate && facts.context == DeliveryContext::RunnerOnly)
            || (movement.from == Base::Plate
                && facts.context == DeliveryContext::UncaughtThirdStrike
                && crate::rules::strikeout_first_ineligible(before, event))
        {
            return Err(TransitionError::IllegalCredit);
        }
    }
    if let Some(starts) = &facts.pre_delivery_steals {
        for start in starts {
            let base = before
                .bases
                .iter()
                .position(|runner| {
                    runner
                        .as_ref()
                        .is_some_and(|r| r.player_id == start.runner_id)
                })
                .ok_or(TransitionError::IllegalCredit)?;
            if destination_rank(start.target) != Some(base as u8 + 2) {
                return Err(TransitionError::IllegalCredit);
            }
        }
    }
    recovery_facts(before, event, facts)?;
    let _ = stolen_base_history(before, event, facts)?;
    if let Some(qualified) = qualification(before, event, facts) {
        if supplied_units(event) != usize::from(qualified) {
            return Err(TransitionError::IllegalCredit);
        }
    }
    Ok(())
}

pub(crate) fn validate(before: &GameState, event: &Event) -> Result<(), TransitionError> {
    let Some(facts) = &event.single_delivery else {
        return if units(event, CreditKind::WildPitch) == 0 {
            Ok(()) // Preserve legacy PB units and labels as partial views.
        } else {
            Err(TransitionError::UnsupportedRule)
        };
    };
    if stolen_base_history(before, event, facts)?
        || (qualification(before, event, facts).is_none() && supplied_units(event) != 0)
    {
        return Err(TransitionError::UnsupportedRule);
    }
    if automatic_recovery_out(before, event, facts)
        && matches!(facts.recovery, RecoveryReview::AfterAdvance { .. })
        && before.half == crate::Half::Bottom
        && before.inning >= 9
        && before.score[1]
            .checked_add(u16::from(event.delta.runs))
            .is_some_and(|home| home > before.score[0])
    {
        // AfterAdvance binds its witness before recovery, not recovery before
        // the game's winning run. End-game/appeal order is still unrepresented;
        // BeforeAnyAdvance, by contrast, supplies that earlier-out ordering.
        return Err(TransitionError::UnsupportedRule);
    }
    Ok(())
}
