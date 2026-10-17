#[allow(dead_code)]
#[path = "../examples/support/full_game.rs"]
mod full_game;
#[allow(dead_code)]
#[path = "../examples/support/full_manager.rs"]
mod full_manager;
#[allow(dead_code)]
#[path = "../examples/support/full_simulation.rs"]
mod full_simulation;
#[path = "../examples/support/full_transcript.rs"]
mod full_transcript;
#[allow(dead_code)]
#[path = "../examples/support/preview.rs"]
mod preview;
#[path = "../examples/support/transcript.rs"]
mod transcript;
use bbsim_engine::*;
use std::collections::BTreeMap;
fn fixture() -> (GameState, [String; 9], BTreeMap<String, String>) {
    let before = GameState {
        inning: 3,
        half: Half::Top,
        outs: 1,
        score: [2, 3],
        teams: ["a", "h"].map(|p| TeamState {
            batting_order: std::array::from_fn(|i| format!("{p}{i}")),
            next_batter: 0,
            pitcher: format!("{p}p"),
            dh_slot: Some(8),
            dh_must_bat: false,
            bench: vec![],
            removed: vec![],
        }),
        bases: [
            Some(Runner {
                player_id: "a5".into(),
                responsible_pitcher: "hp".into(),
            }),
            None,
            None,
        ],
        over: false,
    };
    let fielders = std::array::from_fn(|i| {
        if i == 0 {
            "hp".into()
        } else {
            format!("h{}", i - 1)
        }
    });
    let names = before
        .teams
        .iter()
        .flat_map(|t| t.batting_order.iter().chain(std::iter::once(&t.pitcher)))
        .map(|id| {
            (
                id.clone(),
                format!("Name {}", id.replace('a', "Away ").replace('h', "Home ")),
            )
        })
        .collect();
    (before, fielders, names)
}
fn render(
    step: &full_simulation::Step,
    names: &BTreeMap<String, String>,
) -> std::io::Result<String> {
    let mut text = Vec::new();
    full_transcript::write_step(
        &mut text,
        1,
        step,
        &["Seattle".into(), "Oakland".into()],
        names,
    )?;
    let text = String::from_utf8(text).unwrap();
    let radio = render_radio(step, names)?;
    for debug in ["Before:", "After:", "Batter:", "PA MODEL", "Probabilities:"] {
        assert!(!radio.contains(debug), "{radio}");
    }
    if let Some(directory) = std::env::var_os("BBSIM_TEXT_GAME_CASES") {
        use std::io::Write;
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT_CASE: AtomicUsize = AtomicUsize::new(1);
        std::fs::create_dir_all(&directory)?;
        let path = std::path::PathBuf::from(directory).join(format!(
            "case-{:03}.txt",
            NEXT_CASE.fetch_add(1, Ordering::Relaxed)
        ));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        writeln!(file, "CONSTRUCTED SYNTHETIC-IDENTITY REGRESSION EXAMPLE — NOT AN ORDINARY GAME OR HISTORICAL REPLAY")?;
        writeln!(
            file,
            "Test: {}",
            std::thread::current()
                .name()
                .unwrap_or("constructed regression")
        )?;
        file.write_all(text.as_bytes())?;
    }
    Ok(text)
}
fn render_radio(
    step: &full_simulation::Step,
    names: &BTreeMap<String, String>,
) -> std::io::Result<String> {
    let mut text = Vec::new();
    full_transcript::write_radio_step(
        &mut text,
        1,
        step,
        &["Seattle".into(), "Oakland".into()],
        names,
    )?;
    Ok(String::from_utf8(text).unwrap())
}

fn line_score_game(
    choices: impl IntoIterator<Item = full_game::Ordinary>,
) -> Vec<full_simulation::Step> {
    let (mut state, _, _) = fixture();
    state.inning = 1;
    state.outs = 0;
    state.score = [0, 0];
    state.bases = [None, None, None];
    let mut steps = Vec::new();
    for choice in choices {
        let team = &state.teams[1 - state.half.offense()];
        let fielders = std::array::from_fn(|i| {
            if i == 0 {
                team.pitcher.clone()
            } else {
                team.batting_order[i - 1].clone()
            }
        });
        let step =
            full_simulation::Step::Play(full_game::ordinary(&state, &fielders, choice).unwrap());
        state = step.transition().after().clone();
        steps.push(step);
    }
    assert!(state.over);
    steps
}
fn render_line_score(steps: &[full_simulation::Step]) -> String {
    let mut text = Vec::new();
    full_transcript::write_line_score(
        &mut text,
        steps,
        &["Seattle Mariners".into(), "Oakland Athletics".into()],
    )
    .unwrap();
    let text = String::from_utf8(text).unwrap();
    let rows: Vec<_> = text.lines().filter(|line| line.contains('|')).collect();
    assert_eq!(rows.len(), 3, "{text}");
    assert!(
        rows.iter().all(|line| line.find('|') == rows[0].find('|')),
        "unaligned columns: {text}"
    );
    text
}
fn line_score_cells<'a>(text: &'a str, team: &str) -> Vec<&'a str> {
    text.lines()
        .find_map(|line| line.strip_prefix(team))
        .unwrap()
        .split_whitespace()
        .collect()
}

