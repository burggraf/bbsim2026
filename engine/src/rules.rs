//! Conservative 2019 regulation ending rules. Third-out timing and the
//! out-of-park walk-off exception require explicit future event metadata.
use super::{Event, GameState, Half, TransitionError};

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
    // The transition currently says only that a runner touched home, not why
    // the third out occurred or whether that runner scored first. Reject the
    // ambiguous play instead of crediting a run contrary to Rule 5.08(a).
    if total_outs == 3 && event.delta.runs > 0 {
        return Err(TransitionError::UnsupportedRule);
    }
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
