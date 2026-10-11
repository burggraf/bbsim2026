//! Deterministic, storage-independent 2019 event-state contract.
//! The reducer, statistical accounting and era rules are later work packages.

pub mod accounting;
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
    /// Declared before first pitch; `None` means the pitcher bats.
    /// Once lost, the DH cannot be restored in this game.
    pub dh_slot: Option<u8>,
    /// Starting DH has not yet batted; cleared by his first PA or an opposing pitching change.
    pub dh_must_bat: bool,
    /// Players eligible to enter but not yet used in this game.
    pub bench: Vec<String>,
    /// Players removed from all active roles; re-entry is prohibited.
    pub removed: Vec<String>,
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
    /// Bounded PA: lone occupied runner retired, batter safely to first.
    /// The replacement inherits the retired runner's pitcher charge (9.16(g)).
    FieldersChoice,
}
impl EventKind {
    pub(crate) fn is_pa(self) -> bool {
        matches!(
            self,
            Self::PlateAppearance | Self::OutOfParkHomeRun | Self::FieldersChoice
        )
    }
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
    /// Explicit official at-bat classification when hits/strikeouts do not identify it.
    AtBat,
    HitByPitch,
    SacrificeBunt,
    SacrificeFly,
    InterferenceAward,
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
    /// Batter-runner was tagged after touching first; unlike before-first,
    /// this is a timing out for runners who reached home before the tag.
    BatterAfterFirstTiming,
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
/// Scorer/source review of all fielding chances during this event, including
/// muffed fouls, errors, passed balls, interference and obstruction. NoMisplay
/// asserts none occurred; it is not inferred from absent credits. Unknown
/// cannot certify an errorless inning for earned runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldingReview {
    Unknown,
    NoMisplay,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub kind: EventKind,
    pub participants: Participants,
    pub fielding_review: FieldingReview,
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
            || before
                .teams
                .iter()
                .chain(after.teams.iter())
                .any(|team| !rules::valid_team(team))
            || before.over
        {
            return Err(TransitionError::InvalidState);
        }
        for snapshot in [&before, &after] {
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
        let is_pa = event.kind.is_pa();
        let batter_charge = rules::batter_charge(&before, &event)?;
        if event.fielding_review == FieldingReview::NoMisplay
            && (event.kind == EventKind::RunnerOnly(RunnerEvent::PassedBall)
                || event.delta.credits.iter().any(|c| {
                    matches!(
                        c.kind,
                        CreditKind::Error | CreditKind::PassedBall | CreditKind::InterferenceAward
                    )
                }))
        {
            return Err(TransitionError::IllegalCredit);
        }
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
                    if expected_teams[team.index()].dh_slot.is_some()
                        && expected_teams[team.index()].pitcher == *incoming
                        && expected_teams[team.index()].dh_slot != Some(*slot)
                    {
                        return Err(TransitionError::DeltaMismatch);
                    }
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
                    let order = &mut expected_teams[offense].batting_order;
                    let slot = order
                        .iter()
                        .position(|id| id == &outgoing.player_id)
                        .ok_or(TransitionError::DeltaMismatch)?;
                    if incoming.player_id == expected_teams[offense].pitcher
                        && expected_teams[offense].dh_slot != Some(slot as u8)
                    {
                        return Err(TransitionError::DeltaMismatch);
                    }
                    order[slot] = incoming.player_id.clone();
                    expected_bases[index] = Some(incoming.clone());
                }
            }
        }
        rules::finalize_personnel(&before.teams, &mut expected_teams, &event, offense)?;
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
                    || movement.runner.responsible_pitcher != batter_charge
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
        let defense_team = &before.teams[1 - offense];
        let fielding_eligible = |id: &str| {
            id == defense_team.pitcher
                || defense_team
                    .batting_order
                    .iter()
                    .enumerate()
                    .any(|(slot, player)| {
                        Some(slot) != defense_team.dh_slot.map(usize::from) && player.as_str() == id
                    })
        };
        for credit in &event.delta.credits {
            let eligible = match credit.kind {
                CreditKind::PlateAppearance | CreditKind::Run => true, // checked above
                CreditKind::Hit
                | CreditKind::Walk
                | CreditKind::Strikeout
                | CreditKind::Rbi
                | CreditKind::AtBat
                | CreditKind::HitByPitch
                | CreditKind::SacrificeBunt
                | CreditKind::SacrificeFly
                | CreditKind::InterferenceAward => {
                    is_pa && event.participants.batter.as_deref() == Some(credit.player_id.as_str())
                }
                CreditKind::StolenBase => event.delta.moves.iter().any(|m| {
                    m.runner.player_id == credit.player_id
                        && matches!(
                            (m.from, m.to),
                            (
                                Base::First,
                                Destination::Second | Destination::Third | Destination::Home
                            ) | (Base::Second, Destination::Third | Destination::Home)
                                | (Base::Third, Destination::Home)
                        )
                }),
                CreditKind::CaughtStealing => event.delta.moves.iter().any(|m| {
                    m.from != Base::Plate
                        && m.runner.player_id == credit.player_id
                        && m.to == Destination::Out
                }),
                CreditKind::Putout | CreditKind::Assist | CreditKind::Error => {
                    fielding_eligible(&credit.player_id)
                }
                CreditKind::PassedBall => {
                    credit.player_id != defense_team.pitcher && fielding_eligible(&credit.player_id)
                }
            };
            if !eligible {
                return Err(TransitionError::IllegalCredit);
            }
            // Rules 9.02(a)(18), 9.07(h): CS counts occurrences, including
            // error-saved attempts. One reviewed-clean retirement cannot
            // charge this actor twice; Unknown lacks attempt chronology.
            if credit.kind == CreditKind::CaughtStealing
                && event
                    .delta
                    .credits
                    .iter()
                    .filter(|c| c.kind == credit.kind && c.player_id == credit.player_id)
                    .count()
                    > 1
            {
                return Err(match event.fielding_review {
                    FieldingReview::NoMisplay => TransitionError::IllegalCredit,
                    FieldingReview::Unknown => TransitionError::UnsupportedRule,
                });
            }
        }
        let count = |kind| {
            event
                .delta
                .credits
                .iter()
                .filter(|c| c.kind == kind)
                .count()
        };
        let outcomes =
            count(CreditKind::Hit) + count(CreditKind::Walk) + count(CreditKind::Strikeout);
        let exclusions = count(CreditKind::Walk)
            + count(CreditKind::HitByPitch)
            + count(CreditKind::SacrificeBunt)
            + count(CreditKind::SacrificeFly)
            + count(CreditKind::InterferenceAward);
        if outcomes > 1
            || exclusions > 1
            || count(CreditKind::AtBat) > 1
            || (exclusions > 0
                && count(CreditKind::AtBat) + count(CreditKind::Hit) + count(CreditKind::Strikeout)
                    > 0)
            || count(CreditKind::Rbi) > usize::from(event.delta.runs)
            || count(CreditKind::Putout) > usize::from(event.delta.outs)
        {
            return Err(TransitionError::IllegalCredit);
        }
        // Rule 9.05(b)(1)-(2): a forced retirement on an asserted base
        // hit (including appeal at the first forced base) cannot credit Hit.
        // Other out labels do not establish that scorer condition.
        if count(CreditKind::Hit) > 0
            && event
                .delta
                .third_out
                .as_ref()
                .is_some_and(|out| out.kind == ThirdOutKind::Force)
        {
            return Err(TransitionError::IllegalCredit);
        }
        // Rules 5.05(a)(2), 5.09(a)(3): with first occupied before
        // two outs, the third strike retires the batter automatically. A
        // same-play runner advance cannot make him eligible after the fact.
        if count(CreditKind::Strikeout) > 0
            && before.outs < 2
            && before.bases[0].is_some()
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
        if count(CreditKind::Walk)
            + count(CreditKind::HitByPitch)
            + count(CreditKind::InterferenceAward)
            > 0
            && !event
                .delta
                .moves
                .iter()
                .any(|m| m.from == Base::Plate && m.to != Destination::Out)
        {
            // A batter retired after actually touching first needs touch/order
            // metadata; Plate -> Out alone cannot establish the award.
            return Err(TransitionError::UnsupportedRule);
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
