//! Deterministic, storage-independent 2019 event-state contract.
//! The reducer, statistical accounting and era rules are later work packages.

pub mod accounting;
mod balk;
mod caught;
mod codec;
mod contact;
mod ground;
mod ground_muff;
mod pitch;
mod protected_award;
mod reducer;
mod rules;
mod sole_home_run;
mod sole_safe_hit;
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
    /// Scorer hit-value modifiers; each requires one Hit, not another hit unit.
    Single,
    Double,
    Triple,
    /// Walk classifications are modifiers, not additional base-on-balls units.
    OrdinaryWalk,
    IntentionalWalk,
    /// Scorer-qualified ground force/reverse-force double play, one batter unit.
    GroundedIntoDoublePlay,
    /// One scorer-qualified legal-delivery occurrence, charged to the actual pitcher.
    WildPitch,
    /// One imposed scorer-qualified balk occurrence, charged to the actual pitcher.
    Balk,
    /// Explicit actual-batter no-hit scorer decision; not an AB or fielding fact.
    NoHit,
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
    /// All chances reviewed; only the canonically bound ordinary ground muff.
    SoleGroundMuff,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContactClass {
    Grounded,
    NotGrounded,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContactContinuity {
    Continuous,
    Discontinuous,
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForceLoss {
    /// The force was active immediately before the first named retirement.
    RemovedByFirstOut,
    PreviouslyRemoved,
    NeverForced,
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RetirementCause {
    /// Defensive tag of first base or batter before first touch; not catch,
    /// automatic retirement, interference penalty or appeal.
    BatterPutoutBeforeFirstTouch,
    Force {
        at: Destination,
    },
    Tag {
        at: Destination,
        force_loss: ForceLoss,
    },
    BatterAfterFirstTag {
        at: Destination,
    },
    RunnerInterference,
    BatterByPrecedingRunnerInterference {
        interferer_id: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetirementFact {
    pub runner_id: String,
    pub cause: RetirementCause,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewedErrorPlacement {
    /// All relevant between-out chances were reviewed, including absence.
    pub between_outs: Vec<String>,
    /// Placement of supplied units only; no exhaustive outside/ER review.
    pub outside_interval: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TwoOutContactFacts {
    pub contact: ContactClass,
    /// These fields assert order; movement/credit vector order does not.
    pub first_out: RetirementFact,
    pub second_out: RetirementFact,
    pub continuity: ContactContinuity,
    pub error_placement: Option<ReviewedErrorPlacement>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeliveryContext {
    RunnerOnly,
    UncaughtThirdStrike,
    DeliveredFourthBall,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandlingAssessment {
    PitcherWild,
    CatcherPassed,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandlingAdvance {
    pub runner_id: String,
    pub reached: Destination,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreDeliverySteal {
    pub runner_id: String,
    pub target: Destination,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecoveryReview {
    Unknown,
    NoRecoveryPutout,
    BeforeAnyAdvance {
        runner_id: String,
    },
    AfterAdvance {
        runner_id: String,
        advancer_id: String,
    },
    RecoveredThirdStrikeBatterBeforeFirst,
    /// Actual legal First touch precedes this delivery's catcher throw to a
    /// different receiving fielder for the batter's retirement at `at`.
    /// Handling-caused First acquisition is separately bound in `advances`.
    ThirdStrikeBatterAfterFirstThrow {
        putout_fielder_id: String,
        at: Destination,
    },
}
/// One legal-delivery episode and its supplied WP/PB units; not complete PA/game coverage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SingleDeliveryFacts {
    pub context: DeliveryContext,
    pub handling: HandlingAssessment,
    /// Supplied event-local catcher role, not inferred from lineup order.
    pub catcher_id: String,
    pub advances: Vec<HandlingAdvance>,
    /// None means missing review; Some(empty) asserts reviewed absence.
    pub pre_delivery_steals: Option<Vec<PreDeliverySteal>>,
    pub recovery: RecoveryReview,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BalkCause {
    Unknown,
    PitcherAction602a,
    SqueezeOrStealHomeInterference601g,
    PitchingProhibition602d,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BalkDisposition {
    Unknown,
    PenaltyApplied,
    PlayProceedsWithoutReference,
}
/// Named actual legal touch, not an entitlement or handling-caused advance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedBaseTouch {
    pub runner_id: String,
    pub base: Destination,
}
/// Qualification-threshold assertions, not a complete movement/touch history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BalkProgressFacts {
    pub batter_first_touch: NamedBaseTouch,
    pub runner_next_touches: Vec<NamedBaseTouch>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BalkResolution {
    Unknown,
    /// Ordinary imposed awards originate at before bases with no extra action sequence.
    ExactProtectedAwards,
    /// Named actual first/next touches during the play, before disposition.
    ActualMinimumProgress(BalkProgressFacts),
}
/// One cause-bound balk occurrence; not complete PA/game or fielding-review coverage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BalkFacts {
    pub cause: BalkCause,
    pub disposition: BalkDisposition,
    pub resolution: BalkResolution,
}
/// One ordinary non-bunt fair ground ball, controlled and directly thrown to
/// the named first-covering receiver before the actual batter's first touch.
/// No other contact touch/throw/retirement; before runners hold throughout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrdinaryGroundThrowFacts {
    pub batter_id: String,
    pub thrower_id: String,
    pub receiver_id: String,
}
/// Ordinary legal called/swinging third strike received directly by this
/// catcher, retiring this batter before first with other before runners held.
/// No recovery throw, foul-bunt strikeout or deflection is asserted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaughtStrikeoutFacts {
    pub batter_id: String,
    pub catcher_id: String,
}
/// All runner advancement in this PA is solely the BB/HBP protected force chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProtectedAwardFacts {
    pub batter_id: String,
}
/// Every represented advance/run is exclusively this batter's out-of-park
/// home run, unaided by an error; not an exhaustive fielding/ER review.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SoleHomeRunFacts {
    pub batter_id: String,
}
/// Every represented advance/run is exclusively continuous advancement in
/// this batter's classified safe-hit play, unaided by error or a separate episode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SoleSafeHitFacts {
    pub batter_id: String,
}
/// One mechanical ground-ball muff losing a routine before-first retirement;
/// all represented station-one advances are exclusively caused by that error.
/// No throw/assist chain or recovery putout; whole-chance review is separate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MuffedGroundBallFacts {
    pub batter_id: String,
    pub fielder_id: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub kind: EventKind,
    pub participants: Participants,
    pub fielding_review: FieldingReview,
    pub delta: Delta,
    pub two_out_contact: Option<TwoOutContactFacts>,
    pub single_delivery: Option<SingleDeliveryFacts>,
    pub balk: Option<BalkFacts>,
    pub ordinary_ground_throw: Option<OrdinaryGroundThrowFacts>,
    pub caught_strikeout: Option<CaughtStrikeoutFacts>,
    pub protected_award: Option<ProtectedAwardFacts>,
    pub sole_home_run: Option<SoleHomeRunFacts>,
    pub sole_safe_hit: Option<SoleSafeHitFacts>,
    pub muffed_ground_ball: Option<MuffedGroundBallFacts>,
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
    /// Event-local safe-hit causal proof, not complete segment/box coverage.
    pub fn sole_safe_hit_rbi(&self) -> Option<accounting::RbiContribution> {
        let facts = self.event.sole_safe_hit.as_ref()?;
        let runs = self
            .event
            .delta
            .moves
            .iter()
            .filter(|m| m.to == Destination::Home)
            .count() as u32;
        Some(accounting::RbiContribution {
            runs_batted_in: [(facts.batter_id.clone(), runs)].into(),
        })
    }

    /// Event-local sole-HR proof, not complete segment/box coverage.
    pub fn sole_home_run_rbi(&self) -> Option<accounting::RbiContribution> {
        let facts = self.event.sole_home_run.as_ref()?;
        Some(accounting::RbiContribution {
            runs_batted_in: [(
                facts.batter_id.clone(),
                1 + self.before.bases.iter().flatten().count() as u32,
            )]
            .into(),
        })
    }

    /// Independently classified RBI for one observed PA, not a full box.
    pub fn rbi_contribution(&self) -> Option<accounting::RbiContribution> {
        if !self.event.kind.is_pa() {
            return None;
        }
        if self.event.delta.runs == 0 {
            // Canonical legal-run conservation includes third-out suppression.
            return Some(accounting::RbiContribution {
                runs_batted_in: [(self.event.participants.batter.as_ref()?.clone(), 0)].into(),
            });
        }
        self.protected_award_rbi()
            .or_else(|| self.sole_home_run_rbi())
            .or_else(|| self.sole_safe_hit_rbi())
    }

    /// Event-local protected-award RBI proof, not complete segment/box coverage.
    pub fn protected_award_rbi(&self) -> Option<accounting::RbiContribution> {
        let facts = self.event.protected_award.as_ref()?;
        Some(accounting::RbiContribution {
            runs_batted_in: [(
                facts.batter_id.clone(),
                u32::from(self.before.bases.iter().all(Option::is_some)),
            )]
            .into(),
        })
    }

    /// Bound ordinary muff plus whole-chance review: scoped PO0/A0, not a box.
    pub fn muffed_ground_fielding(&self) -> Option<accounting::FieldingContribution> {
        self.event.muffed_ground_ball.as_ref()?;
        if self.event.fielding_review != FieldingReview::SoleGroundMuff {
            return None;
        }
        Some(accounting::FieldingContribution {
            putouts: Default::default(),
            assists: Default::default(),
        })
    }

    /// Bound named E1/PB0; Unknown prior chances remain unclassified.
    pub fn muffed_ground_misplays(&self) -> Option<accounting::MisplayContribution> {
        let facts = self.event.muffed_ground_ball.as_ref()?;
        if self.event.fielding_review != FieldingReview::SoleGroundMuff {
            return None;
        }
        Some(accounting::MisplayContribution {
            errors: [(facts.fielder_id.clone(), 1)].into(),
            passed_balls: Default::default(),
        })
    }

    pub fn misplay_contribution(&self) -> Option<accounting::MisplayContribution> {
        self.reviewed_misplay_zeros()
            .or_else(|| self.muffed_ground_misplays())
    }

    /// Reviewed event-local E/PB zeroes, independent of PO/A coverage.
    pub fn reviewed_misplay_zeros(&self) -> Option<accounting::MisplayContribution> {
        if self.event.kind == EventKind::Substitution
            || self.event.fielding_review != FieldingReview::NoMisplay
        {
            return None;
        }
        Some(accounting::MisplayContribution {
            errors: Default::default(),
            passed_balls: Default::default(),
        })
    }

    /// Union of qualified event-local PO/A proofs, not whole-game coverage.
    pub fn fielding_contribution(&self) -> Option<accounting::FieldingContribution> {
        self.ordinary_ground_fielding()
            .or_else(|| self.caught_strikeout_fielding())
            .or_else(|| self.sole_safe_hit_fielding())
            .or_else(|| self.protected_award_fielding())
            .or_else(|| self.sole_home_run_fielding())
            .or_else(|| self.muffed_ground_fielding())
    }

    /// Event-local protected-award PO/A; missing cause/review stays unknown.
    /// Empty maps mean zero only in this qualified event's before-defense scope.
    pub fn protected_award_fielding(&self) -> Option<accounting::FieldingContribution> {
        self.event.protected_award.as_ref()?;
        if self.event.fielding_review != FieldingReview::NoMisplay {
            return None;
        }
        Some(accounting::FieldingContribution {
            putouts: Default::default(),
            assists: Default::default(),
        })
    }

    /// Event-local sole-HR PO/A; missing cause/review stays unknown.
    /// Empty maps mean zero only in this qualified event's before-defense scope.
    pub fn sole_home_run_fielding(&self) -> Option<accounting::FieldingContribution> {
        self.event.sole_home_run.as_ref()?;
        if self.event.fielding_review != FieldingReview::NoMisplay {
            return None;
        }
        Some(accounting::FieldingContribution {
            putouts: Default::default(),
            assists: Default::default(),
        })
    }

    /// Event-local sole-safe-hit PO/A proof; missing cause/review stays unknown.
    /// Empty maps mean zero only in this qualified event's before-defense scope.
    pub fn sole_safe_hit_fielding(&self) -> Option<accounting::FieldingContribution> {
        self.event.sole_safe_hit.as_ref()?;
        if self.event.fielding_review != FieldingReview::NoMisplay {
            return None;
        }
        // Canonical exclusive safe-hit facts rule out retirements/extra episodes;
        // full review rules out Rule 9.10's error-prevented-out assist exception.
        Some(accounting::FieldingContribution {
            putouts: Default::default(),
            assists: Default::default(),
        })
    }

    /// Event-local caught-K proof; missing facts/review are not known zero maps.
    pub fn caught_strikeout_fielding(&self) -> Option<accounting::FieldingContribution> {
        if self.event.fielding_review != FieldingReview::NoMisplay {
            return None;
        }
        let facts = self.event.caught_strikeout.as_ref()?;
        Some(accounting::FieldingContribution {
            putouts: [(facts.catcher_id.clone(), 1)].into(),
            assists: Default::default(),
        })
    }

    /// Event-local qualified PO/A only, not a complete-game fielding certificate.
    pub fn ordinary_ground_fielding(&self) -> Option<accounting::FieldingContribution> {
        if self.event.fielding_review != FieldingReview::NoMisplay {
            return None;
        }
        let facts = self.event.ordinary_ground_throw.as_ref()?;
        Some(accounting::FieldingContribution {
            putouts: [(facts.receiver_id.clone(), 1)].into(),
            assists: [(facts.thrower_id.clone(), 1)].into(),
        })
    }

    pub fn try_new(
        before: GameState,
        event: Event,
        after: GameState,
    ) -> Result<Self, TransitionError> {
        rules::validate_snapshot(&before)?;
        rules::validate_snapshot(&after)?;
        if before.over {
            return Err(TransitionError::InvalidState);
        }
        let offense = before.half.offense();
        rules::validate_participants(&before, &event)?;
        let is_pa = event.kind.is_pa();
        let batter_charge = rules::batter_charge(&before, &event)?;
        rules::validate_fielding_review(&event)?;
        let expected_slot = (before.teams[offense].next_batter + u8::from(is_pa)) % 9;
        rules::validate_event_shape(&event)?;
        let mut expected_teams = before.teams.clone();
        expected_teams[offense].next_batter = expected_slot;
        rules::validate_pa_credits(&event)?;
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
        rules::validate_move_origins(&expected_bases, &event, Some(batter_charge))?;
        let total_outs = rules::validate_outs(&before, &event)?;
        rules::validate_home_run(&before, &event)?;
        rules::validate_run_credits(&before, &event, total_outs)?;
        for credit in &event.delta.credits {
            let eligible = match credit.kind {
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
                _ => rules::credit_role_eligible(&before, &event, credit),
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
                    FieldingReview::Unknown | FieldingReview::SoleGroundMuff => {
                        TransitionError::UnsupportedRule
                    }
                });
            }
        }
        rules::validate_credit_counts(&event)?;
        let count = |kind| {
            event
                .delta
                .credits
                .iter()
                .filter(|c| c.kind == kind)
                .count()
        };
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
        rules::validate_hit_value(&before, &event)?;
        rules::validate_strikeout_eligibility(&before, &event)?;
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
        expected_bases = rules::move_destinations(expected_bases, &event)?;
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
