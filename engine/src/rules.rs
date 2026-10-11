//! Bounded 2019 scoring, lineup and regulation-ending rules.
use super::{
    Base, CreditKind, Destination, Event, EventKind, GameState, Half, TeamState, ThirdOutKind,
    TransitionError,
};

pub(crate) fn valid_team(team: &TeamState) -> bool {
    if team.next_batter > 8
        || team.pitcher.is_empty()
        || team.dh_slot.is_some_and(|s| s > 8)
        || (team.dh_must_bat && team.dh_slot.is_none())
    {
        return false;
    }
    let pitcher_count = team
        .batting_order
        .iter()
        .filter(|id| **id == team.pitcher)
        .count();
    if pitcher_count != usize::from(team.dh_slot.is_none()) {
        return false;
    }
    let ids: Vec<&str> = team
        .batting_order
        .iter()
        .chain(team.bench.iter())
        .chain(team.removed.iter())
        .map(String::as_str)
        .collect();
    if ids.iter().any(|id| {
        id.is_empty()
            || *id == team.pitcher
                && (team.bench.iter().any(|b| b == id) || team.removed.iter().any(|r| r == id))
    }) {
        return false;
    }
    ids.iter()
        .enumerate()
        .all(|(i, id)| !ids[i + 1..].contains(id))
}

/// Derive one-way bench/removed transfers from active roles.
pub(crate) fn finalize_personnel(
    before: &[TeamState; 2],
    after: &mut [TeamState; 2],
    event: &Event,
    offense: usize,
) -> Result<(), TransitionError> {
    for (side, (old, new)) in before.iter().zip(after.iter_mut()).enumerate() {
        // Rule 5.04(a)(2): substitutions replace occupants, not the batting
        // slots of players who remain in the order (including pitchers).
        for (slot, id) in old.batting_order.iter().enumerate() {
            if new.batting_order.contains(id) && &new.batting_order[slot] != id {
                return Err(TransitionError::IllegalStateChange);
            }
        }
        if let Some(slot) = old.dh_slot {
            if new.batting_order[slot as usize] == new.pitcher {
                new.dh_slot = None;
            }
            if old.dh_must_bat {
                let dh_batted = side == offense
                    && event.kind.is_pa()
                    && event.participants.batter.as_deref()
                        == Some(old.batting_order[slot as usize].as_str());
                let opposing_pitcher_changed = event.delta.changes.iter().any(|change| {
                    matches!(change, super::PersonnelChange::Pitcher { team, .. } if team.index() != side)
                });
                if !dh_batted
                    && !opposing_pitcher_changed
                    && (new.dh_slot.is_none()
                        || new.batting_order[slot as usize] != old.batting_order[slot as usize])
                {
                    return Err(TransitionError::IllegalStateChange);
                }
                if dh_batted || opposing_pitcher_changed {
                    new.dh_must_bat = false;
                }
            }
        }
        let old_active: Vec<_> = old
            .batting_order
            .iter()
            .chain(std::iter::once(&old.pitcher))
            .collect();
        let new_active: Vec<_> = new
            .batting_order
            .iter()
            .chain(std::iter::once(&new.pitcher))
            .collect();
        for id in &old_active {
            if !new_active.contains(id) && !new.removed.contains(id) {
                new.removed.push((*id).clone());
            }
        }
        for (position, id) in new_active.iter().enumerate() {
            if new_active[..position].contains(id) {
                continue;
            }
            if !old_active.contains(id) {
                let index = new
                    .bench
                    .iter()
                    .position(|candidate| candidate == *id)
                    .ok_or(TransitionError::IllegalStateChange)?;
                new.bench.remove(index);
            }
        }
        if !valid_team(new) {
            return Err(TransitionError::IllegalStateChange);
        }
    }
    Ok(())
}

