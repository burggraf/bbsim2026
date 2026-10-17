//! Closed starting-roster experiment, supplied identities and positional actors.
//! A recorded starting lineup does not prove bench eligibility or player abilities.
use super::{joint, preview};
use bbsim_engine::{Event, EventTransition, GameState, Half, TeamState};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Write},
};
pub const REPORT_SHA: &str = "5385231905a1f0e57b57a736e5229def6e94653c2894a093b4542b131b8df63b";
pub const SNAPSHOT_SHA: &str = "d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528";
pub const SUPPLEMENT_SHA: &str = "554632467416eef202b7a55cadad2d09e5355ce98c83868b26b5267f98852b82";
pub const POLICY_VERSION: &str = "station-held-positional-starting-preview-v1";

struct Player {
    slot: u8,
    position: u8,
    id: String,
    name: String,
}
pub struct Lineup {
    players: [Vec<Player>; 2],
}
impl Lineup {
    pub fn read_tsv(text: &str) -> Result<Self, String> {
        if text
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t')
        {
            return Err("invalid lineup control character".into());
        }
        let mut lines = text.lines();
        if lines.next() != Some("key\tvalue") {
            return Err("expected key,value lineup TSV header".into());
        }
        let mut fields = BTreeMap::new();
        for row in lines {
            let (key, value) = row.split_once('\t').ok_or("invalid lineup row")?;
            if key.is_empty()
                || value.trim().is_empty()
                || value.contains('\t')
                || fields.insert(key, value).is_some()
            {
                return Err("duplicate/invalid lineup row".into());
            }
        }
        if fields.len() != 30 {
            return Err("missing/unknown lineup fields".into());
        }
        for (key, value) in [
            ("format", "closed-starting-lineup-v1"),
            ("game_id", "OAK201903200"),
            ("date", "2019-03-20"),
            ("park_id", "TOK01"),
            ("away_team", "Seattle Mariners"),
            ("home_team", "Oakland Athletics"),
            ("audit_sha256", REPORT_SHA),
            ("snapshot_sha256", SNAPSHOT_SHA),
            ("supplement_sha256", SUPPLEMENT_SHA),
            ("attribution", joint::ATTRIBUTION),
        ] {
            if fields.remove(key) != Some(value) {
                return Err(format!("unsupported lineup {key}"));
            }
        }
        let mut players: [Vec<Player>; 2] = std::array::from_fn(|_| Vec::new());
        let mut ids = BTreeSet::new();
        for (key, name) in fields {
            let parts = key.split(':').collect::<Vec<_>>();
            if parts.len() != 4 {
                return Err("unknown lineup key".into());
            }
            let side = match parts[0] {
                "Away" => 0,
                "Home" => 1,
                _ => return Err("invalid starting side".into()),
            };
            let number = |s: &str| -> Result<u8, String> {
                let n = s.parse::<u8>().map_err(|_| "invalid lineup number")?;
                if n.to_string() != s {
                    return Err("noncanonical lineup number".into());
                }
                Ok(n)
            };
            let slot = number(parts[1])?;
            let position = number(parts[2])?;
            let id = parts[3];
            if slot > 9
                || !(1..=10).contains(&position)
                || (slot == 0) != (position == 1)
                || id.is_empty()
                || !id
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
                || !ids.insert(id.to_string())
            {
                return Err("invalid starting slot/position/identity".into());
            }
            players[side].push(Player {
                slot,
                position,
                id: id.into(),
                name: name.into(),
            });
        }
        for side in &mut players {
            side.sort_by_key(|p| p.slot);
            if side.iter().map(|p| p.slot).collect::<Vec<_>>() != (0..10).collect::<Vec<_>>()
                || side.iter().map(|p| p.position).collect::<BTreeSet<_>>() != (1..=10).collect()
            {
                return Err("invalid starting slot/position grain".into());
            }
        }
        Ok(Self { players })
    }
    pub fn initial(&self) -> GameState {
        let team = |side: usize| TeamState {
            batting_order: std::array::from_fn(|i| self.players[side][i + 1].id.clone()),
            next_batter: 0,
            pitcher: self.players[side][0].id.clone(),
            dh_slot: Some(
                self.players[side]
                    .iter()
                    .find(|p| p.position == 10)
                    .expect("validated DH")
                    .slot
                    - 1,
            ),
            dh_must_bat: true,
            bench: vec![],
            removed: vec![],
        };
        GameState {
            inning: 1,
            half: Half::Top,
            outs: 0,
            score: [0, 0],
            teams: [team(0), team(1)],
            bases: [None, None, None],
            over: false,
        }
    }
    fn fielder(&self, side: usize, position: u8) -> &str {
        &self.players[side]
            .iter()
            .find(|p| p.position == position)
            .expect("validated position")
            .id
    }
    pub fn event_for(&self, state: &GameState, sample: joint::Sample) -> Result<Event, String> {
        if (sample.outcome == preview::Outcome::ContactOut) != sample.contact.is_some() {
            return Err("inconsistent outcome/contact pair".into());
        }
        for (side, team) in state.teams.iter().enumerate() {
            let players = &self.players[side];
            if team
                .batting_order
                .iter()
                .zip(&players[1..])
                .any(|(id, p)| *id != p.id)
                || team.pitcher != players[0].id
                || team.dh_slot
                    != Some(
                        players
                            .iter()
                            .find(|p| p.position == 10)
                            .expect("validated DH")
                            .slot
                            - 1,
                    )
                || !team.bench.is_empty()
                || !team.removed.is_empty()
            {
                return Err("unsupported closed starting roster drift".into());
            }
        }
        let side = 1 - state.half.offense();
        let event = preview::event_for_with_fielders(
            state,
            sample.outcome,
            [
                self.fielder(side, 2),
                self.fielder(side, 6),
                self.fielder(side, 3),
            ],
        )?;
        match sample.contact {
            None => Ok(event),
            Some(kind) => {
                let fielder = match kind {
                    joint::ContactKind::Fly => Some(self.fielder(side, 8)),
                    joint::ContactKind::LineDrive => Some(self.fielder(side, 9)),
                    joint::ContactKind::PopUp => Some(self.fielder(side, 5)),
                    _ => None,
                };
                joint::apply_contact(event, kind, fielder)
            }
        }
    }
    pub fn game(
        &self,
        seed: u64,
        model: &joint::JointModel,
    ) -> Result<Vec<EventTransition>, String> {
        preview::game_from(seed, self.initial(), |state, rng| {
            let sample = model.sample(rng);
            Ok((sample.outcome, self.event_for(state, sample)?))
        })
    }
    pub fn names(&self) -> BTreeMap<String, String> {
        self.players
            .iter()
            .flatten()
            .map(|p| (p.id.clone(), p.name.clone()))
            .collect()
    }
    // Shared test modules do not all exercise presentation.
    #[allow(dead_code)]
    pub fn positions(&self) -> BTreeMap<String, &'static str> {
        let labels = ["", "P", "C", "1B", "2B", "3B", "SS", "LF", "CF", "RF", "DH"];
        self.players
            .iter()
            .flatten()
            .map(|p| (p.id.clone(), labels[p.position as usize]))
            .collect()
    }
    pub fn write_preamble(&self, output: &mut dyn Write) -> io::Result<()> {
        self.write_preamble_with_rates(output, false)
    }
    pub fn write_preamble_with_rates(&self, output: &mut dyn Write, rated: bool) -> io::Result<()> {
        writeln!(output, "LINEUP FORMAT: closed-starting-lineup-v1")?;
        writeln!(output, "Matchup: OAK201903200 | 2019-03-20 | TOK01")?;
        writeln!(output, "Starting witness report SHA256: {REPORT_SHA}")?;
        writeln!(
            output,
            "Snapshot SHA256: {SNAPSHOT_SHA}; supplement SHA256: {SUPPLEMENT_SHA}"
        )?;
        for (side, team, players) in [
            ("Away", "Seattle Mariners", &self.players[0]),
            ("Home", "Oakland Athletics", &self.players[1]),
        ] {
            writeln!(output, "TEAM {side}: {team}")?;
            for p in players {
                writeln!(
                    output,
                    "PLAYER {side} {} {} {}: {}",
                    p.slot, p.position, p.id, p.name
                )?;
            }
        }
        if rated {
            writeln!(output,"Limits: generated exhibition, not the historical result; provisional participant-proxy player effects, not calibrated or predictive; no park effects; Closed starting roster, no substitutions or eligible bench/bullpen claim; starting pitchers stay all game; coarse positional assignments, not historical ball locations or fielding skill.")?;
        } else {
            writeln!(output,"Limits: generated exhibition, not the historical result; pooled probabilities, no player fitting or park effects; Closed starting roster, no substitutions or eligible bench/bullpen claim; starting pitchers stay all game; coarse positional assignments, not historical ball locations or fielding skill.")?;
        }
        writeln!(output, "{}", joint::ATTRIBUTION)
    }
}
/// Strip only a standalone opt-in; preserve legacy and contact option values.
pub fn split_options(args: &[String]) -> Result<(Vec<String>, Option<String>), String> {
    let mut old = Vec::new();
    let mut path = None;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if arg == "--lineup" {
            if path.is_some() {
                return Err("repeated --lineup".into());
            }
            let value = args.next().ok_or("missing lineup path")?;
            if value.is_empty() || value.starts_with("--") {
                return Err("missing lineup path".into());
            }
            path = Some(value.clone());
        } else {
            old.push(arg.clone());
            if matches!(arg.as_str(), "--baseline" | "--seed" | "--contact-counts") {
                if let Some(value) = args.next() {
                    old.push(value.clone());
                }
            }
        }
    }
    Ok((old, path))
}
