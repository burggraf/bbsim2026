//! Opt-in human inspection; immutable native snapshots, not a canonical wire codec.
use bbsim_engine::{
    AirContact, Base, CreditKind, Destination, EventKind, EventTransition, GameState,
};
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

fn base_name(base: Base) -> &'static str {
    match base {
        Base::Plate => "home plate",
        Base::First => "first",
        Base::Second => "second",
        Base::Third => "third",
    }
}

fn compact(state: &GameState) -> String {
    let occupied = state
        .bases
        .iter()
        .zip([Base::First, Base::Second, Base::Third])
        .filter_map(|(runner, base)| {
            runner
                .as_ref()
                .map(|r| format!("{} at {}", r.player_id, base_name(base)))
        })
        .collect::<Vec<_>>();
    let bases = if occupied.is_empty() {
        "bases empty".into()
    } else {
        occupied.join(", ")
    };
    let clock = if state.over {
        "Final".into()
    } else {
        format!(
            "{:?} {} · {} {}",
            state.half,
            state.inning,
            state.outs,
            if state.outs == 1 { "out" } else { "outs" }
        )
    };
    format!(
        "{clock} · Away {}–Home {} · {bases}",
        state.score[0], state.score[1]
    )
}

fn call(t: &EventTransition, positions: &std::collections::BTreeMap<String, &str>) -> String {
    let e = t.event();
    let location = |player: &str| {
        positions
            .get(player)
            .map(|p| format!(" to {p}"))
            .unwrap_or_default()
    };
    let batter = e
        .participants
        .batter
        .as_deref()
        .unwrap_or("batter not recorded");
    let credited = |player: &str, kind| {
        e.delta
            .credits
            .iter()
            .any(|c| c.player_id == player && c.kind == kind)
    };
    let result = if let Some(f) = &e.ordinary_ground_throw {
        format!(
            "ground ball{}{}, the throw to {} — {batter} is out at first!",
            location(&f.thrower_id),
            if positions.contains_key(&f.thrower_id) {
                format!(", fielded by {}", f.thrower_id)
            } else {
                format!(" to {}", f.thrower_id)
            },
            f.receiver_id
        )
    } else if let Some(f) = &e.ordinary_air_catch {
        let contact = match f.contact {
            AirContact::Fly => "fly ball",
            AirContact::LineDrive => "line drive",
            AirContact::PopUp => "pop-up",
        };
        format!(
            "{contact}{}{} caught by {} — {batter} is out!",
            location(&f.fielder_id),
            if positions.contains_key(&f.fielder_id) {
                ","
            } else {
                ""
            },
            f.fielder_id
        )
    } else if e.caught_strikeout.is_some() {
        format!("strike three! {batter} is out.")
    } else if let Some(f) = &e.muffed_ground_ball {
        format!(
            "ground ball{}, mishandled by {}! {batter} reaches first on the error.",
            location(&f.fielder_id),
            f.fielder_id
        )
    } else if e.kind == EventKind::OutOfParkHomeRun {
        "home run!".into()
    } else if credited(batter, CreditKind::Single) {
        "a single!".into()
    } else if credited(batter, CreditKind::Double) {
        "a double!".into()
    } else if credited(batter, CreditKind::Triple) {
        "a triple!".into()
    } else if credited(batter, CreditKind::OrdinaryWalk) {
        format!("ball four! {batter} takes first.")
    } else if credited(batter, CreditKind::HitByPitch) {
        format!("hit by pitch. {batter} takes first.")
    } else {
        "recorded play; see full event facts for the result.".into()
    };
    let mut text = format!("{} to {batter}: {result}", e.participants.pitcher);
    // Unknown causal sequences remain unknown; supplied credits can still name fielders.
    if e.ordinary_ground_throw.is_none()
        && e.ordinary_air_catch.is_none()
        && e.caught_strikeout.is_none()
        && e.muffed_ground_ball.is_none()
    {
        let mut fielders = Vec::new();
        for c in &e.delta.credits {
            let role = match c.kind {
                CreditKind::Putout => "putout",
                CreditKind::Assist => "assist",
                CreditKind::Error => "error",
                CreditKind::PassedBall => "passed ball",
                _ => continue,
            };
            let label = format!("{} ({role})", c.player_id);
            if !fielders.contains(&label) {
                fielders.push(label);
            }
        }
        if !fielders.is_empty() {
            text.push_str(&format!(" Fielders: {}.", fielders.join(", ")));
        }
    }
    // Vec order does not establish play chronology. Describe each supplied disposition independently.
    for movement in e.delta.moves.iter().filter(|m| m.from != Base::Plate) {
        let id = &movement.runner.player_id;
        let from = base_name(movement.from);
        let action = match movement.to {
            Destination::Home if credited(id, CreditKind::Run) => format!("scores from {from}"),
            Destination::Home => format!("crosses home from {from}; run not counted"),
            Destination::Out => format!("is out advancing from {from}"),
            Destination::First => format!("goes from {from} to first"),
            Destination::Second => format!("goes from {from} to second"),
            Destination::Third => format!("goes from {from} to third"),
        };
        text.push_str(&format!(" {id} {action}."));
    }
    for (runner, base) in t
        .before()
        .bases
        .iter()
        .zip([Base::First, Base::Second, Base::Third])
    {
        if let Some(r) = runner {
            if !e
                .delta
                .moves
                .iter()
                .any(|m| m.runner.player_id == r.player_id)
            {
                let action = if t.before().outs + e.delta.outs == 3 {
                    "is stranded at"
                } else {
                    "holds at"
                };
                text.push_str(&format!(" {} {action} {}.", r.player_id, base_name(base)));
            }
        }
    }
    if t.before().outs + e.delta.outs == 3 {
        text.push_str(" Side retired.");
    }
    if t.after().over {
        text.push_str(" The game is over.");
    }
    text
}

/// Concise native presentation alongside (not replacing) the immutable debug snapshots.
pub fn write_commentary(log: &[EventTransition], output: &mut dyn Write) -> io::Result<()> {
    write_commentary_named(log, output, &std::collections::BTreeMap::new())
}

/// Optional token-exact display lookup; snapshots retain canonical identities.
pub fn write_commentary_named(
    log: &[EventTransition],
    output: &mut dyn Write,
    names: &std::collections::BTreeMap<String, String>,
) -> io::Result<()> {
    write_commentary_with_positions(log, output, names, &std::collections::BTreeMap::new())
}

/// Positions are supplied lineup assignments, not measured contact locations.
pub fn write_commentary_with_positions(
    log: &[EventTransition],
    output: &mut dyn Write,
    names: &std::collections::BTreeMap<String, String>,
    positions: &std::collections::BTreeMap<String, &str>,
) -> io::Result<()> {
    let display = |text: &str| {
        if names.is_empty() {
            return text.to_string();
        }
        let mut result = String::new();
        for part in
            text.split_inclusive(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        {
            let token = part
                .trim_end_matches(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'));
            if let Some(name) = names.get(token) {
                result.push_str(name);
            } else {
                result.push_str(token);
            }
            result.push_str(&part[token.len()..]);
        }
        result
    };
    writeln!(output, "COMMENTARY FORMAT: radio-v1")?;
    for (i, t) in log.iter().enumerate() {
        writeln!(
            output,
            "CALL PLAY {}: {}",
            i + 1,
            display(&call(t, positions))
        )?;
        writeln!(
            output,
            "RECAP PLAY {}: Before: {} → After: {}",
            i + 1,
            display(&compact(t.before())),
            display(&compact(t.after()))
        )?;
    }
    Ok(())
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