/// Determine the batter-runner's carried charge, not who faced this PA.
/// Rule 9.16(g): a lone runner retired by a fielder's choice is replaced by
/// the batter with that runner's responsibility. More complex chains need
/// additional play facts; movement-vector order is not a scoring fact.
pub(crate) fn batter_charge<'a>(
    before: &'a GameState,
    event: &Event,
) -> Result<&'a str, TransitionError> {
    // Credit-only contradictions must not be masked by unknown FC lineage.
    let _ = hit_value_credit(event)?;
    let _ = walk_classification(before, event)?;
    let current = before.teams[1 - before.half.offense()].pitcher.as_str();
    if event.kind == EventKind::FieldersChoice {
        if before.bases.iter().flatten().count() != 1
            || event.delta.moves.len() != 2
            || event.delta.outs != 1
            || event.delta.runs != 0
            || !event
                .delta
                .moves
                .iter()
                .any(|m| m.from == Base::Plate && m.to == Destination::First)
        {
            return Err(TransitionError::UnsupportedRule);
        }
        let (index, runner) = before
            .bases
            .iter()
            .enumerate()
            .find_map(|(i, r)| r.as_ref().map(|r| (i, r)))
            .expect("one occupied base");
        let base = [Base::First, Base::Second, Base::Third][index];
        if !event
            .delta
            .moves
            .iter()
            .any(|m| m.from == base && m.to == Destination::Out && m.runner == *runner)
        {
            return Err(TransitionError::UnsupportedRule);
        }
        if event.delta.credits.iter().any(|c| {
            matches!(
                c.kind,
                CreditKind::Hit
                    | CreditKind::Walk
                    | CreditKind::Strikeout
                    | CreditKind::HitByPitch
                    | CreditKind::SacrificeBunt
                    | CreditKind::SacrificeFly
                    | CreditKind::InterferenceAward
                    | CreditKind::CaughtStealing
            )
        }) {
            return Err(TransitionError::IllegalCredit);
        }
        return Ok(&runner.responsible_pitcher);
    }
    if event.kind == EventKind::PlateAppearance
        && (before.outs.saturating_add(event.delta.outs) < 3
            || event.delta.credits.iter().any(|c| {
                c.kind == CreditKind::Run
                    && Some(c.player_id.as_str()) == event.participants.batter.as_deref()
            }))
        && event.delta.moves.iter().any(|m| {
            m.from == Base::Plate
                && matches!(
                    m.to,
                    Destination::First
                        | Destination::Second
                        | Destination::Third
                        | Destination::Home
                )
        })
        && event.delta.moves.iter().any(|m| {
            m.from != Base::Plate
                && m.to == Destination::Out
                && m.runner.responsible_pitcher != current
        })
        && !event.delta.credits.iter().any(|c| {
            matches!(
                c.kind,
                CreditKind::Hit
                    | CreditKind::Walk
                    | CreditKind::HitByPitch
                    | CreditKind::InterferenceAward
            )
        })
    {
        // An unclassified batter reaching while an inherited runner is retired
        // may be a FC replacement. Do not silently erase that charge lineage.
        // No carried/scored batter charge remains on a non-scoring third out;
        // leave those existing timing/force/appeal validations in control.
        return Err(TransitionError::UnsupportedRule);
    }
    Ok(current)
}