#[test]
fn radio_line_score_has_all_nine_innings_and_correct_runs_hits_and_defensive_errors() {
    use full_game::Ordinary::*;
    let steps = line_score_game(
        [FieldingError(6), Single(7), HomeRun(8)]
            .into_iter()
            .chain([CalledStrikeout; 54]),
    );
    let text = render_line_score(&steps);
    assert_eq!(
        line_score_cells(&text, "Team"),
        ["1", "2", "3", "4", "5", "6", "7", "8", "9", "|", "R", "H", "E"]
    );
    assert_eq!(
        line_score_cells(&text, "Seattle Mariners"),
        ["3", "0", "0", "0", "0", "0", "0", "0", "0", "|", "3", "2", "0"]
    );
    assert_eq!(
        line_score_cells(&text, "Oakland Athletics"),
        ["0", "0", "0", "0", "0", "0", "0", "0", "0", "|", "0", "0", "1"]
    );
}

#[test]
fn radio_line_score_marks_an_unplayed_home_ninth_with_x() {
    use full_game::Ordinary::*;
    let steps = line_score_game(
        [CalledStrikeout; 3]
            .into_iter()
            .chain([HomeRun(8)])
            .chain([CalledStrikeout; 48]),
    );
    let text = render_line_score(&steps);
    assert_eq!(
        line_score_cells(&text, "Oakland Athletics"),
        ["1", "0", "0", "0", "0", "0", "0", "0", "X", "|", "1", "1", "0"]
    );
    assert_eq!(line_score_cells(&text, "Team").len(), 13);
}

#[test]
fn radio_line_score_includes_every_extra_inning_and_played_zero() {
    use full_game::Ordinary::*;
    let steps = line_score_game(
        [CalledStrikeout; 60]
            .into_iter()
            .chain([Single(7), HomeRun(9)])
            .chain([CalledStrikeout; 6]),
    );
    let text = render_line_score(&steps);
    assert_eq!(
        line_score_cells(&text, "Team"),
        ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "|", "R", "H", "E"]
    );
    let away = line_score_cells(&text, "Seattle Mariners");
    assert_eq!(&away[9..], ["0", "2", "|", "2", "2", "0"]);
    let home = line_score_cells(&text, "Oakland Athletics");
    assert_eq!(&home[9..], ["0", "0", "|", "0", "0", "0"]);
}

#[test]
fn radio_line_score_counts_the_walkoff_home_inning_and_all_home_run_scores() {
    use full_game::Ordinary::*;
    let steps = line_score_game(
        [CalledStrikeout; 57]
            .into_iter()
            .chain([Single(7), HomeRun(9)]),
    );
    let text = render_line_score(&steps);
    let home = line_score_cells(&text, "Oakland Athletics");
    assert_eq!(&home[9..], ["2", "|", "2", "2", "0"]);
    assert!(!text.contains('X'), "{text}");
}

#[test]
fn radio_line_score_aligns_multi_digit_inning_runs_and_totals() {
    use full_game::Ordinary::*;
    let steps = line_score_game([HomeRun(7); 13].into_iter().chain([CalledStrikeout; 54]));
    let text = render_line_score(&steps);
    let away = line_score_cells(&text, "Seattle Mariners");
    assert_eq!(away[0], "13");
    assert_eq!(&away[9..], ["|", "13", "13", "0"]);
}

#[test]
fn radio_line_score_matches_raw_transition_totals_across_seeded_games() {
    let (mut initial, _, _) = fixture();
    initial.inning = 1;
    initial.outs = 0;
    initial.score = [0, 0];
    initial.bases = [None, None, None];
    let fielders = initial.teams.each_ref().map(|team| {
        std::array::from_fn(|i| {
            if i == 0 {
                team.pitcher.clone()
            } else {
                team.batting_order[i - 1].clone()
            }
        })
    });
    let model = full_simulation::Model::new(
        [42823, 15142, 1984, 25947, 8531, 785, 6776, 1470, 82245, 61],
        [17188, 19528, 5379, 536, 39614],
    )
    .unwrap();
    for seed in 1..=100 {
        let steps = full_simulation::simulate(&initial, &fielders, &model, None, seed).unwrap();
        let text = render_line_score(&steps);
        let mut innings = BTreeMap::<u16, [Option<u32>; 2]>::new();
        let mut hits = [0u32; 2];
        let mut errors = [0u32; 2];
        for step in &steps {
            let t = step.transition();
            let offense = t.before().half.offense();
            *innings.entry(t.before().inning).or_default()[offense].get_or_insert(0) +=
                u32::from(t.event().delta.runs);
            for credit in &t.event().delta.credits {
                if credit.kind == CreditKind::Hit {
                    hits[offense] += 1;
                }
                if credit.kind == CreditKind::Error {
                    errors[1 - offense] += 1;
                }
            }
        }
        for (side, team) in ["Seattle Mariners", "Oakland Athletics"]
            .into_iter()
            .enumerate()
        {
            let cells = line_score_cells(&text, team);
            assert_eq!(cells.len(), innings.len() + 4, "seed {seed}: {text}");
            let mut total = 0;
            for (cell, runs) in cells.iter().zip(innings.values()) {
                assert_eq!(
                    *cell,
                    runs[side].map_or_else(|| "X".into(), |r| r.to_string()),
                    "seed {seed}: {text}"
                );
                total += runs[side].unwrap_or(0);
            }
            assert_eq!(
                total,
                u32::from(steps.last().unwrap().transition().after().score[side])
            );
            let totals = [total, hits[side], errors[side]].map(|n| n.to_string());
            assert_eq!(&cells[innings.len() + 1..], &totals, "seed {seed}: {text}");
        }
        let (_, _, names) = fixture();
        let text = render_box_score(&steps, &names);
        let log: Vec<_> = steps.iter().map(|step| step.transition().clone()).collect();
        let stats = accounting::derive_accounting(&log).unwrap();
        for (side, team) in ["Seattle Mariners", "Oakland Athletics"]
            .into_iter()
            .enumerate()
        {
            let section = format!("{team} — Batting");
            for id in &initial.teams[side].batting_order {
                let b = stats.batters[side].get(id).cloned().unwrap_or_default();
                let expected = [
                    Some(b.plate_appearances),
                    b.at_bats,
                    Some(b.credited_runs),
                    b.classified_hits,
                    b.classified_rbi,
                    Some(b.credited_walks),
                    Some(b.credited_strikeouts),
                ]
                .map(|n| n.map_or("?".into(), |n| n.to_string()));
                assert_eq!(
                    box_cells(&text, &section, &names[id]),
                    expected,
                    "seed {seed}: {text}"
                );
            }
            let batting = box_cells(&text, &section, "Totals");
            assert_eq!(batting[2].parse::<u32>().unwrap(), stats.runs[side]);
            assert_eq!(batting[3].parse::<u32>().unwrap(), hits[side]);
            let section = format!("{team} — Pitching");
            let p = &stats.pitchers[side][&initial.teams[side].pitcher];
            let pitching = box_cells(&text, &section, "Totals");
            assert_eq!(
                pitching[0],
                format!("{}.{}", p.outs_recorded / 3, p.outs_recorded % 3)
            );
            assert_eq!(pitching[1].parse::<u32>().unwrap(), hits[1 - side]);
            assert_eq!(pitching[2].parse::<u32>().unwrap(), stats.runs[1 - side]);
            assert_eq!(
                pitching[3],
                p.earned_runs.map_or("?".into(), |n| n.to_string())
            );
            assert_eq!(
                pitching[7],
                stats.batters[1 - side]
                    .values()
                    .map(|b| b.plate_appearances)
                    .sum::<u32>()
                    .to_string()
            );
        }
    }
}

