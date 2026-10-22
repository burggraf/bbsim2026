//! Offline development command for the connected text-game recovery model.
#[path = "support/pitcher_entry_v2.rs"]
mod pitcher_entry_v2;
#[allow(dead_code)]
#[path = "support/pitcher_policy_v2.rs"]
mod pitcher_policy_v2;
#[allow(dead_code)]
#[path = "support/season_pool.rs"]
mod season_pool;
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
#[path = "support/season_accounting.rs"]
mod season_accounting;
#[path = "support/transcript.rs"]
mod transcript;
#[allow(dead_code)]
#[path = "support/player_calibration_tree.rs"]
mod tree;
use std::{
    collections::BTreeMap,
    fmt::Write as FmtWrite,
    fs::OpenOptions,
    io::{self, Read, Write},
};
const USAGE: &str = "text_game --baseline COUNTS.csv --contact-counts CONTACT.csv --lineup LINEUP.tsv --cast CAST.tsv [--seed N] [--bench BENCH.tsv] [--calibrated-ratings TREE.tsv] [--format detailed|radio|accounting] [--manager-trace TRACE.tsv --event-log EVENTS.tsv | --validate-manager-trace TRACE.tsv --event-log EVENTS.tsv]";
fn parse(args: &[String]) -> Result<Option<BTreeMap<String, String>>, String> {
    if args == ["--help"] {
        return Ok(None);
    }
    let mut fields = BTreeMap::new();
    let (pairs, remainder) = args.as_chunks::<2>();
    for pair in pairs {
        if ![
            "--baseline",
            "--contact-counts",
            "--lineup",
            "--cast",
            "--bench",
            "--seed",
            "--calibrated-ratings",
            "--format",
            "--manager-trace",
            "--event-log",
            "--validate-manager-trace",
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
    if !remainder.is_empty()
        || ["--baseline", "--contact-counts", "--lineup", "--cast"]
            .iter()
            .any(|key| !fields.contains_key(*key))
    {
        return Err(format!("missing input/value; usage: {USAGE}"));
    }
    let seed = fields.entry("--seed".into()).or_insert_with(|| "1".into());
    seed.parse::<u64>()
        .map_err(|_| "seed must be an unsigned 64-bit integer")?;
    let format = fields
        .entry("--format".into())
        .or_insert_with(|| "detailed".into());
    if !matches!(format.as_str(), "detailed" | "radio" | "accounting") {
        return Err("format must be detailed or radio or accounting".into());
    }
    let capture = fields.contains_key("--manager-trace");
    let validate = fields.contains_key("--validate-manager-trace");
    if (capture && validate) || fields.contains_key("--event-log") != (capture || validate) {
        return Err(
            "manager trace and event log must be supplied as one capture or validation pair".into(),
        );
    }
    Ok(Some(fields))
}
fn event_log(
    game: &str,
    seed: u64,
    log: &[bbsim_engine::EventTransition],
) -> Result<String, String> {
    let count = u32::try_from(log.len()).map_err(|_| "event log exceeds u32 bound")?;
    if count == 0 {
        return Err("empty canonical event log".into());
    }
    let mut text =
        format!("format\tcanonical-event-log-v1\ngame\t{game}\nseed\t{seed}\nevents\t{count}\n");
    for (index, transition) in log.iter().enumerate() {
        let mut encoded = String::with_capacity(transition.to_bytes().len() * 2);
        for byte in transition.to_bytes() {
            write!(&mut encoded, "{byte:02x}").unwrap();
        }
        writeln!(&mut text, "event\t{index}\t{encoded}").unwrap();
    }
    Ok(text)
}

fn canonical_hex(value: &str) -> Result<Vec<u8>, String> {
    let (pairs, remainder) = value.as_bytes().as_chunks::<2>();
    if pairs.is_empty()
        || !remainder.is_empty()
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("invalid canonical event hex".into());
    }
    pairs
        .iter()
        .map(|pair| {
            let digit = |b: u8| match b {
                b'0'..=b'9' => Ok(b - b'0'),
                b'a'..=b'f' => Ok(b - b'a' + 10),
                _ => Err("invalid canonical event hex"),
            };
            Ok((digit(pair[0])? << 4) | digit(pair[1])?)
        })
        .collect()
}

fn event_log_read(
    text: &str,
    game: &str,
    seed: u64,
    initial: &bbsim_engine::GameState,
) -> Result<Vec<bbsim_engine::EventTransition>, String> {
    if text.len() > 64 * 1024 * 1024 || text.contains('\r') || !text.ends_with('\n') {
        return Err("invalid canonical event log framing/size".into());
    }
    let lines = text[..text.len() - 1].split('\n').collect::<Vec<_>>();
    if lines.len() < 5
        || lines[0] != "format\tcanonical-event-log-v1"
        || lines[1] != format!("game\t{game}")
        || lines[2] != format!("seed\t{seed}")
    {
        return Err("invalid canonical event log identity".into());
    }
    let count_text = lines[3]
        .strip_prefix("events\t")
        .ok_or("missing canonical event count")?;
    if count_text.is_empty()
        || !count_text.bytes().all(|b| b.is_ascii_digit())
        || (count_text.len() > 1 && count_text.starts_with('0'))
    {
        return Err("invalid canonical event count".into());
    }
    let count = count_text
        .parse::<u32>()
        .map_err(|_| "invalid canonical event count")? as usize;
    if count == 0 || lines.len().checked_sub(4) != Some(count) {
        return Err("canonical event count mismatch".into());
    }
    let mut log = Vec::with_capacity(count);
    for (index, line) in lines[4..].iter().enumerate() {
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 3 || fields[0] != "event" || fields[1] != index.to_string() {
            return Err("invalid canonical event row".into());
        }
        let bytes = canonical_hex(fields[2])?;
        let transition = bbsim_engine::EventTransition::from_bytes(&bytes)
            .map_err(|e| format!("invalid canonical event transition: {e:?}"))?;
        if transition.to_bytes() != bytes {
            return Err("noncanonical event transition".into());
        }
        log.push(transition);
    }
    if log[0].before() != initial
        || log
            .windows(2)
            .any(|pair| pair[0].after() != pair[1].before())
        || !log.last().unwrap().after().over
    {
        return Err("canonical event history is incomplete or discontinuous".into());
    }
    full_simulation::verify_accounting(&log)?;
    Ok(log)
}

fn read_sidecar(path: &str) -> io::Result<String> {
    const MAX_SIDECAR_BYTES: u64 = 64 * 1024 * 1024;
    let file = OpenOptions::new().read(true).open(path)?;
    if file.metadata()?.len() > MAX_SIDECAR_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "sidecar exceeds size bound",
        ));
    }
    let mut text = String::new();
    file.take(MAX_SIDECAR_BYTES + 1).read_to_string(&mut text)?;
    if text.len() as u64 > MAX_SIDECAR_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "sidecar exceeds size bound",
        ));
    }
    Ok(text)
}

