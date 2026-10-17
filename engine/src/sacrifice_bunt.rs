//! Supplied fair grounded bunt, direct First throw and Rule 9.08(a) judgment.
//! This never reinterprets the older explicitly non-bunt ground-throw facts.
use crate::{
    accounting::{FieldingContribution, RbiContribution},
    rules, Base, CreditKind, Destination, Event, EventKind, EventTransition, FieldingReview,
    GameState, TransitionError,
};

/// Supplied scorer judgment, not inferred from endpoints, tactics or SH credits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SacrificeBuntJudgment {
    Unknown,
    SacrificingOwnChance,
    ExclusivelyForBaseHit,
}
/// One fair grounded bunt controlled and directly thrown to a distinct receiver
/// on First before this batter's first touch. Exclusively advances before-First
/// to initially clear Second; any before-Third runner holds throughout. No other
/// touch/throw/retirement, appeal, misplay, runner episode or personnel change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SacrificeBuntThrowFacts {
    pub batter_id: String,
    pub advancing_runner_id: String,
    pub thrower_id: String,
    pub receiver_id: String,
    pub judgment: SacrificeBuntJudgment,
}

pub(crate) fn validate(
    before: &GameState,
    event: &Event,
    facts: &SacrificeBuntThrowFacts,
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
    // Canonical contradictions precede unrelated missing scorer judgment.
    rules::validate_event_shape(event)?;
    rules::validate_move_origins(&before.bases, event, Some(&defense.pitcher))?;
    let total_outs = rules::validate_outs(before, event)?;
    rules::validate_fielding_review(event)?;
    rules::validate_pa_credits(event)?;
    rules::validate_credit_counts(event)?;
    rules::validate_no_hit(event)?;
    rules::validate_run_credits(before, event, total_outs)?;
    if event.ordinary_ground_throw.is_some()
        || event.ordinary_air_catch.is_some()
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
                        | CreditKind::SacrificeBunt
                        | CreditKind::Putout
                        | CreditKind::Assist
                )
        })
    {
        return Err(TransitionError::IllegalCredit);
    }
    let runner = before.bases[0]
        .as_ref()
        .ok_or(TransitionError::UnsupportedRule)?;
    if facts.advancing_runner_id != runner.player_id {
        return Err(TransitionError::InvalidParticipants);
    }
    for (kind, actor) in [
        (CreditKind::PlateAppearance, &facts.batter_id),
        (CreditKind::NoHit, &facts.batter_id),
        (CreditKind::SacrificeBunt, &facts.batter_id),
        (CreditKind::Putout, &facts.receiver_id),
        (CreditKind::Assist, &facts.thrower_id),
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
    if facts.judgment == SacrificeBuntJudgment::ExclusivelyForBaseHit {
        return Err(TransitionError::IllegalCredit);
    }
    crate::balk::preflight(before, event)?;
    crate::pitch::preflight(before, event)?;
    crate::contact::validate(before, event)?;
    if before.outs >= 2
        || before.bases[1].is_some()
        || event.delta.outs != 1
        || event.delta.runs != 0
        || event.delta.third_out.is_some()
        || !event.delta.changes.is_empty()
        || event.delta.moves.len() != 1 + before.bases.iter().flatten().count()
        || event.delta.moves.iter().any(|movement| {
            !matches!(
                (movement.from, movement.to),
                (Base::Plate, Destination::Out)
                    | (Base::First, Destination::Second)
                    | (Base::Third, Destination::Third)
            )
        })
        || event.two_out_contact.is_some()
        || event.single_delivery.is_some()
        || event.balk.is_some()
        || event.fielding_review != FieldingReview::NoMisplay
        || facts.judgment != SacrificeBuntJudgment::SacrificingOwnChance
    {
        return Err(TransitionError::UnsupportedRule);
    }
    Ok(())
}

impl EventTransition {
    pub fn try_new_with_sacrifice_bunt_throw_facts(
        before: GameState,
        event: Event,
        after: GameState,
        facts: SacrificeBuntThrowFacts,
    ) -> Result<Self, TransitionError> {
        Self::try_new_with_bunt_inner(before, event, after, None, None, Some(facts))
    }
    pub fn sacrifice_bunt_throw_facts(&self) -> Option<&SacrificeBuntThrowFacts> {
        self.sacrifice_bunt_throw.as_ref()
    }
    /// Qualified PO1/A1 within this event's before-defense scope, not a full box.
    pub fn sacrifice_bunt_throw_fielding(&self) -> Option<FieldingContribution> {
        let facts = self.sacrifice_bunt_throw.as_ref()?;
        Some(FieldingContribution {
            putouts: [(facts.receiver_id.clone(), 1)].into(),
            assists: [(facts.thrower_id.clone(), 1)].into(),
        })
    }
    /// Context-specific view of the existing legal-zero-run RBI proof.
    pub fn sacrifice_bunt_throw_rbi(&self) -> Option<RbiContribution> {
        self.sacrifice_bunt_throw.as_ref()?;
        self.rbi_contribution()
    }
}