#[test]
fn radio_line_score_refuses_empty_incomplete_and_discontinuous_logs_without_output() {
    use full_game::Ordinary::*;
    let steps = line_score_game([HomeRun(7)].into_iter().chain([CalledStrikeout; 54]));
    let mut broken = steps.clone();
    broken.remove(1);
    for log in [&[][..], &steps[..steps.len() - 1], &steps[1..], &broken] {
        let mut text = Vec::new();
        assert!(full_transcript::write_line_score(
            &mut text,
            log,
            &["Seattle".into(), "Oakland".into()]
        )
        .is_err());
        assert!(text.is_empty());
    }
}

fn render_box_score(steps: &[full_simulation::Step], names: &BTreeMap<String, String>) -> String {
    let mut text = Vec::new();
    full_transcript::write_box_score(
        &mut text,
        steps,
        &["Seattle Mariners".into(), "Oakland Athletics".into()],
        names,
    )
    .unwrap();
    String::from_utf8(text).unwrap()
}
fn box_cells<'a>(text: &'a str, section: &str, player: &str) -> Vec<&'a str> {
    text.split(section)
        .nth(1)
        .unwrap()
        .split("\n\n")
        .next()
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{player} ")))
        .unwrap()
        .split_whitespace()
        .collect()
}

#[test]
fn radio_box_score_lists_lineups_in_order_and_reconciles_batting_pitching_totals() {
    use full_game::Ordinary::*;
    let steps = line_score_game([HomeRun(7)].into_iter().chain([CalledStrikeout; 54]));
    let (_, _, names) = fixture();
    let text = render_box_score(&steps, &names);
    assert!(text.contains("Box score"), "{text}");
    assert_eq!(
        box_cells(&text, "Seattle Mariners — Batting", "Batter"),
        ["PA", "AB", "R", "H", "RBI", "BB", "SO"]
    );
    assert_eq!(
        box_cells(&text, "Seattle Mariners — Batting", "Name Away 0"),
        ["4", "4", "1", "1", "1", "0", "3"]
    );
    assert_eq!(
        box_cells(&text, "Seattle Mariners — Batting", "Totals"),
        ["28", "28", "1", "1", "1", "0", "27"]
    );
    assert_eq!(
        box_cells(&text, "Oakland Athletics — Batting", "Totals"),
        ["27", "27", "0", "0", "0", "0", "27"]
    );
    assert_eq!(
        box_cells(&text, "Oakland Athletics — Pitching", "Pitcher"),
        ["IP", "H", "R", "ER", "BB", "SO", "HR", "BF"]
    );
    assert_eq!(
        box_cells(&text, "Oakland Athletics — Pitching", "Name Home p"),
        ["9.0", "1", "1", "1", "0", "27", "1", "28"]
    );
    assert_eq!(
        box_cells(&text, "Seattle Mariners — Pitching", "Totals"),
        ["9.0", "0", "0", "0", "0", "27", "0", "27"]
    );
    let batting = text
        .split("Seattle Mariners — Batting")
        .nth(1)
        .unwrap()
        .split("\n\n")
        .next()
        .unwrap();
    let mut last = 0;
    for i in 0..9 {
        let index = batting.find(&format!("Name Away {i}")).unwrap();
        assert!(index > last);
        last = index;
    }
    assert!(text.contains("HR: Name Away 0 (1)."), "{text}");
    assert!(text.contains("? = unresolved scoring"), "{text}");
    for unsupported in ["ERA", "AVG", "Winner:", "Save:", "Retrosheet", "PA MODEL"] {
        assert!(!text.contains(unsupported), "{text}");
    }
}

