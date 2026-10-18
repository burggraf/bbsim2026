//! Explicit closed participant-proxy pool; supplied positions are assumptions, not certification.
use super::{
    pinch::{self, Candidate},
    Cast,
};
use bbsim_engine::{DefenseState, GameState, Half, PitcherObligation, PitcherRole};
use std::collections::{BTreeMap, BTreeSet};

pub const AVAILABILITY: &str =
    "same-game participant proxies assumed available at start; not verified eligibility";
pub const DEFENSE_BASIS: &str =
    "supplied observed positions assumed usable; not verified defensive eligibility";
pub struct Bench {
    pub pools: [Vec<Candidate>; 2],
    pub names: BTreeMap<String, String>,
    pub counts: BTreeMap<String, [u64; 9]>,
}
impl Bench {
    pub fn read_tsv(text: &str, game_id: &str) -> Result<Self, String> {
        if text.len() > 65536
            || text
                .chars()
                .any(|c| c.is_control() && c != '\n' && c != '\t')
        {
            return Err("invalid bench controls/size".into());
        }
        let mut lines = text.lines();
        if lines.next() != Some("key\tvalue") {
            return Err("invalid bench header".into());
        }
        let mut fields = BTreeMap::new();
        for row in lines {
            let (key, value) = row.split_once('\t').ok_or("invalid bench row")?;
            if key.is_empty()
                || value.is_empty()
                || key.trim() != key
                || value.trim() != value
                || value.contains('\t')
                || fields.insert(key, value).is_some()
            {
                return Err("duplicate/invalid bench field".into());
            }
        }
        for (key, value) in [
            ("format", "assumed-game-bench-v1"), ("game_id",game_id),
            ("policy",pinch::POLICY_VERSION), ("availability_basis",AVAILABILITY),
            ("defense_basis",DEFENSE_BASIS),
            ("snapshot_sha256","d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528"),
            ("supplement_sha256","554632467416eef202b7a55cadad2d09e5355ce98c83868b26b5267f98852b82"),
            ("opportunity_report_sha256","382be5a84b3e95687e6356b00d3b1b68522f41a5c2a6a065daf876a5a4c6881e"),
            ("attribution","The information used here was obtained free of charge from and is copyrighted by Retrosheet. Interested parties may contact Retrosheet at \"www.retrosheet.org\"."),
        ] {
            if fields.remove(key)!=Some(value) { return Err(format!("bench metadata mismatch: {key}")); }
        }
        let mut records: BTreeMap<(usize, &str), BTreeMap<&str, &str>> = BTreeMap::new();
        for (key, value) in fields {
            let parts: Vec<_> = key.split(':').collect();
            let [side, id, field] = parts.as_slice() else {
                return Err("invalid bench actor key".into());
            };
            let side = match *side {
                "Away" => 0,
                "Home" => 1,
                _ => return Err("invalid bench side".into()),
            };
            if id.is_empty()
                || id.len() > 64
                || !id
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                || !["name", "positions", "counts"].contains(field)
            {
                return Err("invalid bench identity/field".into());
            }
            records.entry((side, id)).or_default().insert(field, value);
        }
        let mut bench = Self {
            pools: [vec![], vec![]],
            names: BTreeMap::new(),
            counts: BTreeMap::new(),
        };
        for ((side, id), fields) in records {
            if fields.len() != 3 || bench.names.contains_key(id) || bench.pools[side].len() >= 50 {
                return Err("incomplete, cross-side or oversized bench".into());
            }
            let positions = if fields["positions"] == "?" {
                vec![]
            } else {
                fields["positions"]
                    .split(',')
                    .map(|v| {
                        let n = v.parse::<u8>().map_err(|_| "invalid bench position")?;
                        if n.to_string() != v || !(2..=9).contains(&n) {
                            return Err("invalid bench position");
                        }
                        Ok(n)
                    })
                    .collect::<Result<Vec<_>, _>>()?
            };
            if positions.iter().collect::<BTreeSet<_>>().len() != positions.len() {
                return Err("duplicate bench position".into());
            }
            let counts: [u64; 9] = fields["counts"]
                .split(',')
                .map(|v| {
                    let n = v.parse::<u64>().map_err(|_| "invalid bench count")?;
                    if n.to_string() != v {
                        return Err("noncanonical bench count");
                    }
                    Ok(n)
                })
                .collect::<Result<Vec<_>, _>>()?
                .try_into()
                .map_err(|_| "expected nine bench counts")?;
            if counts
                .iter()
                .zip([42823, 15142, 1984, 25947, 8531, 785, 6776, 1470, 82245])
                .any(|(n, max)| *n > max)
            {
                return Err("bench count exceeds cohort".into());
            }
            bench.pools[side].push(Candidate {
                id: id.to_owned(),
                positions,
            });
            bench.names.insert(id.to_owned(), fields["name"].to_owned());
            bench.counts.insert(format!("batter:{id}"), counts);
        }
        Ok(bench)
    }
    /// Called after Cast::prepare, exactly once at a complete game origin.
    pub fn prepare(
        &self,
        state: &GameState,
        fielders: &[[String; 9]; 2],
        cast: &Cast,
    ) -> Result<GameState, String> {
        if state.inning != 1
            || state.half != Half::Top
            || state.outs != 0
            || state.score != [0, 0]
            || state.over
            || state.bases.iter().any(Option::is_some)
            || state
                .teams
                .iter()
                .any(|t| t.next_batter != 0 || !t.removed.is_empty())
        {
            return Err("PH preparation requires a complete game origin".into());
        }
        let mut prepared = state.clone();
        let pitchers = cast.names();
        for (side, positions) in fielders.iter().enumerate() {
            for c in &self.pools[side] {
                if pitchers.contains_key(&c.id)
                    || state.teams.iter().any(|t| {
                        t.batting_order.contains(&c.id)
                            || t.bench.contains(&c.id)
                            || t.removed.contains(&c.id)
                            || t.pitcher.active_id() == Some(c.id.as_str())
                    })
                {
                    return Err("bench actor conflicts with initial/cast membership".into());
                }
                prepared.teams[side].bench.push(c.id.clone());
            }
            let team = &mut prepared.teams[side];
            let pitcher = team
                .pitcher
                .active_id()
                .ok_or("missing starting PH-path pitcher")?
                .to_owned();
            team.pitcher = PitcherRole::Active {
                player_id: pitcher,
                obligation: PitcherObligation::StartingBatter,
            };
            team.defense = DefenseState::Assigned(Box::new(positions.clone().map(Some)));
            team.dh_must_bat = team.dh_slot.is_some();
        }
        prepared
            .validate_play_ready()
            .map_err(|e| format!("invalid PH starting personnel: {e:?}"))?;
        Ok(prepared)
    }
}
