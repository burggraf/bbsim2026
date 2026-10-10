//! 2019 scoring and regulation ending rules. A walk-off HR exception still
//! needs explicit contact classification.
use super::{Base, Destination, Event, EventKind, GameState, Half, ThirdOutKind, TransitionError};

/// Derive legally credited runner IDs from movement and explicit third-out
/// cause/timing. Home crossings need not all be runs (Rule 5.08(a)).
pub(crate) fn scored_runners(
    before: &GameState,
    event: &Event,
    total_outs: u8,
) -> Result<Vec<String>, TransitionError> {
    let home: Vec<_> = event
        .delta
        .moves
        .iter()
        .filter(|movement| movement.to == Destination::Home)
        .collect();
    let Some(third) = &event.delta.third_out else {
        if total_outs == 3 && !home.is_empty() {
            return Err(TransitionError::UnsupportedRule);
        }
        return Ok(home.iter().map(|m| m.runner.player_id.clone()).collect());
    };
    if total_outs != 3 || third.runner_id.is_empty() {
        return Err(TransitionError::IllegalStateChange);
    }
    let mut outs = event
        .delta
        .moves
        .iter()
        .filter(|m| m.to == Destination::Out && m.runner.player_id == third.runner_id);
    let out = outs.next().ok_or(TransitionError::DeltaMismatch)?;
    if outs.next().is_some() {
        return Err(TransitionError::DeltaMismatch);
    }
    for (index, id) in third.home_before_out.iter().enumerate() {
        if id.is_empty()
            || third.home_before_out[index + 1..].contains(id)
            || !home.iter().any(|m| m.runner.player_id == *id)
        {
            return Err(TransitionError::DeltaMismatch);
        }
    }
    let scored = match third.kind {
        ThirdOutKind::BatterBeforeFirst => {
            if out.from != Base::Plate {
                return Err(TransitionError::DeltaMismatch);
            }
            Vec::new()
        }
        ThirdOutKind::Force => {
            if event.delta.outs != 1 && !home.is_empty() {
                // Earlier outs can remove a force; their order is not encoded.
                return Err(TransitionError::UnsupportedRule);
            }
            // A force exists while the batter becomes a runner and every
            // preceding base in the chain was occupied before the play.
            let batter_reaches_first = event.kind == EventKind::PlateAppearance
                && event.delta.moves.iter().any(|m| {
                    m.from == Base::Plate
                        && matches!(
                            m.to,
                            Destination::First | Destination::Second | Destination::Third
                        )
                });
            let chain = match out.from {
                Base::Plate => false,
                Base::First => before.bases[0].is_some(),
                Base::Second => before.bases[0].is_some() && before.bases[1].is_some(),
                Base::Third => before.bases.iter().all(Option::is_some),
            };
            if !batter_reaches_first || !chain {
                return Err(TransitionError::DeltaMismatch);
            }
            Vec::new()
        }
        ThirdOutKind::Timing => {
            if out.from == Base::Plate {
                return Err(TransitionError::UnsupportedRule);
            }
            third.home_before_out.clone()
        }
        ThirdOutKind::PrecedingRunnerAppeal => {
            if event.delta.outs != 1 && !home.is_empty() {
                // Fourth-out appeal ordering is not represented yet.
                return Err(TransitionError::UnsupportedRule);
            }
            if out.from == Base::Plate {
                return Err(TransitionError::DeltaMismatch);
            }
            let rank = |base: Base| match base {
                Base::Plate => 0,
                Base::First => 1,
                Base::Second => 2,
                Base::Third => 3,
            };
            third
                .home_before_out
                .iter()
                .filter(|id| {
                    home.iter()
                        .any(|m| m.runner.player_id == **id && rank(m.from) > rank(out.from))
                })
                .cloned()
                .collect()
        }
    };
    Ok(scored)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Ending {
    Continue,
    ThirdOut,
    WalkOff,
}

pub(crate) fn ending(
    before: &GameState,
    event: &Event,
    total_outs: u8,
) -> Result<Ending, TransitionError> {
    if before.inning < 9 {
        return Ok(Ending::Continue);
    }
    let (away, home) = (before.score[0], before.score[1]);
    match before.half {
        Half::Top => {
            if total_outs == 3 && home > away {
                Ok(Ending::ThirdOut)
            } else {
                Ok(Ending::Continue)
            }
        }
        Half::Bottom => {
            if home > away {
                return Err(TransitionError::InvalidState);
            }
            let new_home = home
                .checked_add(u16::from(event.delta.runs))
                .ok_or(TransitionError::DeltaMismatch)?;
            if total_outs == 3 && new_home > away {
                // A later third out after a winning run needs appeal/end-game
                // ordering that is not represented at this boundary yet.
                return Err(TransitionError::UnsupportedRule);
            }
            if new_home > away {
                // Rule 7.01(g)(3): ordinarily stop on the winning run. The
                // out-of-park HR exception is not yet represented in Event.
                let winning_runs = away - home + 1;
                if u16::from(event.delta.runs) != winning_runs {
                    return Err(TransitionError::UnsupportedRule);
                }
                Ok(Ending::WalkOff)
            } else if total_outs == 3 && new_home < away {
                Ok(Ending::ThirdOut)
            } else {
                Ok(Ending::Continue)
            }
        }
    }
}