#[test]
fn radio_box_score_includes_extra_hit_award_and_error_notes_without_inventing_er() {
    use full_game::Ordinary::*;
    let steps = line_score_game(
        [
            Double(7),
            Triple(8),
            HomeRun(9),
            IntentionalWalk,
            Hbp,
            CatcherInterference,
        ]
        .into_iter()
        .chain([CalledStrikeout; 54]),
    );
    let (_, _, names) = fixture();
    let text = render_box_score(&steps, &names);
    for note in [
        "2B: Name Away 0 (1).",
        "3B: Name Away 1 (1).",
        "HR: Name Away 2 (1).",
        "IBB: Name Away 3 (1).",
        "HBP: Name Away 4 (1).",
        "E: Name Home 0 (1).",
    ] {
        assert!(text.contains(note), "missing {note}: {text}");
    }
    assert_eq!(
        box_cells(&text, "Seattle Mariners — Batting", "Totals"),
        ["33", "30", "3", "3", "3", "1", "27"]
    );
    assert_eq!(
        box_cells(&text, "Oakland Athletics — Pitching", "Name Home p"),
        ["9.0", "3", "3", "?", "1", "27", "1", "33"]
    );
    assert_eq!(
        box_cells(&text, "Oakland Athletics — Pitching", "Totals")[3],
        "?"
    );
}

#[test]
fn radio_box_score_keeps_pitcher_entry_order_inherited_runs_and_base_three_ip() {
    use full_game::Ordinary::*;
    let (mut state, _, mut names) = fixture();
    state.inning = 1;
    state.outs = 0;
    state.score = [0, 0];
    state.bases = [None, None, None];
    state.teams[1].bench = vec!["zrelief".into(), "arelief".into()];
    names.insert("zrelief".into(), "First Reliever".into());
    names.insert("arelief".into(), "Second Reliever".into());
    let mut steps = Vec::new();
    for (i, choice) in [Walk, HomeRun(7)]
        .into_iter()
        .chain([CalledStrikeout; 54])
        .enumerate()
    {
        if i == 1 || i == 3 {
            let incoming = if i == 1 { "zrelief" } else { "arelief" };
            let fielders = std::array::from_fn(|p| {
                if p == 0 {
                    state.teams[1].pitcher.clone()
                } else {
                    format!("h{}", p - 1)
                }
            });
            let mut event = full_game::ordinary(&state, &fielders, Walk)
                .unwrap()
                .transition()
                .event()
                .clone();
            event.kind = EventKind::Substitution;
            event.fielding_review = FieldingReview::Unknown;
            event.protected_award = None;
            event.delta = Delta {
                moves: vec![],
                outs: 0,
                runs: 0,
                credits: vec![],
                third_out: None,
                changes: vec![PersonnelChange::Pitcher {
                    team: TeamSide::Home,
                    outgoing: state.teams[1].pitcher.clone(),
                    incoming: incoming.into(),
                }],
            };
            let change = apply(&state, event).unwrap();
            state = change.after().clone();
            steps.push(full_simulation::Step::PitchingChange(change));
        }
        let team = &state.teams[1 - state.half.offense()];
        let fielders = std::array::from_fn(|p| {
            if p == 0 {
                team.pitcher.clone()
            } else {
                team.batting_order[p - 1].clone()
            }
        });
        let play = full_game::ordinary(&state, &fielders, choice).unwrap();
        state = play.transition().after().clone();
        steps.push(full_simulation::Step::Play(play));
    }
    assert!(state.over);
    let text = render_box_score(&steps, &names);
    let section = "Oakland Athletics — Pitching";
    assert_eq!(
        box_cells(&text, section, "Name Home p"),
        ["0.0", "0", "1", "?", "1", "0", "0", "1"]
    );
    assert_eq!(
        box_cells(&text, section, "First Reliever"),
        ["0.1", "1", "1", "?", "0", "1", "1", "2"]
    );
    assert_eq!(
        box_cells(&text, section, "Second Reliever"),
        ["8.2", "0", "0", "0", "0", "26", "0", "26"]
    );
    assert_eq!(
        box_cells(&text, section, "Totals"),
        ["9.0", "1", "2", "?", "1", "27", "1", "29"]
    );
    assert!(text.find("First Reliever").unwrap() < text.find("Second Reliever").unwrap());
}

#[test]
fn radio_box_score_refuses_bad_logs_and_missing_names_without_partial_output() {
    use full_game::Ordinary::*;
    let steps = line_score_game([HomeRun(7)].into_iter().chain([CalledStrikeout; 54]));
    let (_, _, names) = fixture();
    let mut broken = steps.clone();
    broken.remove(1);
    let teams = ["Seattle".into(), "Oakland".into()];
    for log in [&[][..], &steps[..steps.len() - 1], &steps[1..], &broken] {
        let mut text = Vec::new();
        assert!(full_transcript::write_box_score(&mut text, log, &teams, &names).is_err());
        assert!(text.is_empty());
    }
    for missing in ["a8", "hp", "h0"] {
        let mut names = names.clone();
        names.remove(missing);
        let mut text = Vec::new();
        assert!(full_transcript::write_box_score(&mut text, &steps, &teams, &names).is_err());
        assert!(text.is_empty());
    }
}

