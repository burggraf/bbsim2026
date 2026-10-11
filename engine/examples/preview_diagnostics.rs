#[path = "support/diagnostics.rs"]
mod diagnostics;
#[path = "support/preview.rs"]
mod preview;
fn main() -> Result<(), String> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 1 {
        return Err("Usage: preview_diagnostics COUNTS.csv (2000 fixed seeds)".into());
    }
    let baseline = preview::Baseline::read_csv(
        &std::fs::read_to_string(&args[0]).map_err(|e| e.to_string())?,
    )?;
    println!(
        "{} | {} | {}",
        preview::MODEL_VERSION,
        preview::POLICY_VERSION,
        preview::RNG_VERSION
    );
    println!(
        "Counts: {:?}; probabilities: {:?}",
        baseline.counts,
        baseline.probabilities()
    );
    println!("{}", diagnostics::verify(&baseline, 2000)?);
    Ok(())
}
