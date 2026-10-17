//! Bounded pitching changes from one explicitly assumed, recorded cast.
use bbsim_engine::{
    apply, Delta, Event, EventKind, EventTransition, FieldingReview, GameState, Participants,
    PersonnelChange, TeamSide,
};
use std::collections::{BTreeMap, BTreeSet};

const ATTRIBUTION: &str = "The information used here was obtained free of charge from and is copyrighted by Retrosheet. Interested parties may contact Retrosheet at \"www.retrosheet.org\".";
const AVAILABILITY: &str =
    "recorded nonstarters assumed available at start; not verified eligibility";
const WORKLOAD_BASIS: &str = "literal terminal-PA participants per pitcher-game appearance; not pitch counts, official BF or fatigue";
const SOURCE_CAST: [&[(&str, &str)]; 2] = [
    &[
        ("gonzm005", "Marco Gonzales"),
        ("strih001", "Hunter Strickland"),
        ("gearc001", "Cory Gearrin"),
        ("rossz001", "Zac Rosscup"),
        ("rumbn001", "Nick Rumbelow"),
    ],
    &[
        ("fierm001", "Michael Fiers"),
        ("dullr001", "Ryan Dull"),
        ("wendj001", "J.B. Wendelken"),
        ("hendl001", "Liam Hendriks"),
        ("trivl001", "Lou Trivino"),
        ("rodnf001", "Fernando Rodney"),
        ("buchr001", "Ryan Buchter"),
    ],
];

#[derive(Clone, Debug, PartialEq, Eq)]
struct Pitcher {
    id: String,
    name: String,
}

/// The pinned exporter version does not authenticate arbitrary TSV bytes or player eligibility.
#[derive(Clone, Debug)]
pub struct Cast {
    teams: [Vec<Pitcher>; 2],
    names: BTreeMap<String, String>,
    ability_counts: BTreeMap<String, [u64; 9]>,
}

impl Cast {
    #[allow(dead_code)] // Pooled mechanism callers deliberately permit source-cast prefixes.
    pub fn read_complete_tsv(text: &str) -> Result<Self, String> {
        let cast = Self::read_tsv(text)?;
        if cast
            .teams
            .iter()
            .zip(SOURCE_CAST)
            .any(|(team, source)| team.len() != source.len())
        {
            return Err("complete text game requires all 12 pinned cast pitchers".into());
        }
        Ok(cast)
    }