#[test]
fn radio_names_batter_play_out_count_and_runner_holds() {
    let (before, fielders, names) = fixture();
    let step = full_simulation::Step::Play(
        full_game::ordinary(&before, &fielders, full_game::Ordinary::FlyOut(9)).unwrap(),
    );
    let text = render_radio(&step, &names).unwrap();
    assert!(text.contains("Top of inning 3 — Seattle batting"), "{text}");
    assert!(
        text.contains("Name Away 0: Fly ball caught by right fielder Name Home 7."),
        "{text}"
    );
    assert!(text.contains("Two away."), "{text}");
    assert!(text.contains("Name Away 5 holds at first."), "{text}");
    for noisy in ["Before:", "After:", "Play 1:", "is out (from the plate)"] {
        assert!(!text.contains(noisy), "{text}");
    }
}

#[test]
fn radio_walk_uses_a_plain_announcer_sentence() {
    let (before, fielders, names) = fixture();
    let step = full_simulation::Step::Play(
        full_game::ordinary(&before, &fielders, full_game::Ordinary::Walk).unwrap(),
    );
    let text = render_radio(&step, &names).unwrap();
    assert!(text.contains("Name Away 0 walks.\n"), "{text}");
    assert!(!text.contains("Name Away 0:"), "{text}");
    assert!(!text.contains("ordinary walk"), "{text}");
    assert!(text.contains("Name Away 5 advances to second."), "{text}");
}

#[test]
fn radio_walks_omit_implied_first_base_but_keep_forced_advances() {
    let (before, fielders, names) = fixture();
    for choice in [
        full_game::Ordinary::Walk,
        full_game::Ordinary::IntentionalWalk,
    ] {
        let step =
            full_simulation::Step::Play(full_game::ordinary(&before, &fielders, choice).unwrap());
        let text = render_radio(&step, &names).unwrap();
        assert!(!text.contains("Name Away 0 reaches first."), "{text}");
        assert!(text.contains("Name Away 5 advances to second."), "{text}");
    }
}

#[test]
fn radio_runner_events_keep_named_advances_without_a_fake_batter() {
    let (before, fielders, names) = fixture();
    let step = full_simulation::Step::Play(
        full_game::runner(&before, &fielders, full_game::RunnerDecision::WildPitch).unwrap(),
    );
    let text = render_radio(&step, &names).unwrap();
    assert!(text.contains("Wild pitch past the catcher"), "{text}");
    assert!(!text.contains("Name Home 0"), "{text}");
    assert!(text.contains("Name Away 5 advances to second."), "{text}");
    assert!(!text.contains("Name Away 0:"), "{text}");
}

#[test]
fn radio_inning_end_reports_score_stranded_runner_and_next_half() {
    let (mut before, fielders, names) = fixture();
    before.outs = 2;
    let step = full_simulation::Step::Play(
        full_game::ordinary(&before, &fielders, full_game::Ordinary::FlyOut(9)).unwrap(),
    );
    let text = render_radio(&step, &names).unwrap();
    assert!(
        text.contains("Side retired. Seattle 2, Oakland 3."),
        "{text}"
    );
    assert!(text.contains("Name Away 5 is stranded at first."), "{text}");
    assert_eq!(text.matches("Bottom of inning 3").count(), 1, "{text}");
    assert!(!text.contains("Final:"), "{text}");
}

#[test]
fn radio_game_end_uses_final_score_without_announcing_an_unplayed_half() {
    let (mut before, fielders, names) = fixture();
    before.inning = 9;
    before.outs = 2;
    let step = full_simulation::Step::Play(
        full_game::ordinary(&before, &fielders, full_game::Ordinary::FlyOut(9)).unwrap(),
    );
    let text = render_radio(&step, &names).unwrap();
    assert!(text.contains("Final: Seattle 2, Oakland 3."), "{text}");
    assert!(!text.contains("Bottom of inning 9"), "{text}");
    before.half = Half::Bottom;
    before.teams.swap(0, 1);
    before.score = [2, 2];
    let step = full_simulation::Step::Play(
        full_game::ordinary(&before, &fielders, full_game::Ordinary::HomeRun(9)).unwrap(),
    );
    let text = render_radio(&step, &names).unwrap();
    assert!(text.contains("Name Away 0: Home run"), "{text}");
    assert!(!text.contains("every runner scores"), "{text}");
    let detailed = render(&step, &names).unwrap();
    assert!(!detailed.contains("every runner scores"), "{detailed}");
    assert!(text.contains("Name Away 5 scores."), "{text}");
    assert!(text.contains("Name Away 0 scores."), "{text}");
    assert_eq!(
        text.matches("Final: Seattle 2, Oakland 4.").count(),
        1,
        "{text}"
    );
    assert!(!text.contains("inning 10"), "{text}");
}

#[test]
fn radio_announces_the_named_throw_tag_and_runner_out_on_an_extra_base_attempt() {
    let (before, fielders, names) = fixture();
    let step = full_simulation::Step::Play(
        full_game::hit(
            &before,
            &fielders,
            full_game::HitDecision {
                value: 2,
                position: 9,
                runners: [Some(Destination::Out), None, None],
                tag: Some(full_game::TagAttempt {
                    origin: Base::First,
                    attempted_base: Destination::Home,
                    home_before_tag: vec![],
                }),
            },
        )
        .unwrap(),
    );
    let text = render_radio(&step, &names).unwrap();
    for required in [
        "Name Away 0: Double",
        "right fielder Name Home 7 throws to catcher Name Home 0 for the tag at home",
        "Name Away 5 is out (from first).",
        "Two away.",
    ] {
        assert!(text.contains(required), "missing {required}: {text}");
    }
    assert!(!text.contains("Name Away 5 scores"), "{text}");
}

