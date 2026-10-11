//! Terminal-only presentation of validated engine transitions and credited accounting.
use bbsim_engine::{
    accounting::derive_accounting, CreditKind, EventKind, EventTransition, Half, RunnerEvent,
};
use std::io::{self, BufRead, Write};
fn half_name(half: Half) -> &'static str {
    match half {
        Half::Top => "Top",
        Half::Bottom => "Bottom",
    }
}
fn description(t: &EventTransition) -> String {
    let e = t.event();
    let batter = e.participants.batter.as_deref().unwrap_or("unknown");
    let has = |kind| e.delta.credits.iter().any(|c| c.kind == kind);
    let play = if e.kind == EventKind::OutOfParkHomeRun {
        "home run"
    } else if has(CreditKind::Walk) {
        "walk"
    } else if has(CreditKind::HitByPitch) {
        "hit by pitch"
    } else if has(CreditKind::Single) {
        "single"
    } else if has(CreditKind::Double) {
        "double"
    } else if has(CreditKind::Triple) {
        "triple"
    } else if has(CreditKind::Strikeout) {
        "strikeout"
    } else if has(CreditKind::Error) {
        "reaches on error"
    } else if e.delta.outs > 0 {
        "ground/contact out"
    } else {
        "unclassified play"
    };
    if e.kind == EventKind::RunnerOnly(RunnerEvent::StealAttempt) {
        format!("{} steals second", e.delta.moves[0].runner.player_id)
    } else {
        format!("{batter}: {play} ({} run(s))", e.delta.runs)
    }
}
pub fn present(
    log: &[EventTransition],
    input: &mut dyn BufRead,
    output: &mut dyn Write,
    autoplay: bool,
) -> io::Result<()> {
    for (index, t) in log.iter().enumerate() {
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
        let before = t.before();
        let after = t.after();
        writeln!(
            output,
            "Play {:02} | {} {} | {}",
            index + 1,
            half_name(before.half),
            before.inning,
            description(t)
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
    let stats =
        derive_accounting(log).map_err(|e| io::Error::other(format!("accounting: {e:?}")))?;
    writeln!(
        output,
        "\nFINAL: Away {} - Home {}",
        stats.runs[0], stats.runs[1]
    )?;
    writeln!(output, "Line score (observed innings):")?;
    write!(output, "Inning")?;
    for inning in stats.innings.keys() {
        write!(output, " {inning:2}")?;
    }
    writeln!(output)?;
    for (side, name) in ["Away", "Home"].into_iter().enumerate() {
        write!(output, "{name:6}")?;
        for runs in stats.innings.values() {
            match runs[side] {
                Some(r) => write!(output, " {r:2}")?,
                None => write!(output, "  -")?,
            }
        }
        writeln!(output, " | {}", stats.runs[side])?;
    }
    writeln!(output,"\nSupplied-credit statistics — not a complete official box; omitted scoring facts are not known zeroes.")?;
    writeln!(output, "Batter       PA  AB   H   R  BB   K  SB")?;
    for side in &stats.batters {
        for (name, b) in side {
            let ab = b.at_bats.map_or("?".into(), |v| v.to_string());
            writeln!(
                output,
                "{name:12} {:2} {ab:>3} {:3} {:3} {:3} {:3} {:3}",
                b.plate_appearances,
                b.credited_hits,
                b.credited_runs,
                b.credited_walks,
                b.credited_strikeouts,
                b.credited_steals
            )?;
        }
    }
    writeln!(output, "Pitcher          BF  IP   R   H  BB   K  ER")?;
    for side in &stats.pitchers {
        for (name, p) in side {
            let er = p.earned_runs.map_or("?".into(), |v| v.to_string());
            writeln!(
                output,
                "{name:16} {:2} {}.{} {:3} {:3} {:3} {:3} {er:>3}",
                p.batters_faced,
                p.outs_recorded / 3,
                p.outs_recorded % 3,
                p.runs_allowed,
                p.credited_hits_allowed,
                p.credited_walks_allowed,
                p.credited_strikeouts
            )?;
        }
    }
    Ok(())
}
