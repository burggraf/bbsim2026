//! Explicit Rule 9.08(d)(1) caught-SF context, never inferred from labels/endpoints.
use crate::{
    accounting::RbiContribution, rules, Base, CreditKind, Destination, Event, EventKind,
    EventTransition, FieldingReview, GameState, TransitionError,
};

/// Supplied handling criterion, not a position guessed from a lineup index or PO.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SacrificeFlyHandling {
    Unknown,
    QualifyingOutfielderOrInfielderInOutfield,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SacrificeFlyRetouch {
    Unknown,
    /// Legal tag-up; departure at first fielder touch is allowed (Catch Comment).
    Legal,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SacrificeFlyRunTiming {
    Unknown,
    AfterCatch,
    BeforeCatch,
}
/// Supplementary modeled/reviewed assertions bound to this event's named catch.
/// Only one before-Third scorer and held other runners are supported here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaughtSacrificeFlyFacts {
    pub scoring_runner_id: String,
    pub handling: SacrificeFlyHandling,
    pub retouch: SacrificeFlyRetouch,
    pub timing: SacrificeFlyRunTiming,
}

pub(crate) fn validate(
    before: &GameState,
    event: &Event,
    facts: &CaughtSacrificeFlyFacts,
) -> Result<(), TransitionError> {
    rules::validate_snapshot(before)?;
    if before.over {
        return Err(TransitionError::InvalidState);
    }
    rules::validate_participants(before, event)?;
    let air = event
        .ordinary_air_catch
        .as_ref()
        .ok_or(TransitionError::UnsupportedRule)?;
    if event.participants.batter.as_ref() != Some(&air.batter_id) {
        return Err(TransitionError::InvalidParticipants);
    }
    let defense = &before.teams[1 - before.half.offense()];
    if !rules::fielding_eligible(defense, &air.fielder_id) {
        return Err(TransitionError::IllegalCredit);
    }
    if event.kind != EventKind::PlateAppearance {
        return Err(TransitionError::IllegalStateChange);
    }
    // Preserve canonical contradictions before unrelated handling uncertainty.
    rules::validate_event_shape(event)?;
    rules::validate_move_origins(&before.bases, event, defense.pitcher.active_id())?;
    let total_outs = rules::validate_outs(before, event)?;
    rules::validate_fielding_review(event)?;
    rules::validate_pa_credits(event)?;
    rules::validate_credit_counts(event)?;
    rules::validate_no_hit(event)?;
    rules::validate_run_credits(before, event, total_outs)?;
    if event.ordinary_ground_throw.is_some()
        || event.caught_strikeout.is_some()
        || event.protected_award.is_some()
        || event.sole_home_run.is_some()
        || event.sole_safe_hit.is_some()
        || event.muffed_ground_ball.is_some()
        || event.delta.credits.iter().any(|credit| {
            !rules::credit_role_eligible(before, event, credit)
                || !matches!(
                    credit.kind,
                    CreditKind::PlateAppearance
                        | CreditKind::NoHit
                        | CreditKind::SacrificeFly
                        | CreditKind::Rbi
                        | CreditKind::Run
                        | CreditKind::Putout
                )
        })
    {
        return Err(TransitionError::IllegalCredit);
    }
    let scorer = before.bases[2]
        .as_ref()
        .ok_or(TransitionError::UnsupportedRule)?;
    if facts.scoring_runner_id != scorer.player_id {
        return Err(TransitionError::InvalidParticipants);
    }
    for (kind, actor) in [
        (CreditKind::PlateAppearance, &air.batter_id),
        (CreditKind::NoHit, &air.batter_id),
        (CreditKind::SacrificeFly, &air.batter_id),
        (CreditKind::Rbi, &air.batter_id),
        (CreditKind::Run, &scorer.player_id),
        (CreditKind::Putout, &air.fielder_id),
    ] {
        let mut units = event
            .delta
            .credits
            .iter()
            .filter(|credit| credit.kind == kind);
        if units.next().is_none_or(|credit| &credit.player_id != actor) || units.next().is_some() {
            return Err(TransitionError::IllegalCredit);
        }
    }
    if facts.timing == SacrificeFlyRunTiming::BeforeCatch {
        return Err(TransitionError::IllegalCredit);
    }
    crate::balk::preflight(before, event)?;
    crate::pitch::preflight(before, event)?;
    crate::contact::validate(before, event)?;
    if before.outs >= 2
        || event.delta.outs != 1
        || event.delta.runs != 1
        || event.delta.third_out.is_some()
        || event.delta.moves.len() != 1 + before.bases.iter().flatten().count()
        || event.delta.moves.iter().any(|movement| {
            !matches!(
                (movement.from, movement.to),
                (Base::Plate, Destination::Out)
                    | (Base::First, Destination::First)
                    | (Base::Second, Destination::Second)
                    | (Base::Third, Destination::Home)
            )
        })
        || event.two_out_contact.is_some()
        || event.single_delivery.is_some()
        || event.balk.is_some()
        || event.fielding_review != FieldingReview::NoMisplay
        || facts.handling != SacrificeFlyHandling::QualifyingOutfielderOrInfielderInOutfield
        || facts.retouch != SacrificeFlyRetouch::Legal
        || facts.timing != SacrificeFlyRunTiming::AfterCatch
    {
        return Err(TransitionError::UnsupportedRule);
    }
    Ok(())
}

impl EventTransition {
    pub fn try_new_with_caught_sacrifice_fly_facts(
        before: GameState,
        event: Event,
        after: GameState,
        facts: CaughtSacrificeFlyFacts,
    ) -> Result<Self, TransitionError> {
        Self::try_new_inner(before, event, after, None, Some(facts))
    }
    pub fn caught_sacrifice_fly_facts(&self) -> Option<&CaughtSacrificeFlyFacts> {
        self.caught_sacrifice_fly.as_ref()
    }
    /// Causal event-local RBI, not merely a supplied credit or complete game cell.
    pub fn caught_sacrifice_fly_rbi(&self) -> Option<RbiContribution> {
        self.caught_sacrifice_fly.as_ref()?;
        Some(RbiContribution {
            runs_batted_in: [(self.event.participants.batter.as_ref()?.clone(), 1)].into(),
        })
    }
}