#[test]
fn radio_double_play_omits_redundant_outs_but_keeps_other_runners() {
    for outs in [0, 1] {
        let (mut before, fielders, names) = fixture();
        before.outs = outs;
        before.bases[1] = Some(Runner {
            player_id: "a6".into(),
            responsible_pitcher: "hp".into(),
        });
        let step = full_simulation::Step::Play(
            full_game::contact(
                &before,
                &fielders,
                full_game::ContactDecision::GroundForceDoublePlay(GroundForceDoublePlayFacts {
                    batter_id: "a0".into(),
                    forced_runner_id: "a5".into(),
                    initial_fielder_id: fielders[5].clone(),
                    pivot_id: fielders[3].clone(),
                    first_receiver_id: fielders[2].clone(),
                }),
            )
            .unwrap(),
        );
        let text = render_radio(&step, &names).unwrap();
        assert!(text.contains("at second for the force, relay to"), "{text}");
        assert!(text.contains("double play."), "{text}");
        assert!(!text.contains("is out (from"), "{text}");
        assert!(
            text.contains(if outs == 0 {
                "Name Away 6 holds at second."
            } else {
                "Name Away 6 is stranded at second."
            }),
            "{text}"
        );
        let detailed = render(&step, &names).unwrap();
        assert!(
            detailed.contains("Name Away 5 is out (from first)."),
            "{detailed}"
        );
    }
}

#[test]
fn radio_safe_hits_name_the_field_not_the_fielder_or_implied_batter_advance() {
    for (position, location) in [(7, "left field"), (8, "center field"), (9, "right field")] {
        for (value, label, ordinary) in [
            (1, "Single", full_game::Ordinary::Single(position)),
            (2, "Double", full_game::Ordinary::Double(position)),
            (3, "Triple", full_game::Ordinary::Triple(position)),
        ] {
            let (before, fielders, names) = fixture();
            for play in [
                full_game::ordinary(&before, &fielders, ordinary).unwrap(),
                full_game::hit(
                    &before,
                    &fielders,
                    full_game::HitDecision {
                        value,
                        position,
                        runners: [
                            Some(if value == 3 {
                                Destination::Home
                            } else {
                                Destination::Third
                            }),
                            None,
                            None,
                        ],
                        tag: None,
                    },
                )
                .unwrap(),
            ] {
                let step = full_simulation::Step::Play(play);
                let text = render_radio(&step, &names).unwrap();
                assert!(
                    text.contains(&format!("Name Away 0: {label} to {location}.\n")),
                    "{text}"
                );
                assert!(
                    !text.contains(&names[&fielders[usize::from(position) - 1]]),
                    "{text}"
                );
                assert!(!text.contains("Name Away 0 reaches"), "{text}");
                assert!(
                    text.contains("Name Away 5 advances") || text.contains("Name Away 5 scores."),
                    "{text}"
                );
                assert!(render(&step, &names).unwrap().contains("fielded there."));
            }
        }
    }
}

#[test]
fn radio_names_fielders_only_for_recorded_putouts_or_assists() {
    let (before, fielders, names) = fixture();
    for choice in [
        full_game::Ordinary::HomeRun(7),
        full_game::Ordinary::FieldingError(6),
        full_game::Ordinary::ThrowingError(6),
        full_game::Ordinary::CatcherInterference,
    ] {
        let step =
            full_simulation::Step::Play(full_game::ordinary(&before, &fielders, choice).unwrap());
        let text = render_radio(&step, &names).unwrap();
        for id in &fielders[1..] {
            if !step.transition().event().delta.credits.iter().any(|c| {
                c.player_id == *id && matches!(c.kind, CreditKind::Putout | CreditKind::Assist)
            }) {
                assert!(
                    !text.contains(&names[id]),
                    "unexpected fielder {}: {text}",
                    names[id]
                );
            }
        }
    }
    let step = full_simulation::Step::Play(full_game::ground_out(&before, &fielders, 6).unwrap());
    let text = render_radio(&step, &names).unwrap();
    assert!(text.contains("shortstop Name Home 4"), "{text}");
    assert!(text.contains("first baseman Name Home 1"), "{text}");
}

#[test]
fn radio_missing_displayed_actor_refuses_without_partial_output() {
    let (before, fielders, mut names) = fixture();
    let step = full_simulation::Step::Play(full_game::ground_out(&before, &fielders, 6).unwrap());
    names.remove("h4");
    let mut output = Vec::new();
    assert!(full_transcript::write_radio_step(
        &mut output,
        1,
        &step,
        &["Seattle".into(), "Oakland".into()],
        &names
    )
    .is_err());
    assert!(output.is_empty());
}

