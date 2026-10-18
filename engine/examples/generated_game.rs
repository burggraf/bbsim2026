#[path = "support/preview_cli.rs"]
mod cli;
#[path = "support/game_trace.rs"]
mod game_trace;
#[path = "support/joint_contact.rs"]
mod joint;
#[path = "support/presentation.rs"]
mod presentation;
#[path = "support/preview.rs"]
mod preview;
#[path = "support/player_ratings.rs"]
mod ratings;
#[path = "support/real_lineup.rs"]
mod real;
#[allow(dead_code)]
#[path = "support/season_pool.rs"]
mod season_pool;
use std::io::{self, Write};
fn option_error(error: String) -> ! {
    eprintln!(
        "{error}\nUsage: generated_game --baseline COUNTS.csv [--seed N] [--autoplay] [--trace]"
    );
    std::process::exit(2);
}
fn main() -> io::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let (args, ratings_path) =
        ratings::split_options(&args).unwrap_or_else(|error| option_error(error));
    let (args, lineup_path) =
        real::split_options(&args).unwrap_or_else(|error| option_error(error));
    let (args, contact_path) =
        joint::split_options(&args).unwrap_or_else(|error| option_error(error));
    if ratings_path.is_some() && lineup_path.is_none() {
        option_error("--ratings requires --lineup and --contact-counts".into());
    }
    if lineup_path.is_some() && contact_path.is_none() {
        option_error("--lineup requires --contact-counts".into());
    }
    let options = match cli::parse(&args) {
        Ok(Some(options)) => options,
        Ok(None) => {
            println!("Experimental generated game, not a calibrated MLB matchup.\nUsage: generated_game --baseline COUNTS.csv [--seed N] [--autoplay] [--trace]\nDefault seed1. Enter advances, q/EOF stops; rerun same counts/seed to restart.");
            return Ok(());
        }
        Err(error) => option_error(error),
    };
    let baseline = preview::Baseline::read_csv(&std::fs::read_to_string(&options.baseline)?)
        .map_err(io::Error::other)?;
    let joint = contact_path
        .map(|path| {
            joint::JointModel::read_csv(&std::fs::read_to_string(path)?, &baseline)
                .map_err(io::Error::other)
        })
        .transpose()?;
    let lineup = lineup_path
        .map(|path| {
            real::Lineup::read_tsv(&std::fs::read_to_string(path)?).map_err(io::Error::other)
        })
        .transpose()?;
    let ratings = ratings_path
        .map(|path| {
            ratings::Ratings::read_tsv(
                &std::fs::read_to_string(path)?,
                &baseline,
                lineup.as_ref().expect("ratings require lineup"),
            )
            .map_err(io::Error::other)
        })
        .transpose()?;
    let (log, model_version, policy_version) = if let Some(model) = &joint {
        if let (Some(ratings), Some(lineup)) = (&ratings, &lineup) {
            (
                ratings.game(options.seed, &baseline, model, lineup),
                ratings::MODEL_VERSION,
                real::POLICY_VERSION,
            )
        } else if let Some(lineup) = &lineup {
            (
                lineup.game(options.seed, model),
                joint::MODEL_VERSION,
                real::POLICY_VERSION,
            )
        } else {
            (
                joint::game(options.seed, model),
                joint::MODEL_VERSION,
                joint::POLICY_VERSION,
            )
        }
    } else {
        (
            preview::game(options.seed, &baseline),
            preview::MODEL_VERSION,
            preview::POLICY_VERSION,
        )
    };
    let log = log.map_err(io::Error::other)?;
    let mut output = io::stdout().lock();
    if ratings.is_some() {
        writeln!(output,"EXPERIMENTAL GENERATED GAME — recorded Seattle/Oakland starting personnel, shrunk participant-proxy player probabilities; hindsight, not calibrated, predictive or the historical result.")?;
    } else if lineup.is_some() {
        writeln!(output,"EXPERIMENTAL GENERATED GAME — recorded Seattle/Oakland starting personnel, pooled 2019 probabilities; not calibrated or the historical result.")?;
    } else {
        writeln!(output,"EXPERIMENTAL GENERATED GAME — fictional equal teams, pooled 2019 baseline; not calibrated.")?;
    }
    writeln!(
        output,
        "Seed: {} | Model: {} | Policy: {} | RNG: {}",
        options.seed,
        model_version,
        policy_version,
        preview::RNG_VERSION
    )?;
    writeln!(
        output,
        "Counts K/BB/HBP/1B/2B/3B/HR/ROE/contact: {:?}",
        baseline.counts
    )?;
    writeln!(
        output,
        "{}: {:?}",
        if ratings.is_some() {
            "League prior probabilities"
        } else {
            "Posterior probabilities"
        },
        joint
            .as_ref()
            .map_or(*baseline.probabilities(), joint::JointModel::marginals)
    )?;
    if let Some(model) = &joint {
        writeln!(output,"OPT-IN EXPERIMENT — source-shape mixture; synthetic held retirements; residual partial coverage, not calibration.")?;
        writeln!(
            output,
            "Conditional proxy probabilities Ground/Fly/LineDrive/PopUp/residual: {:?}",
            model.conditionals()
        )?;
        writeln!(
            output,
            "{}: {:?}",
            if ratings.is_some() {
                "Reference pooled joint CDF edges"
            } else {
                "Joint CDF edges"
            },
            model.boundaries()
        )?;
        writeln!(output, "{}", joint::ATTRIBUTION)?;
        if ratings.is_some() {
            writeln!(output,"Inspect trace for typed positional air/residual facts. Player effects are provisional participant proxies; no range/arm, tag-ups, fatigue, DP, tactics or runner-only mechanisms.\n")?;
        } else {
            writeln!(output,"Generic narration; inspect trace for typed air/residual facts. No player ratings, range/arm, tag-ups, fatigue, DP, tactics or runner-only mechanisms yet.\n")?;
        }
    } else {
        writeln!(output,"Coarse ground/contact outs and station advancement; no player ratings, fatigue, DP, tactics or runner-only mechanisms yet.\n")?;
    }
    if let Some(lineup) = &lineup {
        if ratings.is_some() {
            lineup.write_preamble_with_rates(&mut output, true)?;
        } else {
            lineup.write_preamble(&mut output)?;
        }
    }
    if let (Some(ratings), Some(lineup)) = (&ratings, &lineup) {
        ratings.write_preamble(&mut output, lineup, &baseline)?;
    }
    if options.trace {
        game_trace::write(&log, &mut output)?;
        if let Some(lineup) = &lineup {
            game_trace::write_commentary_with_positions(
                &log,
                &mut output,
                &lineup.names(),
                &lineup.positions(),
            )?;
        } else {
            game_trace::write_commentary(&log, &mut output)?;
        }
    }
    presentation::present(&log, &mut io::stdin().lock(), &mut output, options.autoplay)
}
