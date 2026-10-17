//! Text presentation of canonical snapshots; never an outcome/state reducer.
use bbsim_engine::{GameState, Half};
use std::{
    collections::BTreeMap,
    io::{self, Write},
};

pub fn write_state(
    output: &mut dyn Write,
    label: &str,
    state: &GameState,
    teams: &[String; 2],
    names: &BTreeMap<String, String>,
) -> io::Result<()> {
    let mut mask = 0;
    let mut runners = Vec::new();
    for (i, runner) in state.bases.iter().enumerate() {
        if let Some(runner) = runner {
            let name = names.get(&runner.player_id).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "missing runner display name")
            })?;
            mask |= 1 << i;
            runners.push(format!("{name} on {}", ["first", "second", "third"][i]));
        }
    }
    let bases = [
        "bases empty",
        "runner on first",
        "runner on second",
        "runners on first and second",
        "runner on third",
        "runners on first and third",
        "runners on second and third",
        "bases loaded",
    ][mask];
    let details = if runners.is_empty() {
        String::new()
    } else {
        format!(" ({})", runners.join("; "))
    };
    let half = match state.half {
        Half::Top => "Top",
        Half::Bottom => "Bottom",
    };
    let final_label = if state.over { "FINAL | " } else { "" };
    writeln!(
        output,
        "{label}: {} {} — {} {} | {final_label}{half} {} | {} out{} | {bases}{details}",
        teams[0],
        state.score[0],
        teams[1],
        state.score[1],
        state.inning,
        state.outs,
        if state.outs == 1 { "" } else { "s" }
    )
}
