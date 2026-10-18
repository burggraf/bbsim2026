//! Bounded pitching changes from one explicitly assumed, recorded cast.
use bbsim_engine::{
    accounting::derive_accounting, apply, Delta, Event, EventKind, EventTransition, FieldingReview,
    GameState, Participants, PersonnelChange, PitcherObligation, PitcherRole, TeamSide,
};
use std::collections::{BTreeMap, BTreeSet};

use crate::season_pool;

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

pub const POLICY_VERSION: &str = "workload-context-2019-v3";

#[derive(Clone, Debug)]
struct Workload {
    pa_sum: u64,
    outings: u64,
    samples: Option<Vec<u64>>,
}
impl Workload {
    fn mean_limit(&self) -> u64 {
        self.pa_sum.div_ceil(self.outings).max(1)
    }
    fn budget(&self, word: u64) -> u64 {
        self.samples.as_ref().map_or_else(
            || self.mean_limit(),
            |samples| samples[((u128::from(word) * samples.len() as u128) >> 64) as usize].max(1),
        )
    }
}

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
    workloads: BTreeMap<(String, bool), Workload>,
    empirical: bool,
    game_id: Option<String>,
    managed: bool,
    starter_scale: [u64; 2],
    relief_pace: BTreeMap<String, i8>,
}

