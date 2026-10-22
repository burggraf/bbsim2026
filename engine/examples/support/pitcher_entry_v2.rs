//! Shared v2 unused-pitcher selection and immutable per-decision evidence.
use super::pitcher_policy_v2::{relief_debt, relief_rank, Clock, Rank, ReliefUsage};
use bbsim_engine::{EventKind, EventTransition, GameState, PersonnelChange, PitcherRole};
use std::collections::BTreeSet;

pub const POLICY: &str = "season-pool-pitcher-usage-v2";
pub const MANAGER: &str = "season-pool-workload-context-2019-v2";

#[derive(Clone, Debug)]
pub struct Candidate {
    pub id: String,
    pub starter: bool,
    pub usage: ReliefUsage,
    pub pa_sum: u32,
    pub outings: u32,
}

#[derive(Clone, Debug)]
struct Evaluated {
    profile: Candidate,
    exclusion: &'static str,
    rank: Option<Rank>,
    position: Option<usize>,
}

pub type ExhaustionKey = (usize, String, Vec<(String, &'static str)>, bool);

#[derive(Clone, Debug)]
pub struct Selection {
    before: GameState,
    side: usize,
    clock: Clock,
    candidates: Vec<Evaluated>,
    chosen: Option<String>,
}

impl Selection {
    pub fn chosen(&self) -> Option<&str> {
        self.chosen.as_deref()
    }

    pub fn exhaustion_key(&self) -> Result<ExhaustionKey, String> {
        let pitcher = self.before.teams[self.side]
            .pitcher
            .active_id()
            .ok_or("v2 exhaustion requires an active pitcher")?;
        let eligibility = self
            .candidates
            .iter()
            .map(|candidate| (candidate.profile.id.clone(), candidate.exclusion))
            .collect();
        let score_gap =
            i64::from(self.before.score[self.side]) - i64::from(self.before.score[1 - self.side]);
        Ok((
            self.side,
            pitcher.to_owned(),
            eligibility,
            self.before.inning <= 5 || score_gap.unsigned_abs() >= 4,
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    Hook,
    PhProbe,
    PhRestore,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Budget {
    pub word: u64,
    pub sample_index: u32,
    pub base: u32,
    pub scale: u32,
    pub lead_extension: u32,
    pub no_hit_floor: Option<u64>,
    pub final_budget: u64,
}

fn actor(id: &str) -> bool {
    (1..=64).contains(&id.len())
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

pub fn select(
    before: &GameState,
    side: usize,
    clock: Clock,
    profiles: &[Candidate],
) -> Result<Selection, String> {
    if side > 1 || before.inning == 0 || before.over {
        return Err("invalid v2 selection situation".into());
    }
    let mut ids = BTreeSet::new();
    if profiles.iter().filter(|p| p.starter).count() != 1
        || profiles.iter().any(|p| {
            !actor(&p.id)
                || !ids.insert(p.id.as_str())
                || (p.outings == 0 && (!p.starter || p.pa_sum != 0))
        })
    {
        return Err("invalid v2 cast candidate profiles".into());
    }
    let team = &before.teams[side];
    let mut candidates = Vec::new();
    for profile in profiles {
        let exclusion = if profile.starter {
            "starter"
        } else if team.pitcher.active_id() == Some(profile.id.as_str()) {
            "active"
        } else if team.removed.contains(&profile.id) {
            "removed"
        } else if !team.bench.contains(&profile.id) {
            "not_benched"
        } else {
            "eligible"
        };
        let rank = if profile.outings > 0 {
            Some(relief_rank(
                clock,
                profile.usage,
                profile.pa_sum,
                profile.outings,
                u32::from(before.inning),
                i64::from(before.score[side]) - i64::from(before.score[1 - side]),
            )?)
        } else {
            None
        };
        candidates.push(Evaluated {
            profile: profile.clone(),
            exclusion,
            rank,
            position: None,
        });
    }
    candidates.sort_by(|a, b| a.profile.id.cmp(&b.profile.id));
    let mut eligible = candidates
        .iter()
        .enumerate()
        .filter_map(|(i, c)| (c.exclusion == "eligible").then_some(i))
        .collect::<Vec<_>>();
    eligible.sort_by(|&a, &b| {
        candidates[b]
            .rank
            .expect("eligible rank")
            .compare(&candidates[a].rank.expect("eligible rank"))
            .then_with(|| candidates[a].profile.id.cmp(&candidates[b].profile.id))
    });
    for (position, &i) in eligible.iter().enumerate() {
        candidates[i].position = Some(position + 1);
    }
    let chosen = eligible.first().map(|&i| candidates[i].profile.id.clone());
    Ok(Selection {
        before: before.clone(),
        side,
        clock,
        candidates,
        chosen,
    })
}

pub fn header(game: &str, seed: u64, events: u32, decisions: u32) -> Result<String, String> {
    let b = game.as_bytes();
    if b.len() != 12
        || !b[..3].iter().all(u8::is_ascii_uppercase)
        || &b[3..7] != b"2019"
        || !b[7..].iter().all(u8::is_ascii_digit)
        || !matches!(b[11], b'0'..=b'2')
        || events == 0
    {
        return Err("invalid v2 trace identity/count".into());
    }
    let month = (b[7] - b'0') * 10 + b[8] - b'0';
    let day = (b[9] - b'0') * 10 + b[10] - b'0';
    let days = match month {
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => return Err("invalid v2 trace date".into()),
    };
    if day == 0 || day > days {
        return Err("invalid v2 trace date".into());
    }
    Ok(format!("format\tpitcher-entry-decisions-v2\ngame\t{game}\nseed\t{seed}\nseason_policy\t{POLICY}\nmanager\t{MANAGER}\nevents\t{events}\ndecisions\t{decisions}\n"))
}

fn hex(transition: &EventTransition) -> String {
    const DIGITS: &[u8] = b"0123456789abcdef";
    transition
        .to_bytes()
        .iter()
        .flat_map(|b| {
            [
                DIGITS[usize::from(b >> 4)] as char,
                DIGITS[usize::from(b & 15)] as char,
            ]
        })
        .collect()
}

pub fn encode_record(
    sequence: u32,
    event_index: u32,
    reason: Reason,
    selection: &Selection,
    budget: Option<&Budget>,
    log: &[EventTransition],
) -> Result<String, String> {
    let witness = log
        .get(event_index as usize)
        .ok_or("v2 trace event index out of bounds")?;
    if witness.before() != &selection.before {
        return Err("v2 trace before-state mismatch".into());
    }
    let before = &selection.before;
    let side = selection.side;
    let (label, applied) = match reason {
        Reason::Hook
            if side == 1 - before.half.offense()
                && before.teams[side].pitcher.active_id().is_some() =>
        {
            ("hook", true)
        }
        Reason::PhProbe if side == before.half.offense() && budget.is_none() => ("ph_probe", false),
        Reason::PhRestore
            if side == 1 - before.half.offense()
                && matches!(before.teams[side].pitcher, PitcherRole::Vacant { .. })
                && budget.is_none() =>
        {
            ("ph_restore", true)
        }
        _ => return Err("invalid v2 trace reason/side/budget".into()),
    };
    if matches!(reason, Reason::PhProbe) && witness.event().kind == EventKind::Substitution {
        let changes = &witness.event().delta.changes;
        if !changes
            .iter()
            .any(|change| matches!(change, PersonnelChange::PinchHitter { .. }))
            || changes.iter().any(|change| {
                matches!(
                    change,
                    PersonnelChange::AssignPitcher { .. } | PersonnelChange::Pitcher { .. }
                )
            })
        {
            return Err("v2 PH probe is not bound to a PH transition".into());
        }
    }
    let old_pitcher = before.teams[side].pitcher.active_id();
    let next_pitcher = witness.after().teams[side].pitcher.active_id();
    match (applied, selection.chosen()) {
        (true, Some(chosen))
            if witness.event().kind != EventKind::Substitution || next_pitcher != Some(chosen) =>
        {
            return Err("v2 trace applied entry mismatch".into());
        }
        (true, None) if next_pitcher != old_pitcher => {
            return Err("v2 trace exhaustion hides an entry".into());
        }
        (false, _) if next_pitcher.is_some() && next_pitcher != old_pitcher => {
            return Err("v2 trace probe hides an entry".into());
        }
        (false, None) if old_pitcher.is_some() && next_pitcher.is_none() => {
            return Err("v2 PH probe leaves a pitcher vacancy without a reserve".into());
        }
        _ => {}
    }
    let wire = hex(witness);
    let chosen = selection.chosen().unwrap_or("-");
    let side_label = ["Away", "Home"][side];
    let mut text = format!(
        "decision\t{sequence}\t{event_index}\t{side_label}\t{label}\t{chosen}\t{}\t{}\t{wire}\n",
        selection.clock.completed(),
        selection.clock.scheduled()
    );
    for c in &selection.candidates {
        let p = &c.profile;
        let target = p.usage.target().map_or("?".into(), |n| n.to_string());
        text.push_str(&format!(
            "candidate\t{sequence}\t{}\t{}\t{}\t{target}\t{}\t{}",
            p.id,
            c.exclusion,
            p.usage.used(),
            p.pa_sum,
            p.outings
        ));
        if let Some(rank) = c.rank {
            let (dn, dd) = rank.debt();
            let (qn, qd) = rank.clipped();
            let (sn, sd) = rank.score();
            debug_assert_eq!(rank.target_known(), p.usage.target().is_some());
            text.push_str(&format!(
                "\t{dn}\t{dd}\t{qn}\t{qd}\t{}\t{}\t{sn}\t{sd}\t{}\n",
                u8::from(rank.long()),
                u8::from(rank.matches()),
                c.position.map_or("-".into(), |n| n.to_string())
            ));
        } else {
            let (dn, dd) = relief_debt(selection.clock, p.usage);
            text.push_str(&format!("\t{dn}\t{dd}\t-\t-\t-\t-\t-\t-\t-\n"));
        }
    }
    if matches!(reason, Reason::Hook) {
        let b = budget.ok_or("missing evaluated v2 hook budget")?;
        let outgoing = before.teams[side]
            .pitcher
            .active_id()
            .ok_or("missing outgoing v2 pitcher")?;
        let starter = selection
            .candidates
            .iter()
            .find(|c| c.profile.id == outgoing)
            .ok_or("outgoing v2 pitcher absent from cast")?
            .profile
            .starter;
        let computed = ((u64::from(b.base) * u64::from(b.scale) / 100).max(1)
            + u64::from(b.lead_extension))
        .max(b.no_hit_floor.unwrap_or(0));
        if b.base == 0
            || !matches!(b.scale, 80 | 100)
            || !matches!(b.lead_extension, 0 | 3)
            || b.no_hit_floor == Some(0)
            || computed != b.final_budget
            || (!starter && (b.scale != 100 || b.lead_extension != 0 || b.no_hit_floor.is_some()))
        {
            return Err("invalid v2 evaluated budget".into());
        }
        let floor = b.no_hit_floor.map_or("-".into(), |n| n.to_string());
        let role = if starter { "start" } else { "relief" };
        text.push_str(&format!(
            "budget\t{sequence}\t{outgoing}\t{role}\t{}\t{}\t{}\t{}\t{}\t{floor}\t{}\n",
            b.word, b.sample_index, b.base, b.scale, b.lead_extension, b.final_budget
        ));
    }
    if applied && selection.chosen().is_some() {
        text.push_str(&format!("applied\t{sequence}\t{wire}\n"));
    }
    text.push_str(&format!(
        "enddecision\t{sequence}\t{}\n",
        selection.candidates.len()
    ));
    Ok(text)
}

#[derive(Clone, Debug)]
pub struct TraceDecision {
    pub event_index: u32,
    pub side: usize,
    pub reason: Reason,
    pub chosen: Option<String>,
    pub budget: Option<Budget>,
}

fn parse_u32(text: &str) -> Result<u32, String> {
    let value = text.parse::<u32>().map_err(|_| "invalid trace integer")?;
    if value.to_string() != text {
        return Err("noncanonical trace integer".into());
    }
    Ok(value)
}

fn parse_u64(text: &str) -> Result<u64, String> {
    let value = text.parse::<u64>().map_err(|_| "invalid trace integer")?;
    if value.to_string() != text {
        return Err("noncanonical trace integer".into());
    }
    Ok(value)
}

fn parse_reason(text: &str) -> Result<Reason, String> {
    match text {
        "hook" => Ok(Reason::Hook),
        "ph_probe" => Ok(Reason::PhProbe),
        "ph_restore" => Ok(Reason::PhRestore),
        _ => Err("unknown v2 trace decision reason".into()),
    }
}

fn decode_hex(text: &str) -> Result<Vec<u8>, String> {
    let (pairs, remainder) = text.as_bytes().as_chunks::<2>();
    if pairs.is_empty() || !remainder.is_empty() {
        return Err("invalid v2 transition hex".into());
    }
    pairs
        .iter()
        .map(|pair| {
            let digit = |b: u8| match b {
                b'0'..=b'9' => Ok(b - b'0'),
                b'a'..=b'f' => Ok(b - b'a' + 10),
                _ => Err("noncanonical v2 transition hex"),
            };
            Ok((digit(pair[0])? << 4) | digit(pair[1])?)
        })
        .collect()
}

fn parse_budget(line: &str, sequence: u32) -> Result<Budget, String> {
    let fields = line.split('\t').collect::<Vec<_>>();
    if fields.len() != 11 || fields[0] != "budget" || parse_u32(fields[1])? != sequence {
        return Err("invalid v2 trace budget row".into());
    }
    Ok(Budget {
        word: parse_u64(fields[4])?,
        sample_index: parse_u32(fields[5])?,
        base: parse_u32(fields[6])?,
        scale: parse_u32(fields[7])?,
        lead_extension: parse_u32(fields[8])?,
        no_hit_floor: if fields[9] == "-" {
            None
        } else {
            Some(parse_u64(fields[9])?)
        },
        final_budget: parse_u64(fields[10])?,
    })
}

fn side_index(side: bbsim_engine::TeamSide) -> usize {
    match side {
        bbsim_engine::TeamSide::Away => 0,
        bbsim_engine::TeamSide::Home => 1,
    }
}

/// Validate the complete decision stream against its pinned game inputs and event history.
pub fn validate_trace(
    text: &str,
    game: &str,
    seed: u64,
    log: &[EventTransition],
    profiles: &[Vec<Candidate>; 2],
    clocks: &[Clock; 2],
) -> Result<Vec<TraceDecision>, String> {
    if text.contains('\r')
        || !text.ends_with('\n')
        || log.is_empty()
        || log.len() > u32::MAX as usize
    {
        return Err("invalid v2 trace framing or event count".into());
    }
    for pair in log.windows(2) {
        if pair[0].after() != pair[1].before() {
            return Err("v2 trace event history is discontinuous".into());
        }
    }
    for transition in log {
        let bytes = transition.to_bytes();
        let decoded = EventTransition::from_bytes(&bytes)
            .map_err(|e| format!("invalid canonical v2 event transition: {e:?}"))?;
        if decoded != *transition {
            return Err("v2 event transition is not canonical".into());
        }
    }

    let lines = text[..text.len() - 1].split('\n').collect::<Vec<_>>();
    if lines.len() < 7 {
        return Err("truncated v2 trace header".into());
    }
    let event_count = lines[5]
        .strip_prefix("events\t")
        .ok_or("invalid v2 trace event-count row")?;
    let decision_count = lines[6]
        .strip_prefix("decisions\t")
        .ok_or("invalid v2 trace decision-count row")?;
    if parse_u32(event_count)? as usize != log.len() {
        return Err("v2 trace event count does not match history".into());
    }
    let decision_count = parse_u32(decision_count)?;
    let expected_header = header(game, seed, log.len() as u32, decision_count)?;
    if !text.starts_with(&expected_header) {
        return Err("v2 trace header does not match pinned run".into());
    }

    let mut cursor = 7;
    let mut decisions = Vec::new();
    let mut previous_event = 0;
    for sequence in 0..decision_count {
        let start = cursor;
        let row = lines.get(cursor).ok_or("truncated v2 decision row")?;
        let fields = row.split('\t').collect::<Vec<_>>();
        if fields.len() != 9 || fields[0] != "decision" || parse_u32(fields[1])? != sequence {
            return Err("invalid or unordered v2 decision row".into());
        }
        let event_index = parse_u32(fields[2])?;
        if event_index as usize >= log.len() || (sequence > 0 && event_index < previous_event) {
            return Err("invalid or unordered v2 decision event index".into());
        }
        previous_event = event_index;
        let side = match fields[3] {
            "Away" => 0,
            "Home" => 1,
            _ => return Err("invalid v2 trace side".into()),
        };
        let reason = parse_reason(fields[4])?;
        let witness_bytes = decode_hex(fields[8])?;
        let witness = EventTransition::from_bytes(&witness_bytes)
            .map_err(|e| format!("invalid v2 trace witness: {e:?}"))?;
        if witness.to_bytes() != log[event_index as usize].to_bytes()
            || fields[8] != hex(&log[event_index as usize])
        {
            return Err("v2 trace witness does not match indexed event".into());
        }
        let row_clock = Clock::read(&format!("{},{}", fields[6], fields[7]))?;
        if row_clock.completed() != clocks[side].completed()
            || row_clock.scheduled() != clocks[side].scheduled()
        {
            return Err("v2 trace clock does not match pinned club clock".into());
        }
        let selection = select(witness.before(), side, clocks[side], &profiles[side])?;
        let mut actual = String::new();
        actual.push_str(row);
        actual.push('\n');
        cursor += 1;
        for _ in 0..profiles[side].len() {
            let candidate = lines.get(cursor).ok_or("truncated v2 candidate rows")?;
            if !candidate.starts_with("candidate\t") {
                return Err("missing v2 trace candidate row".into());
            }
            actual.push_str(candidate);
            actual.push('\n');
            cursor += 1;
        }
        let budget = if lines
            .get(cursor)
            .is_some_and(|line| line.starts_with("budget\t"))
        {
            let line = lines[cursor];
            actual.push_str(line);
            actual.push('\n');
            cursor += 1;
            Some(parse_budget(line, sequence)?)
        } else {
            None
        };
        if lines
            .get(cursor)
            .is_some_and(|line| line.starts_with("applied\t"))
        {
            actual.push_str(lines[cursor]);
            actual.push('\n');
            cursor += 1;
        }
        let end = lines.get(cursor).ok_or("missing v2 decision terminator")?;
        let end_fields = end.split('\t').collect::<Vec<_>>();
        if end_fields.len() != 3
            || end_fields[0] != "enddecision"
            || parse_u32(end_fields[1])? != sequence
            || parse_u32(end_fields[2])? as usize != profiles[side].len()
        {
            return Err("invalid v2 decision terminator".into());
        }
        actual.push_str(end);
        actual.push('\n');
        cursor += 1;
        let expected = encode_record(
            sequence,
            event_index,
            reason,
            &selection,
            budget.as_ref(),
            log,
        )?;
        if actual != expected {
            return Err("v2 trace decision does not match recomputed selection".into());
        }
        decisions.push(TraceDecision {
            event_index,
            side,
            reason,
            chosen: selection.chosen().map(str::to_owned),
            budget,
        });
        debug_assert!(cursor > start);
    }
    if cursor != lines.len() {
        return Err("extra or uncounted v2 trace rows".into());
    }

    for (event_index, transition) in log.iter().enumerate() {
        for change in &transition.event().delta.changes {
            let install = match change {
                PersonnelChange::Pitcher { team, incoming, .. } => {
                    Some((side_index(*team), Reason::Hook, incoming.as_str()))
                }
                PersonnelChange::AssignPitcher { team, incoming } => {
                    Some((side_index(*team), Reason::PhRestore, incoming.as_str()))
                }
                _ => None,
            };
            if let Some((side, reason, incoming)) = install {
                let matches = decisions
                    .iter()
                    .filter(|decision| {
                        decision.event_index as usize == event_index
                            && decision.side == side
                            && decision.reason == reason
                            && decision.chosen.as_deref() == Some(incoming)
                    })
                    .count();
                if matches != 1 {
                    return Err("actual pitcher entry lacks one matching v2 decision".into());
                }
            }
        }
        let offense = transition.before().half.offense();
        if transition.event().delta.changes.iter().any(|change| {
            matches!(change, PersonnelChange::PinchHitter { .. })
                && transition.before().teams[offense]
                    .pitcher
                    .active_id()
                    .is_some()
                && transition.after().teams[offense]
                    .pitcher
                    .active_id()
                    .is_none()
        }) && !decisions.iter().any(|decision| {
            decision.event_index as usize == event_index
                && decision.side == offense
                && decision.reason == Reason::PhProbe
                && decision.chosen.is_some()
        }) {
            return Err("pitcher-replacing PH lacks a successful v2 reserve probe".into());
        }
    }
    Ok(decisions)
}
