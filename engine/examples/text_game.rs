//! Offline development command for the connected text-game recovery model.
// These shared legacy modules also contain the unchanged preview command paths.
#[path = "support/calibrated_ratings.rs"]
mod calibrated;
#[allow(dead_code)]
#[path = "support/player_calibration.rs"]
mod calibration;
#[path = "support/full_game.rs"]
mod full_game;
#[path = "support/full_manager.rs"]
mod full_manager;
#[path = "support/full_simulation.rs"]
mod full_simulation;
#[path = "support/full_transcript.rs"]
mod full_transcript;
#[allow(dead_code)]
#[path = "support/joint_contact.rs"]
mod joint;
#[allow(dead_code)]
#[path = "support/preview.rs"]
mod preview;
#[allow(dead_code)]
#[path = "support/player_ratings.rs"]
mod ratings;
#[allow(dead_code)]
#[path = "support/real_lineup.rs"]
mod real;
#[path = "support/transcript.rs"]
mod transcript;
#[allow(dead_code)]
#[path = "support/player_calibration_tree.rs"]
mod tree;
use std::{
    collections::BTreeMap,
    io::{self, Write},
};
const USAGE: &str = "text_game --baseline COUNTS.csv --contact-counts CONTACT.csv --lineup LINEUP.tsv --cast CAST.tsv [--seed N] [--calibrated-ratings TREE.tsv]";
fn parse(args: &[String]) -> Result<Option<BTreeMap<String, String>>, String> {
    if args == ["--help"] {
        return Ok(None);
    }
    let mut fields = BTreeMap::new();
    let mut pairs = args.chunks_exact(2);
    for pair in &mut pairs {
        if ![
            "--baseline",
            "--contact-counts",
            "--lineup",
            "--cast",
            "--seed",
            "--calibrated-ratings",
        ]
        .contains(&pair[0].as_str())
            || pair[1].is_empty()
            || fields.insert(pair[0].clone(), pair[1].clone()).is_some()
        {
            return Err(format!(
                "unknown, empty or duplicate option; usage: {USAGE}"
            ));
        }
    }
    if !pairs.remainder().is_empty()
        || ["--baseline", "--contact-counts", "--lineup", "--cast"]
            .iter()
            .any(|key| !fields.contains_key(*key))
    {
        return Err(format!("missing input/value; usage: {USAGE}"));
    }
    let seed = fields.entry("--seed".into()).or_insert_with(|| "1".into());
    seed.parse::<u64>()
        .map_err(|_| "seed must be an unsigned 64-bit integer")?;
    Ok(Some(fields))
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(options) = parse(&std::env::args().skip(1).collect::<Vec<_>>())? else {
        println!("{USAGE}");
        return Ok(());
    };
    let baseline_text = std::fs::read_to_string(&options["--baseline"])?;
    let baseline = preview::Baseline::read_csv(&baseline_text)?;
    let contact_text = std::fs::read_to_string(&options["--contact-counts"])?;
    joint::JointModel::read_csv(&contact_text, &baseline)?;
    let exclusion = |key: &str| -> Result<u64, String> {
        baseline_text
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{key},")))
            .ok_or("missing baseline exclusion")?
            .parse()
            .map_err(|_| "invalid baseline exclusion".into())
    };
    if exclusion("excluded_interference")? != 61 || exclusion("excluded_ibb")? != 753 {
        return Err("baseline exclusion/cohort differs from pinned contact source".into());
    }
    let mut counts = [0; 10];
    counts[..9].copy_from_slice(&baseline.counts);
    counts[9] = 61;
    let model = full_simulation::Model::new(counts, [17188, 19528, 5379, 536, 39614])?;
    let lineup = real::Lineup::read_tsv(&std::fs::read_to_string(&options["--lineup"])?)?;
    let cast =
        full_manager::Cast::read_complete_tsv(&std::fs::read_to_string(&options["--cast"])?)?;
    let initial = lineup.initial();
    let required = initial
        .teams
        .iter()
        .flat_map(|team| team.batting_order.iter().map(|id| format!("batter:{id}")))
        .chain(cast.names().keys().map(|id| format!("pitcher:{id}")))
        .collect();
    let ratings =
        ratings::Ratings::from_counts(cast.ability_counts().clone(), &baseline, &required)?;
    let calibrated = options
        .get("--calibrated-ratings")
        .map(|path| {
            calibrated::Ratings::read_tsv(
                &std::fs::read_to_string(path).map_err(|e| e.to_string())?,
                cast.ability_counts(),
                &baseline,
                &required,
            )
        })
        .transpose()?;
    let positions = lineup.positions();
    let labels = ["P", "C", "1B", "2B", "3B", "SS", "LF", "CF", "RF"];
    let fielders = initial.teams.each_ref().map(|team| {
        std::array::from_fn(|i| {
            team.batting_order
                .iter()
                .chain(std::iter::once(&team.pitcher))
                .find(|id| positions.get(*id) == Some(&labels[i]))
                .expect("validated starting position")
                .clone()
        })
    });
    let mut names = lineup.names();
    for (id, name) in cast.names() {
        if names.get(&id).is_some_and(|old| old != &name) {
            return Err("lineup/cast display-name mismatch".into());
        }
        names.insert(id, name);
    }
    let seed = options["--seed"].parse::<u64>()?;
    let full_simulation::Simulation { steps, draws } =
        full_simulation::simulate_with_probabilities(
            &initial,
            &fielders,
            &model,
            Some(&cast),
            seed,
            |state| {
                let probabilities = match &calibrated {
                    Some(r) => r.probabilities(state)?,
                    None => ratings.probabilities(state, *baseline.probabilities())?,
                };
                Ok(Some(model.player_probabilities(probabilities)?))
            },
        )?;
    let draws: BTreeMap<_, _> = draws.into_iter().collect();
    let mut output = io::stdout().lock();
    writeln!(
        output,
        "DEVELOPMENT TEXT GAME — INCOMPLETE RECOVERY MODEL, NOT ACCEPTED OR CALIBRATED"
    )?;
    let label = if calibrated.is_some() {
        calibrated::MODEL_VERSION
    } else {
        "individual-text-game-proxy-v1-incomplete"
    };
    writeln!(
        output,
        "Seed: {seed} | Model: {label} | RNG: {}",
        preview::RNG_VERSION
    )?;
    writeln!(output,"Seattle Mariners at Oakland Athletics; recorded 2019-03-20 starting personnel, not the historical result.")?;
    if calibrated.is_some() {
        writeln!(output,"Individual batter/current-pitcher probabilities from an explicit opponent-adjusted outcome-tree development artifact. Full2019 hindsight refit, outcome-specific strengths; one parent draw. Reused retrospective test benchmark, not fresh independent validation, official PA/BF, true ability or forecasting; no handedness adjustment or propagated ability uncertainty.")?;
    } else {
        writeln!(output,"Individual batter/current-pitcher PA probabilities from cast ability counts; model {} with fixed kappa200. Selected-season hindsight participant proxies, not official PA/BF, calibrated player fidelity or forecasting; no handedness adjustment or propagated rating uncertainty.", ratings::MODEL_VERSION)?;
    }
    writeln!(output,"Pooled reference counts: {counts:?}; interference alone retains pooled parent mass. Generated contact geometry/handling, runner/error/steal conditionals remain shared provisional priors; not observed mechanisms or validated individual speed/arm/fielding.")?;
    writeln!(output,"PA MODEL records the actual consumed vector, raw RNG word and selected parent (K, BB, HBP, 1B, 2B, 3B, HR, ROE, contact, interference). Tactical IBB overrides and runner-only events have no PA draw. Credited walkoff hit value may be shorter than the selected parent.")?;
    writeln!(output,"Recorded nonstarters assumed available at start; not verified eligibility. Starter/relief targets27/6 PA; boundary changes from seventh; exhausted cast keeps last pitcher.")?;
    writeln!(output,"KNOWN MISSING GENERATION: automatic uncaught-K out plus same-delivery runner advance, pinch hitting/running/defensive replacements, ground-rule doubles, inside-park home runs and appeal double-offs. Joint hits/runner tags and eligible recovered-K advances use provisional priors. No complete major-event coverage claim.")?;
    writeln!(output, "Source witness OAK201903200; accepted 2019 snapshot d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528; supplement 554632467416eef202b7a55cadad2d09e5355ce98c83868b26b5267f98852b82.")?;
    writeln!(output, "INTEGRITY CHECKS: contiguous canonical log; event-summed runs/PA/BF/outs/cursors and observed PO/A/E/PB reconciliation. Not complete official accounting.")?;
    writeln!(output, "Engine {} | Mechanics {} | 2019 pilot rules; incomplete official accounting; individual participant-proxy means, not validated player abilities.",env!("CARGO_PKG_VERSION"), full_simulation::MODEL_VERSION)?;
    writeln!(output, "{}\n", joint::ATTRIBUTION)?;
    for (i, step) in steps.iter().enumerate() {
        if let Some(draw) = draws.get(&(i + 1)) {
            let event = step.transition().event();
            let batter = event
                .participants
                .batter
                .as_ref()
                .ok_or("ordinary PA lacks batter")?;
            let pitcher = &event.participants.pitcher;
            writeln!(output, "PA MODEL {} | Batter: {} ({batter}) | Pitcher: {} ({pitcher}) | Word: {} | Parent: {} | Probabilities: {:?}",i+1,names.get(batter).ok_or("missing rated batter name")?,names.get(pitcher).ok_or("missing rated pitcher name")?,draw.word,draw.parent,draw.probabilities)?;
        }
        full_transcript::write_step(
            &mut output,
            i + 1,
            step,
            &["Seattle".into(), "Oakland".into()],
            &names,
        )?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::parse;
    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| s.to_string()).collect()
    }
    #[test]
    fn options_require_all_offline_inputs_and_a_valid_seed_without_duplicate_or_unknown_flags() {
        let good = args(&[
            "--baseline",
            "base.csv",
            "--contact-counts",
            "contact.csv",
            "--lineup",
            "lineup.tsv",
            "--cast",
            "cast.tsv",
            "--seed",
            "3",
        ]);
        assert_eq!(parse(&good).unwrap().unwrap()["--seed"], "3");
        let calibrated = [good.clone(), args(&["--calibrated-ratings", "tree.tsv"])].concat();
        assert_eq!(
            parse(&calibrated).unwrap().unwrap()["--calibrated-ratings"],
            "tree.tsv"
        );
        assert!(
            parse(&[calibrated, args(&["--calibrated-ratings", "second.tsv"])].concat()).is_err()
        );
        assert!(parse(&args(&["--help"])).unwrap().is_none());
        for bad in [
            args(&[]),
            args(&["--seed", "1"]),
            [good.clone(), args(&["--seed", "2"])].concat(),
            [good.clone(), args(&["--other", "x"])].concat(),
            {
                let mut a = good.clone();
                a.pop();
                a
            },
            {
                let mut a = good.clone();
                *a.last_mut().unwrap() = "-1".into();
                a
            },
        ] {
            assert!(parse(&bad).is_err());
        }
    }
}