impl Cast {
    #[allow(dead_code)] // Pooled mechanism callers deliberately permit source-cast prefixes.
    pub fn read_complete_tsv(text: &str) -> Result<Self, String> {
        let cast = Self::read_tsv(text)?;
        if cast.game_id.is_none()
            && cast
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

        let (empirical, generic, managed) = match (
            fields.remove("format").as_deref(),
            fields.remove("policy").as_deref(),
        ) {
            (Some("closed-pitching-cast-v1"), Some("pa-mean-closed-cast-v1")) => {
                (false, false, false)
            }
            (Some("closed-pitching-cast-v2"), Some("empirical-pa-closed-cast-v2")) => {
                (true, false, false)
            }
            (Some("closed-pitching-cast-v3"), Some("empirical-pa-closed-cast-v2")) => {
                (true, true, false)
            }
            (Some("season-pool-pitching-cast-v1"), Some("empirical-pa-closed-cast-v2")) => {
                (true, true, true)
            }
            _ => return Err("unsupported pitching cast format/policy".into()),
        };
        let mut starter_scale = [100; 2];
        if managed {
            if fields.remove("season_policy").as_deref() != Some(season_pool::POLICY) {
                return Err("unsupported cast season_policy".into());
            }
            for (side, label) in ["Away", "Home"].iter().enumerate() {
                starter_scale[side] =
                    match fields.remove(&format!("starter_scale:{label}")).as_deref() {
                        Some("80") => 80,
                        Some("100") => 100,
                        _ => return Err("invalid or missing starter scale".into()),
                    };
            }
        }
        let mut game_id = None;
        let mut sizes = None;
        if generic {
            let id = fields.remove("game_id").ok_or("missing cast game_id")?;
            if id.len() != 12
                || &id.as_bytes()[3..7] != b"2019"
                || !id.bytes().take(3).all(|b| b.is_ascii_uppercase())
                || !id.bytes().skip(7).all(|b| b.is_ascii_digit())
                || !matches!(id.as_bytes()[11], b'0'..=b'2')
            {
                return Err("invalid cast game_id".into());
            }
            game_id = Some(id);
            let counts =
                numeric_fields(&fields.remove("cast_sizes").ok_or("missing cast sizes")?, 2)?;
            if counts.iter().any(|n| !(1..=50).contains(n)) {
                return Err("invalid cast sizes".into());
            }
            sizes = Some(counts);
            for (key, expected) in [
                (
                    "workload_recipe_sha256",
                    "512f542cee70c5824c6fff92f1278b51aad2be6a5a066db18b306091a3fae48f",
                ),
                (
                    "outcome_events_sha256",
                    "ea3fd660ac87deb02c61cddf28977db59f85f9a88f77aaabc0c07ba0f1831d1a",
                ),
            ] {
                if fields.remove(key).as_deref() != Some(expected) {
                    return Err(format!("unsupported pitching cast {key}"));
                }
            }
        }
        let valid_actor = |id: &str| {
            valid_id(id)
                || (generic
                    && !id.is_empty()
                    && id
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'))
        };
        for (key, expected) in [
            ("attribution", ATTRIBUTION),
            (
                "availability_basis",
                if managed {
                    season_pool::AVAILABILITY
                } else {
                    AVAILABILITY
                },
            ),
            (
                "bullpen_report_sha256",
                "758186cceb67815573e7e109763f7be7d5e57b9ec9583bc8a3cca299d39edc88",
            ),
            ("exhaustion", "last_pitcher_stays"),
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
            (
                "snapshot_sha256",
                "d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528",
            ),
            (
                "supplement_sha256",
                "554632467416eef202b7a55cadad2d09e5355ce98c83868b26b5267f98852b82",
            ),
            (
                "workload_basis",
                if managed {
                    season_pool::WORKLOAD_BASIS
                } else {
                    WORKLOAD_BASIS
                },
            ),
            (
                "workload_report_sha256",
                "125768bd82ce54ead277d92301fbb7434e823a8fc4432777087c15b8e46ad2f9",
            ),
            ("batter_kappa", "200"),
            ("pitcher_kappa", "200"),
        ] {
            if generic && ["bullpen_report_sha256", "workload_report_sha256"].contains(&key) {
                continue;
            }
            if fields.remove(key).as_deref() != Some(expected) {
                return Err(format!("unsupported pitching cast {key}"));
            }
        }

        let mut cast_rows: [Vec<(usize, Pitcher)>; 2] = std::array::from_fn(|_| Vec::new());
        let mut ability_counts = BTreeMap::new();
        let mut pitcher_profiles = BTreeSet::new();
        let mut workloads = BTreeMap::new();
        let mut usage = BTreeMap::new();
        let mut relief_pace = BTreeMap::new();
        let mut budget_priors = BTreeSet::new();
        for (key, value) in fields {
            let parts = key.split(':').collect::<Vec<_>>();
            match parts.as_slice() {
                ["relief_pace", id] if managed && valid_actor(id) => {
                    relief_pace.insert((*id).to_owned(), season_pool::pace(&value)?);
                }
                ["budget_prior", role @ ("start" | "relief"), id]
                    if managed && valid_actor(id) && value == "true" =>
                {
                    budget_priors.insert(((*id).to_owned(), *role == "start"));
                }
                ["cast", side, rank_text, id] => {
                    let side = side_index(side)?;
                    let rank = rank_text
                        .parse::<usize>()
                        .map_err(|_| "invalid pitching cast rank")?;
                    if rank.to_string() != *rank_text || !valid_actor(id) || value.trim().is_empty()
                    {
                        return Err("invalid pitching cast identity/name".into());
                    }
                    if !generic
                        && !SOURCE_CAST[side].get(rank).is_some_and(
                            |(expected_id, expected_name)| {
                                *expected_id == *id && *expected_name == value
                            },
                        )
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
                [role @ ("pitcher" | "batter"), id] if valid_actor(id) => {
                    let counts = numeric_fields(&value, 9)?
                        .try_into()
                        .map_err(|_| "expected nine ability counts")?;
                    ability_counts.insert(key.clone(), counts);
                    if *role == "pitcher" && !pitcher_profiles.insert((*id).to_owned()) {
                        return Err("duplicate pitcher profile".into());
                    }
                }
                ["workload", role, id] if ["start", "relief"].contains(role) && valid_actor(id) => {
                    let numbers = numeric_fields(&value, 2)?;
                    let (pa_sum, outings) = (numbers[0], numbers[1]);
                    if pa_sum > 1 << 52 || outings > 2429 || (outings == 0 && pa_sum != 0) {
                        return Err("invalid pitcher workload sum/denominator".into());
                    }
                    if workloads
                        .insert(
                            ((*id).to_owned(), *role == "start"),
                            Workload {
                                pa_sum,
                                outings,
                                samples: None,
                            },
                        )
                        .is_some()
                    {
                        return Err("duplicate pitcher workload profile".into());
                    }
                }
                ["usage", role, id]
                    if empirical && ["start", "relief"].contains(role) && valid_actor(id) =>
                {
                    let samples = if value == "-" {
                        Vec::new()
                    } else {
                        let n = value.split(',').count();
                        if n > 2429 {
                            return Err("too many workload samples".into());
                        }
                        numeric_fields(&value, n)?
                    };
                    usage.insert(((*id).to_owned(), *role == "start"), samples);
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
        if sizes.is_some_and(|sizes| {
            teams
                .iter()
                .zip(sizes)
                .any(|(team, n)| team.len() as u64 != n)
        }) {
            return Err("incomplete declared pitching cast".into());
        }
        let cast_ids = teams
            .iter()
            .flatten()
            .map(|pitcher| pitcher.id.clone())
            .collect::<BTreeSet<_>>();
        let expected_workloads = cast_ids
            .iter()
            .flat_map(|id| [(id.clone(), true), (id.clone(), false)])
            .collect::<BTreeSet<_>>();
        if pitcher_profiles != cast_ids
            || workloads.keys().cloned().collect::<BTreeSet<_>>() != expected_workloads
        {
            return Err("pitcher profiles do not match recorded cast".into());
        }
        if managed
            && (relief_pace.keys().cloned().collect::<BTreeSet<_>>() != cast_ids
                || !budget_priors.is_subset(&expected_workloads))
        {
            return Err("season-pool profiles do not match cast".into());
        }
        if empirical {
            if usage.keys().cloned().collect::<BTreeSet<_>>() != expected_workloads {
                return Err("usage profiles do not match recorded cast".into());
            }
            for (key, samples) in usage {
                let workload = workloads.get_mut(&key).expect("verified workload key");
                if samples.len() as u64 != workload.outings
                    || samples.iter().map(|n| u128::from(*n)).sum::<u128>()
                        != u128::from(workload.pa_sum)
                    || !samples.windows(2).all(|pair| pair[0] <= pair[1])
                {
                    return Err("usage samples do not reconcile with role workload".into());
                }
                workload.samples = Some(samples);
            }
        }
        for team in &teams {
            for (rank, pitcher) in team.iter().enumerate() {
                if workloads[&(pitcher.id.clone(), rank == 0)].outings == 0 {
                    return Err("missing selected pitcher workload role".into());
                }
            }
        }
        Ok(Self {
            teams,
            names,
            ability_counts,
            workloads,
            empirical,
            game_id,
            managed,
            starter_scale,
            relief_pace,
        })
    }

    #[allow(dead_code)]
    pub fn is_managed(&self) -> bool {
        self.managed
    }

    #[allow(dead_code)]
    pub fn policy_version(&self) -> &'static str {
        if self.managed {
            season_pool::MANAGER
        } else {
            POLICY_VERSION
        }
    }

    #[allow(dead_code)] // Only game-input callers need the source game ID.
    pub fn game_id(&self) -> Option<&str> {
        self.game_id.as_deref()
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
            if initial.teams[side].pitcher.active_id() != Some(pitchers[0].id.as_str()) {
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
                    if team.pitcher.active_id() == Some(pitcher.id.as_str()) && !(own && rank == 0)
                    {
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

    #[allow(dead_code)] // Optional PH entry points also compile in isolated pitching-manager tests.
    pub fn next_available<'a>(&'a self, before: &GameState, side: usize) -> Option<&'a str> {
        let available = self.teams[side].iter().skip(1).filter(|p| {
            before.teams[side].bench.contains(&p.id) && !before.teams[side].removed.contains(&p.id)
        });
        if self.managed {
            // Workload-role classes, not measured fatigue or historical leverage.
            let long_needed = before.inning <= 5 || before.score[0].abs_diff(before.score[1]) >= 4;
            available
                .min_by_key(|p| {
                    let w = &self.workloads[&(p.id.clone(), false)];
                    let long = u128::from(w.pa_sum) >= 6 * u128::from(w.outings);
                    (long != long_needed, -self.relief_pace[&p.id], &p.id)
                })
                .map(|p| p.id.as_str())
        } else {
            available.into_iter().next().map(|p| p.id.as_str())
        }
    }

    /// A PH vacancy is resolved before defensive readiness or any entry/play RNG word.
    #[allow(dead_code)]
    pub fn restore_pitcher(&self, before: &GameState) -> Result<Option<EventTransition>, String> {
        let side = 1 - before.half.offense();
        if !matches!(before.teams[side].pitcher, PitcherRole::Vacant { .. }) {
            return Ok(None);
        }
        let incoming = self
            .next_available(before, side)
            .ok_or("no unused cast pitcher for PH vacancy")?;
        self.substitution(before, side, incoming).map(Some)
    }

    pub fn workload_mode(&self) -> &'static str {
        if self.managed {
            "source empirical role budgets with explicitly marked pooled priors"
        } else if self.empirical {
            "empirical training outing budgets"
        } else {
            "individual role means (v1 fallback; no empirical spread)"
        }
    }

    /// Compatibility caller: a supplied zero word selects the first empirical outing.
    #[allow(dead_code)]
    pub fn next_change(
        &self,
        before: &GameState,
        log: &[EventTransition],
    ) -> Result<Option<EventTransition>, String> {
        self.next_change_with_word(before, log, 0)
    }

    /// Decide before the next event; the caller keeps one manager word per entry.
    pub fn next_change_with_word(
        &self,
        before: &GameState,
        log: &[EventTransition],
        usage_word: u64,
    ) -> Result<Option<EventTransition>, String> {
        if before.over {
            return Ok(None);
        }
        let side = 1 - before.half.offense();
        let team = &before.teams[side];
        if matches!(
            team.pitcher,
            PitcherRole::Active {
                obligation: PitcherObligation::StartingBatter | PitcherObligation::ReliefBatter,
                ..
            }
        ) {
            return Ok(None);
        }
        let pitchers = &self.teams[side];
        let Some(role) = pitchers
            .iter()
            .position(|pitcher| Some(pitcher.id.as_str()) == team.pitcher.active_id())
        else {
            return Err("current pitcher is not in supplied cast".into());
        };
        let pa_count = log
            .iter()
            .filter(|transition| {
                transition.event().participants.pitcher.as_deref() == team.pitcher.active_id()
                    && matches!(
                        transition.event().kind,
                        EventKind::PlateAppearance
                            | EventKind::OutOfParkHomeRun
                            | EventKind::FieldersChoice
                    )
            })
            .count();
        let workload = &self.workloads[&(
            team.pitcher
                .active_id()
                .expect("validated active pitcher")
                .to_owned(),
            role == 0,
        )];
        let mut target = workload.budget(usage_word);
        if self.managed && role == 0 {
            target = (target * self.starter_scale[side] / 100).max(1);
        }
        let runs: u64 = log
            .iter()
            .filter(|t| t.event().participants.pitcher.as_deref() == team.pitcher.active_id())
            .map(|t| u64::from(t.event().delta.runs))
            .sum();
        let lead = i64::from(before.score[side]) - i64::from(before.score[1 - side]);
        // Observed outing budgets already include short/ineffective removals; do not shorten twice.
        if (pa_count as u64) < target {
            return Ok(None);
        }
        if role == 0 && runs == 0 {
            if lead > 3 {
                target += 3;
            }
            let complete_origin = log.first().is_some_and(|t| {
                let s = t.before();
                s.inning == 1
                    && s.half == bbsim_engine::Half::Top
                    && s.outs == 0
                    && s.bases.iter().all(Option::is_none)
                    && s.score == [0, 0]
            });
            if complete_origin && !log.is_empty() {
                let stats = derive_accounting(log)
                    .map_err(|e| format!("manager accounting refused: {e:?}"))?;
                if stats.pitchers[side]
                    .get(
                        team.pitcher
                            .active_id()
                            .ok_or("no active manager pitcher")?,
                    )
                    .is_some_and(|p| p.classified_hits_allowed == Some(0))
                {
                    target =
                        target.max((3 * workload.pa_sum).div_ceil(2 * workload.outings).max(1));
                }
            }
            if (pa_count as u64) < target {
                return Ok(None);
            }
        }
        let Some(incoming) = self.next_available(before, side) else {
            return Ok(None);
        };
        self.substitution(before, side, incoming).map(Some)
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
        let team = &before.teams[side];
        let mut changes = match &team.pitcher {
            PitcherRole::Active { player_id, .. } => vec![PersonnelChange::Pitcher {
                team: team_side,
                outgoing: player_id.clone(),
                incoming: incoming.to_owned(),
            }],
            PitcherRole::Vacant { batting_slot } => vec![
                PersonnelChange::AssignPitcher {
                    team: team_side,
                    incoming: incoming.to_owned(),
                },
                PersonnelChange::BattingSlot {
                    team: team_side,
                    slot: *batting_slot,
                    outgoing: team.batting_order[*batting_slot as usize].clone(),
                    incoming: incoming.to_owned(),
                },
            ],
        };
        if let Some(outgoing) = team.pitcher.active_id().filter(|_| team.dh_slot.is_none()) {
            let slot = team
                .batting_order
                .iter()
                .position(|id| id == outgoing)
                .ok_or("non-DH pitcher is absent from batting order")?;
            changes.push(PersonnelChange::BattingSlot {
                team: team_side,
                slot: slot as u8,
                outgoing: outgoing.to_owned(),
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
                pitcher: before.teams[side].pitcher.active_id().map(str::to_owned),
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
