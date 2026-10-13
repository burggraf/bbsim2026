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
use std::io::{self, Write};
fn option_error(error: String) -> ! {
    eprintln!(
        "{error}\nUsage: generated_game --baseline COUNTS.csv [--seed N] [--autoplay] [--trace]"
    );
    std::process::exit(2);
}
fn main() -> io::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let (args, contact_path) =
        joint::split_options(&args).unwrap_or_else(|error| option_error(error));
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
    let (log, model_version, policy_version) = if let Some(model) = &joint {
        (
            joint::game(options.seed, model),
            joint::MODEL_VERSION,
            joint::POLICY_VERSION,
        )
    } else {
        (
            preview::game(options.seed, &baseline),
            preview::MODEL_VERSION,
            preview::POLICY_VERSION,
        )
    };
    let log = log.map_err(io::Error::other)?;
    let mut output = io::stdout().lock();
    writeln!(output,"EXPERIMENTAL GENERATED GAME — fictional equal teams, pooled 2019 baseline; not calibrated.")?;
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
        "Posterior probabilities: {:?}",
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
        writeln!(output, "Joint CDF edges: {:?}", model.boundaries())?;
        writeln!(output, "{}", joint::ATTRIBUTION)?;
        writeln!(output,"Generic narration; inspect trace for typed air/residual facts. No player ratings, range/arm, tag-ups, fatigue, DP, tactics or runner-only mechanisms yet.\n")?;
    } else {
        writeln!(output,"Coarse ground/contact outs and station advancement; no player ratings, fatigue, DP, tactics or runner-only mechanisms yet.\n")?;
    }
    if options.trace {
        game_trace::write(&log, &mut output)?;
        game_trace::write_commentary(&log, &mut output)?;
    }
    presentation::present(&log, &mut io::stdin().lock(), &mut output, options.autoplay)
}
