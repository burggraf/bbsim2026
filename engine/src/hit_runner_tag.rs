//! Supplied safe-hit attainment followed by a separate advancing-runner tag.
use crate::*;

/// Explicit chronology, not movement-vector order or an inferred touch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitRunnerTagOrder {
    BatterAttainedCreditedBaseBeforeTag,
}

/// One clean non-force throw/tag after the batter attained the scored hit base.
/// The runner legally touched intervening bases, then was tagged before acquiring
/// attempted_base. No error, appeal, overslide, separate force-removal episode
/// or other out cause.
/// Defensive eligibility is not evidence of real field positions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HitRunnerTagFacts {
    pub batter_id: String,
    pub runner_id: String,
    pub attempted_base: Destination,
    pub thrower_id: String,
    pub tagger_id: String,
    pub order: HitRunnerTagOrder,
    /// Physical home crossings before this tag, explicitly supplied even if empty.
    pub home_before_tag: Vec<String>,
}

pub fn apply_with_hit_runner_tag_facts(
    before: &GameState,
    event: Event,
    facts: HitRunnerTagFacts,
) -> Result<EventTransition, TransitionError> {
    crate::reducer::apply_with_hit_runner_tag_facts(before, event, facts)
}

impl EventTransition {
    pub fn try_new_with_hit_runner_tag_facts(
        before: GameState,
        event: Event,
        after: GameState,
        facts: HitRunnerTagFacts,
    ) -> Result<Self, TransitionError> {
        Self::try_new_with_hit_runner_tag_inner(
            before,
            event,
            after,
            None,
            None,
            None,
            None,
            Some(facts),
        )
    }
    pub fn hit_runner_tag_facts(&self) -> Option<&HitRunnerTagFacts> {
        self.hit_runner_tag.as_ref()
    }
}

fn origin(base: Base) -> u8 {
    match base {
        Base::Plate => 0,
        Base::First => 1,
        Base::Second => 2,
        Base::Third => 3,
    }
}
fn target(base: Destination) -> u8 {
    match base {
        Destination::First => 1,
        Destination::Second => 2,
        Destination::Third => 3,
        Destination::Home => 4,
        Destination::Out => 0,
    }
}