#[test]
fn joint_hit_narration_names_direction_throw_tag_and_winning_official_credit() {
    let (mut before, defense, names) = fixture();
    let decision = full_game::HitDecision {
        value: 2,
        position: 9,
        runners: [Some(Destination::Out), None, None],
        tag: Some(full_game::TagAttempt {
            origin: Base::First,
            attempted_base: Destination::Home,
            home_before_tag: vec![],
        }),
    };
    let step = full_simulation::Step::Play(full_game::hit(&before, &defense, decision).unwrap());
    let text = render(&step, &names).unwrap();
    for required in [
        "Double",
        "right fielder Name Home 7",
        "catcher Name Home 0",
        "tag at home",
        "Name Away 5 is out",
        "Name Away 0 reaches second",
    ] {
        assert!(text.contains(required), "missing {required}: {text}");
    }
    before.half = Half::Bottom;
    before.teams.swap(0, 1);
    before.score = [0, 0];
    before.inning = 9;
    before.bases[2] = before.bases[0].take();
    let decision = full_game::HitDecision {
        value: 3,
        position: 7,
        runners: [None, None, Some(Destination::Home)],
        tag: None,
    };
    let step = full_simulation::Step::Play(full_game::hit(&before, &defense, decision).unwrap());
    let text = render(&step, &names).unwrap();
    for required in [
        "Single",
        "left fielder Name Home 5",
        "winning run",
        "Name Away 5 scores",
        "game over",
    ] {
        assert!(text.contains(required), "missing {required}: {text}");
    }
    assert!(
        !text.contains("Triple"),
        "official walkoff credit must not retain sampled value: {text}"
    );
}

#[test]
fn uncaught_k_narration_distinguishes_automatic_safe_hold_and_advance_before_recovery() {
    use full_game::StrikeoutDecision::*;
    for (decision, outs, first_empty, expected) in [
        (
            Automatic,
            1,
            false,
            ["automatically out", "catcher Name Home 0"],
        ),
        (
            SafeReach {
                handling: HandlingAssessment::PitcherWild,
            },
            2,
            false,
            ["wild pitch", "reaches first safely"],
        ),
        (
            RecoveredBeforeFirst {
                handling: HandlingAssessment::CatcherPassed,
            },
            1,
            true,
            ["before first", "runners hold"],
        ),
        (
            RecoveredBeforeFirstAfterAdvances {
                handling: HandlingAssessment::PitcherWild,
            },
            1,
            true,
            ["wild pitch", "advances before the recovery throw"],
        ),
        (
            RecoveredBeforeFirstAfterAdvances {
                handling: HandlingAssessment::CatcherPassed,
            },
            2,
            true,
            ["passed ball", "crosses home; run does not count"],
        ),
    ] {
        let (mut before, defense, names) = fixture();
        before.outs = outs;
        if first_empty {
            before.bases.swap(0, 1);
        }
        if outs == 2 && matches!(decision, RecoveredBeforeFirstAfterAdvances { .. }) {
            before.bases[2] = before.bases[1].take();
        }
        let step = full_simulation::Step::Play(
            full_game::uncaught_strikeout(&before, &defense, decision).unwrap(),
        );
        let text = render(&step, &names).unwrap();
        assert!(text.contains("Strike three is not caught"), "{text}");
        for required in expected {
            assert!(text.contains(required), "missing {required}: {text}");
        }
        if matches!(decision, RecoveredBeforeFirst { .. } | Automatic) {
            assert!(
                !text.contains("passed ball") && !text.contains("wild pitch"),
                "unearned handling unit: {text}"
            );
        }
    }
}

#[test]
fn connected_rendering_names_matchup_ground_mechanism_every_runner_and_continuous_snapshots() {
    let (before, fielders, names) = fixture();
    let first = full_simulation::Step::Play(full_game::ground_out(&before, &fielders, 6).unwrap());
    let text = render(&first, &names).unwrap();
    for required in [
        "Before: Seattle 2 — Oakland 3",
        "Top 3 | 1 out",
        "Batter: Name Away 0",
        "Pitcher: Name Home p",
        "shortstop",
        "Name Home 4",
        "first baseman",
        "Name Home 1",
        "Name Away 5 holds at first",
        "After:",
    ] {
        assert!(text.contains(required), "missing {required}: {text}");
    }
    let second = full_simulation::Step::Play(
        full_game::ground_out(first.transition().after(), &fielders, 3).unwrap(),
    );
    let next = render(&second, &names).unwrap();
    assert!(next.contains("Name Away 5 is stranded at first"), "{next}");
    let after = text
        .lines()
        .find_map(|l| l.strip_prefix("After: "))
        .unwrap();
    let before = next
        .lines()
        .find_map(|l| l.strip_prefix("Before: "))
        .unwrap();
    assert_eq!(after, before);
    assert!(!text.contains("recorded play"));
}
#[test]
fn throwing_error_and_failed_bunt_force_choice_are_concrete_not_fielding_muff_or_sacrifice() {
    let (before, fielders, names) = fixture();
    let step = full_simulation::Step::Play(
        full_game::ordinary(&before, &fielders, full_game::Ordinary::ThrowingError(6)).unwrap(),
    );
    let text = render(&step, &names).unwrap();
    assert!(text.contains("wild throw"), "{text}");
    assert!(!text.contains("muffed"), "{text}");
    let step =
        full_simulation::Step::Play(full_game::force_choice(&before, &fielders, 6, true).unwrap());
    let text = render(&step, &names).unwrap();
    assert!(text.contains("Failed sacrifice bunt"), "{text}");
    assert!(text.contains("force at second"), "{text}");
    assert!(text.contains("Name Away 5 is out"), "{text}");
    assert!(text.contains("Name Away 0 reaches first"), "{text}");
}

