//! Apply a fully specified, deterministic event. The E-01 constructor is the
//! canonical validator; no partial update is returned on failure.
use super::*;

/// Derive the after-state from a before-state and explicit event effects.
/// This does not sample outcomes or infer missing decisions.
pub fn apply(before: &GameState, event: Event) -> Result<EventTransition, TransitionError> {
    let mut after = before.clone();
    let offense = before.half.offense();
    if event.kind == EventKind::PlateAppearance {
        let cursor = after.teams[offense].next_batter;
        if cursor >= 9 {
            return Err(TransitionError::InvalidState);
        }
        after.teams[offense].next_batter = (cursor + 1) % 9;
    }
    for change in &event.delta.changes {
        match change {
            PersonnelChange::Pitcher {
                team,
                outgoing,
                incoming,
            } => {
                let pitcher = &mut after.teams[team.index()].pitcher;
                if pitcher != outgoing {
                    return Err(TransitionError::DeltaMismatch);
                }
                *pitcher = incoming.clone();
            }
            PersonnelChange::BattingSlot {
                team,
                slot,
                outgoing,
                incoming,
            } => {
                let batter = after.teams[team.index()]
                    .batting_order
                    .get_mut(*slot as usize)
                    .ok_or(TransitionError::DeltaMismatch)?;
                if batter != outgoing {
                    return Err(TransitionError::DeltaMismatch);
                }
                *batter = incoming.clone();
            }
            PersonnelChange::PinchRunner {
                base,
                outgoing,
                incoming,
            } => {
                let index = match base {
                    Base::First => 0,
                    Base::Second => 1,
                    Base::Third => 2,
                    Base::Plate => return Err(TransitionError::DeltaMismatch),
                };
                if after.bases[index].as_ref() != Some(outgoing) {
                    return Err(TransitionError::DeltaMismatch);
                }
                after.bases[index] = Some(incoming.clone());
            }
        }
    }
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
    EventTransition::try_new(before.clone(), event, after)
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
