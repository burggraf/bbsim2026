//! Scripted presentation only: all state changes and statistics belong to the engine.
use bbsim_engine::{
    accounting::derive_accounting, apply, Base, Credit, CreditKind, Delta, Destination, Event,
    EventKind, EventTransition, FieldingReview, GameState, Half, Participants, Runner, RunnerEvent,
    RunnerMove, TeamState, TransitionError,
};
use std::io::{self, BufRead, Write};

fn team(name: &str) -> TeamState {
    TeamState {
        batting_order: std::array::from_fn(|i| format!("{name}-{}", i + 1)),
        next_batter: 0,
        pitcher: format!("{name}-pitcher"),
        dh_slot: Some(8),
        dh_must_bat: true,
        bench: vec![],
        removed: vec![],
    }
}

fn credit(id: &str, kind: CreditKind) -> Credit {
    Credit {
        player_id: id.into(),
        kind,
    }
}

pub fn scripted_game() -> Result<Vec<EventTransition>, TransitionError> {
    let mut state = GameState {
        inning: 1,
        half: Half::Top,
        outs: 0,
        score: [0, 0],
        teams: [team("Away"), team("Home")],
        bases: [None, None, None],
        over: false,
    };
    let mut log = Vec::new();
    while !state.over {
        // A fixture guard, not a rule or simulated game-length limit.
        if log.len() >= 80 {
            return Err(TransitionError::UnsupportedRule);
        }
        let offense = state.half.offense();
        let batter =
            state.teams[offense].batting_order[state.teams[offense].next_batter as usize].clone();
        let pitcher = state.teams[1 - offense].pitcher.clone();
        let mut kind = EventKind::PlateAppearance;
        let mut moves = vec![RunnerMove {
            runner: Runner {
                player_id: batter.clone(),
                responsible_pitcher: pitcher.clone(),
            },
            from: Base::Plate,
            to: Destination::Out,
        }];
        let mut credits = vec![credit(&batter, CreditKind::PlateAppearance)];
        let mut outs = 0;
        let mut runs = 0;
        match log.len() {
            0 => {
                moves[0].to = Destination::First;
                credits.extend([
                    credit(&batter, CreditKind::Hit),
                    credit(&batter, CreditKind::Single),
                ]);
            }
            1 => {
                kind = EventKind::RunnerOnly(RunnerEvent::StealAttempt);
                let runner = state.bases[0]
                    .clone()
                    .expect("script single precedes steal");
                credits = vec![credit(&runner.player_id, CreditKind::StolenBase)];
                moves = vec![RunnerMove {
                    runner,
                    from: Base::First,
                    to: Destination::Second,
                }];
            }
            2 => {
                moves[0].to = Destination::First;
                credits.extend([
                    credit(&batter, CreditKind::Walk),
                    credit(&batter, CreditKind::OrdinaryWalk),
                ]);
            }
            _ if log.len() == 3
                || (state.inning == 1
                    && state.half == Half::Bottom
                    && state.outs == 0
                    && state.score[1] == 0) =>
            {
                kind = EventKind::OutOfParkHomeRun;
                moves[0].to = Destination::Home;
                credits.extend([
                    credit(&batter, CreditKind::Hit),
                    credit(&batter, CreditKind::Run),
                ]);
                runs = 1;
                for (base, runner) in [Base::First, Base::Second, Base::Third]
                    .into_iter()
                    .zip(&state.bases)
                {
                    if let Some(runner) = runner {
                        moves.push(RunnerMove {
                            runner: runner.clone(),
                            from: base,
                            to: Destination::Home,
                        });
                        credits.push(credit(&runner.player_id, CreditKind::Run));
                        runs += 1;
                    }
                }
            }
            _ => {
                outs = 1;
                credits.push(credit(&batter, CreditKind::Strikeout));
            }
        }
        let event = Event {
            kind,
            participants: Participants {
                batter: Some(batter),
                pitcher,
            },
            delta: Delta {
                moves,
                outs,
                runs,
                credits,
                changes: vec![],
                third_out: None,
            },
            fielding_review: FieldingReview::NoMisplay,
            two_out_contact: None,
            single_delivery: None,
            balk: None,
        };
        let transition = apply(&state, event)?;
        state = transition.after().clone();
        log.push(transition);
    }
    Ok(log)
}

fn half_name(half: Half) -> &'static str {
    match half {
        Half::Top => "Top",
        Half::Bottom => "Bottom",
    }
}

