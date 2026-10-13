//! Provisional Dirichlet/generalized-log5 participant-proxy means, not forecasting.
use super::{joint, preview, real};
use bbsim_engine::{EventTransition, GameState};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Write},
};
pub const MODEL_VERSION: &str = "dirichlet-log5-player-proxy-preview-v1";
pub const REPORT_SHA: &str = "382be5a84b3e95687e6356b00d3b1b68522f41a5c2a6a065daf876a5a4c6881e";
pub const BASIS: &str = "literal terminal-event participants; not official credits or PA/BF";
const KAPPA: f64 = 200.0;
const COUNTS: [u64; 9] = [42823, 15142, 1984, 25947, 8531, 785, 6776, 1470, 82245];
fn valid_probability(p: &[f64; 9]) -> Result<(), String> {
    if p.iter().any(|x| !x.is_finite() || *x <= 0.0 || *x >= 1.0)
        || (p.iter().sum::<f64>() - 1.0).abs() > 32.0 * f64::EPSILON
    {
        return Err("invalid player probability distribution".into());
    }
    Ok(())
}
fn exposure(counts: [u64; 9]) -> Result<u64, String> {
    let n = counts
        .into_iter()
        .try_fold(0u64, |sum, n| sum.checked_add(n))
        .ok_or("player count overflow")?;
    if n > (1u64 << 52) {
        return Err("unsupported player exposure".into());
    }
    Ok(n)
}
pub fn posterior(counts: [u64; 9], league: [f64; 9]) -> Result<[f64; 9], String> {
    valid_probability(&league)?;
    let n = exposure(counts)?;
    if n == 0 {
        return Ok(league);
    }
    let means =
        std::array::from_fn(|k| (counts[k] as f64 + KAPPA * league[k]) / (n as f64 + KAPPA));
    valid_probability(&means)?;
    Ok(means)
}
pub fn matchup(b: [u64; 9], p: [u64; 9], league: [f64; 9]) -> Result<[f64; 9], String> {
    let b = posterior(b, league)?;
    let p = posterior(p, league)?;
    let w: [f64; 9] = std::array::from_fn(|k| b[k] * p[k] / league[k]);
    let total: f64 = w.iter().sum();
    let result = w.map(|x| x / total);
    valid_probability(&result)?;
    Ok(result)
}
pub struct Ratings {
    counts: BTreeMap<String, [u64; 9]>,
}
fn roles(lineup: &real::Lineup) -> BTreeSet<String> {
    let state = lineup.initial();
    state
        .teams
        .iter()
        .flat_map(|t| {
            t.batting_order
                .iter()
                .map(|id| format!("batter:{id}"))
                .chain(std::iter::once(format!("pitcher:{}", t.pitcher)))
        })
        .collect()
}
impl Ratings {
    pub fn read_tsv(
        text: &str,
        base: &preview::Baseline,
        lineup: &real::Lineup,
    ) -> Result<Self, String> {
        if base.counts != COUNTS
            || *base.probabilities() != *preview::Baseline::from_counts(COUNTS)?.probabilities()
        {
            return Err("rating league baseline mismatch".into());
        }
        if text
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t')
        {
            return Err("invalid rating control".into());
        }
        let mut lines = text.lines();
        if lines.next() != Some("key\tvalue") {
            return Err("expected rating key,value TSV header".into());
        }
        let mut fields = BTreeMap::new();
        for line in lines {
            let (key, value) = line.split_once('\t').ok_or("invalid rating row")?;
            if key.is_empty()
                || value.is_empty()
                || value.contains('\t')
                || fields.insert(key, value).is_some()
            {
                return Err("duplicate/invalid rating field".into());
            }
        }
        for (key, value) in [
            ("format", "player-proxy-counts-v1"),
            ("model", MODEL_VERSION),
            ("opportunity_report_sha256", REPORT_SHA),
            ("lineup_report_sha256", real::REPORT_SHA),
            ("snapshot_sha256", real::SNAPSHOT_SHA),
            ("supplement_sha256", real::SUPPLEMENT_SHA),
            ("batter_kappa", "200"),
            ("pitcher_kappa", "200"),
            (
                "league_counts",
                "42823,15142,1984,25947,8531,785,6776,1470,82245",
            ),
            ("attribution", joint::ATTRIBUTION),
            ("opportunity_basis", BASIS),
        ] {
            if fields.remove(key) != Some(value) {
                return Err(format!("unsupported rating {key}"));
            }
        }
        if fields
            .keys()
            .map(|s| s.to_string())
            .collect::<BTreeSet<_>>()
            != roles(lineup)
        {
            return Err("rating role/identity set differs from starting lineup".into());
        }
        let mut counts = BTreeMap::new();
        for (key, value) in fields {
            let values = value
                .split(',')
                .map(|s| {
                    let n = s.parse::<u64>().map_err(|_| "invalid rating count")?;
                    if n.to_string() != s {
                        return Err("noncanonical rating count");
                    }
                    Ok(n)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let array: [u64; 9] = values
                .try_into()
                .map_err(|_| "expected nine rating counts")?;
            if array.iter().zip(COUNTS).any(|(n, max)| *n > max) {
                return Err("rating count exceeds source cohort".into());
            }
            exposure(array)?;
            counts.insert(key.to_string(), array);
        }
        Ok(Self { counts })
    }
    pub fn probabilities(&self, state: &GameState, league: [f64; 9]) -> Result<[f64; 9], String> {
        let offense = state.half.offense();
        let batter = state.teams[offense]
            .batting_order
            .get(state.teams[offense].next_batter as usize)
            .ok_or("invalid rating batting cursor")?;
        let pitcher = &state.teams[1 - offense].pitcher;
        let b = *self
            .counts
            .get(&format!("batter:{batter}"))
            .ok_or("missing current batter rating")?;
        let p = *self
            .counts
            .get(&format!("pitcher:{pitcher}"))
            .ok_or("missing current pitcher rating")?;
        matchup(b, p, league)
    }
    pub fn game(
        &self,
        seed: u64,
        base: &preview::Baseline,
        model: &joint::JointModel,
        lineup: &real::Lineup,
    ) -> Result<Vec<EventTransition>, String> {
        preview::game_from(seed, lineup.initial(), |state, rng| {
            let adjusted =
                model.with_marginals(self.probabilities(state, *base.probabilities())?)?;
            let sample = adjusted.sample(rng);
            Ok((sample.outcome, lineup.event_for(state, sample)?))
        })
    }
    pub fn write_preamble(
        &self,
        output: &mut dyn Write,
        lineup: &real::Lineup,
        base: &preview::Baseline,
    ) -> io::Result<()> {
        writeln!(output, "RATINGS FORMAT: player-proxy-counts-v1")?;
        writeln!(output, "Rating prior strength: batter200 pitcher200")?;
        writeln!(output, "Opportunity report SHA256: {REPORT_SHA}")?;
        writeln!(output, "Rating basis: {BASIS}")?;
        for (key, counts) in &self.counts {
            let (role, id) = key.split_once(':').expect("validated role key");
            let n = exposure(*counts).map_err(io::Error::other)?;
            let mean = posterior(*counts, *base.probabilities()).map_err(io::Error::other)?;
            let sd = mean.map(|p| (p * (1.0 - p) / (n as f64 + KAPPA + 1.0)).sqrt());
            writeln!(
                output,
                "RATING {role} {id} | N: {n} | Weight: {} | Mean: {mean:?} | SD: {sd:?} | Counts: {counts:?}",
                n as f64 / (n as f64 + KAPPA)
            )?;
        }
        for (side, name) in [(0, "Away"), (1, "Home")] {
            let mut state = lineup.initial();
            state.half = if side == 0 {
                bbsim_engine::Half::Top
            } else {
                bbsim_engine::Half::Bottom
            };
            for slot in 0..9 {
                state.teams[side].next_batter = slot;
                let p = self
                    .probabilities(&state, *base.probabilities())
                    .map_err(io::Error::other)?;
                writeln!(
                    output,
                    "MATCHUP {name} {} {} {}: {p:?}",
                    slot + 1,
                    state.teams[side].batting_order[slot as usize],
                    state.teams[1 - side].pitcher
                )?;
            }
        }
        writeln!(output,"Rating limits: selected-season hindsight reconstruction, not calibrated or predictive. Fixed provisional kappa200, not tuned or proven best. Independent marginals retain opponent/park/defense confounding; no handedness effects. Component SDs are assumed Dirichlet uncertainty, not validated ability intervals or propagated matchup uncertainty. Counts are participant proxies, not official PA/BF. Same closed roster/starter-all-game/coarse contact limitations; no new fielding roll.")
    }
}
/// Preserve known option values before removing the standalone ratings opt-in.
pub fn split_options(args: &[String]) -> Result<(Vec<String>, Option<String>), String> {
    let mut old = Vec::new();
    let mut path = None;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if arg == "--ratings" {
            if path.is_some() {
                return Err("repeated --ratings".into());
            }
            let value = args.next().ok_or("missing ratings path")?;
            if value.is_empty() || value.starts_with("--") {
                return Err("missing ratings path".into());
            }
            path = Some(value.clone());
        } else {
            old.push(arg.clone());
            if matches!(
                arg.as_str(),
                "--baseline" | "--seed" | "--contact-counts" | "--lineup"
            ) {
                if let Some(value) = args.next() {
                    old.push(value.clone());
                }
            }
        }
    }
    Ok((old, path))
}
