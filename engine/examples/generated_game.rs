#[path = "support/preview_cli.rs"]
mod cli;
#[path = "support/presentation.rs"]
mod presentation;
#[path = "support/preview.rs"]
mod preview;
use std::io::{self, Write};
fn main() -> io::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let options = match cli::parse(&args) {
        Ok(Some(options)) => options,
        Ok(None) => {
            println!("Experimental generated game, not a calibrated MLB matchup.\nUsage: generated_game --baseline COUNTS.csv [--seed N] [--autoplay]\nDefault seed1. Enter advances, q/EOF stops; rerun same counts/seed to restart.");
            return Ok(());
        }
        Err(error) => {
            eprintln!(
                "{error}\nUsage: generated_game --baseline COUNTS.csv [--seed N] [--autoplay]"
            );
            std::process::exit(2);
        }
    };
    let baseline = preview::Baseline::read_csv(&std::fs::read_to_string(&options.baseline)?)
        .map_err(io::Error::other)?;
    let log = preview::game(options.seed, &baseline).map_err(io::Error::other)?;
    let mut output = io::stdout().lock();
    writeln!(output,"EXPERIMENTAL GENERATED GAME — fictional equal teams, pooled 2019 baseline; not calibrated.")?;
    writeln!(
        output,
        "Seed: {} | Model: {} | Policy: {} | RNG: {}",
        options.seed,
        preview::MODEL_VERSION,
        preview::POLICY_VERSION,
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
        baseline.probabilities()
    )?;
    writeln!(output,"Coarse ground/contact outs and station advancement; no player ratings, fatigue, DP, tactics or runner-only mechanisms yet.\n")?;
    presentation::present(&log, &mut io::stdin().lock(), &mut output, options.autoplay)
}
