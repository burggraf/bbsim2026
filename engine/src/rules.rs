//! Bounded 2019 scoring, lineup and regulation-ending rules.
use super::{
    Base, CreditKind, Destination, Event, EventKind, GameState, Half, Runner, TeamState,
    ThirdOutKind, TransitionError,
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

/// Existing canonical state predicates, shared before deferred balk history.
pub(crate) fn validate_snapshot(snapshot: &GameState) -> Result<(), TransitionError> {
    if snapshot.inning == 0
        || snapshot.outs > 2
        || snapshot.teams.iter().any(|team| !valid_team(team))
    {
        return Err(TransitionError::InvalidState);
    }
    for (index, runner) in snapshot.bases.iter().enumerate() {
        if let Some(runner) = runner {
            if runner.player_id.is_empty()
                || runner.responsible_pitcher.is_empty()
                || !snapshot.teams[snapshot.half.offense()]
                    .batting_order
                    .contains(&runner.player_id)
                || snapshot.teams[snapshot.half.offense()].batting_order
                    [snapshot.teams[snapshot.half.offense()].next_batter as usize]
                    == runner.player_id
                || snapshot.bases[index + 1..]
                    .iter()
                    .flatten()
                    .any(|other| other.player_id == runner.player_id)
            {
                return Err(TransitionError::InvalidState);
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_participants(
    before: &GameState,
    event: &Event,
) -> Result<(), TransitionError> {
    let offense = before.half.offense();
    if event.participants.pitcher != before.teams[1 - offense].pitcher
        || event.participants.batter.as_deref()
            != Some(
                before.teams[offense].batting_order[before.teams[offense].next_batter as usize]
                    .as_str(),
            )
    {
        return Err(TransitionError::InvalidParticipants);
    }
    Ok(())
}

pub(crate) fn validate_pa_credits(event: &Event) -> Result<(), TransitionError> {
    let pa = event
        .delta
        .credits
        .iter()
        .filter(|c| c.kind == CreditKind::PlateAppearance)
        .count();
    if pa != usize::from(event.kind.is_pa())
        || event.delta.credits.iter().any(|c| {
            c.player_id.is_empty()
                || (c.kind == CreditKind::PlateAppearance
                    && event.participants.batter.as_deref() != Some(c.player_id.as_str()))
        })
    {
        return Err(TransitionError::IllegalCredit);
    }
    Ok(())
}

pub(crate) fn validate_fielding_review(event: &Event) -> Result<(), TransitionError> {
    if event.fielding_review == super::FieldingReview::NoMisplay
        && (event.kind == EventKind::RunnerOnly(super::RunnerEvent::PassedBall)
            || event.delta.credits.iter().any(|c| {
                matches!(
                    c.kind,
                    CreditKind::Error | CreditKind::PassedBall | CreditKind::InterferenceAward
                )
            }))
    {
        return Err(TransitionError::IllegalCredit);
    }
    Ok(())
}

pub(crate) fn validate_credit_counts(event: &Event) -> Result<(), TransitionError> {
    let count = |kind| {
        event
            .delta
            .credits
            .iter()
            .filter(|c| c.kind == kind)
            .count()
    };
    let outcomes = count(CreditKind::Hit) + count(CreditKind::Walk) + count(CreditKind::Strikeout);
    let exclusions = count(CreditKind::Walk)
        + count(CreditKind::HitByPitch)
        + count(CreditKind::SacrificeBunt)
        + count(CreditKind::SacrificeFly)
        + count(CreditKind::InterferenceAward);
    if outcomes > 1
        || exclusions > 1
        || count(CreditKind::AtBat) > 1
        || (exclusions > 0
            && count(CreditKind::AtBat) + count(CreditKind::Hit) + count(CreditKind::Strikeout) > 0)
        || count(CreditKind::Rbi) > usize::from(event.delta.runs)
        || count(CreditKind::Putout) > usize::from(event.delta.outs)
    {
        return Err(TransitionError::IllegalCredit);
    }
    Ok(())
}

/// Actor roles only; runner timing/retirement eligibility remains canonical below.
pub(crate) fn credit_role_eligible(
    before: &GameState,
    event: &Event,
    credit: &super::Credit,
) -> bool {
    let defense = &before.teams[1 - before.half.offense()];
    match credit.kind {
        CreditKind::PlateAppearance
        | CreditKind::Run
        | CreditKind::StolenBase
        | CreditKind::CaughtStealing => true,
        CreditKind::Hit
        | CreditKind::Walk
        | CreditKind::Strikeout
        | CreditKind::Rbi
        | CreditKind::AtBat
        | CreditKind::NoHit
        | CreditKind::HitByPitch
        | CreditKind::SacrificeBunt
        | CreditKind::SacrificeFly
        | CreditKind::InterferenceAward
        | CreditKind::Single
        | CreditKind::Double
        | CreditKind::Triple
        | CreditKind::OrdinaryWalk
        | CreditKind::IntentionalWalk
        | CreditKind::GroundedIntoDoublePlay => {
            event.kind.is_pa()
                && event.participants.batter.as_deref() == Some(credit.player_id.as_str())
        }
        CreditKind::Putout | CreditKind::Assist | CreditKind::Error => {
            fielding_eligible(defense, &credit.player_id)
        }
        CreditKind::PassedBall => {
            credit.player_id != defense.pitcher && fielding_eligible(defense, &credit.player_id)
        }
        CreditKind::WildPitch | CreditKind::Balk => credit.player_id == defense.pitcher,
    }
}

pub(crate) fn strikeout_first_ineligible(before: &GameState, event: &Event) -> bool {
    // Rules 5.05(a)(2), 5.09(a)(3): before occupancy, not same-play vacating.
    event
        .delta
        .credits
        .iter()
        .any(|c| c.kind == CreditKind::Strikeout)
        && before.outs < 2
        && before.bases[0].is_some()
}

pub(crate) fn validate_strikeout_eligibility(
    before: &GameState,
    event: &Event,
) -> Result<(), TransitionError> {
    if strikeout_first_ineligible(before, event)
        && (event
            .delta
            .moves
            .iter()
            .any(|m| m.from == Base::Plate && m.to != Destination::Out)
            || event
                .delta
                .third_out
                .as_ref()
                .is_some_and(|out| out.kind == ThirdOutKind::BatterAfterFirstTiming))
    {
        return Err(TransitionError::IllegalStateChange);
    }
    Ok(())
}

pub(crate) fn fielding_eligible(team: &TeamState, id: &str) -> bool {
    id == team.pitcher
        || team.batting_order.iter().enumerate().any(|(slot, player)| {
            Some(slot) != team.dh_slot.map(usize::from) && player.as_str() == id
        })
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

pub(crate) fn validate_no_hit(event: &Event) -> Result<(), TransitionError> {
    let mut decisions = event
        .delta
        .credits
        .iter()
        .filter(|c| c.kind == CreditKind::NoHit);
    let Some(decision) = decisions.next() else {
        return Ok(());
    };
    if !event.kind.is_pa()
        || decisions.next().is_some()
        || Some(decision.player_id.as_str()) != event.participants.batter.as_deref()
        || event
            .delta
            .credits
            .iter()
            .any(|c| c.kind == CreditKind::Hit)
    {
        return Err(TransitionError::IllegalCredit);
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
    // Known bound contact facts precede unrelated occurrence/charge uncertainty.
    super::ground::validate(before, event)?;
    // Credit-only contradictions must not be masked by unknown FC lineage.
    super::balk::preflight(before, event)?;
    super::pitch::preflight(before, event)?;
    validate_no_hit(event)?;
    let _ = hit_value_credit(event)?;
    let _ = walk_classification(before, event)?;
    super::contact::validate(before, event)?;
    super::balk::validate(before, event)?;
    super::pitch::validate(before, event)?;
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
            if event.delta.outs != 1 && !home.is_empty() && event.two_out_contact.is_none() {
                // Legacy records lack order. Typed contact facts have already
                // validated that the first retirement did not remove this force.
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

/// Canonical event/personnel shape, independent of unresolved occurrence facts.
pub(crate) fn validate_event_shape(event: &Event) -> Result<(), TransitionError> {
    if event.kind == EventKind::Substitution {
        if event.delta.changes.is_empty()
            || !event.delta.moves.is_empty()
            || event.delta.outs != 0
            || event.delta.runs != 0
            || !event.delta.credits.is_empty()
        {
            return Err(TransitionError::IllegalStateChange);
        }
    } else if !event.delta.changes.is_empty() {
        return Err(TransitionError::IllegalStateChange);
    }
    Ok(())
}

/// Canonical unique origins, actor identity and preserved before-runner charge.
/// None defers only the batter's potentially ambiguous replacement charge.
pub(crate) fn validate_move_origins(
    bases: &[Option<Runner>; 3],
    event: &Event,
    batter_charge: Option<&str>,
) -> Result<(), TransitionError> {
    let is_pa = event.kind.is_pa();
    let mut occupied_origins = [false; 4];
    for movement in &event.delta.moves {
        let origin = match movement.from {
            Base::Plate => 0,
            Base::First => 1,
            Base::Second => 2,
            Base::Third => 3,
        };
        if occupied_origins[origin]
            || movement.runner.player_id.is_empty()
            || movement.runner.responsible_pitcher.is_empty()
        {
            return Err(TransitionError::DeltaMismatch);
        }
        occupied_origins[origin] = true;
        if origin == 0 {
            if !is_pa
                || event.participants.batter.as_deref() != Some(&movement.runner.player_id)
                || batter_charge.is_some_and(|charge| movement.runner.responsible_pitcher != charge)
            {
                return Err(TransitionError::DeltaMismatch);
            }
        } else if bases[origin - 1].as_ref() != Some(&movement.runner) {
            return Err(TransitionError::DeltaMismatch);
        }
    }
    if occupied_origins[0] != is_pa {
        return Err(TransitionError::DeltaMismatch);
    }
    Ok(())
}

/// Canonical simultaneous destination reconstruction after origin validation.
/// Remove every origin before inserting destinations; held/duplicate bases conflict.
pub(crate) fn move_destinations(
    mut bases: [Option<Runner>; 3],
    event: &Event,
) -> Result<[Option<Runner>; 3], TransitionError> {
    let mut destinations = Vec::new();
    for movement in &event.delta.moves {
        match movement.from {
            Base::Plate => {}
            Base::First => bases[0] = None,
            Base::Second => bases[1] = None,
            Base::Third => bases[2] = None,
        }
        match movement.to {
            Destination::Out | Destination::Home => {}
            destination => destinations.push((destination, movement.runner.clone())),
        }
    }
    for (destination, runner) in destinations {
        let index = match destination {
            Destination::First => 0,
            Destination::Second => 1,
            Destination::Third => 2,
            _ => unreachable!(),
        };
        if bases[index].is_some() {
            return Err(TransitionError::DeltaMismatch);
        }
        bases[index] = Some(runner);
    }
    Ok(bases)
}

/// Represented retirement conservation; does not infer a retirement's cause/order.
pub(crate) fn validate_outs(before: &GameState, event: &Event) -> Result<u8, TransitionError> {
    let outs_made = u8::try_from(
        event
            .delta
            .moves
            .iter()
            .filter(|m| m.to == Destination::Out)
            .count(),
    )
    .map_err(|_| TransitionError::DeltaMismatch)?;
    let total_outs = before
        .outs
        .checked_add(event.delta.outs)
        .ok_or(TransitionError::DeltaMismatch)?;
    if outs_made != event.delta.outs || total_outs > 3 {
        return Err(TransitionError::DeltaMismatch);
    }
    Ok(total_outs)
}

/// Shared named legal-run conservation; unknown third-out history stays unknown.
pub(crate) fn validate_run_credits(
    before: &GameState,
    event: &Event,
    total_outs: u8,
) -> Result<(), TransitionError> {
    let mut scored = scored_runners(before, event, total_outs)?;
    if scored.len() != usize::from(event.delta.runs) {
        return Err(TransitionError::DeltaMismatch);
    }
    let mut run_credits: Vec<_> = event
        .delta
        .credits
        .iter()
        .filter(|credit| credit.kind == CreditKind::Run)
        .map(|credit| credit.player_id.as_str())
        .collect();
    scored.sort_unstable();
    run_credits.sort_unstable();
    if scored.iter().map(String::as_str).collect::<Vec<_>>() != run_credits {
        return Err(TransitionError::IllegalCredit);
    }
    Ok(())
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
            let new_away = away
                .checked_add(u16::from(event.delta.runs))
                .ok_or(TransitionError::DeltaMismatch)?;
            if total_outs == 3 && home > new_away {
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
