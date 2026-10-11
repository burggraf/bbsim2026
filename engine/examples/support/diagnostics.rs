//! Native preview validation harness, not an outcome-generation component.
use super::preview::{game, Baseline, Rng, ALL};
use bbsim_engine::{accounting::derive_accounting, EventTransition};
pub fn verify(baseline: &Baseline, games: u64) -> Result<String, String> {
    if games == 0 {
        return Err("diagnostics need games".into());
    }
    let mut sum_runs = 0u64;
    let mut sum_events = 0usize;
    let mut max_events = 0usize;
    for seed in 0..games {
        let log = game(seed, baseline)?;
        for pair in log.windows(2) {
            pair[1]
                .follows(&pair[0])
                .map_err(|e| format!("seed{seed}: {e:?}"))?;
        }
        for t in &log {
            if EventTransition::from_bytes(&t.to_bytes())
                .map_err(|e| format!("seed{seed}: {e:?}"))?
                != *t
            {
                return Err(format!("seed{seed}: codec mismatch"));
            }
        }
        let stats = derive_accounting(&log).map_err(|e| format!("seed{seed}: {e:?}"))?;
        if !stats.complete_game
            || stats.runs.map(|n| n as u16) != log.last().ok_or("empty game")?.after().score
        {
            return Err(format!("seed{seed}: incomplete/run mismatch"));
        }
        for side in 0..2 {
            let batters = &stats.batters[side];
            let pitchers = &stats.pitchers[1 - side];
            if batters.values().map(|b| b.plate_appearances).sum::<u32>()
                != pitchers.values().map(|p| p.batters_faced).sum::<u32>()
                || batters.values().map(|b| b.credited_runs).sum::<u32>() != stats.runs[side]
                || pitchers.values().map(|p| p.runs_allowed).sum::<u32>() != stats.runs[side]
                || pitchers.values().map(|p| p.outs_recorded).sum::<u32>()
                    != stats.fielders[1 - side]
                        .values()
                        .map(|f| f.credited_putouts)
                        .sum::<u32>()
                || batters.values().any(|b| {
                    b.credited_hit_breakdown.as_ref().map(|h| h.hits()) != Some(b.credited_hits)
                        || b.at_bats
                            != Some(
                                b.plate_appearances - b.credited_walks - b.credited_hit_by_pitch,
                            )
                })
            {
                return Err(format!("seed{seed}: conservation failed"));
            }
        }
        sum_runs += stats.runs.iter().map(|n| u64::from(*n)).sum::<u64>();
        sum_events += log.len();
        max_events = max_events.max(log.len());
    }
    let mut rng = Rng::new(19);
    let mut draws = [0u32; 9];
    for _ in 0..100_000 {
        let outcome = baseline.sample(&mut rng);
        let i = ALL
            .iter()
            .position(|o| *o == outcome)
            .ok_or("unknown sample")?;
        draws[i] += 1;
    }
    for (i, p) in baseline.probabilities().iter().enumerate() {
        let observed = f64::from(draws[i]) / 100_000.0;
        if (observed - p).abs() > 6.0 * (p * (1.0 - p) / 100_000.0).sqrt() + 1.0 / 100_000.0 {
            return Err(format!("sampler diagnostic failed: {i}"));
        }
    }
    Ok(format!("Verified {games} seeds: termination/continuity/codec/available-stat conservation.\nSampler:100000 draws passed fixed six-sigma diagnostic; draws={draws:?}.\nExploratory only: runs/game={:.4}, PA-events/game={:.4}, max-events={max_events}; not calibration or complete boxes.",sum_runs as f64/games as f64,sum_events as f64/games as f64))
}
