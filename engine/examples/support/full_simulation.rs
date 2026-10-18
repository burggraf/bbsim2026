//! Versioned, provisional ordinary-game policy; canonical apply owns every state change.
use super::{
    full_game::{
        self, ContactDecision, HitDecision, Ordinary, ResolvedPlay, RunnerDecision,
        StrikeoutDecision, TagAttempt,
    },
    full_manager::Cast,
    preview::Rng,
};
use bbsim_engine::{
    AirContact, Base, Destination, EventTransition, GameState, GroundForceDoublePlayFacts,
    SacrificeBuntJudgment, SacrificeBuntThrowFacts,
};

#[allow(dead_code)] // Only opt-in personnel callers consume the bench schema and PH policy.
#[path = "full_bench.rs"]
pub mod bench;
#[allow(dead_code)]
#[path = "full_pinch.rs"]
pub mod pinch;

pub const MODEL_VERSION: &str = "text-game-recovery-v1-joint-hit-k-incomplete";
// Both variants retain an inline immutable transition; boxing saves little but adds per-event allocations.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    Play(ResolvedPlay),
    PitchingChange(EventTransition),
    PinchHit(EventTransition),
}
impl Step {
    pub fn transition(&self) -> &EventTransition {
        match self {
            Self::Play(p) => p.transition(),
            Self::PitchingChange(t) | Self::PinchHit(t) => t,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct PaDraw {
    pub probabilities: [f64; 10],
    pub word: u64,
    pub parent: usize,
}
pub struct Simulation {
    pub steps: Vec<Step>,
    #[allow(dead_code)] // Pooled-only callers do not inspect rated draw receipts.
    pub draws: Vec<(usize, PaDraw)>,
}
pub struct Model {
    pa: [u64; 10],
    contact: [u64; 4],
}
impl Model {
    pub fn new(counts: [u64; 10], contact: [u64; 5]) -> Result<Self, String> {
        let total = counts
            .iter()
            .try_fold(0u64, |sum, n| sum.checked_add(*n))
            .ok_or("PA count overflow")?;
        if total == 0 || total > (1u64 << 52) {
            return Err("empty or unsupported PA counts".into());
        }
        let contact_total = contact
            .iter()
            .try_fold(0u64, |sum, n| sum.checked_add(*n))
            .ok_or("contact count overflow")?;
        if contact_total == 0 || contact_total != counts[8] {
            return Err("contact/PA mass mismatch".into());
        }
        Ok(Self {
            pa: counts.map(|n| 2 * n + 1),
            contact: std::array::from_fn(|i| 100 * contact[i] + [45, 35, 15, 5][i] * contact[4]),
        })
    }
    /// Retain pooled interference and scale all nine rated categories without losing mass.
    #[allow(dead_code)] // Pooled mechanism callers do not use player probabilities.
    pub fn player_probabilities(&self, player: [f64; 9]) -> Result<[f64; 10], String> {
        let interference = self.pa[9] as f64 / self.pa.iter().sum::<u64>() as f64;
        let probabilities = std::array::from_fn(|k| {
            if k == 9 {
                interference
            } else {
                player[k] * (1.0 - interference)
            }
        });
        validate_pa(&probabilities)?;
        Ok(probabilities)
    }
    fn caught_air(
        &self,
        before: &GameState,
        fielders: &[String; 9],
        contact: AirContact,
        position: u8,
        rng: &mut Rng,
    ) -> Result<ResolvedPlay, String> {
        if before.outs < 2 {
            if contact == AirContact::Fly
                && position >= 7
                && before.bases[2].is_some()
                && unit(rng) < 0.60
            {
                return full_game::air_advance(
                    before,
                    fielders,
                    contact,
                    position,
                    Base::Third,
                    false,
                );
            }
            let available =
                |i: usize| before.bases[i].is_some() && (i == 2 || before.bases[i + 1].is_none());
            if let Some(i) = (0..3).rev().find(|&i| available(i)) {
                if unit(rng) < 0.02 {
                    return full_game::air_advance(
                        before,
                        fielders,
                        contact,
                        position,
                        [Base::First, Base::Second, Base::Third][i],
                        true,
                    );
                }
            }
            if let Some(i) = (0..2).rev().find(|&i| available(i)) {
                if unit(rng) < 0.20 {
                    return full_game::air_advance(
                        before,
                        fielders,
                        contact,
                        position,
                        [Base::First, Base::Second][i],
                        unit(rng) < 0.10,
                    );
                }
            }
        }
        full_game::ordinary(
            before,
            fielders,
            match contact {
                AirContact::Fly => Ordinary::FlyOut(position),
                AirContact::LineDrive => Ordinary::LineOut(position),
                AirContact::PopUp => Ordinary::PopOut(position),
            },
        )
    }
    fn hit(
        before: &GameState,
        fielders: &[String; 9],
        value: u8,
        position: u8,
        rng: &mut Rng,
    ) -> Result<ResolvedPlay, String> {
        let mut runners = [None; 3];
        for i in (0..3).rev().filter(|&i| before.bases[i].is_some()) {
            runners[i] = Some(match (value, i) {
                (1, 0) if runners[1] != Some(Destination::Third) && unit(rng) < 0.30 => {
                    Destination::Third
                }
                (1, 0) => Destination::Second,
                (1, 1) if unit(rng) >= 0.60 => Destination::Third,
                (2, 0) if unit(rng) >= 0.50 => Destination::Third,
                _ => Destination::Home,
            });
        }
        let safe_runs = runners
            .iter()
            .filter(|d| **d == Some(Destination::Home))
            .count() as u32;
        let winning = before.half == bbsim_engine::Half::Bottom
            && before.inning >= 9
            && u32::from(before.score[1]) + safe_runs > u32::from(before.score[0]);
        let mut decision = HitDecision {
            value,
            position,
            runners,
            tag: None,
        };
        // Stop the winning safe opportunity before selecting any optional throw.
        if !winning {
            if let Some(i) = (0..2).rev().find(|&i| {
                matches!(
                    (value, i, runners[i]),
                    (1, 1, Some(Destination::Home))
                        | (1, 0, Some(Destination::Third))
                        | (2, 0, Some(Destination::Home))
                )
            }) {
                if unit(rng) < 0.10 {
                    let attempted_base = runners[i].ok_or("missing extra-base endpoint")?;
                    decision.runners[i] = Some(Destination::Out);
                    decision.tag = Some(TagAttempt {
                        origin: [Base::First, Base::Second][i],
                        attempted_base,
                        home_before_tag: (0..3)
                            .rev()
                            .filter(|&i| decision.runners[i] == Some(Destination::Home))
                            .map(|i| [Base::First, Base::Second, Base::Third][i])
                            .collect(),
                    });
                }
            }
        }
        full_game::hit(before, fielders, decision)
    }
    fn uncaught(
        before: &GameState,
        fielders: &[String; 9],
        rng: &mut Rng,
    ) -> Result<ResolvedPlay, String> {
        if before.outs < 2 && before.bases[0].is_some() {
            return full_game::uncaught_strikeout(before, fielders, StrikeoutDecision::Automatic);
        }
        let handling = if unit(rng) < 0.75 {
            bbsim_engine::HandlingAssessment::PitcherWild
        } else {
            bbsim_engine::HandlingAssessment::CatcherPassed
        };
        let occupied = before.bases.iter().any(Option::is_some);
        let advance = occupied && unit(rng) < 0.75;
        let decision = if advance {
            let winning = before.outs < 2
                && before.half == bbsim_engine::Half::Bottom
                && before.inning >= 9
                && before.score[0] == before.score[1]
                && before.bases[2].is_some();
            if winning || unit(rng) < 2.0 / 3.0 {
                StrikeoutDecision::SafeReach { handling }
            } else {
                StrikeoutDecision::RecoveredBeforeFirstAfterAdvances { handling }
            }
        } else if !occupied && unit(rng) < 0.50 {
            StrikeoutDecision::SafeReach { handling }
        } else {
            StrikeoutDecision::RecoveredBeforeFirst { handling }
        };
        full_game::uncaught_strikeout(before, fielders, decision)
    }
    #[allow(dead_code)] // Direct pooled sampling is also exercised by mechanism tests.
    pub(crate) fn next(
        &self,
        before: &GameState,
        fielders: &[String; 9],
        rng: &mut Rng,
    ) -> Result<ResolvedPlay, String> {
        self.next_with_probabilities(before, fielders, rng, &mut |_| Ok(None), &mut None)
    }
    fn next_with_probabilities(
        &self,
        before: &GameState,
        fielders: &[String; 9],
        rng: &mut Rng,
        probabilities: &mut impl FnMut(&GameState) -> Result<Option<[f64; 10]>, String>,
        audit: &mut Option<PaDraw>,
    ) -> Result<ResolvedPlay, String> {
        full_game::validate_defense(before, fielders)?;
        let offense = before.half.offense();
        let batter = before.teams[offense]
            .batting_order
            .get(usize::from(before.teams[offense].next_batter))
            .ok_or("invalid batting cursor before generation")?
            .clone();
        if before.bases.iter().any(Option::is_some) {
            let u = unit(rng);
            let decision = if u < 0.025 {
                let i = (0..3)
                    .rev()
                    .find(|i| {
                        before.bases[*i].is_some() && (*i == 2 || before.bases[*i + 1].is_none())
                    })
                    .ok_or("no legal steal target")?;
                Some(RunnerDecision::Steal {
                    origin: [Base::First, Base::Second, Base::Third][i],
                    target: [Destination::Second, Destination::Third, Destination::Home][i],
                    caught: unit(rng) >= 0.75,
                })
            } else if u < 0.029 {
                let i = (0..3)
                    .rev()
                    .find(|i| before.bases[*i].is_some())
                    .ok_or("no pickoff runner")?;
                Some(RunnerDecision::Pickoff {
                    origin: [Base::First, Base::Second, Base::Third][i],
                    caught: unit(rng) < 0.15,
                })
            } else if u < 0.041 {
                Some(RunnerDecision::WildPitch)
            } else if u < 0.044 {
                Some(RunnerDecision::PassedBall)
            } else if u < 0.045 {
                Some(RunnerDecision::Balk)
            } else {
                None
            };
            if let Some(decision) = decision {
                return full_game::runner(before, fielders, decision);
            }
        }
        if before.inning >= 7
            && before.outs < 2
            && before.bases[0].is_none()
            && (before.bases[1].is_some() || before.bases[2].is_some())
            && before.score[0].abs_diff(before.score[1]) <= 2
            && unit(rng) < 0.03
        {
            return full_game::ordinary(before, fielders, Ordinary::IntentionalWalk);
        }
        let parent = match probabilities(before)? {
            Some(p) => {
                let word = rng.next_u64();
                let parent = sample_pa(p, word)?;
                *audit = Some(PaDraw {
                    probabilities: p,
                    word,
                    parent,
                });
                parent
            }
            None => draw(&self.pa, rng),
        };
        let outfield = |rng: &mut Rng| 7 + draw(&[1, 1, 1], rng) as u8;
        let choice = match parent {
            0 => {
                let u = unit(rng);
                if u < 0.20 {
                    Ordinary::CalledStrikeout
                } else if u < 0.98 {
                    Ordinary::SwingingStrikeout
                } else {
                    return Self::uncaught(before, fielders, rng);
                }
            }
            1 => Ordinary::Walk,
            2 => Ordinary::Hbp,
            3..=5 => return Self::hit(before, fielders, parent as u8 - 2, outfield(rng), rng),
            6 => Ordinary::HomeRun(outfield(rng)),
            7 => {
                let position = 1 + draw(&[5, 1, 12, 25, 22, 35], rng) as u8;
                if unit(rng) < 0.60 {
                    Ordinary::FieldingError(position)
                } else {
                    Ordinary::ThrowingError(position)
                }
            }
            8 => match draw(&self.contact, rng) {
                0 => {
                    let u = unit(rng);
                    if before.outs < 2 && before.bases[0].is_some() {
                        if u < 0.18 {
                            let position = draw(&[5, 1, 12, 25, 22, 35], rng);
                            let pivot = if position == 3 { 5 } else { 3 };
                            return full_game::contact(
                                before,
                                fielders,
                                ContactDecision::GroundForceDoublePlay(
                                    GroundForceDoublePlayFacts {
                                        batter_id: batter,
                                        forced_runner_id: before.bases[0]
                                            .as_ref()
                                            .unwrap()
                                            .player_id
                                            .clone(),
                                        initial_fielder_id: fielders[position].clone(),
                                        pivot_id: fielders[pivot].clone(),
                                        first_receiver_id: fielders[2].clone(),
                                    },
                                ),
                            );
                        }
                        if (0.18..0.30).contains(&u) {
                            return full_game::force_choice(
                                before,
                                fielders,
                                1 + draw(&[5, 1, 12, 25, 22, 35], rng) as u8,
                                false,
                            );
                        }
                        if (0.30..0.306).contains(&u) {
                            return full_game::force_choice(
                                before,
                                fielders,
                                1 + draw(&[5, 1, 12, 25, 22, 35], rng) as u8,
                                true,
                            );
                        }
                        if (0.306..0.33).contains(&u) && before.bases[1].is_none() {
                            let thrower = if unit(rng) < 0.5 { 0 } else { 4 };
                            return full_game::contact(
                                before,
                                fielders,
                                ContactDecision::SacrificeBunt(SacrificeBuntThrowFacts {
                                    batter_id: batter,
                                    advancing_runner_id: before.bases[0]
                                        .as_ref()
                                        .unwrap()
                                        .player_id
                                        .clone(),
                                    thrower_id: fielders[thrower].clone(),
                                    receiver_id: fielders[2].clone(),
                                    judgment: SacrificeBuntJudgment::SacrificingOwnChance,
                                }),
                            );
                        }
                    }
                    return full_game::ground_out(
                        before,
                        fielders,
                        1 + draw(&[5, 1, 12, 25, 22, 35], rng) as u8,
                    );
                }
                1 => return self.caught_air(before, fielders, AirContact::Fly, outfield(rng), rng),
                2 => {
                    return self.caught_air(
                        before,
                        fielders,
                        AirContact::LineDrive,
                        1 + draw(&[2, 0, 9, 14, 12, 16, 15, 16, 16], rng) as u8,
                        rng,
                    )
                }
                3 => {
                    return self.caught_air(
                        before,
                        fielders,
                        AirContact::PopUp,
                        1 + draw(&[2, 23, 15, 15, 15, 25, 2, 1, 2], rng) as u8,
                        rng,
                    )
                }
                _ => unreachable!("four contact weights"),
            },
            9 => Ordinary::CatcherInterference,
            _ => unreachable!("ten PA weights"),
        };
        full_game::ordinary(before, fielders, choice)
    }
}
// Reconcile generated evidence without projecting bases or re-deciding a play.
pub(crate) fn verify_accounting(log: &[EventTransition]) -> Result<(), String> {
    use bbsim_engine::{accounting, CreditKind, EventKind};
    use std::collections::BTreeMap;
    let first = log.first().ok_or("empty generated log")?;
    if log
        .windows(2)
        .any(|pair| pair[0].after() != pair[1].before())
    {
        return Err("generated log is discontinuous".into());
    }
    let stats = accounting::derive_accounting(log)
        .map_err(|e| format!("generated accounting refused: {e:?}"))?;
    let mut runs = [0u32; 2];
    let mut outs = [0u32; 2];
    let mut pa = [0u32; 2];
    let mut fielding: [BTreeMap<&str, [u32; 4]>; 2] = std::array::from_fn(|_| BTreeMap::new());
    for t in log {
        let event = t.event();
        let offense = t.before().half.offense();
        let defense = 1 - offense;
        let is_pa = matches!(
            event.kind,
            EventKind::PlateAppearance | EventKind::OutOfParkHomeRun | EventKind::FieldersChoice
        );
        runs[offense] += u32::from(event.delta.runs);
        outs[defense] += u32::from(event.delta.outs);
        pa[offense] += u32::from(is_pa);
        let units = |kind| {
            event
                .delta
                .credits
                .iter()
                .filter(|c| c.kind == kind)
                .count() as u32
        };
        if units(CreditKind::Putout) != u32::from(event.delta.outs) {
            return Err("generated retirement lacks exact named putout evidence".into());
        }
        if units(CreditKind::PlateAppearance) != u32::from(is_pa) {
            return Err("generated PA credit mismatch".into());
        }
        for (i, kind) in [
            CreditKind::Putout,
            CreditKind::Assist,
            CreditKind::Error,
            CreditKind::PassedBall,
        ]
        .into_iter()
        .enumerate()
        {
            for credit in event.delta.credits.iter().filter(|c| c.kind == kind) {
                fielding[defense].entry(&credit.player_id).or_default()[i] += 1;
            }
        }
        for side in 0..2 {
            let expected = if is_pa && side == offense {
                (t.before().teams[side].next_batter + 1) % 9
            } else {
                t.before().teams[side].next_batter
            };
            if t.after().teams[side].next_batter != expected {
                return Err("generated batting cursor mismatch".into());
            }
        }
    }
    let final_score = log
        .last()
        .ok_or("empty generated log")?
        .after()
        .score
        .map(u32::from);
    if stats.runs != runs
        || (0..2)
            .any(|side| u32::from(first.before().score[side]) + runs[side] != final_score[side])
    {
        return Err("generated run/score accounting mismatch".into());
    }
    for side in 0..2 {
        let observed: BTreeMap<_, _> = stats.fielders[side]
            .iter()
            .map(|(id, f)| {
                (
                    id.as_str(),
                    [
                        f.credited_putouts,
                        f.credited_assists,
                        f.credited_errors,
                        f.credited_passed_balls,
                    ],
                )
            })
            .collect();
        if stats.batters[side]
            .values()
            .map(|b| b.plate_appearances)
            .sum::<u32>()
            != pa[side]
            || stats.pitchers[side]
                .values()
                .map(|p| p.outs_recorded)
                .sum::<u32>()
                != outs[side]
            || stats.pitchers[side]
                .values()
                .map(|p| p.batters_faced)
                .sum::<u32>()
                != pa[1 - side]
            || observed != fielding[side]
        {
            return Err("generated PA/BF/outs/fielding accounting mismatch".into());
        }
    }
    Ok(())
}
fn unit(rng: &mut Rng) -> f64 {
    (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64
}
// One full word, integer multiply-high partition; no rejection or lost category.
fn draw(weights: &[u64], rng: &mut Rng) -> usize {
    let total: u64 = weights.iter().sum();
    let mut target = ((u128::from(rng.next_u64()) * u128::from(total)) >> 64) as u64;
    for (i, weight) in weights.iter().enumerate() {
        if target < *weight {
            return i;
        }
        target -= weight;
    }
    unreachable!("validated nonzero categorical mass")
}
fn validate_pa(probabilities: &[f64; 10]) -> Result<(), String> {
    if probabilities
        .iter()
        .any(|p| !p.is_finite() || *p <= 0.0 || *p >= 1.0)
        || (probabilities.iter().sum::<f64>() - 1.0).abs() > 32.0 * f64::EPSILON
    {
        return Err("invalid complete-game PA probabilities".into());
    }
    Ok(())
}
/// One supplied word, high-53-bit CDF partition, with the final interval retaining rounding residue.
pub fn sample_pa(probabilities: [f64; 10], word: u64) -> Result<usize, String> {
    validate_pa(&probabilities)?;
    let unit = (word >> 11) as f64 / (1u64 << 53) as f64;
    let mut edge = 0.0;
    for (k, p) in probabilities[..9].iter().enumerate() {
        edge += p;
        if unit < edge {
            return Ok(k);
        }
    }
    Ok(9)
}
#[allow(dead_code)] // The rated command and pooled mechanism tests share the implementation below.
pub fn simulate(
    initial: &GameState,
    fielders: &[[String; 9]; 2],
    model: &Model,
    cast: Option<&Cast>,
    seed: u64,
) -> Result<Vec<Step>, String> {
    simulate_with_probabilities(initial, fielders, model, cast, seed, |_| Ok(None))
        .map(|simulation| simulation.steps)
}
pub fn simulate_with_probabilities(
    initial: &GameState,
    fielders: &[[String; 9]; 2],
    model: &Model,
    cast: Option<&Cast>,
    seed: u64,
    probabilities: impl FnMut(&GameState) -> Result<Option<[f64; 10]>, String>,
) -> Result<Simulation, String> {
    simulate_with_personnel(initial, fielders, model, cast, None, seed, probabilities)
}

pub fn simulate_with_personnel(
    initial: &GameState,
    fielders: &[[String; 9]; 2],
    model: &Model,
    cast: Option<&Cast>,
    bench: Option<&bench::Bench>,
    seed: u64,
    mut probabilities: impl FnMut(&GameState) -> Result<Option<[f64; 10]>, String>,
) -> Result<Simulation, String> {
    let mut state = match cast {
        Some(cast) => cast.prepare(initial)?,
        None => initial.clone(),
    };
    if let Some(bench) = bench {
        state = bench.prepare(
            &state,
            fielders,
            cast.ok_or("PH requires an explicit pitching cast")?,
        )?;
    }
    let mut rng = Rng::new(seed);
    // Manager draws must not consume or shift the outcome RNG stream.
    let mut manager_rng = Rng::new(seed ^ 0x6d61_6e61_6765_7232);
    let mut entries: [(String, u64); 2] = std::array::from_fn(|_| (String::new(), 0));
    let mut steps = Vec::new();
    let mut draws = Vec::new();
    let mut log = Vec::new();
    while !state.over {
        if steps.len() >= 10_000 {
            return Err(format!(
                "seed {seed}: explicit 10000-event resource limit; no winner fabricated"
            ));
        }
        if state.teams[state.half.offense()].next_batter > 8 {
            return Err("invalid batting cursor".into());
        }
        if bench.is_some() {
            if let Some(change) = cast
                .ok_or("PH requires an explicit pitching cast")?
                .restore_pitcher(&state)?
            {
                state = change.after().clone();
                log.push(change.clone());
                steps.push(Step::PitchingChange(change));
                continue;
            }
        }
        state
            .validate_play_ready()
            .map_err(|e| format!("unready personnel before generation: {e:?}"))?;
        let side = 1 - state.half.offense();
        if cast.is_some() && Some(entries[side].0.as_str()) != state.teams[side].pitcher.active_id()
        {
            entries[side] = (
                state.teams[side]
                    .pitcher
                    .active_id()
                    .expect("validated active pitcher")
                    .to_owned(),
                manager_rng.next_u64(),
            );
        }
        let step = if let Some(change) = cast
            .map(|cast| cast.next_change_with_word(&state, &log, entries[side].1))
            .transpose()?
            .flatten()
        {
            Step::PitchingChange(change)
        } else if let Some(change) = bench
            .map(|bench| {
                let offense = state.half.offense();
                let reserve = cast
                    .ok_or("PH requires an explicit pitching cast")?
                    .next_available(&state, offense);
                pinch::decide_states(&state, &bench.pools[offense], reserve, |candidate| {
                    let p = probabilities(candidate)?
                        .ok_or("PH requires individual current-matchup ratings")?;
                    validate_pa(&p)?;
                    Ok(std::array::from_fn(|i| p[i] / (1.0 - p[9])))
                })
            })
            .transpose()?
            .flatten()
        {
            Step::PinchHit(change)
        } else {
            let side = 1 - state.half.offense();
            let mut defense = match &state.teams[side].defense {
                bbsim_engine::DefenseState::LegacyUnassigned => fielders[side].clone(),
                bbsim_engine::DefenseState::Assigned(positions) => {
                    (*positions.clone()).map(|id| id.expect("validated complete defense"))
                }
            };
            defense[0] = state.teams[side]
                .pitcher
                .active_id()
                .expect("validated active pitcher")
                .to_owned();
            let mut draw = None;
            let play = model
                .next_with_probabilities(&state, &defense, &mut rng, &mut probabilities, &mut draw)
                .map_err(|e| {
                    format!(
                        "seed {seed}, event {}, state {state:?}: {e}",
                        steps.len() + 1
                    )
                })?;
            if let Some(draw) = draw {
                draws.push((steps.len() + 1, draw));
            }
            Step::Play(play)
        };
        state = step.transition().after().clone();
        log.push(step.transition().clone());
        steps.push(step);
    }
    verify_accounting(&log)?;
    Ok(Simulation { steps, draws })
}
