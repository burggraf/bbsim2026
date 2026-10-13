//! Opt-in source-shape mixture and synthetic held-retirement policy, not calibration.
use super::preview::{self, Baseline, Outcome, Rng};
use bbsim_engine::{
    AirContact, Credit, CreditKind, Event, FieldingReview, GameState, OrdinaryAirCatchFacts,
};

pub const MODEL_VERSION: &str = "pooled-jeffreys-joint-proxy-preview-v1";
pub const POLICY_VERSION: &str = "station-held-joint-proxy-preview-v1";
pub const ATTRIBUTION: &str = "The information used here was obtained free of charge from and is copyrighted by Retrosheet. Interested parties may contact Retrosheet at \"www.retrosheet.org\".";
// Version1 is deliberately pinned to the accepted internal pilot's logical contents.
const PROVENANCE: [(&str, &str); 12] = [
    ("format", "joint-proxy-counts-v1"),
    ("cohort_format", "2019-joint-contact-proxies-v1"),
    (
        "catalog_sha256",
        "2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583",
    ),
    ("source_commit", "b449f7bec49f852c27803e096303a4c78ac661b9"),
    (
        "snapshot_sha256",
        "d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528",
    ),
    (
        "supplement_sha256",
        "554632467416eef202b7a55cadad2d09e5355ce98c83868b26b5267f98852b82",
    ),
    (
        "recipe_sha256",
        "174e9b959f59359f9fa86c7881dbbf9aefd5981f9376d08b60b975126ff4afc3",
    ),
    (
        "artifact_sha256",
        "1573fb845ac24d644ac722cc267b3729b2174fc114ff9f6748e2435502fe29ca",
    ),
    ("included_pa", "185703"),
    ("excluded_ibb", "753"),
    ("excluded_interference", "61"),
    ("attribution", ATTRIBUTION),
];
const COUNTS: [(&str, u64); 9] = [
    ("strikeout", 42823),
    ("walk", 15142),
    ("hbp", 1984),
    ("single", 25947),
    ("double", 8531),
    ("triple", 785),
    ("home_run", 6776),
    ("roe", 1470),
    ("contact_out", 82245),
];
const BINS: [(&str, u64); 5] = [
    ("ground_candidate", 17188),
    ("fly_candidate", 19528),
    ("line_candidate", 5379),
    ("pop_candidate", 536),
    ("residual", 39614),
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContactKind {
    Ground,
    Fly,
    LineDrive,
    PopUp,
    Residual,
}
const CONTACTS: [ContactKind; 5] = [
    ContactKind::Ground,
    ContactKind::Fly,
    ContactKind::LineDrive,
    ContactKind::PopUp,
    ContactKind::Residual,
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sample {
    pub outcome: Outcome,
    pub contact: Option<ContactKind>,
}
pub struct JointModel {
    marginals: [f64; 9],
    conditionals: [f64; 5],
    boundaries: [f64; 13],
}
impl JointModel {
    pub fn read_csv(text: &str, base: &Baseline) -> Result<Self, String> {
        let mut lines = text.lines();
        if lines.next() != Some("key,value") {
            return Err("expected joint key,value header".into());
        }
        let mut fields = std::collections::BTreeMap::new();
        for line in lines {
            let (key, value) = line.split_once(',').ok_or("invalid joint count row")?;
            if value.contains(',') || fields.insert(key, value).is_some() {
                return Err("duplicate/invalid joint count row".into());
            }
        }
        if fields.len() != PROVENANCE.len() + COUNTS.len() + BINS.len()
            || PROVENANCE
                .iter()
                .any(|(key, value)| fields.get(key) != Some(value))
        {
            return Err("unsupported joint provenance/cohort or missing/unknown fields".into());
        }
        for (key, expected) in COUNTS.iter().chain(BINS.iter()) {
            let value = fields.get(key).ok_or("missing joint count")?;
            if value.is_empty()
                || !value.bytes().all(|b| b.is_ascii_digit())
                || value.parse::<u64>().map_err(|_| "joint count overflow")? != *expected
            {
                return Err(format!("invalid or changed audited joint count: {key}"));
            }
        }
        if base.counts != COUNTS.map(|(_, n)| n) {
            return Err("joint source-shape cohort disagrees with legacy baseline".into());
        }
        Self::from_counts(base, BINS.map(|(_, n)| n))
    }
    pub fn from_counts(base: &Baseline, counts: [u64; 5]) -> Result<Self, String> {
        let checked = Baseline::from_counts(base.counts)?;
        if checked.probabilities() != base.probabilities() {
            return Err("inconsistent legacy baseline counts/probabilities".into());
        }
        let n = counts
            .iter()
            .try_fold(0u64, |sum, n| sum.checked_add(*n))
            .ok_or("contact count overflow")?;
        if n != base.counts[8] {
            return Err("contact counts do not match baseline marginal".into());
        }
        let marginals = *base.probabilities();
        let conditionals = counts.map(|n_j| (n_j as f64 + 0.5) / (n as f64 + 2.5));
        let mut boundaries = [1.0; 13];
        let mut cumulative = 0.0;
        for (edge, p) in boundaries[..8].iter_mut().zip(&marginals[..8]) {
            cumulative += p;
            *edge = cumulative;
        }
        let start = cumulative;
        cumulative = 0.0;
        for (edge, q) in boundaries[8..12].iter_mut().zip(&conditionals[..4]) {
            cumulative += q;
            *edge = start + (1.0 - start) * cumulative;
        }
        if boundaries.iter().any(|p| !p.is_finite() || *p <= 0.0)
            || boundaries.windows(2).any(|pair| pair[1] <= pair[0])
        {
            return Err("numerically unsupported joint intervals".into());
        }
        Ok(Self {
            marginals,
            conditionals,
            boundaries,
        })
    }
    pub fn marginals(&self) -> [f64; 9] {
        self.marginals
    }
    pub fn conditionals(&self) -> [f64; 5] {
        self.conditionals
    }
    pub fn boundaries(&self) -> [f64; 13] {
        self.boundaries
    }
    pub fn sample(&self, rng: &mut Rng) -> Sample {
        let u = (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        self.sample_unit(u).expect("SplitMix64 draw is in [0,1)")
    }
    pub fn sample_unit(&self, u: f64) -> Result<Sample, String> {
        if !u.is_finite() || !(0.0..1.0).contains(&u) {
            return Err("joint draw must be finite and in [0,1)".into());
        }
        let index = self
            .boundaries
            .iter()
            .position(|edge| u < *edge)
            .expect("validated CDF ends at1");
        Ok(if index < 8 {
            Sample {
                outcome: preview::ALL[index],
                contact: None,
            }
        } else {
            Sample {
                outcome: Outcome::ContactOut,
                contact: Some(CONTACTS[index - 8]),
            }
        })
    }
}
/// Preserve legacy option values (including flag-looking filenames) before stripping opt-in.
pub fn split_options(args: &[String]) -> Result<(Vec<String>, Option<String>), String> {
    let mut old = Vec::new();
    let mut path = None;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if arg == "--contact-counts" {
            if path.is_some() {
                return Err("repeated --contact-counts".into());
            }
            let value = args.next().ok_or("missing contact-counts path")?;
            if value.is_empty() || value.starts_with("--") {
                return Err("missing contact-counts path".into());
            }
            path = Some(value.clone());
        } else {
            old.push(arg.clone());
            if matches!(arg.as_str(), "--baseline" | "--seed") {
                if let Some(value) = args.next() {
                    old.push(value.clone());
                }
            }
        }
    }
    Ok((old, path))
}
pub fn game(seed: u64, model: &JointModel) -> Result<Vec<bbsim_engine::EventTransition>, String> {
    preview::game_with(seed, |state, rng| {
        let sample = model.sample(rng);
        Ok((sample.outcome, event_for(state, sample)?))
    })
}
pub fn event_for(state: &GameState, sample: Sample) -> Result<Event, String> {
    if (sample.outcome == Outcome::ContactOut) != sample.contact.is_some() {
        return Err("inconsistent outcome/contact pair".into());
    }
    let mut event = preview::event_for(state, sample.outcome)?;
    let Some(kind) = sample.contact else {
        return Ok(event);
    };
    if kind == ContactKind::Ground {
        return Ok(event);
    }
    event.ordinary_ground_throw = None;
    event
        .delta
        .credits
        .retain(|c| !matches!(c.kind, CreditKind::Assist | CreditKind::Putout));
    event.fielding_review = FieldingReview::Unknown;
    let (ordinal, contact) = match kind {
        ContactKind::Fly => (6, AirContact::Fly),
        ContactKind::LineDrive => (7, AirContact::LineDrive),
        ContactKind::PopUp => (2, AirContact::PopUp),
        ContactKind::Residual => return Ok(event),
        ContactKind::Ground => unreachable!("ground path returned above"),
    };
    let defense = &state.teams[1 - state.half.offense()];
    let mut defenders = defense
        .batting_order
        .iter()
        .enumerate()
        .filter(|(i, _)| defense.dh_slot != Some(*i as u8))
        .map(|(_, id)| id.clone())
        .collect::<Vec<_>>();
    if !defenders.contains(&defense.pitcher) {
        defenders.push(defense.pitcher.clone());
    }
    let fielder_id = defenders
        .get(ordinal)
        .ok_or("no eligible synthetic defender")?
        .clone();
    event.delta.credits.push(Credit {
        player_id: fielder_id.clone(),
        kind: CreditKind::Putout,
    });
    event.ordinary_air_catch = Some(OrdinaryAirCatchFacts {
        batter_id: event
            .participants
            .batter
            .clone()
            .ok_or("missing current batter")?,
        fielder_id,
        contact,
    });
    event.fielding_review = FieldingReview::NoMisplay;
    Ok(event)
}