/// The exception is only for a batted ball leaving the park, not an
/// inside-the-park hit or a generic multi-run advancement.
pub(crate) fn validate_home_run(before: &GameState, event: &Event) -> Result<(), TransitionError> {
    if event.kind != EventKind::OutOfParkHomeRun {
        return Ok(());
    }
    let batter = event.participants.batter.as_deref();
    if event.delta.outs != 0
        || event.delta.third_out.is_some()
        || event.delta.moves.len() != 1 + before.bases.iter().flatten().count()
        || event.delta.moves.iter().any(|m| m.to != Destination::Home)
        || !event
            .delta
            .moves
            .iter()
            .any(|m| m.from == Base::Plate && Some(m.runner.player_id.as_str()) == batter)
    {
        return Err(TransitionError::DeltaMismatch);
    }
    if event
        .delta
        .credits
        .iter()
        .filter(|c| c.kind == CreditKind::Hit)
        .count()
        != 1
        || !event
            .delta
            .credits
            .iter()
            .any(|c| c.kind == CreditKind::Hit && Some(c.player_id.as_str()) == batter)
    {
        return Err(TransitionError::IllegalCredit);
    }
    Ok(())
}

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
            let batter_reaches_first = event.kind.is_pa()
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
        ThirdOutKind::BatterAfterFirstTiming => {
            if out.from != Base::Plate {
                return Err(TransitionError::DeltaMismatch);
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

/// Credit-only prerequisites, safe before any movement/charge inference.
fn hit_value_credit(event: &Event) -> Result<Option<u8>, TransitionError> {
    let mut values = event
        .delta
        .credits
        .iter()
        .filter_map(|credit| match credit.kind {
            CreditKind::Single => Some(1),
            CreditKind::Double => Some(2),
            CreditKind::Triple => Some(3),
            _ => None,
        });
    let Some(value) = values.next() else {
        return Ok(None);
    };
    if values.next().is_some()
        || event.kind != EventKind::PlateAppearance
        || event
            .delta
            .credits
            .iter()
            .filter(|c| c.kind == CreditKind::Hit)
            .count()
            != 1
    {
        return Err(TransitionError::IllegalCredit);
    }
    Ok(Some(value))
}

/// Validate explicit walk classification before charge ambiguity can mask a
/// missing or conflicting Walk credit. No classification is inferred from play shape.
pub(crate) fn walk_classification(
    before: &GameState,
    event: &Event,
) -> Result<Option<CreditKind>, TransitionError> {
    let modifiers: Vec<_> = event
        .delta
        .credits
        .iter()
        .filter(|credit| {
            matches!(
                credit.kind,
                CreditKind::OrdinaryWalk | CreditKind::IntentionalWalk
            )
        })
        .collect();
    if modifiers.is_empty() {
        return Ok(None);
    }
    let offense = before.half.offense();
    let batter =
        before.teams[offense].batting_order[before.teams[offense].next_batter as usize].as_str();
    if modifiers.len() != 1
        || event.kind != EventKind::PlateAppearance
        || event
            .delta
            .credits
            .iter()
            .filter(|credit| credit.kind == CreditKind::Walk)
            .count()
            != 1
        || event.participants.batter.as_deref() != Some(batter)
        || modifiers[0].player_id != batter
        || event.delta.credits.iter().any(|credit| {
            matches!(
                credit.kind,
                CreditKind::Hit
                    | CreditKind::Strikeout
                    | CreditKind::AtBat
                    | CreditKind::HitByPitch
                    | CreditKind::SacrificeBunt
                    | CreditKind::SacrificeFly
                    | CreditKind::InterferenceAward
            )
        })
    {
        return Err(TransitionError::IllegalCredit);
    }
    Ok(Some(modifiers[0].kind))
}

/// Rule 9.06: supplied scorer value is not inferred from final advancement.
/// This first subset lacks retirement location/touch and misplay-cause facts.
pub(crate) fn validate_hit_value(before: &GameState, event: &Event) -> Result<(), TransitionError> {
    let Some(value) = hit_value_credit(event)? else {
        return Ok(());
    };
    let batter = event
        .delta
        .moves
        .iter()
        .find(|m| m.from == Base::Plate)
        .expect("validated PA");
    let destination = match batter.to {
        Destination::First => 1,
        Destination::Second => 2,
        Destination::Third => 3,
        Destination::Home => 4,
        Destination::Out => return Err(TransitionError::UnsupportedRule),
    };
    if value > destination {
        return Err(TransitionError::IllegalCredit);
    }
    // Ordinary walk-off value cannot exceed the winning runner's advance.
    // Without order facts, test every possible winner rather than Vec order.
    if before.inning >= 9
        && before.half == Half::Bottom
        && before.score[1] <= before.score[0]
        && before.score[1]
            .checked_add(u16::from(event.delta.runs))
            .is_some_and(|home| home > before.score[0])
    {
        let ceilings: Vec<_> = event
            .delta
            .moves
            .iter()
            .filter(|m| {
                m.to == Destination::Home
                    && event
                        .delta
                        .credits
                        .iter()
                        .any(|c| c.kind == CreditKind::Run && c.player_id == m.runner.player_id)
            })
            .map(|m| match m.from {
                Base::Plate => 4,
                Base::First => 3,
                Base::Second => 2,
                Base::Third => 1,
            })
            .collect();
        if ceilings.iter().all(|ceiling| value > *ceiling) {
            return Err(TransitionError::IllegalCredit);
        }
        if ceilings.iter().any(|ceiling| value > *ceiling) {
            return Err(TransitionError::UnsupportedRule);
        }
    }
    if event.delta.outs > 0
        || (value > 1 && event.fielding_review != super::FieldingReview::NoMisplay)
    {
        return Err(TransitionError::UnsupportedRule);
    }
    Ok(())
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
                // Rule 7.01(g)(3): ordinarily stop on the winning run;
                // an out-of-park HR allows every runner and batter to score.
                let winning_runs = away - home + 1;
                if event.kind != EventKind::OutOfParkHomeRun
                    && u16::from(event.delta.runs) != winning_runs
                {
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
