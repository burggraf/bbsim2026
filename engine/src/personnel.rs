//! Canonical active roles; legacy records carry explicitly untracked usage/defense.
use crate::{
    Base, Event, EventKind, GameState, PersonnelChange, Runner, TeamState, TransitionError,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PitcherObligation {
    UntrackedLegacy,
    StartingBatter,
    ReliefBatter,
    Satisfied,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PitcherRole {
    Active {
        player_id: String,
        obligation: PitcherObligation,
    },
    Vacant {
        batting_slot: u8,
    },
}
impl PitcherRole {
    pub fn active_id(&self) -> Option<&str> {
        match self {
            Self::Active { player_id, .. } => Some(player_id),
            Self::Vacant { .. } => None,
        }
    }
    fn replaceable(&self, legacy: bool) -> bool {
        matches!(
            self,
            Self::Active {
                obligation: PitcherObligation::Satisfied,
                ..
            }
        ) || legacy
            && matches!(
                self,
                Self::Active {
                    obligation: PitcherObligation::UntrackedLegacy,
                    ..
                }
            )
    }
}
impl From<String> for PitcherRole {
    fn from(player_id: String) -> Self {
        Self::Active {
            player_id,
            obligation: PitcherObligation::UntrackedLegacy,
        }
    }
}
impl From<&str> for PitcherRole {
    fn from(player_id: &str) -> Self {
        player_id.to_owned().into()
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum DefenseState {
    #[default]
    LegacyUnassigned,
    /// Index zero is P, followed by C, 1B, 2B, 3B, SS, LF, CF, RF. None is pending.
    Assigned(Box<[Option<String>; 9]>),
}
impl TeamState {
    pub(crate) fn active_ids(&self) -> impl Iterator<Item = &str> {
        self.batting_order
            .iter()
            .map(String::as_str)
            .chain(self.pitcher.active_id())
    }
    pub(crate) fn legacy_personnel(&self) -> bool {
        self.defense == DefenseState::LegacyUnassigned
            && matches!(
                self.pitcher,
                PitcherRole::Active {
                    obligation: PitcherObligation::UntrackedLegacy,
                    ..
                }
            )
    }
}
impl GameState {
    /// Must succeed before a producer consumes any play randomness.
    pub fn validate_play_ready(&self) -> Result<(), TransitionError> {
        crate::rules::validate_snapshot(self)?;
        let team = &self.teams[1 - self.half.offense()];
        if self.over
            || team.pitcher.active_id().is_none()
            || matches!(&team.defense, DefenseState::Assigned(positions) if positions.iter().any(Option::is_none))
        {
            return Err(TransitionError::IllegalStateChange);
        }
        Ok(())
    }
}
pub(crate) fn valid_team(team: &TeamState) -> bool {
    if team.next_batter > 8
        || team.dh_slot.is_some_and(|s| s > 8)
        || team.dh_must_bat && team.dh_slot.is_none()
    {
        return false;
    }
    match &team.pitcher {
        PitcherRole::Active { player_id, .. } => {
            if player_id.is_empty()
                || team
                    .batting_order
                    .iter()
                    .filter(|id| *id == player_id)
                    .count()
                    != usize::from(team.dh_slot.is_none())
                || team.bench.contains(player_id)
                || team.removed.contains(player_id)
            {
                return false;
            }
        }
        PitcherRole::Vacant { batting_slot } => {
            if *batting_slot > 8
                || team.dh_slot.is_some()
                || matches!(team.defense, DefenseState::LegacyUnassigned)
            {
                return false;
            }
        }
    }
    let ids: Vec<_> = team
        .batting_order
        .iter()
        .chain(&team.bench)
        .chain(&team.removed)
        .collect();
    if ids.iter().any(|id| id.is_empty())
        || ids
            .iter()
            .enumerate()
            .any(|(i, id)| ids[i + 1..].contains(id))
    {
        return false;
    }
    if let DefenseState::Assigned(positions) = &team.defense {
        let mut seen = std::collections::BTreeSet::new();
        for (position, id) in positions.iter().enumerate() {
            if let Some(id) = id {
                if id.is_empty()
                    || !seen.insert(id)
                    || !team.active_ids().any(|active| active == id)
                    || (position == 0 && team.pitcher.active_id() != Some(id.as_str()))
                    || (position != 0
                        && (team.pitcher.active_id() == Some(id.as_str())
                            || team
                                .dh_slot
                                .is_some_and(|s| team.batting_order[s as usize] == *id)))
                {
                    return false;
                }
            }
        }
    }
    true
}
pub(crate) fn fielding_eligible(team: &TeamState, id: &str) -> bool {
    match &team.defense {
        DefenseState::Assigned(positions) => positions.iter().flatten().any(|player| player == id),
        DefenseState::LegacyUnassigned => {
            team.pitcher.active_id() == Some(id)
                || team
                    .batting_order
                    .iter()
                    .enumerate()
                    .any(|(slot, player)| Some(slot as u8) != team.dh_slot && player == id)
        }
    }
}

pub(crate) fn catcher_eligible(team: &TeamState, id: &str) -> bool {
    match &team.defense {
        DefenseState::Assigned(positions) => positions[1].as_deref() == Some(id),
        DefenseState::LegacyUnassigned => {
            team.pitcher.active_id() != Some(id) && fielding_eligible(team, id)
        }
    }
}

pub(crate) fn legacy_event(event: &Event) -> bool {
    event.participants.batter.is_some()
        && event.participants.pitcher.is_some()
        && event.delta.changes.iter().all(|change| {
            matches!(
                change,
                PersonnelChange::Pitcher { .. }
                    | PersonnelChange::BattingSlot { .. }
                    | PersonnelChange::PinchRunner { .. }
            )
        })
}

pub(crate) fn project_personnel(
    before: &GameState,
    event: &Event,
    teams: &mut [TeamState; 2],
    bases: &mut [Option<Runner>; 3],
) -> Result<(), TransitionError> {
    let offense = before.half.offense();
    let legacy = before.teams.iter().all(TeamState::legacy_personnel) && legacy_event(event);
    let mut slots = [[false; 9]; 2];
    let mut pitchers = [false; 2];
    let mut positions = [[false; 9]; 2];
    // Slot and pitching assignments bind the original snapshot, never an intermediate entrant.
    for change in &event.delta.changes {
        let (side, slot, outgoing, incoming) = match change {
            PersonnelChange::BattingSlot {
                team,
                slot,
                outgoing,
                incoming,
            } => (team.index(), *slot, outgoing.as_str(), incoming.as_str()),
            PersonnelChange::PinchHitter { outgoing, incoming } => {
                if !matches!(before.teams[offense].defense, DefenseState::Assigned(_))
                    || !before.teams[offense].bench.contains(incoming)
                {
                    return Err(TransitionError::IllegalStateChange);
                }
                let old = &before.teams[offense];
                if old.pitcher.active_id() == Some(outgoing.as_str())
                    && !old.pitcher.replaceable(false)
                {
                    return Err(TransitionError::IllegalStateChange);
                }
                (
                    offense,
                    old.next_batter,
                    outgoing.as_str(),
                    incoming.as_str(),
                )
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
                if before.bases[index].as_ref() != Some(outgoing)
                    || bases[index].as_ref() != Some(outgoing)
                    || incoming.responsible_pitcher != outgoing.responsible_pitcher
                {
                    return Err(TransitionError::DeltaMismatch);
                }
                let slot = before.teams[offense]
                    .batting_order
                    .iter()
                    .position(|id| *id == outgoing.player_id)
                    .ok_or(TransitionError::DeltaMismatch)?;
                bases[index] = Some(incoming.clone());
                (
                    offense,
                    slot as u8,
                    outgoing.player_id.as_str(),
                    incoming.player_id.as_str(),
                )
            }
            _ => continue,
        };
        let old = &before.teams[side];
        if slot > 8
            || slots[side][slot as usize]
            || old.batting_order[slot as usize] != outgoing
            || incoming.is_empty()
            || outgoing == incoming
            || (legacy
                && old.dh_slot.is_some()
                && old.pitcher.active_id() == Some(incoming)
                && old.dh_slot != Some(slot))
        {
            return Err(TransitionError::DeltaMismatch);
        }
        slots[side][slot as usize] = true;
        teams[side].batting_order[slot as usize] = incoming.into();
        if matches!(change, PersonnelChange::PinchHitter { .. })
            && old.pitcher.active_id() == Some(outgoing)
        {
            teams[side].pitcher = PitcherRole::Vacant { batting_slot: slot };
        }
    }
    for change in &event.delta.changes {
        let (side, outgoing, incoming) = match change {
            PersonnelChange::Pitcher {
                team,
                outgoing,
                incoming,
            } => (team.index(), Some(outgoing.as_str()), incoming),
            PersonnelChange::AssignPitcher { team, incoming } => (team.index(), None, incoming),
            _ => continue,
        };
        let old = &before.teams[side];
        if pitchers[side]
            || incoming.is_empty()
            || old.pitcher.active_id() == Some(incoming.as_str())
        {
            return Err(TransitionError::DeltaMismatch);
        }
        match (&old.pitcher, outgoing) {
            (PitcherRole::Active { player_id, .. }, Some(outgoing))
                if player_id == outgoing
                    && old.pitcher.replaceable(legacy)
                    && (legacy || old.bench.contains(incoming)) => {}
            (PitcherRole::Vacant { batting_slot }, None)
                if old.bench.contains(incoming)
                    && teams[side].batting_order[*batting_slot as usize] == *incoming => {}
            _ => return Err(TransitionError::IllegalStateChange),
        }
        // PH and a pitching replacement cannot both consume the same outgoing role.
        if matches!(teams[side].pitcher, PitcherRole::Vacant { .. }) && outgoing.is_some() {
            return Err(TransitionError::DeltaMismatch);
        }
        pitchers[side] = true;
        teams[side].pitcher = if legacy {
            incoming.clone().into()
        } else {
            PitcherRole::Active {
                player_id: incoming.clone(),
                obligation: PitcherObligation::ReliefBatter,
            }
        };
        if let DefenseState::Assigned(defense) = &mut teams[side].defense {
            defense[0] = Some(incoming.clone());
        }
    }
    for (side, team) in teams.iter_mut().enumerate() {
        let active: Vec<String> = team.active_ids().map(str::to_owned).collect();
        if let DefenseState::Assigned(defense) = &mut team.defense {
            for id in defense.iter_mut() {
                if id.as_ref().is_some_and(|id| !active.contains(id)) {
                    *id = None;
                }
            }
            if matches!(team.pitcher, PitcherRole::Vacant { .. }) {
                defense[0] = None;
            }
        }
        for change in &event.delta.changes {
            if let PersonnelChange::AssignDefense {
                team: target,
                position,
                outgoing,
                incoming,
            } = change
            {
                if target.index() != side {
                    continue;
                }
                let index = position
                    .checked_sub(1)
                    .filter(|p| *p < 9)
                    .ok_or(TransitionError::DeltaMismatch)? as usize;
                let DefenseState::Assigned(original) = &before.teams[side].defense else {
                    return Err(TransitionError::IllegalStateChange);
                };
                let DefenseState::Assigned(defense) = &mut team.defense else {
                    return Err(TransitionError::IllegalStateChange);
                };
                if positions[side][index] || &original[index] != outgoing || incoming.is_empty() {
                    return Err(TransitionError::DeltaMismatch);
                }
                positions[side][index] = true;
                if index == 0 && pitchers[side] && defense[0].as_ref() != Some(incoming) {
                    return Err(TransitionError::DeltaMismatch);
                }
                defense[index] = Some(incoming.clone());
            }
        }
    }
    finalize(before, event, teams)?;
    if event.kind != EventKind::Substitution {
        let batter =
            &before.teams[offense].batting_order[before.teams[offense].next_batter as usize];
        let completed = event
            .delta
            .moves
            .iter()
            .any(|m| m.from == Base::Plate && &m.runner.player_id == batter);
        if let PitcherRole::Active { obligation, .. } = &mut teams[1 - offense].pitcher {
            if (*obligation == PitcherObligation::StartingBatter && completed)
                || (*obligation == PitcherObligation::ReliefBatter
                    && (completed || before.outs.checked_add(event.delta.outs) == Some(3)))
            {
                *obligation = PitcherObligation::Satisfied;
            }
        }
    }
    Ok(())
}
fn finalize(
    before: &GameState,
    event: &Event,
    teams: &mut [TeamState; 2],
) -> Result<(), TransitionError> {
    for (side, (old, new)) in before.teams.iter().zip(teams.iter_mut()).enumerate() {
        for (slot, id) in old.batting_order.iter().enumerate() {
            if new.batting_order.contains(id) && &new.batting_order[slot] != id {
                return Err(TransitionError::IllegalStateChange);
            }
        }
        if let Some(slot) = old.dh_slot {
            let dh = &new.batting_order[slot as usize];
            if new
                .pitcher
                .active_id()
                .is_some_and(|p| new.batting_order.iter().any(|id| id == p))
                || matches!(&new.defense, DefenseState::Assigned(defense) if defense.iter().flatten().any(|id| id == dh))
            {
                new.dh_slot = None;
            }
            if old.dh_must_bat {
                let batted = side == before.half.offense()
                    && event.kind.is_pa()
                    && event.participants.batter.as_deref()
                        == Some(old.batting_order[slot as usize].as_str());
                let opposing_change = before.teams[1 - side].pitcher.active_id()
                    != teams_opposing_id(before, event, side);
                if !batted
                    && !opposing_change
                    && (new.dh_slot.is_none()
                        || new.batting_order[slot as usize] != old.batting_order[slot as usize])
                {
                    return Err(TransitionError::IllegalStateChange);
                }
                if batted || opposing_change {
                    new.dh_must_bat = false;
                }
            }
        }
        let old_active: Vec<_> = old.active_ids().collect();
        let new_active: Vec<String> = new.active_ids().map(str::to_owned).collect();
        for id in &old_active {
            if !new_active.iter().any(|new| new == id)
                && !new.removed.iter().any(|removed| removed == id)
            {
                new.removed.push((*id).into());
            }
        }
        for (index, id) in new_active.iter().enumerate() {
            if new_active[..index].contains(id) || old_active.contains(&id.as_str()) {
                continue;
            }
            let position = new
                .bench
                .iter()
                .position(|candidate| candidate == id)
                .ok_or(TransitionError::IllegalStateChange)?;
            new.bench.remove(position);
        }
        if !valid_team(new) {
            return Err(TransitionError::IllegalStateChange);
        }
    }
    Ok(())
}
fn teams_opposing_id<'a>(before: &'a GameState, event: &'a Event, side: usize) -> Option<&'a str> {
    event
        .delta
        .changes
        .iter()
        .find_map(|change| match change {
            PersonnelChange::Pitcher { team, incoming, .. }
            | PersonnelChange::AssignPitcher { team, incoming }
                if team.index() != side =>
            {
                Some(incoming.as_str())
            }
            _ => None,
        })
        .or_else(|| before.teams[1 - side].pitcher.active_id())
}
