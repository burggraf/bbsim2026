//! Narration reads retained mechanisms and canonical deltas; it never generates a result.
use super::{
    full_game::{ContactDecision, Mechanism, Ordinary, RunnerDecision, StrikeoutDecision},
    full_simulation::Step,
    transcript,
};
use bbsim_engine::{Base, CreditKind, Destination, GameState, Half, PersonnelChange};
use std::{
    collections::BTreeMap,
    io::{self, Write},
};
fn name<'a>(names: &'a BTreeMap<String, String>, id: &str) -> io::Result<&'a str> {
    names
        .get(id)
        .map(String::as_str)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing actor display name"))
}
const POSITIONS: [&str; 9] = [
    "pitcher",
    "catcher",
    "first baseman",
    "second baseman",
    "third baseman",
    "shortstop",
    "left fielder",
    "center fielder",
    "right fielder",
];
const LOCATIONS: [&str; 9] = [
    "the pitcher",
    "the catcher",
    "first base",
    "second base",
    "third base",
    "shortstop",
    "left field",
    "center field",
    "right field",
];
fn base(from: Base) -> &'static str {
    match from {
        Base::Plate => "the plate",
        Base::First => "first",
        Base::Second => "second",
        Base::Third => "third",
    }
}
fn target(to: Destination) -> &'static str {
    match to {
        Destination::First => "first",
        Destination::Second => "second",
        Destination::Third => "third",
        Destination::Home => "home",
        Destination::Out => "out",
    }
}
fn matchup(
    output: &mut dyn Write,
    state: &GameState,
    names: &BTreeMap<String, String>,
) -> io::Result<()> {
    if state.over {
        return writeln!(output, "No next plate appearance — game over.");
    }
    let offense = state.half.offense();
    let batter = state.teams[offense]
        .batting_order
        .get(usize::from(state.teams[offense].next_batter))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid batting cursor"))?;
    writeln!(
        output,
        "Batter: {} | Pitcher: {}",
        name(names, batter)?,
        name(names, &state.teams[1 - offense].pitcher)?
    )
}
fn box_count(value: Option<u32>) -> String {
    value.map_or_else(|| "?".into(), |n| n.to_string())
}
fn write_box_table(
    output: &mut dyn Write,
    heading: &str,
    headers: &[&str],
    rows: &[Vec<String>],
) -> io::Result<()> {
    let widths: Vec<_> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| {
            rows.iter()
                .map(|r| r[i].chars().count())
                .max()
                .unwrap_or(0)
                .max(h.len())
        })
        .collect();
    writeln!(output, "\n{heading}")?;
    for row in std::iter::once(headers.iter().map(|s| s.to_string()).collect::<Vec<_>>())
        .chain(rows.iter().cloned())
    {
        for (i, cell) in row.iter().enumerate() {
            let width = widths[i];
            if i == 0 {
                write!(output, "{cell:<width$}")?;
            } else {
                write!(output, " {cell:>width$}")?;
            }
        }
        writeln!(output)?;
    }
    Ok(())
}
fn write_box_note<'a>(
    output: &mut dyn Write,
    label: &str,
    values: impl Iterator<Item = (&'a str, Option<u32>)>,
    names: &BTreeMap<String, String>,
) -> io::Result<()> {
    let mut entries = Vec::new();
    for (id, value) in values {
        if value != Some(0) {
            entries.push(format!("{} ({})", name(names, id)?, box_count(value)));
        }
    }
    if !entries.is_empty() {
        writeln!(output, "{label}: {}.", entries.join(", "))?;
    }
    Ok(())
}