fn create_sidecar(path: &str, text: &str) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(text.as_bytes())
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
    if lineup.policy_version() != cast.season_policy() {
        return Err("lineup/cast mode mismatch".into());
    }
    match (lineup.is_generic(), cast.game_id()) {
        (false, None) => {}
        (true, Some(id)) if id == lineup.game_id() => {}
        _ => return Err("lineup/cast game or format mismatch".into()),
    }
    let bench = options
        .get("--bench")
        .map(|path| {
            full_simulation::bench::Bench::read_tsv(
                &std::fs::read_to_string(path).map_err(|e| e.to_string())?,
                lineup.game_id(),
            )
        })
        .transpose()?;
    if bench
        .as_ref()
        .is_some_and(|bench| bench.season_policy() != cast.season_policy())
    {
        return Err("bench/cast mode mismatch".into());
    }
    if cast.is_v2() && bench.is_none() {
        return Err("v2 personnel requires a complete bench input".into());
    }
    let teams = lineup.team_names().map(str::to_owned);
    let team_labels = lineup.team_labels().map(str::to_owned);
    let initial = lineup.initial();
    let mut required: std::collections::BTreeSet<_> = initial
        .teams
        .iter()
        .flat_map(|team| team.batting_order.iter().map(|id| format!("batter:{id}")))
        .chain(cast.names().keys().map(|id| format!("pitcher:{id}")))
        .collect();
    if initial.teams[0].dh_slot.is_none() {
        required.extend(cast.names().keys().map(|id| format!("batter:{id}")));
    }
    let mut ability_counts = cast.ability_counts().clone();
    if let Some(bench) = &bench {
        for (id, counts) in &bench.counts {
            if ability_counts.insert(id.clone(), *counts).is_some() {
                return Err("bench/cast batting-count conflict".into());
            }
            required.insert(id.clone());
        }
    }
    let ratings = ratings::Ratings::from_counts(ability_counts.clone(), &baseline, &required)?;
    let calibrated = options
        .get("--calibrated-ratings")
        .map(|path| {
            calibrated::Ratings::read_tsv(
                &std::fs::read_to_string(path).map_err(|e| e.to_string())?,
                &ability_counts,
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
                .map(String::as_str)
                .chain(team.pitcher.active_id())
                .find(|id| positions.get(*id) == Some(&labels[i]))
                .expect("validated starting position")
                .to_owned()
        })
    });
    let mut names = lineup.names();
    cast.merge_names(&mut names)?;
    if let Some(bench) = &bench {
        for (id, name) in &bench.names {
            if names.insert(id.clone(), name.clone()).is_some() {
                return Err("bench/initial display identity conflict".into());
            }
        }
    }
    if cast.is_v2() {
        cast.validate_v2_clocks(lineup.team_labels())?;
        let mut names = lineup.names();
        cast.merge_names(&mut names)?;
        let prepared = cast.prepare(&initial)?;
        let prepared = bench
            .as_ref()
            .expect("v2 requires a bench")
            .prepare(&prepared, &fielders, &cast)?;
        if options["--format"] != "accounting" {
            return Err(
                "v2 personnel currently requires accounting output and trace evidence".into(),
            );
        }
        if let Some(trace_path) = options.get("--validate-manager-trace") {
            let log = event_log_read(
                &read_sidecar(&options["--event-log"])?,
                lineup.game_id(),
                options["--seed"].parse::<u64>()?,
                &prepared,
            )?;
            let trace = read_sidecar(trace_path)?;
            cast.validate_v2_trace(
                &trace,
                lineup.game_id(),
                options["--seed"].parse::<u64>()?,
                &log,
            )?;
            return Ok(());
        }
        if !options.contains_key("--manager-trace") {
            return Err("v2 simulation requires --manager-trace and --event-log sidecars".into());
        }
    } else if options.contains_key("--manager-trace")
        || options.contains_key("--validate-manager-trace")
    {
        return Err("manager trace options require v2 personnel".into());
    }
    let seed = options["--seed"].parse::<u64>()?;
    let full_simulation::Simulation {
        steps,
        draws,
        manager_trace,
    } = full_simulation::simulate_with_personnel_options(
        &initial,
        &fielders,
        &model,
        Some(&cast),
        bench.as_ref(),
        full_simulation::SimulationOptions {
            seed,
            capture_v2_trace: cast.is_v2(),
        },
        |state| {
            let probabilities = match &calibrated {
                Some(r) => r.probabilities(state)?,
                None => ratings.probabilities(state, *baseline.probabilities())?,
            };
            Ok(Some(model.player_probabilities(probabilities)?))
        },
    )?;
    if cast.is_v2() {
        let trace = manager_trace
            .as_deref()
            .ok_or("v2 simulation omitted manager trace")?;
        let log = steps
            .iter()
            .map(|step| step.transition().clone())
            .collect::<Vec<_>>();
        let events = event_log(lineup.game_id(), seed, &log)?;
        create_sidecar(&options["--manager-trace"], trace)?;
        create_sidecar(&options["--event-log"], &events)?;
    }
    if options["--format"] == "accounting" {
        let log: Vec<_> = steps.iter().map(|s| s.transition().clone()).collect();
        let stats = bbsim_engine::accounting::derive_accounting(&log)
            .map_err(|e| format!("machine accounting: {e:?}"))?;
        let ids = if lineup.is_generic() {
            team_labels.clone()
        } else {
            ["SEA".into(), "OAK".into()]
        };
        let mut text = Vec::new();
        if cast.is_v2() {
            season_accounting::write_managed_v2(
                &mut text,
                &stats,
                lineup.game_id(),
                seed,
                &ids,
                if calibrated.is_some() {
                    calibrated::MODEL_VERSION
                } else {
                    ratings::MODEL_VERSION
                },
                bench.is_some(),
            )?;
        } else if cast.is_managed() {
            season_accounting::write_managed(
                &mut text,
                &stats,
                lineup.game_id(),
                seed,
                &ids,
                if calibrated.is_some() {
                    calibrated::MODEL_VERSION
                } else {
                    ratings::MODEL_VERSION
                },
                bench.is_some(),
            )?;
        } else if bench.is_some() {
            season_accounting::write_with_personnel(
                &mut text,
                &stats,
                lineup.game_id(),
                seed,
                &ids,
                Some((
                    full_simulation::pinch::POLICY_VERSION,
                    full_simulation::bench::AVAILABILITY,
                    full_simulation::bench::DEFENSE_BASIS,
                    if calibrated.is_some() {
                        calibrated::MODEL_VERSION
                    } else {
                        ratings::MODEL_VERSION
                    },
                )),
            )?;
        } else {
            season_accounting::write(&mut text, &stats, lineup.game_id(), seed, &ids)?;
        }
        io::stdout().lock().write_all(&text)?;
        return Ok(());
    }
    let draws: BTreeMap<_, _> = draws.into_iter().collect();
    let mut output = io::stdout().lock();
    if options["--format"] == "radio" {
        writeln!(output, "{} at {} — radio transcript", teams[0], teams[1])?;
        if bench.is_some() {
            writeln!(
                output,
                "PH policy: {} | {} | {}",
                if cast.is_managed() {
                    season_pool::PINCH_HITTING
                } else {
                    full_simulation::pinch::POLICY_VERSION
                },
                if cast.is_managed() {
                    season_pool::AVAILABILITY
                } else {
                    full_simulation::bench::AVAILABILITY
                },
                full_simulation::bench::DEFENSE_BASIS
            )?;
        } else if cast.is_managed() {
            writeln!(
                output,
                "Season policy: {} | {} | pinch hitting disabled",
                season_pool::POLICY,
                season_pool::AVAILABILITY
            )?;
        } else if lineup.is_generic() {
            writeln!(output,"Controlled historical lineup/closed cast, not season fidelity; no pinch hitting or daily eligibility. Explicit zero-count roles use the neutral prior.")?;
        }
        writeln!(
            output,
            "Simulated exhibition | Seed: {seed} | Ratings: {}",
            if calibrated.is_some() {
                "experimental v2"
            } else {
                "standard"
            }
        )?;
        writeln!(
            output,
            "Uses 2019 stats; fielding and baserunning are simplified."
        )?;
        writeln!(
            output,
            "Manager: {} | {}; PA workload proxy, not pitch counts; no extra performance cutoff.",
            cast.policy_version(),
            cast.workload_mode()
        )?;
        for (i, step) in steps.iter().enumerate() {
            full_transcript::write_radio_step(&mut output, i + 1, step, &team_labels, &names)?;
        }
        full_transcript::write_line_score(&mut output, &steps, &teams)?;
        full_transcript::write_box_score(&mut output, &steps, &teams, &names)?;
        return Ok(());
    }
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
    if cast.is_managed() {
        writeln!(
            output,
            "{} at {}; modeled {} season-pool starting personnel, not the historical result.",
            teams[0],
            teams[1],
            lineup.date()
        )?;
    } else {
        writeln!(
            output,
            "{} at {}; recorded {} starting personnel, not the historical result.",
            teams[0],
            teams[1],
            lineup.date()
        )?;
    }
    if bench.is_some() {
        writeln!(
            output,
            "PH policy: {} | {} | {}",
            if cast.is_managed() {
                season_pool::PINCH_HITTING
            } else {
                full_simulation::pinch::POLICY_VERSION
            },
            if cast.is_managed() {
                season_pool::AVAILABILITY
            } else {
                full_simulation::bench::AVAILABILITY
            },
            full_simulation::bench::DEFENSE_BASIS
        )?;
    } else if cast.is_managed() {
        writeln!(
            output,
            "Season policy: {} | {} | pinch hitting disabled",
            season_pool::POLICY,
            season_pool::AVAILABILITY
        )?;
    } else if lineup.is_generic() {
        writeln!(output,"Controlled historical lineup/closed cast, not season fidelity; no pinch hitting or daily eligibility. Explicit zero-count roles use the neutral prior.")?;
    }
    if calibrated.is_some() {
        writeln!(output,"Individual batter/current-pitcher probabilities from an explicit opponent-adjusted outcome-tree development artifact. Full2019 hindsight refit, outcome-specific strengths; one parent draw. Reused retrospective test benchmark, not fresh independent validation, official PA/BF, true ability or forecasting; no handedness adjustment or propagated ability uncertainty.")?;
    } else {
        writeln!(output,"Individual batter/current-pitcher PA probabilities from cast ability counts; model {} with fixed kappa200. Selected-season hindsight participant proxies, not official PA/BF, calibrated player fidelity or forecasting; no handedness adjustment or propagated rating uncertainty.", ratings::MODEL_VERSION)?;
    }
    writeln!(output,"Pooled reference counts: {counts:?}; interference alone retains pooled parent mass. Generated contact geometry/handling, runner/error/steal conditionals remain shared provisional priors; not observed mechanisms or validated individual speed/arm/fielding.")?;
    writeln!(output,"PA MODEL records the actual consumed vector, raw RNG word and selected parent (K, BB, HBP, 1B, 2B, 3B, HR, ROE, contact, interference). Tactical IBB overrides and runner-only events have no PA draw. Credited walkoff hit value may be shorter than the selected parent.")?;
    if cast.is_managed() {
        writeln!(output, "Manager: {} | {}. {}. Starter budget scaling and workload-role/pace preferences, not measured fatigue or historical leverage; bounded scoreless/no-hit extensions; no hard IP quota; exhausted cast keeps last pitcher.", cast.policy_version(), cast.workload_mode(), season_pool::AVAILABILITY)?;
    } else {
        writeln!(output,"Manager: {} | {}. Recorded nonstarters assumed available at start; not verified eligibility. Individual outing budgets with bounded scoreless/no-hit extensions; no additional early-performance hook, measured pitch counts or fatigue. No automatic seventh-inning hook; exhausted cast keeps last pitcher.", full_manager::POLICY_VERSION, cast.workload_mode())?;
    }
    let missing_personnel = if bench.is_some() {
        "pinch running/tactical defensive replacements"
    } else {
        "pinch hitting/running/defensive replacements"
    };
    writeln!(output,"KNOWN MISSING GENERATION: automatic uncaught-K out plus same-delivery runner advance, {missing_personnel}, ground-rule doubles, inside-park home runs and appeal double-offs. Joint hits/runner tags and eligible recovered-K advances use provisional priors. No complete major-event coverage claim.")?;
    writeln!(output, "Source witness {}; accepted 2019 snapshot d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528; supplement 554632467416eef202b7a55cadad2d09e5355ce98c83868b26b5267f98852b82.",lineup.game_id())?;
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
            let pitcher = event
                .participants
                .pitcher
                .as_deref()
                .ok_or("ordinary PA lacks pitcher")?;
            writeln!(output, "PA MODEL {} | Batter: {} ({batter}) | Pitcher: {} ({pitcher}) | Word: {} | Parent: {} | Probabilities: {:?}",i+1,names.get(batter).ok_or("missing rated batter name")?,names.get(pitcher).ok_or("missing rated pitcher name")?,draw.word,draw.parent,draw.probabilities)?;
        }
        full_transcript::write_step(&mut output, i + 1, step, &team_labels, &names)?;
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
    fn radio_format_is_explicit_strict_and_keeps_detailed_default() {
        let good = args(&[
            "--baseline",
            "base.csv",
            "--contact-counts",
            "contact.csv",
            "--lineup",
            "lineup.tsv",
            "--cast",
            "cast.tsv",
        ]);
        assert_eq!(parse(&good).unwrap().unwrap()["--format"], "detailed");
        for format in ["radio", "detailed", "accounting"] {
            let explicit = [good.clone(), args(&["--format", format])].concat();
            let parsed = parse(&explicit).unwrap().unwrap();
            assert_eq!(parsed["--format"], format);
            assert_eq!(parsed["--seed"], "1");
            assert!(parse(&[explicit, args(&["--format", "radio"])].concat()).is_err());
        }
        for format in ["", "Radio", "brief", "--seed"] {
            assert!(parse(&[good.clone(), args(&["--format", format])].concat()).is_err());
        }
        assert!(parse(&[good, args(&["--format"])].concat()).is_err());
    }
    #[test]
    fn trace_capture_and_validation_require_distinct_paired_sidecars() {
        let base = args(&[
            "--baseline",
            "base.csv",
            "--contact-counts",
            "contact.csv",
            "--lineup",
            "lineup.tsv",
            "--cast",
            "cast.tsv",
            "--format",
            "accounting",
        ]);
        for (mode, trace) in [
            ("--manager-trace", "trace.tsv"),
            ("--validate-manager-trace", "trace.tsv"),
        ] {
            let explicit = [
                base.clone(),
                args(&[mode, trace, "--event-log", "events.tsv"]),
            ]
            .concat();
            assert_eq!(parse(&explicit).unwrap().unwrap()[mode], trace);
        }
        for bad in [
            [base.clone(), args(&["--manager-trace", "trace.tsv"])].concat(),
            [base.clone(), args(&["--event-log", "events.tsv"])].concat(),
            [
                base.clone(),
                args(&[
                    "--manager-trace",
                    "trace.tsv",
                    "--event-log",
                    "events.tsv",
                    "--validate-manager-trace",
                    "trace.tsv",
                ]),
            ]
            .concat(),
        ] {
            assert!(parse(&bad).is_err());
        }
    }
    #[test]
    fn malformed_pairs_keep_error_precedence_over_a_trailing_missing_value() {
        let good = args(&[
            "--baseline",
            "base.csv",
            "--contact-counts",
            "contact.csv",
            "--lineup",
            "lineup.tsv",
            "--cast",
            "cast.tsv",
        ]);
        for tail in [
            args(&["--other", "x", "--seed"]),
            args(&["--seed", "", "--format"]),
            args(&["--cast", "duplicate.tsv", "--seed"]),
        ] {
            assert_eq!(
                parse(&[good.clone(), tail].concat()).unwrap_err(),
                format!(
                    "unknown, empty or duplicate option; usage: {}",
                    super::USAGE
                )
            );
        }
        assert_eq!(
            parse(&[good, args(&["--seed"])].concat()).unwrap_err(),
            format!("missing input/value; usage: {}", super::USAGE)
        );
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
