//! Opt-in human inspection; immutable native snapshots, not a canonical wire codec.
use bbsim_engine::{EventTransition, GameState};
use std::io::{self, Write};

fn summary(state: &GameState) -> String {
    let bases = state.bases.each_ref().map(|runner| {
        runner.as_ref().map_or("empty".into(), |r| {
            format!("{} (charged to {})", r.player_id, r.responsible_pitcher)
        })
    });
    format!(
        "{:?} {} | Outs: {} | Away {} - Home {} | Bases: 1={} 2={} 3={} | Game over: {}",
        state.half,
        state.inning,
        state.outs,
        state.score[0],
        state.score[1],
        bases[0],
        bases[1],
        bases[2],
        state.over
    )
}

pub fn write(log: &[EventTransition], output: &mut dyn Write) -> io::Result<()> {
    writeln!(output, "TRACE FORMAT: engine-debug-v1")?;
    for (i, t) in log.iter().enumerate() {
        writeln!(output, "TRACE PLAY {} BEGIN", i + 1)?;
        writeln!(output, "Before state: {}", summary(t.before()))?;
        writeln!(output, "After state: {}", summary(t.after()))?;
        writeln!(
            output,
            "Before:\n{:#?}\nEvent:\n{:#?}\nAfter:\n{:#?}",
            t.before(),
            t.event(),
            t.after()
        )?;
        writeln!(output, "TRACE PLAY {} END", i + 1)?;
    }
    Ok(())
}