pub fn write_box_score(
    output: &mut dyn Write,
    steps: &[Step],
    teams: &[String; 2],
    names: &BTreeMap<String, String>,
) -> io::Result<()> {
    let log: Vec<_> = steps.iter().map(|step| step.transition().clone()).collect();
    let stats = bbsim_engine::accounting::derive_accounting(&log).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("box score accounting: {e:?}"),
        )
    })?;
    if !stats.complete_game {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "box score requires a complete game",
        ));
    }
    // Buffer the entire box so missing identities cannot leave a partial table.
    let mut text = Vec::new();
    writeln!(
        text,
        "\nBox score\nRecorded game statistics; ? = unresolved scoring."
    )?;
    for (side, team) in teams.iter().enumerate() {
        let mut batters = Vec::new();
        for slot in 0..9 {
            for state in std::iter::once(log[0].before()).chain(log.iter().map(|t| t.after())) {
                let id = &state.teams[side].batting_order[slot];
                if !batters.contains(id) {
                    batters.push(id.clone());
                }
            }
        }
        let mut rows = Vec::new();
        let mut batting = Vec::new();
        for id in &batters {
            let b = stats.batters[side].get(id).cloned().unwrap_or_default();
            let cells = [
                Some(b.plate_appearances),
                b.at_bats,
                Some(b.credited_runs),
                b.classified_hits,
                b.classified_rbi,
                Some(b.credited_walks),
                Some(b.credited_strikeouts),
            ];
            let mut row = vec![name(names, id)?.to_string()];
            row.extend(cells.map(box_count));
            rows.push(row);
            batting.push(cells);
        }
        let mut totals = vec!["Totals".into()];
        totals.extend((0..7).map(|i| box_count(batting.iter().map(|b| b[i]).sum())));
        rows.push(totals);
        write_box_table(
            &mut text,
            &format!("{team} — Batting"),
            &["Batter", "PA", "AB", "R", "H", "RBI", "BB", "SO"],
            &rows,
        )?;
        let notes: Vec<_> = batters
            .iter()
            .map(|id| {
                let b = stats.batters[side].get(id).cloned().unwrap_or_default();
                let h = b.credited_hit_breakdown.as_ref();
                (
                    id.as_str(),
                    [
                        h.map(|h| h.doubles),
                        h.map(|h| h.triples),
                        h.map(|h| h.home_runs),
                        b.credited_intentional_walks,
                        Some(b.credited_hit_by_pitch),
                        Some(b.credited_sacrifice_bunts),
                        Some(b.credited_sacrifice_flies),
                        Some(b.credited_grounded_into_double_plays),
                        Some(b.credited_steals),
                        Some(b.credited_caught_stealing),
                        Some(b.credited_interference_awards),
                    ],
                )
            })
            .collect();
        for (i, label) in [
            "2B", "3B", "HR", "IBB", "HBP", "SH", "SF", "GDP", "SB", "CS", "CI",
        ]
        .iter()
        .enumerate()
        {
            write_box_note(
                &mut text,
                label,
                notes.iter().map(|(id, cells)| (*id, cells[i])),
                names,
            )?;
        }
        write_box_note(
            &mut text,
            "E",
            stats.fielders[side]
                .iter()
                .map(|(id, f)| (id.as_str(), Some(f.credited_errors))),
            names,
        )?;
        write_box_note(
            &mut text,
            "PB",
            stats.fielders[side]
                .iter()
                .map(|(id, f)| (id.as_str(), Some(f.credited_passed_balls))),
            names,
        )?;

        let mut pitchers = Vec::new();
        for state in std::iter::once(log[0].before()).chain(log.iter().map(|t| t.after())) {
            let id = &state.teams[side].pitcher;
            if !pitchers.contains(id) {
                pitchers.push(id.clone());
            }
        }
        let mut rows = Vec::new();
        let mut pitching = Vec::new();
        for id in &pitchers {
            let p = stats.pitchers[side].get(id).cloned().unwrap_or_default();
            let cells = [
                Some(p.outs_recorded),
                p.classified_hits_allowed,
                Some(p.runs_allowed),
                p.earned_runs,
                Some(p.credited_walks_allowed),
                Some(p.credited_strikeouts),
                p.credited_hit_breakdown_allowed
                    .as_ref()
                    .map(|h| h.home_runs),
                Some(p.batters_faced),
            ];
            let mut row = vec![
                name(names, id)?.to_string(),
                format!("{}.{}", p.outs_recorded / 3, p.outs_recorded % 3),
            ];
            row.extend(cells[1..].iter().copied().map(box_count));
            rows.push(row);
            pitching.push(cells);
        }
        let outs: u32 = pitching.iter().map(|p| p[0].unwrap()).sum();
        let mut totals = vec!["Totals".into(), format!("{}.{}", outs / 3, outs % 3)];
        totals.extend((1..8).map(|i| box_count(pitching.iter().map(|p| p[i]).sum())));
        rows.push(totals);
        write_box_table(
            &mut text,
            &format!("{team} — Pitching"),
            &["Pitcher", "IP", "H", "R", "ER", "BB", "SO", "HR", "BF"],
            &rows,
        )?;
        write_box_note(
            &mut text,
            "WP",
            stats.pitchers[side]
                .iter()
                .map(|(id, p)| (id.as_str(), Some(p.credited_wild_pitches))),
            names,
        )?;
        write_box_note(
            &mut text,
            "BK",
            stats.pitchers[side]
                .iter()
                .map(|(id, p)| (id.as_str(), Some(p.credited_balks))),
            names,
        )?;
    }
    output.write_all(&text)
}

