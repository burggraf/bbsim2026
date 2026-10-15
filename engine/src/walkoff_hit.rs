//! Supplied ordinary safe-hit ending facts; no inferred clock or generator policy.
use super::*;

fn origin(base: Base) -> u8 {
    match base {
        Base::Plate => 0,
        Base::First => 1,
        Base::Second => 2,
        Base::Third => 3,
    }
}

fn destination(base: Destination) -> u8 {
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
    batter: &RunnerMove,
    value: u8,
    facts: &WalkoffHitFacts,
) -> Result<(), TransitionError> {
    if before.over {
        return Err(TransitionError::InvalidState);
    }
    if before.inning < 9
        || before.half != Half::Bottom
        || before.score[1] > before.score[0]
        || event.kind != EventKind::PlateAppearance
        || event.sole_safe_hit.is_none()
        || !matches!(value, 1 | 2)
        || event.delta.outs != 0
        || event.delta.runs == 0
        || u32::from(event.delta.runs)
            != u32::from(before.score[0]) - u32::from(before.score[1]) + 1
    {
        return Err(TransitionError::UnsupportedRule);
    }
    if facts.batter_touch.runner_id != batter.runner.player_id {
        return Err(TransitionError::InvalidParticipants);
    }
    let winner = event
        .delta
        .moves
        .iter()
        .find(|m| {
            m.runner.player_id == facts.winning_runner_id
                && m.from != Base::Plate
                && m.to == Destination::Home
                && event
                    .delta
                    .credits
                    .iter()
                    .any(|c| c.kind == CreditKind::Run && c.player_id == m.runner.player_id)
        })
        .ok_or(TransitionError::InvalidParticipants)?;
    // Earlier canonical origin/credit validation binds this named actor to before.
    // Known scorer ceiling contradictions precede unavailable timing/review.
    if value > 4 - origin(winner.from) {
        return Err(TransitionError::IllegalCredit);
    }
    if event.fielding_review != FieldingReview::NoMisplay
        || facts.timing == NaturalCourseTouchTiming::Unknown
        || facts.batter_touch.base != batter.to
        || !matches!(
            batter.to,
            Destination::First | Destination::Second | Destination::Third
        )
        || event
            .delta
            .moves
            .iter()
            .any(|m| m.to == Destination::Home && origin(m.from) < origin(winner.from))
        || event.delta.moves.iter().any(|trailing| {
            event.delta.moves.iter().any(|leading| {
                origin(trailing.from) < origin(leading.from)
                    && (destination(trailing.to) > destination(leading.to)
                        || trailing.to == leading.to && trailing.to != Destination::Home)
            })
        })
    {
        // This family has no passing/appeals/extra physical continuation.
        // Origin order is an invariant, never the order of a supplied Vec.
        return Err(TransitionError::UnsupportedRule);
    }
    Ok(())
}
