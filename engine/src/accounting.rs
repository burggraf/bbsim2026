//! Independent accounting over validated, contiguous event transitions.
//! Credited counts are not complete official box cells: the event vocabulary
//! cannot yet distinguish every PA/fielding/earned-run outcome. Unknown stays
//! `None`, not zero. No game-state score is used to accumulate a line.
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    Base, CreditKind, Destination, Event, EventKind, EventTransition, FieldingReview, GameState,
    Half, RunnerEvent,
};

/// Complete PO/A within one qualified event's before-defense scope only.
/// Absent actors have zero in that scope; None at the API means missing proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldingContribution {
    pub putouts: BTreeMap<String, u32>,
    pub assists: BTreeMap<String, u32>,
}

/// Complete E/PB within one reviewed event's before-defense scope only.
/// Empty maps mean zero in that scope, not a whole-game certificate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MisplayContribution {
    pub errors: BTreeMap<String, u32>,
    pub passed_balls: BTreeMap<String, u32>,
}

/// Independently classified RBI for the named batter in one PA, not a full box.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RbiContribution {
    pub runs_batted_in: BTreeMap<String, u32>,
}

/// Breakdown of supplied Hit credits, not proof of complete official H/TB.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HitBreakdown {
    pub singles: u32,
    pub doubles: u32,
    pub triples: u32,
    pub home_runs: u32,
}
impl HitBreakdown {
    pub fn hits(&self) -> u32 {
        self.singles + self.doubles + self.triples + self.home_runs
    }
    pub fn total_bases(&self) -> u32 {
        self.singles + 2 * self.doubles + 3 * self.triples + 4 * self.home_runs
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BatterLine {
    pub plate_appearances: u32,
    pub credited_hits: u32,
    /// H across this actor's observed PAs; None if any lacks hit/no-hit classification.
    /// Not a complete official game/fielding coverage certificate.
    pub classified_hits: Option<u32>,
    /// None if any supplied Hit in this player's segment lacks scored value.
    pub credited_hit_breakdown: Option<HitBreakdown>,
    pub credited_walks: u32,
    /// None if a supplied Walk in this batter segment lacks intent classification.
    pub credited_intentional_walks: Option<u32>,
    pub credited_grounded_into_double_plays: u32,
    pub credited_strikeouts: u32,
    pub credited_runs: u32,
    pub credited_rbi: u32,
    /// RBI across observed PAs; None if any lacks independent scoring-cause proof.
    pub classified_rbi: Option<u32>,
    pub credited_steals: u32,
    pub credited_caught_stealing: u32,
    pub credited_hit_by_pitch: u32,
    pub credited_sacrifice_bunts: u32,
    pub credited_sacrifice_flies: u32,
    pub credited_interference_awards: u32,
    /// None if any PA for this player in the segment lacks AB classification.
    pub at_bats: Option<u32>,
}
impl Default for BatterLine {
    fn default() -> Self {
        Self {
            plate_appearances: 0,
            credited_hits: 0,
            classified_hits: Some(0),
            credited_hit_breakdown: Some(HitBreakdown::default()),
            credited_walks: 0,
            credited_intentional_walks: Some(0),
            credited_grounded_into_double_plays: 0,
            credited_strikeouts: 0,
            credited_runs: 0,
            credited_rbi: 0,
            classified_rbi: Some(0),
            credited_steals: 0,
            credited_caught_stealing: 0,
            credited_hit_by_pitch: 0,
            credited_sacrifice_bunts: 0,
            credited_sacrifice_flies: 0,
            credited_interference_awards: 0,
            at_bats: Some(0),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PitcherLine {
    pub outs_recorded: u32,
    pub batters_faced: u32,
    pub at_bats_against: Option<u32>,
    pub runs_allowed: u32,
    pub credited_hits_allowed: u32,
    /// Observed PA hit/no-hit coverage for the actual pitcher, not carried R charges.
    pub classified_hits_allowed: Option<u32>,
    /// Actual PA pitcher's supplied Hit breakdown; not carried run charges.
    pub credited_hit_breakdown_allowed: Option<HitBreakdown>,
    pub credited_walks_allowed: u32,
    /// None if a faced Walk in this pitcher segment lacks intent classification.
    pub credited_intentional_walks_allowed: Option<u32>,
    pub credited_strikeouts: u32,
    pub credited_wild_pitches: u32,
    pub credited_balks: u32,
    pub credited_hit_batters: u32,
    pub credited_sacrifice_bunts: u32,
    pub credited_sacrifice_flies: u32,
    /// Segment ER: positive counts require a completed, fully observed half
    /// with reviewed clean plays. Any uncertain charged run makes this None.
    /// Some(0) means no runs charged in this segment, not known career/game ER.
    pub earned_runs: Option<u32>,
}
impl Default for PitcherLine {
    fn default() -> Self {
        Self {
            outs_recorded: 0,
            batters_faced: 0,
            at_bats_against: Some(0),
            runs_allowed: 0,
            credited_hits_allowed: 0,
            classified_hits_allowed: Some(0),
            credited_hit_breakdown_allowed: Some(HitBreakdown::default()),
            credited_walks_allowed: 0,
            credited_intentional_walks_allowed: Some(0),
            credited_strikeouts: 0,
            credited_wild_pitches: 0,
            credited_balks: 0,
            credited_hit_batters: 0,
            credited_sacrifice_bunts: 0,
            credited_sacrifice_flies: 0,
            earned_runs: Some(0),
        }
    }
}
/// Independent fielding coverage across this actor's observed defensive events.
/// Known totals do not certify a complete official fielding line or game.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldingCoverageLine {
    pub observed_events: u32,
    pub putouts: Option<u32>,
    pub assists: Option<u32>,
    pub errors: Option<u32>,
    pub passed_balls: Option<u32>,
}
impl Default for FieldingCoverageLine {
    fn default() -> Self {
        Self {
            observed_events: 0,
            putouts: Some(0),
            assists: Some(0),
            errors: Some(0),
            passed_balls: Some(0),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FielderLine {
    pub credited_putouts: u32,
    pub credited_assists: u32,
    pub credited_errors: u32,
    pub credited_passed_balls: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameAccounting {
    /// Inning → [away, home]. `None` means that half was not in the log;
    /// `Some(0)` means it was played with no runs.
    pub innings: BTreeMap<u16, [Option<u16>; 2]>,
    /// Runs in this log segment, not an initial-score-inclusive total.
    pub runs: [u32; 2],
    pub batters: [BTreeMap<String, BatterLine>; 2],
    pub pitchers: [BTreeMap<String, PitcherLine>; 2],
    pub fielders: [BTreeMap<String, FielderLine>; 2],
    /// Separate observed-defense coverage; legacy credited maps stay unchanged.
    pub classified_fielding: [BTreeMap<String, FieldingCoverageLine>; 2],
    /// True only if the contiguous log starts at an empty 1st/top 0-0
    /// and ends in a terminal snapshot. This does not certify credit coverage.
    pub complete_game: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountingError {
    EmptyLog,
    Discontinuous,
    ScoreMismatch,
}

fn hit_value(event: &Event) -> Option<usize> {
    if event.kind == EventKind::OutOfParkHomeRun {
        Some(4)
    } else {
        event.delta.credits.iter().find_map(|c| match c.kind {
            CreditKind::Single => Some(1),
            CreditKind::Double => Some(2),
            CreditKind::Triple => Some(3),
            _ => None,
        })
    }
}

fn add_hit(breakdown: &mut Option<HitBreakdown>, event: &Event) {
    match (breakdown.as_mut(), hit_value(event)) {
        (_, None) => *breakdown = None,
        (Some(line), Some(1)) => line.singles += 1,
        (Some(line), Some(2)) => line.doubles += 1,
        (Some(line), Some(3)) => line.triples += 1,
        (Some(line), Some(4)) => line.home_runs += 1,
        _ => {} // An earlier unclassified Hit cannot be restored by later facts.
    }
}

fn starts_game(state: &GameState) -> bool {
    state.inning == 1
        && state.half == Half::Top
        && state.outs == 0
        && state.score == [0, 0]
        && state.bases.iter().all(Option::is_none)
}

// These new safe-PA shapes do not certify extra episodes or fielding chances.
fn simple_safe_pa_episode(event: &Event) -> bool {
    event.delta.outs == 0
        && event.delta.changes.is_empty()
        && event.two_out_contact.is_none()
        && event.single_delivery.is_none()
        && event.balk.is_none()
        && !event.delta.credits.iter().any(|c| {
            matches!(
                c.kind,
                CreditKind::StolenBase
                    | CreditKind::CaughtStealing
                    | CreditKind::WildPitch
                    | CreditKind::Balk
                    | CreditKind::Putout
                    | CreditKind::Assist
            )
        })
}

// Canonical transitions already validate actors, origins, charges, scorer
// value, runs and collisions. Require the complete station advancement shape;
// a safe final snapshot alone does not supply the omitted play history.
fn clean_station_hit(before: &GameState, event: &Event) -> bool {
    let Some(value @ 1..=3) = hit_value(event) else {
        return false;
    };
    let destinations = [
        Destination::First,
        Destination::Second,
        Destination::Third,
        Destination::Home,
    ];
    simple_safe_pa_episode(event)
        && event.delta.moves.len() == before.bases.iter().filter(|b| b.is_some()).count() + 1
        && event.delta.moves.iter().all(|m| {
            let from = match m.from {
                Base::Plate => 0,
                Base::First => 1,
                Base::Second => 2,
                Base::Third => 3,
            };
            m.to == destinations[(from + value - 1).min(3)]
        })
}

// Classifications supply the protected first-base award; only its exact force
// chain is reconstructed here. Unforced runners may be omitted or held, never
// advanced by this proof. Canonical transitions already preserve actor/charges.
fn clean_forced_award(before: &GameState, event: &Event) -> bool {
    if !simple_safe_pa_episode(event)
        || !event
            .delta
            .credits
            .iter()
            .any(|c| matches!(c.kind, CreditKind::Walk | CreditKind::HitByPitch))
    {
        return false;
    }
    crate::rules::protected_award_moves(before, event)
}

/// A deliberately narrow errorless-play proof. Absence of an Error credit
/// alone is not proof: generic contact/advancement may conceal a misplay.
fn clean_play(before: &GameState, event: &Event) -> bool {
    if event.fielding_review != FieldingReview::NoMisplay {
        return false;
    }
    if event.delta.credits.iter().any(|c| {
        matches!(
            c.kind,
            CreditKind::Error | CreditKind::PassedBall | CreditKind::InterferenceAward
        )
    }) {
        return false;
    }
    match event.kind {
        EventKind::Substitution | EventKind::OutOfParkHomeRun => true,
        // Canonical validation bounds FC to one retired runner, batter to
        // first, one out and no runs; explicit review excludes missed chances.
        EventKind::FieldersChoice => true,
        EventKind::PlateAppearance => {
            let m = &event.delta.moves;
            // Bound retirement facts prove one before-first out with held
            // runners; sole-hit facts bind continuous unaided safe advances.
            // Whole-chance review excludes erased misplays in both cases.
            // Credits/endpoints alone never supply these methods.
            event.ordinary_ground_throw.is_some()
                || event.ordinary_air_catch.is_some()
                || event.caught_strikeout.is_some()
                || event.sole_safe_hit.is_some()
                || clean_station_hit(before, event)
                || clean_forced_award(before, event)
                || (m.len() == 1
                    && m[0].from == Base::Plate
                    && event.delta.runs == 0
                    && ((m[0].to == Destination::Out
                        && event.delta.outs == 1
                        && event
                            .delta
                            .credits
                            .iter()
                            .any(|c| c.kind == CreditKind::Strikeout))
                        || (m[0].to == Destination::First
                            && event.delta.outs == 0
                            && event.delta.credits.iter().any(|c| {
                                c.kind == CreditKind::Walk
                                    || (c.kind == CreditKind::HitByPitch
                                        && simple_safe_pa_episode(event))
                            }))))
        }
        EventKind::RunnerOnly(RunnerEvent::Balk) => {
            event.balk.as_ref().is_some_and(crate::balk::exact_awards)
        }
        EventKind::RunnerOnly(RunnerEvent::WildPitch) => {
            let m = &event.delta.moves;
            m.len() == 1
                && event.delta.outs == 0
                && matches!(
                    (m[0].from, m[0].to),
                    (Base::First, Destination::Second)
                        | (Base::Second, Destination::Third)
                        | (Base::Third, Destination::Home)
                )
        }
        _ => false,
    }
}

fn finish_half(
    accounting: &mut GameAccounting,
    runs: &BTreeMap<(usize, String), u32>,
    certified: bool,
    unearned: bool,
    reconstructed: Option<u32>,
) {
    for ((side, id), count) in runs {
        let pitcher = accounting.pitchers[*side].entry(id.clone()).or_default();
        if certified {
            if let Some(earned) = &mut pitcher.earned_runs {
                *earned += count;
            }
        } else if let Some(contribution) = reconstructed {
            // The empty-base muff certificate has one unchanged pitcher and
            // no inherited charges; its scalar contribution belongs to him.
            if let Some(earned) = &mut pitcher.earned_runs {
                *earned += contribution;
            }
        } else if !unearned {
            pitcher.earned_runs = None;
        }
        // A known unearned contribution adds zero; it cannot repair None from
        // an earlier positive half with missing reconstruction.
    }
}

pub fn derive_accounting(log: &[EventTransition]) -> Result<GameAccounting, AccountingError> {
    let first = log.first().ok_or(AccountingError::EmptyLog)?;
    if log
        .windows(2)
        .any(|pair| pair[1].follows(&pair[0]).is_err())
    {
        return Err(AccountingError::Discontinuous);
    }
    let mut result = GameAccounting {
        innings: BTreeMap::new(),
        runs: [0, 0],
        batters: std::array::from_fn(|_| BTreeMap::new()),
        pitchers: std::array::from_fn(|_| BTreeMap::new()),
        fielders: std::array::from_fn(|_| BTreeMap::new()),
        classified_fielding: std::array::from_fn(|_| BTreeMap::new()),
        complete_game: starts_game(first.before()) && log.last().unwrap().after().over,
    };
    let mut frame = (first.before().inning, first.before().half);
    // A later zero-out/empty-base snapshot alone is not evidence of a half's
    // start (it could follow an unobserved solo HR). Later halves must follow
    // an observed boundary in this contiguous log.
    let mut clean_half = starts_game(first.before());
    // Conservative missed-third-out certificate: observed clean zero-run
    // prefix, a reviewed muff at two outs, clean/reviewed-muff suffix, no
    // personnel changes and one actual pitcher. Rule9.16(i) forbids relief
    // inheritance. Later reviewed errors cannot reopen the virtual inning.
    let mut unearned_half = clean_half;
    let mut missed_third_out = false;
    let mut half_pitcher = first.event().participants.pitcher.clone();
    let mut empty_muff_half = clean_half;
    let mut error_batter = None;
    let mut error_runner = None;
    let mut empty_muff_earned = 0;
    // Virtual inning clock only, not a limit on actual errors/events/credits.
    let mut lost_muff_outs = 0;
    let mut half_runs: BTreeMap<(usize, String), u32> = BTreeMap::new();
    for (index, transition) in log.iter().enumerate() {
        let before = transition.before();
        let event = transition.event();
        if index > 0 && (before.inning, before.half) != frame {
            frame = (before.inning, before.half);
            clean_half = before.outs == 0 && before.bases.iter().all(Option::is_none);
            unearned_half = clean_half;
            missed_third_out = false;
            half_pitcher = event.participants.pitcher.clone();
            empty_muff_half = clean_half;
            error_batter = None;
            error_runner = None;
            empty_muff_earned = 0;
            lost_muff_outs = 0;
            half_runs.clear();
        }
        // Error-affected reconstruction still needs the older bounded geometry;
        // whole-chance review alone cannot reconstruct a misplayed inning.
        let clean = transition.caught_sacrifice_fly_facts().is_none()
            && transition.sacrifice_bunt_throw_facts().is_none()
            && clean_play(before, event);
        if (before.outs < 2 || error_batter.is_some())
            && before.bases.iter().all(Option::is_none)
            && event.fielding_review == FieldingReview::SoleGroundMuff
            && event.muffed_ground_ball.is_some()
        {
            // Earlier clean scoring stays earned. Seed once, not at later
            // empty-base episodes: their prior totals include unearned reaches.
            // The scalar certificate cannot cover inherited/other charges.
            if error_batter.is_none() {
                empty_muff_half &= half_runs.keys().all(|(side, pitcher)| {
                    *side == 1 - before.half.offense() && pitcher == &half_pitcher
                });
                empty_muff_earned = half_runs.values().sum();
            }
            lost_muff_outs = (lost_muff_outs + 1).min(3);
            error_batter = event
                .muffed_ground_ball
                .as_ref()
                .map(|f| f.batter_id.clone());
            error_runner = error_batter.clone();
        } else if error_batter.is_some() {
            // Exact station/balk advances and forces behind the leading error
            // runner survive its deletion. Other advancement stays unproved.
            empty_muff_half &= clean
                && (event.caught_strikeout.is_some()
                    || event.ordinary_ground_throw.is_some()
                    || event.sole_home_run.is_some()
                    || (event.sole_safe_hit.is_some() && clean_station_hit(before, event))
                    || (event.protected_award.is_some() && clean_forced_award(before, event))
                    || event.balk.as_ref().is_some_and(crate::balk::exact_awards));
        } else {
            empty_muff_half &= clean;
        }
        if before.outs == 2
            && event.muffed_ground_ball.is_some()
            && event.fielding_review == FieldingReview::SoleGroundMuff
        {
            // Only the first missed retirement needs a zero-run prefix.
            // Further bound muffs occur after the virtual inning already ended;
            // this conjunction still cannot repair earlier unknown evidence.
            unearned_half &= missed_third_out || half_runs.is_empty();
            missed_third_out = true;
        } else {
            unearned_half &= clean;
        }
        // NoMisplay explicitly reviews all chances, not just recorded errors.
        // A canonical substitution contains no play, credit or scoring chance.
        clean_half &= event.kind == EventKind::Substitution
            || event.fielding_review == FieldingReview::NoMisplay;
        let offense = before.half.offense();
        let defense = 1 - offense;
        let unchanged_personnel = event.delta.changes.is_empty()
            && event.participants.pitcher == half_pitcher
            && transition.after().teams[defense].pitcher == half_pitcher;
        unearned_half &= unchanged_personnel;
        empty_muff_half &= unchanged_personnel;
        if event.kind != EventKind::Substitution {
            let team = &before.teams[defense];
            // A no-DH pitcher is also in the batting order; observe him once.
            let actors: BTreeSet<_> = team
                .batting_order
                .iter()
                .chain([&team.pitcher])
                .filter(|id| crate::rules::fielding_eligible(team, id))
                .collect();
            let contribution = transition.fielding_contribution();
            let misplays = transition.misplay_contribution();
            for id in actors {
                let line = result.classified_fielding[defense]
                    .entry(id.clone())
                    .or_default();
                line.observed_events += 1;
                line.putouts = line
                    .putouts
                    .zip(contribution.as_ref())
                    .map(|(total, proof)| total + proof.putouts.get(id).copied().unwrap_or(0));
                line.assists = line
                    .assists
                    .zip(contribution.as_ref())
                    .map(|(total, proof)| total + proof.assists.get(id).copied().unwrap_or(0));
                line.errors = line
                    .errors
                    .zip(misplays.as_ref())
                    .map(|(total, proof)| total + proof.errors.get(id).copied().unwrap_or(0));
                line.passed_balls = line
                    .passed_balls
                    .zip(misplays.as_ref())
                    .map(|(total, proof)| total + proof.passed_balls.get(id).copied().unwrap_or(0));
            }
        }
        let credited_runs = event
            .delta
            .credits
            .iter()
            .filter(|c| c.kind == CreditKind::Run)
            .count();
        let credited_runs =
            u16::try_from(credited_runs).map_err(|_| AccountingError::ScoreMismatch)?;
        let line = &mut result.innings.entry(before.inning).or_insert([None, None])[offense];
        *line = Some(line.unwrap_or(0) + credited_runs);
        result.runs[offense] += u32::from(credited_runs);
        let current_pitcher = event.participants.pitcher.as_str();
        result.pitchers[defense]
            .entry(current_pitcher.into())
            .or_default()
            .outs_recorded += u32::from(event.delta.outs);
        if event.kind.is_pa() {
            let has = |kind| event.delta.credits.iter().any(|credit| credit.kind == kind);
            let excluded = [
                CreditKind::Walk,
                CreditKind::HitByPitch,
                CreditKind::SacrificeBunt,
                CreditKind::SacrificeFly,
                CreditKind::InterferenceAward,
            ]
            .into_iter()
            .any(has);
            let counted = matches!(
                event.kind,
                EventKind::OutOfParkHomeRun | EventKind::FieldersChoice
            ) || [
                CreditKind::AtBat,
                CreditKind::Hit,
                CreditKind::Strikeout,
                CreditKind::GroundedIntoDoublePlay,
            ]
            .into_iter()
            .any(has);
            let at_bat = if excluded {
                Some(false)
            } else if counted {
                Some(true)
            } else {
                None
            };
            // A missing Hit is not a no-hit scorer decision. Bare AB/GIDP,
            // endpoints and whole-chance review do not fill this coverage gap.
            let classified_hit = if has(CreditKind::Hit) {
                Some(1)
            } else if excluded
                || event.kind == EventKind::FieldersChoice
                || has(CreditKind::Strikeout)
                || has(CreditKind::NoHit)
            {
                Some(0)
            } else {
                None
            };
            let batter = event.participants.batter.as_deref().expect("validated PA");
            let batting_line = result.batters[offense].entry(batter.into()).or_default();
            let pitching_line = result.pitchers[defense]
                .entry(current_pitcher.into())
                .or_default();
            pitching_line.batters_faced += 1;
            batting_line.classified_rbi = batting_line
                .classified_rbi
                .zip(transition.rbi_contribution())
                .map(|(total, contribution)| total + contribution.runs_batted_in[batter]);
            batting_line.classified_hits = batting_line
                .classified_hits
                .zip(classified_hit)
                .map(|(total, hit)| total + hit);
            pitching_line.classified_hits_allowed = pitching_line
                .classified_hits_allowed
                .zip(classified_hit)
                .map(|(total, hit)| total + hit);
            match at_bat {
                Some(true) => {
                    if let Some(n) = &mut batting_line.at_bats {
                        *n += 1;
                    }
                    if let Some(n) = &mut pitching_line.at_bats_against {
                        *n += 1;
                    }
                }
                None => {
                    batting_line.at_bats = None;
                    pitching_line.at_bats_against = None;
                }
                Some(false) => {}
            }
        }
        for credit in &event.delta.credits {
            if credit.kind == CreditKind::Balk {
                result.pitchers[defense]
                    .entry(current_pitcher.into())
                    .or_default()
                    .credited_balks += 1;
                continue;
            }
            if credit.kind == CreditKind::WildPitch {
                result.pitchers[defense]
                    .entry(current_pitcher.into())
                    .or_default()
                    .credited_wild_pitches += 1;
                continue;
            }
            if matches!(
                credit.kind,
                CreditKind::Putout
                    | CreditKind::Assist
                    | CreditKind::Error
                    | CreditKind::PassedBall
            ) {
                let fielder = result.fielders[defense]
                    .entry(credit.player_id.clone())
                    .or_default();
                match credit.kind {
                    CreditKind::Putout => fielder.credited_putouts += 1,
                    CreditKind::Assist => fielder.credited_assists += 1,
                    CreditKind::Error => fielder.credited_errors += 1,
                    CreditKind::PassedBall => fielder.credited_passed_balls += 1,
                    _ => unreachable!(),
                }
                continue;
            }
            let batter = result.batters[offense]
                .entry(credit.player_id.clone())
                .or_default();
            match credit.kind {
                CreditKind::PlateAppearance => batter.plate_appearances += 1,
                CreditKind::Hit => {
                    batter.credited_hits += 1;
                    add_hit(&mut batter.credited_hit_breakdown, event);
                    let pitcher = result.pitchers[defense]
                        .entry(current_pitcher.into())
                        .or_default();
                    pitcher.credited_hits_allowed += 1;
                    add_hit(&mut pitcher.credited_hit_breakdown_allowed, event);
                }
                CreditKind::Walk => {
                    batter.credited_walks += 1;
                    let classification = event.delta.credits.iter().find_map(|credit| {
                        matches!(
                            credit.kind,
                            CreditKind::OrdinaryWalk | CreditKind::IntentionalWalk
                        )
                        .then_some(credit.kind)
                    });
                    match classification {
                        Some(CreditKind::OrdinaryWalk) => {}
                        Some(CreditKind::IntentionalWalk) => {
                            if let Some(intentional_walks) = &mut batter.credited_intentional_walks
                            {
                                *intentional_walks += 1;
                            }
                        }
                        None => batter.credited_intentional_walks = None,
                        _ => unreachable!("walk classification is validated"),
                    }
                    let pitcher = result.pitchers[defense]
                        .entry(current_pitcher.into())
                        .or_default();
                    pitcher.credited_walks_allowed += 1;
                    match classification {
                        Some(CreditKind::OrdinaryWalk) => {}
                        Some(CreditKind::IntentionalWalk) => {
                            if let Some(intentional_walks) =
                                &mut pitcher.credited_intentional_walks_allowed
                            {
                                *intentional_walks += 1;
                            }
                        }
                        None => pitcher.credited_intentional_walks_allowed = None,
                        _ => unreachable!("walk classification is validated"),
                    }
                }
                CreditKind::Strikeout => {
                    batter.credited_strikeouts += 1;
                    result.pitchers[defense]
                        .entry(current_pitcher.into())
                        .or_default()
                        .credited_strikeouts += 1;
                }
                CreditKind::Run => {
                    // Only the original error reach is unearned; clear it
                    // on scoring so that actor's later clean reach can earn.
                    // One lost out ends the virtual inning at two actual outs.
                    let error_reach = error_runner.as_ref() == Some(&credit.player_id);
                    if empty_muff_half
                        && error_batter.is_some()
                        && !error_reach
                        && before.outs + lost_muff_outs < 3
                    {
                        empty_muff_earned += 1;
                    }
                    if error_reach {
                        error_runner = None;
                    }
                    batter.credited_runs += 1;
                    let scorer = event
                        .delta
                        .moves
                        .iter()
                        .find(|movement| {
                            movement.to == Destination::Home
                                && movement.runner.player_id == credit.player_id
                        })
                        .expect("validated Run credit");
                    let responsible = scorer.runner.responsible_pitcher.clone();
                    result.pitchers[defense]
                        .entry(responsible.clone())
                        .or_default()
                        .runs_allowed += 1;
                    *half_runs.entry((defense, responsible)).or_default() += 1;
                }
                CreditKind::Rbi => batter.credited_rbi += 1,
                CreditKind::StolenBase => batter.credited_steals += 1,
                CreditKind::CaughtStealing => batter.credited_caught_stealing += 1,
                CreditKind::AtBat
                | CreditKind::NoHit
                | CreditKind::Single
                | CreditKind::Double
                | CreditKind::Triple
                | CreditKind::OrdinaryWalk
                | CreditKind::IntentionalWalk => {}
                CreditKind::GroundedIntoDoublePlay => {
                    batter.credited_grounded_into_double_plays += 1
                }
                CreditKind::HitByPitch => {
                    batter.credited_hit_by_pitch += 1;
                    result.pitchers[defense]
                        .entry(current_pitcher.into())
                        .or_default()
                        .credited_hit_batters += 1;
                }
                CreditKind::SacrificeBunt => {
                    batter.credited_sacrifice_bunts += 1;
                    result.pitchers[defense]
                        .entry(current_pitcher.into())
                        .or_default()
                        .credited_sacrifice_bunts += 1;
                }
                CreditKind::SacrificeFly => {
                    batter.credited_sacrifice_flies += 1;
                    result.pitchers[defense]
                        .entry(current_pitcher.into())
                        .or_default()
                        .credited_sacrifice_flies += 1;
                }
                CreditKind::InterferenceAward => batter.credited_interference_awards += 1,
                CreditKind::WildPitch
                | CreditKind::Balk
                | CreditKind::Putout
                | CreditKind::Assist
                | CreditKind::Error
                | CreditKind::PassedBall => unreachable!(),
            }
        }
        let ended = transition.after().over
            || (transition.after().inning, transition.after().half) != frame;
        if ended || index + 1 == log.len() {
            finish_half(
                &mut result,
                &half_runs,
                clean_half && ended,
                unearned_half && missed_third_out && ended,
                (empty_muff_half && error_batter.is_some() && ended).then_some(empty_muff_earned),
            );
        }
    }
    let last = log.last().unwrap().after();
    if (0..2).any(|side| {
        u32::from(last.score[side]) - u32::from(first.before().score[side]) != result.runs[side]
    }) {
        return Err(AccountingError::ScoreMismatch);
    }
    Ok(result)
}