    pub fn read_tsv(text: &str) -> Result<Self, String> {
        if text
            .chars()
            .any(|character| character.is_control() && character != '\n' && character != '\t')
        {
            return Err("invalid pitching cast control character".into());
        }
        let mut lines = text.lines();
        if lines.next() != Some("key\tvalue") {
            return Err("expected key,value pitching cast TSV header".into());
        }
        let mut fields = BTreeMap::new();
        for row in lines {
            let (key, value) = row.split_once('\t').ok_or("invalid pitching cast row")?;
            if key.is_empty()
                || value.is_empty()
                || key.trim() != key
                || value.trim() != value
                || value.contains('\t')
                || fields.insert(key.to_owned(), value.to_owned()).is_some()
            {
                return Err("duplicate/invalid pitching cast row".into());
            }
        }

        for (key, expected) in [
            ("attribution", ATTRIBUTION),
            ("availability_basis", AVAILABILITY),
            (
                "bullpen_report_sha256",
                "758186cceb67815573e7e109763f7be7d5e57b9ec9583bc8a3cca299d39edc88",
            ),
            ("exhaustion", "last_pitcher_stays"),
            ("format", "closed-pitching-cast-v1"),
            (
                "league_counts",
                "42823,15142,1984,25947,8531,785,6776,1470,82245",
            ),
            (
                "lineup_report_sha256",
                "5385231905a1f0e57b57a736e5229def6e94653c2894a093b4542b131b8df63b",
            ),
            ("model", "dirichlet-log5-player-proxy-preview-v1"),
            (
                "opportunity_basis",
                "literal terminal-event participants; not official credits or PA/BF",
            ),
            (
                "opportunity_report_sha256",
                "382be5a84b3e95687e6356b00d3b1b68522f41a5c2a6a065daf876a5a4c6881e",
            ),
            ("policy", "pa-mean-closed-cast-v1"),
            (
                "snapshot_sha256",
                "d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528",
            ),
            (
                "supplement_sha256",
                "554632467416eef202b7a55cadad2d09e5355ce98c83868b26b5267f98852b82",
            ),
            ("workload_basis", WORKLOAD_BASIS),
            (
                "workload_report_sha256",
                "125768bd82ce54ead277d92301fbb7434e823a8fc4432777087c15b8e46ad2f9",
            ),
            ("batter_kappa", "200"),
            ("pitcher_kappa", "200"),
        ] {
            if fields.remove(key).as_deref() != Some(expected) {
                return Err(format!("unsupported pitching cast {key}"));
            }
        }

        let mut cast_rows: [Vec<(usize, Pitcher)>; 2] = std::array::from_fn(|_| Vec::new());
        let mut ability_counts = BTreeMap::new();
        let mut pitcher_profiles = BTreeSet::new();
        let mut workload_profiles = BTreeSet::new();
        for (key, value) in fields {
            let parts = key.split(':').collect::<Vec<_>>();
            match parts.as_slice() {
                ["cast", side, rank_text, id] => {
                    let side = side_index(side)?;
                    let rank = rank_text
                        .parse::<usize>()
                        .map_err(|_| "invalid pitching cast rank")?;
                    if rank.to_string() != *rank_text || !valid_id(id) || value.trim().is_empty() {
                        return Err("invalid pitching cast identity/name".into());
                    }
                    if !SOURCE_CAST[side]
                        .get(rank)
                        .is_some_and(|(expected_id, expected_name)| {
                            *expected_id == *id && *expected_name == value
                        })
                    {
                        return Err("unsupported pitching cast identity/name".into());
                    }
                    cast_rows[side].push((
                        rank,
                        Pitcher {
                            id: (*id).to_owned(),
                            name: value,
                        },
                    ));
                }
                [role @ ("pitcher" | "batter"), id] if valid_id(id) => {
                    let counts = numeric_fields(&value, 9)?
                        .try_into()
                        .map_err(|_| "expected nine ability counts")?;
                    ability_counts.insert(key.clone(), counts);
                    if *role == "pitcher" && !pitcher_profiles.insert((*id).to_owned()) {
                        return Err("duplicate pitcher profile".into());
                    }
                }
                ["workload", role, id] if ["start", "relief"].contains(role) && valid_id(id) => {
                    numeric_fields(&value, 2)?;
                    if !workload_profiles.insert(((*id).to_owned(), (*role).to_owned())) {
                        return Err("duplicate pitcher workload profile".into());
                    }
                }
                _ => return Err("unknown pitching cast field".into()),
            }
        }

        let mut names = BTreeMap::new();
        let teams = cast_rows.map(|mut rows| {
            rows.sort_by_key(|(rank, _)| *rank);
            rows.into_iter()
                .enumerate()
                .map(|(expected, (rank, pitcher))| {
                    if rank != expected
                        || names
                            .insert(pitcher.id.clone(), pitcher.name.clone())
                            .is_some()
                    {
                        return Err("noncontiguous or duplicate pitching cast identity".to_owned());
                    }
                    Ok(pitcher)
                })
                .collect::<Result<Vec<_>, _>>()
        });
        let teams = [teams[0].clone()?, teams[1].clone()?];
        if teams.iter().any(Vec::is_empty) {
            return Err("missing pitching cast starter".into());
        }
        let cast_ids = teams
            .iter()
            .flatten()
            .map(|pitcher| pitcher.id.clone())
            .collect::<BTreeSet<_>>();
        let expected_workloads = cast_ids
            .iter()
            .flat_map(|id| {
                [
                    (id.clone(), "start".to_owned()),
                    (id.clone(), "relief".to_owned()),
                ]
            })
            .collect::<BTreeSet<_>>();
        if pitcher_profiles != cast_ids || workload_profiles != expected_workloads {
            return Err("pitcher profiles do not match recorded cast".into());
        }
        Ok(Self {
            teams,
            names,
            ability_counts,
        })
    }

    /// Literal supplied counts; the rating loader still checks the complete required role set and bounds.
    #[allow(dead_code)] // The standalone manager tests and old preview do not consume abilities.
    pub fn ability_counts(&self) -> &BTreeMap<String, [u64; 9]> {
        &self.ability_counts
    }

    /// Only names explicitly present in the source cast rows are exposed.
    pub fn names(&self) -> BTreeMap<String, String> {
        self.names.clone()
    }

    /// Add exactly the supplied nonstarters after native starter/membership checks.
    pub fn prepare(&self, initial: &GameState) -> Result<GameState, String> {
        for side in 0..2 {
            let pitchers = &self.teams[side];
            if initial.teams[side].pitcher != pitchers[0].id {
                return Err("pitching cast starter does not match initial pitcher".into());
            }
            for (rank, pitcher) in pitchers.iter().enumerate() {
                for (team_index, team) in initial.teams.iter().enumerate() {
                    let own = team_index == side;
                    if team.bench.contains(&pitcher.id) || team.removed.contains(&pitcher.id) {
                        return Err(
                            "pitching cast identity conflicts with bench/removed players".into(),
                        );
                    }
                    if team.pitcher == pitcher.id && !(own && rank == 0) {
                        return Err(
                            "pitching cast identity conflicts with pitcher membership".into()
                        );
                    }
                    let lineup_count = team
                        .batting_order
                        .iter()
                        .filter(|player| **player == pitcher.id)
                        .count();
                    let is_own_no_dh_starter = own && rank == 0 && team.dh_slot.is_none();
                    if lineup_count != usize::from(is_own_no_dh_starter) {
                        return Err(
                            "pitching cast identity conflicts with batting membership".into()
                        );
                    }
                }
                if initial
                    .bases
                    .iter()
                    .flatten()
                    .any(|runner| runner.player_id == pitcher.id)
                {
                    return Err("pitching cast identity conflicts with a base runner".into());
                }
            }
        }
        let mut prepared = initial.clone();
        for side in 0..2 {
            prepared.teams[side].bench.extend(
                self.teams[side]
                    .iter()
                    .skip(1)
                    .map(|pitcher| pitcher.id.clone()),
            );
        }
        Ok(prepared)
    }

