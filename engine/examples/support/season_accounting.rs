//! Machine-readable canonical counts; unresolved coverage stays unknown.
use crate::season_pool;
use bbsim_engine::accounting::GameAccounting;
use std::io::{self, Write};

fn row(
    output: &mut dyn Write,
    role: &str,
    side: &str,
    id: &str,
    cells: &[Option<u32>],
) -> io::Result<()> {
    write!(output, "{role}\t{side}\t{id}")?;
    for value in cells {
        match value {
            Some(n) => write!(output, "\t{n}")?,
            None => write!(output, "\t?")?,
        }
    }
    writeln!(output)
}

pub fn write(
    output: &mut dyn Write,
    stats: &GameAccounting,
    game: &str,
    seed: u64,
    teams: &[String; 2],
) -> io::Result<()> {
    write_with_personnel(output, stats, game, seed, teams, None)
}

pub fn write_with_personnel(
    output: &mut dyn Write,
    stats: &GameAccounting,
    game: &str,
    seed: u64,
    teams: &[String; 2],
    ph: Option<(&str, &str, &str, &str)>,
) -> io::Result<()> {
    write_inner(output, stats, game, seed, teams, ph, None)
}

#[allow(dead_code)] // Only season-pool CLI inputs select this explicit wire format.
pub fn write_managed(
    output: &mut dyn Write,
    stats: &GameAccounting,
    game: &str,
    seed: u64,
    teams: &[String; 2],
    ratings: &str,
    pinch_hitting: bool,
) -> io::Result<()> {
    write_inner(
        output,
        stats,
        game,
        seed,
        teams,
        None,
        Some((
            ratings,
            pinch_hitting,
            season_pool::POLICY,
            season_pool::MANAGER,
        )),
    )
}

#[allow(dead_code)]
pub fn write_managed_v2(
    output: &mut dyn Write,
    stats: &GameAccounting,
    game: &str,
    seed: u64,
    teams: &[String; 2],
    ratings: &str,
    pinch_hitting: bool,
) -> io::Result<()> {
    write_inner(
        output,
        stats,
        game,
        seed,
        teams,
        None,
        Some((
            ratings,
            pinch_hitting,
            season_pool::PITCHER_USAGE_POLICY_V2,
            season_pool::MANAGER_V2,
        )),
    )
}

fn write_inner(
    output: &mut dyn Write,
    stats: &GameAccounting,
    game: &str,
    seed: u64,
    teams: &[String; 2],
    ph: Option<(&str, &str, &str, &str)>,
    managed: Option<(&str, bool, &str, &str)>,
) -> io::Result<()> {
    if !stats.complete_game {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "accounting requires a complete game",
        ));
    }
    let format = if let Some((_, _, policy, _)) = managed {
        if policy == season_pool::PITCHER_USAGE_POLICY_V2 {
            "controlled-accounting-managed-v2"
        } else {
            "controlled-accounting-managed-v1"
        }
    } else if ph.is_some() {
        "controlled-accounting-ph-v1"
    } else {
        "controlled-accounting-v1"
    };
    writeln!(output, "format\t{format}\ngame\t{game}\nseed\t{seed}")?;
    if let Some((policy, availability, defense, _ratings)) = ph {
        writeln!(
            output,
            "pinch_hitting\t{policy}\navailability_basis\t{availability}\ndefense_basis\t{defense}"
        )?;
    }
    if let Some((_, ph, policy, _)) = managed {
        writeln!(
            output,
            "season_policy\t{}\navailability_basis\t{}\ndefense_basis\t{}\npinch_hitting\t{}",
            policy,
            season_pool::AVAILABILITY,
            season_pool::DEFENSE_BASIS,
            if ph {
                season_pool::PINCH_HITTING
            } else {
                "disabled"
            }
        )?;
    }
    for (key, value) in [
        ("engine", env!("CARGO_PKG_VERSION")),
        ("mechanics", super::full_simulation::MODEL_VERSION),
        (
            "ratings",
            managed
                .map(|(model, _, _, _)| model)
                .or_else(|| ph.map(|(_, _, _, model)| model))
                .unwrap_or(super::ratings::MODEL_VERSION),
        ),
        (
            "manager",
            if let Some((_, _, _, manager)) = managed {
                manager
            } else {
                super::full_manager::POLICY_VERSION
            },
        ),
        ("rng", super::preview::RNG_VERSION),
        ("rules", "2019"),
    ] {
        writeln!(output, "{key}\t{value}")?;
    }
    for (side, label) in ["Away", "Home"].iter().enumerate() {
        writeln!(
            output,
            "team\t{label}\t{}\t{}",
            teams[side], stats.runs[side]
        )?;
        for (id, b) in &stats.batters[side] {
            let h = b.credited_hit_breakdown.as_ref();
            row(
                output,
                "batter",
                label,
                id,
                &[
                    Some(b.plate_appearances),
                    b.at_bats,
                    Some(b.credited_runs),
                    b.classified_hits,
                    b.classified_rbi,
                    Some(b.credited_walks),
                    Some(b.credited_strikeouts),
                    h.map(|h| h.singles),
                    h.map(|h| h.doubles),
                    h.map(|h| h.triples),
                    h.map(|h| h.home_runs),
                    b.credited_intentional_walks,
                    Some(b.credited_hit_by_pitch),
                    Some(b.credited_sacrifice_bunts),
                    Some(b.credited_sacrifice_flies),
                    Some(b.credited_grounded_into_double_plays),
                    Some(b.credited_steals),
                    Some(b.credited_caught_stealing),
                    Some(b.credited_interference_awards),
                ],
            )?;
        }
        for (id, p) in &stats.pitchers[side] {
            let h = p.credited_hit_breakdown_allowed.as_ref();
            row(
                output,
                "pitcher",
                label,
                id,
                &[
                    Some(p.batters_faced),
                    Some(p.outs_recorded),
                    p.at_bats_against,
                    Some(p.runs_allowed),
                    p.earned_runs,
                    p.classified_hits_allowed,
                    Some(p.credited_walks_allowed),
                    Some(p.credited_strikeouts),
                    h.map(|h| h.singles),
                    h.map(|h| h.doubles),
                    h.map(|h| h.triples),
                    h.map(|h| h.home_runs),
                    p.credited_intentional_walks_allowed,
                    Some(p.credited_hit_batters),
                    Some(p.credited_sacrifice_bunts),
                    Some(p.credited_sacrifice_flies),
                    Some(p.credited_wild_pitches),
                    Some(p.credited_balks),
                ],
            )?;
        }
        for (id, f) in &stats.classified_fielding[side] {
            row(
                output,
                "fielder",
                label,
                id,
                &[
                    Some(f.observed_events),
                    f.putouts,
                    f.assists,
                    f.errors,
                    f.passed_balls,
                ],
            )?;
        }
    }
    Ok(())
}
