//! Supplied fair, non-bunt direct force/First relay; never inferred from credits.
use crate::{
    accounting::{FieldingContribution, RbiContribution},
    rules, *,
};

/// Clean control and direct throws: initial fielder to pivot touching Second
/// before the First runner, then pivot to First receiver before batter touch.
/// No other contact, runner episode or misplay. Initial may equal First receiver.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroundForceDoublePlayFacts {
    pub batter_id: String,
    pub forced_runner_id: String,
    pub initial_fielder_id: String,
    pub pivot_id: String,
    pub first_receiver_id: String,
}

pub fn apply_with_ground_force_double_play_facts(
    before: &GameState,
    event: Event,
    facts: GroundForceDoublePlayFacts,
) -> Result<EventTransition, TransitionError> {
    crate::reducer::apply_with_ground_force_double_play_facts(before, event, facts)
}

pub(crate) fn validate(
    before: &GameState,
    event: &Event,
    facts: &GroundForceDoublePlayFacts,
) -> Result<(), TransitionError> {
    rules::validate_snapshot(before)?;
    if before.over {
        return Err(TransitionError::InvalidState);
    }
    rules::validate_participants(before, event)?;
    if event.participants.batter.as_ref() != Some(&facts.batter_id) {
        return Err(TransitionError::InvalidParticipants);
    }
    let defense = &before.teams[1 - before.half.offense()];
    let offense = &before.teams[before.half.offense()];
    for actor in [
        &facts.initial_fielder_id,
        &facts.pivot_id,
        &facts.first_receiver_id,
    ] {
        if !rules::fielding_eligible(defense, actor)
            || offense.batting_order.contains(actor)
            || &offense.pitcher == actor
            || offense.bench.contains(actor)
            || offense.removed.contains(actor)
        {
            return Err(TransitionError::IllegalCredit);
        }
    }
    if facts.initial_fielder_id == facts.pivot_id || facts.first_receiver_id == facts.pivot_id {
        return Err(TransitionError::DeltaMismatch);
    }
    if event.kind != EventKind::PlateAppearance {
        return Err(TransitionError::IllegalStateChange);
    }
    // Preserve known canonical contradictions before missing contact history or
    // an unsupported terminal context. No guessed retirement order/credit cause.
    rules::validate_event_shape(event)?;
    rules::validate_move_origins(&before.bases, event, Some(&defense.pitcher))?;
    let outs = rules::validate_outs(before, event)?;
    rules::validate_fielding_review(event)?;
    rules::validate_pa_credits(event)?;
    rules::validate_credit_counts(event)?;
    rules::validate_no_hit(event)?;
    rules::validate_run_credits(before, event, outs)?;
    let expected = [
        (CreditKind::PlateAppearance, &facts.batter_id),
        (CreditKind::AtBat, &facts.batter_id),
        (CreditKind::NoHit, &facts.batter_id),
        (CreditKind::GroundedIntoDoublePlay, &facts.batter_id),
        (CreditKind::Putout, &facts.pivot_id),
        (CreditKind::Putout, &facts.first_receiver_id),
        (CreditKind::Assist, &facts.initial_fielder_id),
        (CreditKind::Assist, &facts.pivot_id),
    ];
    if event.delta.credits.len() != expected.len()
        || expected.iter().any(|(kind, actor)| {
            event
                .delta
                .credits
                .iter()
                .filter(|c| c.kind == *kind && &c.player_id == *actor)
                .count()
                != 1
        })
        || event.ordinary_ground_throw.is_some()
        || event.ordinary_air_catch.is_some()
        || event.caught_strikeout.is_some()
        || event.protected_award.is_some()
        || event.sole_home_run.is_some()
        || event.sole_safe_hit.is_some()
        || event.muffed_ground_ball.is_some()
    {
        return Err(TransitionError::IllegalCredit);
    }
    crate::balk::preflight(before, event)?;
    crate::pitch::preflight(before, event)?;
    crate::contact::validate(before, event)?;
    let runner = before.bases[0]
        .as_ref()
        .ok_or(TransitionError::UnsupportedRule)?;
    if runner.player_id != facts.forced_runner_id {
        return Err(TransitionError::InvalidParticipants);
    }
    let contact = TwoOutContactFacts {
        contact: ContactClass::Grounded,
        continuity: ContactContinuity::Continuous,
        first_out: RetirementFact {
            runner_id: facts.forced_runner_id.clone(),
            cause: RetirementCause::Force {
                at: Destination::Second,
            },
        },
        second_out: RetirementFact {
            runner_id: facts.batter_id.clone(),
            cause: RetirementCause::BatterPutoutBeforeFirstTouch,
        },
        error_placement: Some(ReviewedErrorPlacement {
            between_outs: vec![],
            outside_interval: vec![],
        }),
    };
    if before.outs >= 2
        || before.bases[1].is_some()
        || (before.half == Half::Bottom && before.inning >= 9)
        || event.delta.outs != 2
        || event.delta.runs != 0
        || !event.delta.changes.is_empty()
        || event.single_delivery.is_some()
        || event.balk.is_some()
        || event.fielding_review != FieldingReview::NoMisplay
        || event.two_out_contact.as_ref() != Some(&contact)
        || event.delta.moves.len() != 1 + before.bases.iter().flatten().count()
        || event.delta.moves.iter().any(|m| {
            !matches!(
                (m.from, m.to),
                (Base::Plate, Destination::Out)
                    | (Base::First, Destination::Out)
                    | (Base::Third, Destination::Third)
            )
        })
        || (before.outs == 1
            && event.delta.third_out.as_ref().is_none_or(|out| {
                out.runner_id != facts.batter_id
                    || out.kind != ThirdOutKind::BatterBeforeFirst
                    || !out.home_before_out.is_empty()
            }))
        || rules::ending(before, event, outs)? != rules::Ending::Continue
    {
        return Err(TransitionError::UnsupportedRule);
    }
    Ok(())
}

impl EventTransition {
    pub fn try_new_with_ground_force_double_play_facts(
        before: GameState,
        event: Event,
        after: GameState,
        facts: GroundForceDoublePlayFacts,
    ) -> Result<Self, TransitionError> {
        Self::try_new_with_ground_force_inner(before, event, after, None, None, None, Some(facts))
    }
    pub fn ground_force_double_play_facts(&self) -> Option<&GroundForceDoublePlayFacts> {
        self.ground_force_double_play.as_ref()
    }
    /// Complete PO2/A2 within this supplied episode's before-defense scope only.
    pub fn ground_force_double_play_fielding(&self) -> Option<FieldingContribution> {
        let facts = self.ground_force_double_play.as_ref()?;
        Some(FieldingContribution {
            putouts: [
                (facts.pivot_id.clone(), 1),
                (facts.first_receiver_id.clone(), 1),
            ]
            .into(),
            assists: [
                (facts.initial_fielder_id.clone(), 1),
                (facts.pivot_id.clone(), 1),
            ]
            .into(),
        })
    }
    /// Exposes existing legal-zero-run PA proof; no new generic RBI coverage.
    pub fn ground_force_double_play_rbi(&self) -> Option<RbiContribution> {
        self.ground_force_double_play.as_ref()?;
        self.rbi_contribution()
    }
}