fn description(transition: &EventTransition) -> String {
    let event = transition.event();
    let batter = event.participants.batter.as_deref().unwrap_or("unknown");
    match event.kind {
        EventKind::OutOfParkHomeRun => format!("{batter}: home run ({} run(s))", event.delta.runs),
        EventKind::RunnerOnly(RunnerEvent::StealAttempt) => {
            format!("{} steals second", event.delta.moves[0].runner.player_id)
        }
        _ if event
            .delta
            .credits
            .iter()
            .any(|c| c.kind == CreditKind::Walk) =>
        {
            format!("{batter}: walk")
        }
        _ if event
            .delta
            .credits
            .iter()
            .any(|c| c.kind == CreditKind::Single) =>
        {
            format!("{batter}: single")
        }
        _ => format!("{batter}: strikeout"),
    }
}

pub fn run(input: &mut dyn BufRead, output: &mut dyn Write, autoplay: bool) -> io::Result<()> {
    let log = scripted_game().map_err(|error| io::Error::other(format!("script: {error:?}")))?;
    writeln!(
        output,
        "SCRIPTED ENGINE DEMO — fictional teams, not a probability model."
    )?;
    writeln!(
        output,
        "Opening: single, steal, walk, three-run HR; Home answers with a solo HR."
    )?;
    writeln!(
        output,
        "Remaining outs are scripted strikeouts. Same game on every restart.\n"
    )?;
    for (index, transition) in log.iter().enumerate() {
        if !autoplay {
            write!(output, "Enter: next play | q: quit > ")?;
            output.flush()?;
            let mut line = String::new();
            if input.read_line(&mut line)? == 0 || line.trim().eq_ignore_ascii_case("q") {
                writeln!(
                    output,
                    "Stopped before play {}. Rerun to restart.",
                    index + 1
                )?;
                return Ok(());
            }
        }
        let before = transition.before();
        let after = transition.after();
        writeln!(
            output,
            "Play {:02} | {} {} | {}",
            index + 1,
            half_name(before.half),
            before.inning,
            description(transition)
        )?;
        if after.over {
            writeln!(
                output,
                "  Away {} - Home {} | GAME OVER",
                after.score[0], after.score[1]
            )?;
        } else {
            writeln!(
                output,
                "  Away {} - Home {} | Now: {} {} | Outs: {}",
                after.score[0],
                after.score[1],
                half_name(after.half),
                after.inning,
                after.outs
            )?;
        }
        let bases = after
            .bases
            .each_ref()
            .map(|r| r.as_ref().map_or("-", |r| r.player_id.as_str()));
        writeln!(
            output,
            "  Bases: 1={} 2={} 3={}",
            bases[0], bases[1], bases[2]
        )?;
    }
    let stats = derive_accounting(&log)
        .map_err(|error| io::Error::other(format!("accounting: {error:?}")))?;
    writeln!(
        output,
        "\nFINAL: Away {} - Home {}",
        stats.runs[0], stats.runs[1]
    )?;
    writeln!(output, "Line score (innings 1–9):")?;
    for (side, name) in ["Away", "Home"].into_iter().enumerate() {
        write!(output, "{name:5}")?;
        for runs in stats.innings.values() {
            match runs[side] {
                Some(runs) => write!(output, " {runs:2}")?,
                None => write!(output, "  -")?,
            }
        }
        writeln!(output, " | {}", stats.runs[side])?;
    }
    writeln!(
        output,
        "\nSupplied-credit statistics — not a complete official box (PO/A/RBI omitted)."
    )?;
    writeln!(output, "Batter       PA  AB   H   R  BB   K  SB")?;
    for side in &stats.batters {
        for (name, line) in side {
            let ab = line.at_bats.map_or("?".into(), |value| value.to_string());
            writeln!(
                output,
                "{name:12} {:2} {ab:>3} {:3} {:3} {:3} {:3} {:3}",
                line.plate_appearances,
                line.credited_hits,
                line.credited_runs,
                line.credited_walks,
                line.credited_strikeouts,
                line.credited_steals
            )?;
        }
    }
    writeln!(output, "Pitcher          BF  IP   R   H  BB   K")?;
    for side in &stats.pitchers {
        for (name, line) in side {
            writeln!(
                output,
                "{name:16} {:2} {}.{} {:3} {:3} {:3} {:3}",
                line.batters_faced,
                line.outs_recorded / 3,
                line.outs_recorded % 3,
                line.runs_allowed,
                line.credited_hits_allowed,
                line.credited_walks_allowed,
                line.credited_strikeouts
            )?;
        }
    }
    Ok(())
}
