//! Apply a fully specified, deterministic event. The E-01 constructor is the
//! canonical validator; no partial update is returned on failure.
use super::*;

/// Apply a fully specified ordinary safe-hit ending with supplied chronology facts.
/// Does not choose a winner, clip movements or reclassify a sampled hit.
pub fn apply_with_walkoff_hit_facts(
    before: &GameState,
    event: Event,
    facts: WalkoffHitFacts,
) -> Result<EventTransition, TransitionError> {
    apply_inner(before, event, Some(facts), None, None, None, None)
}

/// Apply the bounded direct catch/Third-score episode with supplied Rule 9.08(d) facts.
/// Does not infer handling, retouch legality, scoring chronology or attempt policy.
pub fn apply_with_caught_sacrifice_fly_facts(
    before: &GameState,
    event: Event,
    facts: CaughtSacrificeFlyFacts,
) -> Result<EventTransition, TransitionError> {
    apply_inner(before, event, None, Some(facts), None, None, None)
}

/// Apply a supplied grounded sacrifice bunt/First throw, not an attempt policy.
pub fn apply_with_sacrifice_bunt_throw_facts(
    before: &GameState,
    event: Event,
    facts: SacrificeBuntThrowFacts,
) -> Result<EventTransition, TransitionError> {
    apply_inner(before, event, None, None, Some(facts), None, None)
}

/// Derive the after-state from a before-state and explicit event effects.
/// This does not sample outcomes or infer missing decisions.
pub fn apply(before: &GameState, event: Event) -> Result<EventTransition, TransitionError> {
    apply_inner(before, event, None, None, None, None, None)
}

pub(crate) fn apply_with_ground_force_double_play_facts(
    before: &GameState,
    event: Event,
    facts: GroundForceDoublePlayFacts,
) -> Result<EventTransition, TransitionError> {
    apply_inner(before, event, None, None, None, Some(facts), None)
}

pub(crate) fn apply_with_hit_runner_tag_facts(
    before: &GameState,
    event: Event,
    facts: HitRunnerTagFacts,
) -> Result<EventTransition, TransitionError> {
    apply_inner(before, event, None, None, None, None, Some(facts))
}

fn apply_inner(
    before: &GameState,
    event: Event,
    walkoff_hit: Option<WalkoffHitFacts>,
    caught_sacrifice_fly: Option<CaughtSacrificeFlyFacts>,
    sacrifice_bunt_throw: Option<SacrificeBuntThrowFacts>,
    ground_force_double_play: Option<GroundForceDoublePlayFacts>,
    hit_runner_tag: Option<HitRunnerTagFacts>,
) -> Result<EventTransition, TransitionError> {
    if before.teams.iter().any(|team| !rules::valid_team(team)) {
        return Err(TransitionError::InvalidState);
    }
    rules::batter_charge(
        before,
        &event,
        walkoff_hit.as_ref(),
        caught_sacrifice_fly.as_ref(),
        sacrifice_bunt_throw.as_ref(),
        ground_force_double_play.as_ref(),
        hit_runner_tag.as_ref(),
    )?;
    let mut after = before.clone();
    let offense = before.half.offense();
    if event.kind.is_pa() {
        let cursor = after.teams[offense].next_batter;
        if cursor >= 9 {
            return Err(TransitionError::InvalidState);
        }
        after.teams[offense].next_batter = (cursor + 1) % 9;
    }
    rules::project_personnel(before, &event, &mut after.teams, &mut after.bases)?;
    // Remove all origins before adding destinations: a forced runner may move
    // into a base just vacated by another runner on the same play.
    for movement in &event.delta.moves {
        match movement.from {
            Base::Plate => {}
            base => {
                let index = base_index(base);
                if after.bases[index].as_ref() != Some(&movement.runner) {
                    return Err(TransitionError::DeltaMismatch);
                }
                after.bases[index] = None;
            }
        }
    }
    for movement in &event.delta.moves {
        match movement.to {
            Destination::Out | Destination::Home => {}
            destination => {
                let index = destination_index(destination);
                if after.bases[index].is_some() {
                    return Err(TransitionError::DeltaMismatch);
                }
                after.bases[index] = Some(movement.runner.clone());
            }
        }
    }
    after.score[offense] = after.score[offense]
        .checked_add(u16::from(event.delta.runs))
        .ok_or(TransitionError::DeltaMismatch)?;
    let outs = before
        .outs
        .checked_add(event.delta.outs)
        .ok_or(TransitionError::DeltaMismatch)?;
    if outs > 3 {
        return Err(TransitionError::DeltaMismatch);
    }
    match rules::ending(before, &event, outs)? {
        rules::Ending::ThirdOut => {
            after.over = true;
            after.outs = 0;
            after.bases = [None, None, None];
        }
        rules::Ending::WalkOff => {
            after.over = true;
            after.outs = outs;
        }
        rules::Ending::Continue if outs == 3 => {
            after.outs = 0;
            after.bases = [None, None, None];
            match before.half {
                Half::Top => after.half = Half::Bottom,
                Half::Bottom => {
                    after.half = Half::Top;
                    after.inning = before
                        .inning
                        .checked_add(1)
                        .ok_or(TransitionError::DeltaMismatch)?;
                }
            }
        }
        rules::Ending::Continue => after.outs = outs,
    }
    EventTransition::try_new_with_hit_runner_tag_inner(
        before.clone(),
        event,
        after,
        walkoff_hit,
        caught_sacrifice_fly,
        sacrifice_bunt_throw,
        ground_force_double_play,
        hit_runner_tag,
    )
}

fn base_index(base: Base) -> usize {
    match base {
        Base::First => 0,
        Base::Second => 1,
        Base::Third => 2,
        Base::Plate => unreachable!(),
    }
}
fn destination_index(destination: Destination) -> usize {
    match destination {
        Destination::First => 0,
        Destination::Second => 1,
        Destination::Third => 2,
        Destination::Home | Destination::Out => unreachable!(),
    }
}