    /// Decide before the next event, using only the current accepted transition log.
    pub fn next_change(
        &self,
        before: &GameState,
        log: &[EventTransition],
    ) -> Result<Option<EventTransition>, String> {
        if before.over {
            return Ok(None);
        }
        let side = 1 - before.half.offense();
        let team = &before.teams[side];
        let pitchers = &self.teams[side];
        let Some(role) = pitchers
            .iter()
            .position(|pitcher| pitcher.id == team.pitcher)
        else {
            return Err("current pitcher is not in supplied cast".into());
        };
        let pa_count = log
            .iter()
            .filter(|transition| {
                transition.event().participants.pitcher == team.pitcher
                    && matches!(
                        transition.event().kind,
                        EventKind::PlateAppearance
                            | EventKind::OutOfParkHomeRun
                            | EventKind::FieldersChoice
                    )
            })
            .count();
        let target = if role == 0 { 27 } else { 6 };
        let half_boundary = log.last().is_some_and(|last| {
            last.after() == before
                && (last.before().inning != before.inning || last.before().half != before.half)
        });
        if pa_count < target && !(before.inning >= 7 && half_boundary) {
            return Ok(None);
        }
        let Some(incoming) = pitchers.iter().skip(1).find(|pitcher| {
            team.bench.contains(&pitcher.id) && !team.removed.contains(&pitcher.id)
        }) else {
            return Ok(None);
        };
        self.substitution(before, side, &incoming.id).map(Some)
    }

    fn substitution(
        &self,
        before: &GameState,
        side: usize,
        incoming: &str,
    ) -> Result<EventTransition, String> {
        let team_side = if side == 0 {
            TeamSide::Away
        } else {
            TeamSide::Home
        };
        let outgoing = before.teams[side].pitcher.clone();
        let mut changes = vec![PersonnelChange::Pitcher {
            team: team_side,
            outgoing: outgoing.clone(),
            incoming: incoming.to_owned(),
        }];
        if before.teams[side].dh_slot.is_none() {
            let slot = before.teams[side]
                .batting_order
                .iter()
                .position(|player| player == &outgoing)
                .ok_or("non-DH pitcher is absent from batting order")?;
            changes.push(PersonnelChange::BattingSlot {
                team: team_side,
                slot: slot as u8,
                outgoing,
                incoming: incoming.to_owned(),
            });
        }
        let offense = before.half.offense();
        let batter = before.teams[offense]
            .batting_order
            .get(usize::from(before.teams[offense].next_batter))
            .ok_or("invalid batting cursor before pitching change")?
            .clone();
        let event = Event {
            kind: EventKind::Substitution,
            participants: Participants {
                batter: Some(batter),
                pitcher: before.teams[side].pitcher.clone(),
            },
            fielding_review: FieldingReview::Unknown,
            delta: Delta {
                moves: Vec::new(),
                outs: 0,
                runs: 0,
                credits: Vec::new(),
                changes,
                third_out: None,
            },
            two_out_contact: None,
            single_delivery: None,
            balk: None,
            caught_strikeout: None,
            protected_award: None,
            sole_home_run: None,
            sole_safe_hit: None,
            muffed_ground_ball: None,
            ordinary_air_catch: None,
            ordinary_ground_throw: None,
        };
        apply(before, event)
            .map_err(|error| format!("canonical pitching substitution refused: {error:?}"))
    }
}

fn side_index(side: &str) -> Result<usize, String> {
    match side {
        "Away" => Ok(0),
        "Home" => Ok(1),
        _ => Err("invalid pitching cast side".into()),
    }
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
}

fn numeric_fields(value: &str, expected: usize) -> Result<Vec<u64>, String> {
    let fields = value
        .split(',')
        .map(|field| {
            let number = field
                .parse::<u64>()
                .map_err(|_| "invalid pitching cast numeric row")?;
            if number.to_string() != field {
                return Err("invalid pitching cast numeric row".to_owned());
            }
            Ok(number)
        })
        .collect::<Result<Vec<_>, String>>()?;
    if fields.len() != expected {
        return Err("invalid pitching cast numeric row".into());
    }
    Ok(fields)
}