pub fn write_line_score(
    output: &mut dyn Write,
    steps: &[Step],
    teams: &[String; 2],
) -> io::Result<()> {
    let log: Vec<_> = steps.iter().map(|step| step.transition().clone()).collect();
    let stats = bbsim_engine::accounting::derive_accounting(&log).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("line score accounting: {e:?}"),
        )
    })?;
    if !stats.complete_game {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "line score requires a complete game",
        ));
    }
    let hits: [u32; 2] =
        std::array::from_fn(|side| stats.batters[side].values().map(|b| b.credited_hits).sum());
    let errors: [u32; 2] = std::array::from_fn(|side| {
        stats.fielders[side]
            .values()
            .map(|f| f.credited_errors)
            .sum()
    });
    let team_width = teams
        .iter()
        .map(|team| team.chars().count())
        .max()
        .unwrap_or(0)
        .max(4);
    let cell_width = stats
        .runs
        .into_iter()
        .chain(hits)
        .chain(errors)
        .chain(stats.innings.keys().map(|inning| u32::from(*inning)))
        .max()
        .unwrap_or(0)
        .to_string()
        .len()
        .max(2);
    let mut text = Vec::new();
    writeln!(text, "\nLine score")?;
    write!(text, "{:<team_width$}", "Team")?;
    for inning in stats.innings.keys() {
        write!(text, " {inning:>cell_width$}")?;
    }
    write!(text, " |")?;
    for label in ["R", "H", "E"] {
        write!(text, " {label:>cell_width$}")?;
    }
    writeln!(text)?;
    for (side, team) in teams.iter().enumerate() {
        write!(text, "{team:<team_width$}")?;
        for runs in stats.innings.values() {
            let value = runs[side].map_or_else(|| "X".into(), |r| r.to_string());
            write!(text, " {value:>cell_width$}")?;
        }
        write!(text, " |")?;
        for total in [stats.runs[side], hits[side], errors[side]] {
            write!(text, " {total:>cell_width$}")?;
        }
        writeln!(text)?;
    }
    output.write_all(&text)
}