#[test]
fn caught_ball_tag_double_play_names_the_catch_retouch_throw_or_unassisted_tag() {
    let (before, fielders, names) = fixture();
    for p in [4, 8] {
        let step = full_simulation::Step::Play(
            full_game::air_advance(
                &before,
                &fielders,
                AirContact::LineDrive,
                p,
                Base::First,
                true,
            )
            .unwrap(),
        );
        let text = render(&step, &names).unwrap();
        assert!(text.contains("Line drive"), "{text}");
        assert!(text.contains("legal retouch"), "{text}");
        assert!(text.contains("Double play"), "{text}");
        let radio = render_radio(&step, &names).unwrap();
        assert!(radio.contains("Double play"), "{radio}");
        assert!(!radio.contains("is out (from"), "{radio}");
        if p == 4 {
            assert!(text.contains("keeps the ball"), "{text}");
            assert!(!text.contains("throws"), "{text}");
        } else {
            assert!(
                text.contains("throws to second baseman Name Home 2"),
                "{text}"
            );
        }
    }
}

#[test]
fn safe_hit_endpoints_keep_their_advances_and_stranding_at_the_third_out() {
    use full_game::{HitDecision, TagAttempt};
    for outs in [1, 2] {
        let (mut before, fielders, names) = fixture();
        before.outs = outs;
        before.bases[1] = Some(Runner {
            player_id: "a6".into(),
            responsible_pitcher: "hp".into(),
        });
        let step = full_simulation::Step::Play(
            full_game::hit(
                &before,
                &fielders,
                HitDecision {
                    value: 1,
                    position: 8,
                    runners: [Some(Destination::Second), Some(Destination::Out), None],
                    tag: Some(TagAttempt {
                        origin: Base::Second,
                        attempted_base: Destination::Home,
                        home_before_tag: vec![],
                    }),
                },
            )
            .unwrap(),
        );
        let text = render(&step, &names).unwrap();
        assert!(text.contains("Name Away 0 reaches first"), "{text}");
        assert!(text.contains("Name Away 5 advances to second"), "{text}");
        assert!(text.contains("Name Away 6 is out (from second)"), "{text}");
        if outs == 2 {
            assert!(
                text.contains("Name Away 0 reaches first; is stranded at first."),
                "{text}"
            );
            assert!(
                text.contains("Name Away 5 advances to second; is stranded at second."),
                "{text}"
            );
            assert!(step.transition().after().bases.iter().all(Option::is_none));
        } else {
            assert!(!text.contains("stranded"), "{text}");
            assert!(step.transition().after().bases[0].is_some());
            assert!(step.transition().after().bases[1].is_some());
        }
    }
}

#[test]
fn recovered_k_advances_are_stranded_but_a_suppressed_home_crossing_is_not() {
    let (mut before, fielders, names) = fixture();
    before.outs = 2;
    before.bases[1] = Some(Runner {
        player_id: "a6".into(),
        responsible_pitcher: "hp".into(),
    });
    before.bases[2] = Some(Runner {
        player_id: "a7".into(),
        responsible_pitcher: "hp".into(),
    });
    let step = full_simulation::Step::Play(
        full_game::uncaught_strikeout(
            &before,
            &fielders,
            full_game::StrikeoutDecision::RecoveredBeforeFirstAfterAdvances {
                handling: bbsim_engine::HandlingAssessment::PitcherWild,
            },
        )
        .unwrap(),
    );
    let text = render(&step, &names).unwrap();
    assert!(
        text.contains("Name Away 5 advances to second; is stranded at second."),
        "{text}"
    );
    assert!(
        text.contains("Name Away 6 advances to third; is stranded at third."),
        "{text}"
    );
    assert!(
        text.contains("Name Away 7 crosses home; run does not count."),
        "{text}"
    );
    assert!(!text.contains("Name Away 7 is stranded"), "{text}");
    assert_eq!(step.transition().after().score, before.score);
    assert!(step.transition().after().bases.iter().all(Option::is_none));
}

#[test]
fn missing_actor_names_refuse_without_partial_or_id_fallback_output() {
    let (before, fielders, mut names) = fixture();
    let step = full_simulation::Step::Play(full_game::ground_out(&before, &fielders, 6).unwrap());
    names.remove("h4");
    let mut output = Vec::new();
    let error = full_transcript::write_step(
        &mut output,
        1,
        &step,
        &["Seattle".into(), "Oakland".into()],
        &names,
    )
    .unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(output.is_empty());
}

#[test]
fn home_steal_tag_has_no_catcher_self_throw_and_explicit_holds_never_become_advances() {
    let (mut before, fielders, names) = fixture();
    before.bases.swap(0, 2);
    let step = full_simulation::Step::Play(
        full_game::runner(
            &before,
            &fielders,
            full_game::RunnerDecision::Steal {
                origin: Base::Third,
                target: Destination::Home,
                caught: true,
            },
        )
        .unwrap(),
    );
    let text = render(&step, &names).unwrap();
    assert!(text.contains("tags the runner at home"), "{text}");
    assert!(!text.contains("throws to"), "{text}");
}

#[test]
fn failed_pickoff_explicit_holds_never_become_advances() {
    let (mut before, fielders, names) = fixture();
    before.bases.swap(0, 2);
    let step = full_simulation::Step::Play(
        full_game::runner(
            &before,
            &fielders,
            full_game::RunnerDecision::Pickoff {
                origin: Base::Third,
                caught: false,
            },
        )
        .unwrap(),
    );
    let text = render(&step, &names).unwrap();
    assert!(text.contains("Name Away 5 holds at third"), "{text}");
    assert!(!text.contains("advances to"), "{text}");
    let radio = render_radio(&step, &names).unwrap();
    assert!(radio.contains("Name Away 5 holds at third."), "{radio}");
}
