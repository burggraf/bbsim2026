//! Deterministic, storage-independent 2019 event-state contract.
//! The reducer, statistical accounting and era rules are later work packages.

mod codec;
mod reducer;
mod rules;
pub use reducer::apply;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeamSide {
    Away,
    Home,
}
impl TeamSide {
    fn index(self) -> usize {
        match self {
            Self::Away => 0,
            Self::Home => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Half {
    Top,
    Bottom,
}

impl Half {
    pub fn offense(self) -> usize {
        match self {
            Self::Top => 0,
            Self::Bottom => 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Runner {
    pub player_id: String,
    pub responsible_pitcher: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeamState {
    pub batting_order: [String; 9],
    pub next_batter: u8,
    pub pitcher: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    pub inning: u16,
    pub half: Half,
    pub outs: u8,
    /// Away, home.
    pub score: [u16; 2],
    /// Away, home. The pitcher's team is the defensive team.
    pub teams: [TeamState; 2],
    /// First, second, third.
    pub bases: [Option<Runner>; 3],
    pub over: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Base {
    Plate,
    First,
    Second,
    Third,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Destination {
    First,
    Second,
    Third,
    Home,
    Out,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunnerMove {
    pub runner: Runner,
    pub from: Base,
    pub to: Destination,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunnerEvent {
    WildPitch,
    PassedBall,
    StealAttempt,
    Pickoff,
    Balk,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventKind {
    PlateAppearance,
    RunnerOnly(RunnerEvent),
    Substitution,
    /// A plate appearance on an out-of-park HR; all runners may score on a walk-off.
    OutOfParkHomeRun,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Participants {
    pub batter: Option<String>,
    pub pitcher: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreditKind {
    PlateAppearance,
    Strikeout,
    Walk,
    Hit,
    Run,
    Rbi,
    StolenBase,
    CaughtStealing,
    Putout,
    Assist,
    Error,
    PassedBall,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Credit {
    pub player_id: String,
    pub kind: CreditKind,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PersonnelChange {
    Pitcher {
        team: TeamSide,
        outgoing: String,
        incoming: String,
    },
    BattingSlot {
        team: TeamSide,
        slot: u8,
        outgoing: String,
        incoming: String,
    },
    PinchRunner {
        base: Base,
        outgoing: Runner,
        incoming: Runner,
    },
}
/// Why the final out occurred. A timing out can preserve runs that crossed
/// first; a force or batter-runner out before first cannot (2019 Rule 5.08(a)).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThirdOutKind {
    BatterBeforeFirst,
    Force,
    Timing,
    PrecedingRunnerAppeal,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThirdOut {
    pub runner_id: String,
    pub kind: ThirdOutKind,
    /// Home-crossing runner IDs that reached home before the final out.
    /// This is observed/sampled play order, not inferred from Vec order.
    pub home_before_out: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Delta {
    pub moves: Vec<RunnerMove>,
    pub outs: u8,
    /// Legally credited runs, potentially fewer than physical home crossings.
    pub runs: u8,
    pub credits: Vec<Credit>,
    pub changes: Vec<PersonnelChange>,
    pub third_out: Option<ThirdOut>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub kind: EventKind,
    pub participants: Participants,
    pub delta: Delta,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransitionError {
    InvalidState,
    InvalidParticipants,
    IllegalStateChange,
    DeltaMismatch,
    IllegalCredit,
    Discontinuous,
    InvalidEncoding,
    UnsupportedRule,
}

/// Construct once; only immutable access is exposed to consumers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventTransition {
    before: GameState,
    event: Event,
    after: GameState,
}

impl EventTransition {
    pub fn try_new(
        before: GameState,
        event: Event,
        after: GameState,
    ) -> Result<Self, TransitionError> {
        if before.inning == 0
            || after.inning == 0
            || before.outs > 2
            || after.outs > 2
            || before.teams.iter().chain(after.teams.iter()).any(|team| {
                team.next_batter > 8
                    || team.pitcher.is_empty()
                    || team.batting_order.iter().any(String::is_empty)
            })
            || before.over
        {
            return Err(TransitionError::InvalidState);
        }
        for snapshot in [&before, &after] {
            for (index, runner) in snapshot.bases.iter().enumerate() {
                if let Some(runner) = runner {
                    if runner.player_id.is_empty()
                        || runner.responsible_pitcher.is_empty()
                        || snapshot.bases[index + 1..]
                            .iter()
                            .flatten()
                            .any(|other| other.player_id == runner.player_id)
                    {
                        return Err(TransitionError::InvalidState);
                    }
                }
            }
        }
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
        let is_pa = matches!(
            event.kind,
            EventKind::PlateAppearance | EventKind::OutOfParkHomeRun
        );
        let expected_slot = (before.teams[offense].next_batter + u8::from(is_pa)) % 9;
        let is_substitution = event.kind == EventKind::Substitution;
        if is_substitution {
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
        let mut expected_teams = before.teams.clone();
        expected_teams[offense].next_batter = expected_slot;
        let pa_credits = event
            .delta
            .credits
            .iter()
            .filter(|c| c.kind == CreditKind::PlateAppearance)
            .count();
        if pa_credits != usize::from(is_pa)
            || event.delta.credits.iter().any(|c| {
                c.player_id.is_empty()
                    || (c.kind == CreditKind::PlateAppearance
                        && event.participants.batter.as_deref() != Some(&c.player_id))
            })
        {
            return Err(TransitionError::IllegalCredit);
        }
        let mut expected_bases = before.bases.clone();
        for change in &event.delta.changes {
            match change {
                PersonnelChange::Pitcher {
                    team,
                    outgoing,
                    incoming,
                } => {
                    let pitcher = &mut expected_teams[team.index()].pitcher;
                    if pitcher != outgoing || incoming.is_empty() || outgoing == incoming {
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
                    let batter = expected_teams[team.index()]
                        .batting_order
                        .get_mut(*slot as usize)
                        .ok_or(TransitionError::DeltaMismatch)?;
                    if batter != outgoing || incoming.is_empty() || outgoing == incoming {
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
                    if expected_bases[index].as_ref() != Some(outgoing)
                        || incoming.player_id.is_empty()
                        || incoming.player_id == outgoing.player_id
                        || incoming.responsible_pitcher != outgoing.responsible_pitcher
                    {
                        return Err(TransitionError::DeltaMismatch);
                    }
                    expected_bases[index] = Some(incoming.clone());
                }
            }
        }
        if after.teams != expected_teams {
            return Err(TransitionError::IllegalStateChange);
        }
        let mut destinations = Vec::new();
        let mut occupied_origins = [false; 4];
        let mut outs_made = 0u8;
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
                    || movement.runner.responsible_pitcher != event.participants.pitcher
                {
                    return Err(TransitionError::DeltaMismatch);
                }
            } else if expected_bases[origin - 1].as_ref() == Some(&movement.runner) {
                expected_bases[origin - 1] = None;
            } else {
                return Err(TransitionError::DeltaMismatch);
            }
            match movement.to {
                Destination::Out => {
                    outs_made = outs_made
                        .checked_add(1)
                        .ok_or(TransitionError::DeltaMismatch)?
                }
                Destination::Home => {}
                destination => destinations.push((destination, movement.runner.clone())),
            }
        }
        if occupied_origins[0] != is_pa || outs_made != event.delta.outs {
            return Err(TransitionError::DeltaMismatch);
        }
        let total_outs = before
            .outs
            .checked_add(event.delta.outs)
            .ok_or(TransitionError::DeltaMismatch)?;
        if total_outs > 3 {
            return Err(TransitionError::DeltaMismatch);
        }
        rules::validate_home_run(&before, &event)?;
        let mut scored = rules::scored_runners(&before, &event, total_outs)?;
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
        for (destination, runner) in destinations {
            let index = match destination {
                Destination::First => 0,
                Destination::Second => 1,
                Destination::Third => 2,
                _ => unreachable!(),
            };
            if expected_bases[index].is_some() {
                return Err(TransitionError::DeltaMismatch);
            }
            expected_bases[index] = Some(runner);
        }
        if before.score[1 - offense] != after.score[1 - offense]
            || before.score[offense].checked_add(u16::from(event.delta.runs))
                != Some(after.score[offense])
        {
            return Err(TransitionError::DeltaMismatch);
        }
        let ending = rules::ending(&before, &event, total_outs)?;
        if after.over != (ending != rules::Ending::Continue) {
            return Err(TransitionError::IllegalStateChange);
        }
        let expected_frame = match ending {
            rules::Ending::ThirdOut => (before.inning, before.half, 0),
            rules::Ending::WalkOff => (before.inning, before.half, total_outs),
            rules::Ending::Continue if total_outs == 3 => match before.half {
                Half::Top => (before.inning, Half::Bottom, 0),
                Half::Bottom => (
                    before
                        .inning
                        .checked_add(1)
                        .ok_or(TransitionError::DeltaMismatch)?,
                    Half::Top,
                    0,
                ),
            },
            rules::Ending::Continue => (before.inning, before.half, total_outs),
        };
        if (after.inning, after.half, after.outs) != expected_frame
            || (total_outs == 3 && after.bases.iter().any(Option::is_some))
            || (total_outs < 3 && after.bases != expected_bases)
        {
            return Err(TransitionError::IllegalStateChange);
        }
        Ok(Self {
            before,
            event,
            after,
        })
    }
    pub fn before(&self) -> &GameState {
        &self.before
    }
    pub fn event(&self) -> &Event {
        &self.event
    }
    pub fn after(&self) -> &GameState {
        &self.after
    }
    pub fn follows(&self, prior: &Self) -> Result<(), TransitionError> {
        if self.before == prior.after {
            Ok(())
        } else {
            Err(TransitionError::Discontinuous)
        }
    }
}