pub(crate) fn validate(
    before: &GameState,
    event: &Event,
    facts: &HitRunnerTagFacts,
) -> Result<(), TransitionError> {
    rules::validate_snapshot(before)?;
    rules::validate_participants(before, event)?;
    if event.participants.batter.as_ref() != Some(&facts.batter_id) {
        return Err(TransitionError::InvalidParticipants);
    }
    if event.kind != EventKind::PlateAppearance {
        return Err(TransitionError::IllegalStateChange);
    }
    if event.ordinary_ground_throw.is_some()
        || event.ordinary_air_catch.is_some()
        || event.caught_strikeout.is_some()
        || event.protected_award.is_some()
        || event.sole_home_run.is_some()
        || event.sole_safe_hit.is_some()
        || event.muffed_ground_ball.is_some()
        || event.two_out_contact.is_some()
        || event.single_delivery.is_some()
        || event.balk.is_some()
    {
        return Err(TransitionError::UnsupportedRule);
    }
    rules::validate_event_shape(event)?;
    rules::validate_pa_credits(event)?;
    rules::validate_credit_counts(event)?;
    rules::validate_no_hit(event)?;
    rules::validate_fielding_review(event)?;
    let value = rules::hit_value_credit(event)?.ok_or(TransitionError::IllegalCredit)?;
    let defense = &before.teams[1 - before.half.offense()];
    let offense = &before.teams[before.half.offense()];
    for actor in [&facts.thrower_id, &facts.tagger_id] {
        if !rules::fielding_eligible(defense, actor)
            || offense.batting_order.contains(actor)
            || &offense.pitcher == actor
            || offense.bench.contains(actor)
            || offense.removed.contains(actor)
        {
            return Err(TransitionError::IllegalCredit);
        }
    }
    if facts.thrower_id == facts.tagger_id {
        return Err(TransitionError::DeltaMismatch);
    }
    let required = [
        (CreditKind::PlateAppearance, &facts.batter_id),
        (CreditKind::Hit, &facts.batter_id),
        (
            [CreditKind::Single, CreditKind::Double, CreditKind::Triple][usize::from(value - 1)],
            &facts.batter_id,
        ),
        (CreditKind::Assist, &facts.thrower_id),
        (CreditKind::Putout, &facts.tagger_id),
    ];
    if required.iter().any(|(kind, actor)| {
        event
            .delta
            .credits
            .iter()
            .filter(|c| c.kind == *kind && &c.player_id == *actor)
            .count()
            != 1
    }) || event.delta.credits.iter().any(|c| {
        !rules::credit_role_eligible(before, event, c)
            || (!required
                .iter()
                .any(|(kind, actor)| c.kind == *kind && &c.player_id == *actor)
                && !matches!(
                    c.kind,
                    CreditKind::AtBat | CreditKind::Run | CreditKind::Rbi
                ))
    }) {
        return Err(TransitionError::IllegalCredit);
    }
    rules::validate_move_origins(&before.bases, event, Some(&defense.pitcher))?;
    let total_outs = rules::validate_outs(before, event)?;
    let retired = event
        .delta
        .moves
        .iter()
        .find(|m| {
            m.runner.player_id == facts.runner_id
                && m.from != Base::Plate
                && m.to == Destination::Out
        })
        .ok_or(TransitionError::DeltaMismatch)?;
    let batter = event
        .delta
        .moves
        .iter()
        .find(|m| m.from == Base::Plate)
        .ok_or(TransitionError::DeltaMismatch)?;
    if event.delta.outs != 1
        || event.delta.moves.len() != before.bases.iter().flatten().count() + 1
        || target(batter.to) != value
        || event.delta.moves.iter().any(|m| {
            m.from != Base::Plate && m.to != Destination::Out && target(m.to) < origin(m.from)
        })
    {
        return Err(TransitionError::DeltaMismatch);
    }
    let attempted = target(facts.attempted_base);
    if !matches!(
        facts.attempted_base,
        Destination::Second | Destination::Third | Destination::Home
    ) || attempted <= origin(retired.from)
    {
        return Err(TransitionError::DeltaMismatch);
    }
    // No force-release/overslide proof: a tag at the still-forced base is excluded.
    let forced = before.bases[..usize::from(origin(retired.from))]
        .iter()
        .all(Option::is_some);
    if (forced && attempted == origin(retired.from) + 1)
        || attempted <= value
        || event.delta.moves.iter().any(|m| {
            if m.from == Base::Plate || m.to == Destination::Out {
                return false;
            }
            let end = target(m.to);
            // No extra runner chronology: do not certify passing or a later catch-up.
            end <= value
                || (origin(m.from) < origin(retired.from) && end >= attempted)
                || (origin(m.from) > origin(retired.from) && end < attempted)
        })
    {
        return Err(TransitionError::UnsupportedRule);
    }
    crate::rules::validate_nonpassing_runners(before, event)?;
    let mut crossings = facts.home_before_tag.clone();
    crossings.sort_unstable();
    if crossings.windows(2).any(|pair| pair[0] == pair[1])
        || crossings.iter().any(|id| {
            !event
                .delta
                .moves
                .iter()
                .any(|m| m.to == Destination::Home && m.runner.player_id == *id)
        })
    {
        return Err(TransitionError::DeltaMismatch);
    }
    // A trailing Home crossing cannot precede an omitted leading runner without passing.
    let mut omitted_leader = false;
    for runner in before.bases.iter().rev().flatten() {
        if event
            .delta
            .moves
            .iter()
            .any(|m| m.runner.player_id == runner.player_id && m.to == Destination::Home)
        {
            if crossings.contains(&runner.player_id) {
                if omitted_leader {
                    return Err(TransitionError::UnsupportedRule);
                }
            } else {
                omitted_leader = true;
            }
        }
    }
    if total_outs == 3 {
        let third = event
            .delta
            .third_out
            .as_ref()
            .ok_or(TransitionError::UnsupportedRule)?;
        let mut before_out = third.home_before_out.clone();
        before_out.sort_unstable();
        if third.runner_id != facts.runner_id
            || third.kind != ThirdOutKind::Timing
            || before_out != crossings
        {
            return Err(TransitionError::DeltaMismatch);
        }
    }
    rules::validate_run_credits(before, event, total_outs)?;
    // No winning-run stop/extra-throw chronology is admitted by this proof.
    if event.fielding_review != FieldingReview::NoMisplay
        || (before.half == Half::Bottom
            && before.inning >= 9
            && before.score[1]
                .checked_add(u16::from(event.delta.runs))
                .is_some_and(|home| home > before.score[0]))
    {
        return Err(TransitionError::UnsupportedRule);
    }
    Ok(())
}