pub fn write_radio_step(
    output: &mut dyn Write,
    number: usize,
    step: &Step,
    teams: &[String; 2],
    names: &BTreeMap<String, String>,
) -> io::Result<()> {
    write_step_mode(output, number, step, teams, names, true)
}
pub fn write_step(
    output: &mut dyn Write,
    number: usize,
    step: &Step,
    teams: &[String; 2],
    names: &BTreeMap<String, String>,
) -> io::Result<()> {
    write_step_mode(output, number, step, teams, names, false)
}
fn radio_heading(
    output: &mut dyn Write,
    state: &GameState,
    teams: &[String; 2],
    names: &BTreeMap<String, String>,
) -> io::Result<()> {
    writeln!(
        output,
        "\n{} of inning {} — {} batting; {} pitching.",
        match state.half {
            Half::Top => "Top",
            Half::Bottom => "Bottom",
        },
        state.inning,
        teams[state.half.offense()],
        name(names, &state.teams[1 - state.half.offense()].pitcher)?
    )
}
fn write_step_mode(
    output: &mut dyn Write,
    number: usize,
    step: &Step,
    teams: &[String; 2],
    names: &BTreeMap<String, String>,
    radio: bool,
) -> io::Result<()> {
    let t = step.transition();
    let actor = |names: &BTreeMap<String, String>,
                 fielders: &[String; 9],
                 id: &str|
     -> io::Result<String> {
        let position = fielders.iter().position(|p| p == id).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "mechanism actor absent from positional defense",
            )
        })?;
        let who = name(names, id)?;
        let credited = t.event().delta.credits.iter().any(|c| {
            c.player_id == id && matches!(c.kind, CreditKind::Putout | CreditKind::Assist)
        });
        Ok(if radio && !credited {
            format!("the {}", POSITIONS[position])
        } else {
            format!("{} {who}", POSITIONS[position])
        })
    };
    let ordinary_walk = t
        .event()
        .delta
        .credits
        .iter()
        .any(|c| c.kind == CreditKind::OrdinaryWalk);
    // Buffer one complete event so missing identities never leak raw IDs or half a play.
    let mut text = Vec::new();
    if radio {
        if number == 1 {
            radio_heading(&mut text, t.before(), teams, names)?;
        }
        if t.event()
            .delta
            .credits
            .iter()
            .any(|c| c.kind == CreditKind::PlateAppearance)
        {
            let batter = t.event().participants.batter.as_ref().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "plate appearance lacks batter")
            })?;
            write!(
                text,
                "{}{}",
                name(names, batter)?,
                if ordinary_walk { " " } else { ": " }
            )?;
        }
    } else {
        transcript::write_state(&mut text, "Before", t.before(), teams, names)?;
        matchup(&mut text, t.before(), names)?;
        write!(text, "Play {number}: ")?;
    }
    match step {
        Step::PitchingChange(_) => {
            for change in &t.event().delta.changes {
                match change {
                    PersonnelChange::Pitcher {
                        outgoing, incoming, ..
                    } => write!(
                        text,
                        "Pitching change: {} replaces {}. ",
                        name(names, incoming)?,
                        name(names, outgoing)?
                    )?,
                    PersonnelChange::BattingSlot { slot, incoming, .. } => write!(
                        text,
                        "{} takes batting slot {}. ",
                        name(names, incoming)?,
                        slot + 1
                    )?,
                    _ => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "unexpected managerial change",
                        ))
                    }
                }
            }
            writeln!(text)?;
        }
        Step::Play(play) => {
            match play.mechanism() {
                Mechanism::Hit { decision, fielders } => {
                    let result=t.event().delta.credits.iter().find_map(|c|match c.kind {CreditKind::Single=>Some("Single"),CreditKind::Double=>Some("Double"),CreditKind::Triple=>Some("Triple"),_=>None}).ok_or_else(||io::Error::new(io::ErrorKind::InvalidData,"hit mechanism lacks native hit value"))?;
                    if radio { writeln!(text,"{result} to {}.",LOCATIONS[usize::from(decision.position)-1])?; }
                    else { writeln!(text,"{result} toward {}, fielded there.",actor(names,fielders,&fielders[usize::from(decision.position)-1])?)?; }
                    if let Some(facts)=t.hit_runner_tag_facts() {
                        writeln!(text,"{} throws to {} for the tag at {}; the hitter attained the credited base before the tag.",actor(names,fielders,&facts.thrower_id)?,actor(names,fielders,&facts.tagger_id)?,target(facts.attempted_base))?;
                    }
                    if t.walkoff_hit_facts().is_some() {writeln!(text,"The winning run ends the game; no later throw or out is generated.")?;}
                }
                Mechanism::UncaughtStrikeout { decision, fielders } => {
                    write!(text,"Strike three is not caught by {}. ",actor(names,fielders,&fielders[1])?)?;
                    if t.event().delta.credits.iter().any(|c|c.kind==CreditKind::WildPitch) {write!(text,"It is a wild pitch. ")?;}
                    if t.event().delta.credits.iter().any(|c|c.kind==CreditKind::PassedBall) {write!(text,"It is a passed ball charged to {}. ",actor(names,fielders,&fielders[1])?)?;}
                    match decision {
                        StrikeoutDecision::Automatic=>writeln!(text,"First was occupied with fewer than two outs: the batter is automatically out; runners hold.")?,
                        StrikeoutDecision::SafeReach { .. }=>writeln!(text,"The batter reaches first safely on the uncaught third strike.")?,
                        StrikeoutDecision::RecoveredBeforeFirst { .. }|StrikeoutDecision::RecoveredBeforeFirstAfterAdvances { .. }=>{
                            if matches!(decision,StrikeoutDecision::RecoveredBeforeFirstAfterAdvances { .. }) {write!(text,"Runners complete their advances before the recovery throw. ")?;}
                            let receiver=t.event().delta.credits.iter().find(|c|c.kind==CreditKind::Putout).ok_or_else(||io::Error::new(io::ErrorKind::InvalidData,"recovered K lacks named putout"))?;
                            write!(text,"{} throws to {} to retire the batter before first.",actor(names,fielders,&fielders[1])?,actor(names,fielders,&receiver.player_id)?)?;
                            if matches!(decision,StrikeoutDecision::RecoveredBeforeFirst { .. }) {write!(text," The runners hold.")?;}
                            writeln!(text)?;
                        }
                    }
                }
                Mechanism::GroundOut{fielder,receiver,position}=>{
                    write!(text,"Ground ball to {} {}",POSITIONS[usize::from(*position)-1],name(names,fielder)?)?;
                    if fielder==receiver {writeln!(text,"; steps on first for the unassisted out.")?;}
                    else {writeln!(text,"; throw to first baseman {} at first for the out.",name(names,receiver)?)?;}
                }
                Mechanism::Ordinary{choice,fielders}=>{
                    let direction=|p: u8| -> io::Result<String> {actor(names,fielders,&fielders[usize::from(p)-1])};
                    match choice {
                        Ordinary::CalledStrikeout=>writeln!(text,"Called strike three, caught by {}.",direction(2)?)?,
                        Ordinary::SwingingStrikeout=>writeln!(text,"Swing and a miss for strike three, caught by {}.",direction(2)?)?,
                        Ordinary::Walk=>writeln!(text,"{}",if radio {"walks."} else {"Ball four; an ordinary walk."})?,
                        Ordinary::IntentionalWalk=>writeln!(text,"The manager orders an intentional walk.")?,
                        Ordinary::Hbp=>writeln!(text,"Hit by the pitch; awarded first.")?,
                        Ordinary::Single(p)|Ordinary::Double(p)|Ordinary::Triple(p)=>{
                            let credits=&t.event().delta.credits;
                            let result=if credits.iter().any(|c|c.kind==CreditKind::Triple) {"Triple"} else if credits.iter().any(|c|c.kind==CreditKind::Double) {"Double"} else {"Single"};
                            if radio { writeln!(text,"{result} to {}.",LOCATIONS[usize::from(*p)-1])?; }
                            else { writeln!(text,"{result} toward {}, fielded there.",direction(*p)?)?; }
                        }
                        Ordinary::HomeRun(p)=>if radio { writeln!(text,"Home run to {}.",LOCATIONS[usize::from(*p)-1])?; } else { writeln!(text,"Home run over the fence toward {}.",direction(*p)?)?; },
                        Ordinary::FlyOut(p)=>writeln!(text,"Fly ball caught by {}.",direction(*p)?)?,
                        Ordinary::LineOut(p)=>writeln!(text,"Line drive caught by {}.",direction(*p)?)?,
                        Ordinary::PopOut(p)=>writeln!(text,"Popup caught by {}.",direction(*p)?)?,
                        Ordinary::ThrowingError(p)=>writeln!(text,"Ground ball fielded by {}; wild throw toward {} covering first, throwing error; batter reaches first.",direction(*p)?,direction(if *p==3 {1} else {3})?)?,
                        Ordinary::FieldingError(p)=>writeln!(text,"Ground ball muffed by {}; fielding error, batter reaches first.",direction(*p)?)?,
                        Ordinary::CatcherInterference=>writeln!(text,"Catcher interference by {}; first-base award and catcher error.",direction(2)?)?,
                    }
                }
                Mechanism::AirAdvance{contact,position,origin,caught,fielders}=>{
                    let i=match origin {Base::First=>0,Base::Second=>1,Base::Third=>2,Base::Plate=>return Err(io::Error::new(io::ErrorKind::InvalidData,"invalid air origin"))};
                    let destination=[Destination::Second,Destination::Third,Destination::Home][i];
                    let fielder=&fielders[usize::from(*position)-1];
                    let receiver=&fielders[[3,4,1][i]];
                    let runner=t.before().bases[i].as_ref().ok_or_else(||io::Error::new(io::ErrorKind::InvalidData,"missing air runner"))?;
                    write!(text,"{} caught by {}; {} makes a legal retouch and attempts {}. ",match contact {bbsim_engine::AirContact::Fly=>"Fly ball",bbsim_engine::AirContact::LineDrive=>"Line drive",bbsim_engine::AirContact::PopUp=>"Popup"},actor(names,fielders,fielder)?,name(names,&runner.player_id)?,target(destination))?;
                    if *caught {
                        if fielder==receiver {writeln!(text,"{} keeps the ball and tags the runner at {}. Double play.",actor(names,fielders,fielder)?,target(destination))?;}
                        else {writeln!(text,"{} throws to {} for the tag at {}. Double play.",actor(names,fielders,fielder)?,actor(names,fielders,receiver)?,target(destination))?;}
                    } else {writeln!(text,"{} reaches {} safely after the catch.",name(names,&runner.player_id)?,target(destination))?;}
                },
                Mechanism::ForceChoice{fielder,receiver,position,pivot_position,failed_bunt}=>writeln!(text,"{} to {} {}; throw to {} {} for the force at second, batter reaches on a fielder's choice.",if *failed_bunt {"Failed sacrifice bunt"} else {"Ground ball"},POSITIONS[usize::from(*position)-1],name(names,fielder)?,POSITIONS[usize::from(*pivot_position)-1],name(names,receiver)?)?,
                Mechanism::Contact{decision,fielders}=>match decision {
                    ContactDecision::CaughtSacrificeFly{catch,..}=>writeln!(text,"Fly ball caught by {}; sacrifice fly with a legal retouch and run after the catch.",actor(names,fielders,&catch.fielder_id)?)?,
                    ContactDecision::SacrificeBunt(facts)=>writeln!(text,"Sacrifice bunt fielded by {}; throw to {} at first retires the batter as the first-base runner advances.",actor(names,fielders,&facts.thrower_id)?,actor(names,fielders,&facts.receiver_id)?)?,
                    ContactDecision::GroundForceDoublePlay(facts)=>writeln!(text,"Ground ball to {}; throw to {} at second for the force, relay to {} at first; double play.",actor(names,fielders,&facts.initial_fielder_id)?,actor(names,fielders,&facts.pivot_id)?,actor(names,fielders,&facts.first_receiver_id)?)?,
                },
                Mechanism::Runner{decision,fielders}=>match decision {
                    RunnerDecision::Steal{target,caught,..}=>{
                        let receiver=match target {Destination::Second=>3,Destination::Third=>4,Destination::Home=>1,_=>return Err(io::Error::new(io::ErrorKind::InvalidData,"invalid steal target"))};
                        if *caught && *target == Destination::Home {writeln!(text,"Steal attempt; {} tags the runner at home; caught stealing.",actor(names,fielders,&fielders[1])?)?;}
                        else if *caught {writeln!(text,"Steal attempt; {} throws to {} at {} for the tag; caught stealing.",actor(names,fielders,&fielders[1])?,actor(names,fielders,&fielders[receiver])?,self::target(*target))?;}
                        else {writeln!(text,"Runner steals {}; no retirement.",self::target(*target))?;}
                    }
                    RunnerDecision::Pickoff{origin,caught}=>{
                        if *caught {writeln!(text,"{} throws to {} at {} for the pickoff tag.",actor(names,fielders,&fielders[0])?,actor(names,fielders,&fielders[match origin {Base::First=>2,Base::Second=>3,Base::Third=>4,_=>return Err(io::Error::new(io::ErrorKind::InvalidData,"invalid pickoff origin"))}])?,base(*origin))?;}
                        else {writeln!(text,"Pickoff attempt at {}; runner returns safely, no error.",base(*origin))?;}
                    }
                    RunnerDecision::WildPitch=>writeln!(text,"Wild pitch past {}; each runner advances one base.",actor(names,fielders,&fielders[1])?)?,
                    RunnerDecision::PassedBall=>writeln!(text,"Passed ball charged to {}; each runner advances one base.",actor(names,fielders,&fielders[1])?)?,
                    RunnerDecision::Balk=>writeln!(text,"Balk by {}; play stopped, each runner awarded one base.",actor(names,fielders,&fielders[0])?)?,
                },
            }
            let third_out = t.before().outs + t.event().delta.outs == 3;
            for movement in &t.event().delta.moves {
                let who = name(names, &movement.runner.player_id)?;
                if radio
                    && movement.from == Base::Plate
                    && matches!(
                        movement.to,
                        Destination::First | Destination::Second | Destination::Third
                    )
                {
                    continue;
                }
                match movement.to {
                    Destination::Out => {
                        let double_play = matches!(
                            play.mechanism(),
                            Mechanism::Contact {
                                decision: ContactDecision::GroundForceDoublePlay(_),
                                ..
                            } | Mechanism::AirAdvance { caught: true, .. }
                        );
                        if !radio || (movement.from != Base::Plate && !double_play) {
                            writeln!(text, "  {who} is out (from {}).", base(movement.from))?;
                        }
                    }
                    Destination::Home => {
                        let counted = t.event().delta.credits.iter().any(|c| {
                            c.kind == CreditKind::Run && c.player_id == movement.runner.player_id
                        });
                        writeln!(
                            text,
                            "  {who} {}.",
                            if counted {
                                "scores"
                            } else {
                                "crosses home; run does not count"
                            }
                        )?;
                    }
                    to => {
                        let holds = base(movement.from) == target(to);
                        write!(
                            text,
                            "  {who} {} {}",
                            if movement.from == Base::Plate {
                                "reaches"
                            } else if holds {
                                if third_out {
                                    "is stranded at"
                                } else {
                                    "holds at"
                                }
                            } else {
                                "advances to"
                            },
                            target(to)
                        )?;
                        if third_out && !holds {
                            write!(text, "; is stranded at {}", target(to))?;
                        }
                        writeln!(text, ".")?;
                    }
                }
            }
            for (i, runner) in t.before().bases.iter().enumerate() {
                if let Some(runner) = runner {
                    if !t
                        .event()
                        .delta
                        .moves
                        .iter()
                        .any(|m| m.runner.player_id == runner.player_id)
                    {
                        writeln!(
                            text,
                            "  {} {} {}.",
                            name(names, &runner.player_id)?,
                            if third_out {
                                "is stranded at"
                            } else {
                                "holds at"
                            },
                            ["first", "second", "third"][i]
                        )?;
                    }
                }
            }
        }
    }
    if radio {
        let before = t.before();
        let after = t.after();
        if after.over {
            writeln!(
                text,
                "Final: {} {}, {} {}.",
                teams[0], after.score[0], teams[1], after.score[1]
            )?;
        } else if before.inning != after.inning || before.half != after.half {
            writeln!(
                text,
                "Side retired. {} {}, {} {}.",
                teams[0], after.score[0], teams[1], after.score[1]
            )?;
            radio_heading(&mut text, after, teams, names)?;
        } else {
            if before.score != after.score {
                writeln!(
                    text,
                    "{} {}, {} {}.",
                    teams[0], after.score[0], teams[1], after.score[1]
                )?;
            }
            if t.event().delta.outs > 0 {
                writeln!(
                    text,
                    "{} away.",
                    if after.outs == 1 { "One" } else { "Two" }
                )?;
            }
        }
    } else {
        transcript::write_state(&mut text, "After", t.after(), teams, names)?;
        matchup(&mut text, t.after(), names)?;
        writeln!(text)?;
    }
    output.write_all(&text)
}
