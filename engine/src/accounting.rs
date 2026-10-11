//! Independent accounting over validated, contiguous event transitions.
//! Credited counts are not complete official box cells: the event vocabulary
//! cannot yet distinguish every PA/fielding/earned-run outcome. Unknown stays
//! `None`, not zero. No game-state score is used to accumulate a line.
use std::collections::BTreeMap;

use crate::{
    Base, CreditKind, Destination, Event, EventKind, EventTransition, FieldingReview, GameState,
    Half, RunnerEvent,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BatterLine {
    pub plate_appearances: u32,
    pub credited_hits: u32,
    pub credited_walks: u32,
    pub credited_strikeouts: u32,
    pub credited_runs: u32,
    pub credited_rbi: u32,
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
            credited_walks: 0,
            credited_strikeouts: 0,
            credited_runs: 0,
            credited_rbi: 0,
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
    pub credited_walks_allowed: u32,
    pub credited_strikeouts: u32,
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
            credited_walks_allowed: 0,
            credited_strikeouts: 0,
            credited_hit_batters: 0,
            credited_sacrifice_bunts: 0,
            credited_sacrifice_flies: 0,
            earned_runs: Some(0),
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

fn starts_game(state: &GameState) -> bool {
    state.inning == 1
        && state.half == Half::Top
        && state.outs == 0
        && state.score == [0, 0]
        && state.bases.iter().all(Option::is_none)
}

/// A deliberately narrow errorless-play proof. Absence of an Error credit
/// alone is not proof: generic contact/advancement may conceal a misplay.
fn clean_play(event: &Event) -> bool {
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
            m.len() == 1
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
                        && event
                            .delta
                            .credits
                            .iter()
                            .any(|c| c.kind == CreditKind::Walk)))
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
) {
    for ((side, id), count) in runs {
        let pitcher = accounting.pitchers[*side].entry(id.clone()).or_default();
        if certified {
            if let Some(earned) = &mut pitcher.earned_runs {
                *earned += count;
            }
        } else {
            pitcher.earned_runs = None;
        }
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
        complete_game: starts_game(first.before()) && log.last().unwrap().after().over,
    };
    let mut frame = (first.before().inning, first.before().half);
    // A later zero-out/empty-base snapshot alone is not evidence of a half's
    // start (it could follow an unobserved solo HR). Later halves must follow
    // an observed boundary in this contiguous log.
    let mut clean_half = starts_game(first.before());
    let mut half_runs: BTreeMap<(usize, String), u32> = BTreeMap::new();
    for (index, transition) in log.iter().enumerate() {
        let before = transition.before();
        let event = transition.event();
        if index > 0 && (before.inning, before.half) != frame {
            frame = (before.inning, before.half);
            clean_half = before.outs == 0 && before.bases.iter().all(Option::is_none);
            half_runs.clear();
        }
        clean_half &= clean_play(event);
        let offense = before.half.offense();
        let defense = 1 - offense;
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
            ) || [CreditKind::AtBat, CreditKind::Hit, CreditKind::Strikeout]
                .into_iter()
                .any(has);
            let at_bat = if excluded {
                Some(false)
            } else if counted {
                Some(true)
            } else {
                None
            };
            let batter = event.participants.batter.as_deref().expect("validated PA");
            let batting_line = result.batters[offense].entry(batter.into()).or_default();
            let pitching_line = result.pitchers[defense]
                .entry(current_pitcher.into())
                .or_default();
            pitching_line.batters_faced += 1;
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
                    result.pitchers[defense]
                        .entry(current_pitcher.into())
                        .or_default()
                        .credited_hits_allowed += 1;
                }
                CreditKind::Walk => {
                    batter.credited_walks += 1;
                    result.pitchers[defense]
                        .entry(current_pitcher.into())
                        .or_default()
                        .credited_walks_allowed += 1;
                }
                CreditKind::Strikeout => {
                    batter.credited_strikeouts += 1;
                    result.pitchers[defense]
                        .entry(current_pitcher.into())
                        .or_default()
                        .credited_strikeouts += 1;
                }
                CreditKind::Run => {
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
                CreditKind::AtBat => {} // already included in the PA classification
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
                CreditKind::Putout
                | CreditKind::Assist
                | CreditKind::Error
                | CreditKind::PassedBall => unreachable!(),
            }
        }
        let ended = transition.after().over
            || (transition.after().inning, transition.after().half) != frame;
        if ended || index + 1 == log.len() {
            finish_half(&mut result, &half_runs, clean_half && ended);
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
