//! Strict optional development artifact; never a fallback for missing current personnel.
use super::{joint, preview, ratings, real, tree};
use bbsim_engine::GameState;
use std::collections::{BTreeMap, BTreeSet};
pub const MODEL_VERSION: &str = "outcome-tree-player-proxy-v2-development";
pub const CAST_SHA: &str = "d284339066222212dc79c57b284bb764d49a156937b2b6762ac943a7ccec2bec";
pub const STRENGTHS: &str = "20,20;20,20;20,20;20,20;20,20;200,200;200,200;200,200";
pub struct Ratings {
    model: tree::Tree,
    batters: BTreeMap<String, usize>,
    pitchers: BTreeMap<String, usize>,
}
fn vector(text: &str) -> Result<[f64; 8], String> {
    let values = text
        .split(',')
        .map(str::parse::<f64>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "invalid calibrated conditional rates")?;
    let q: [f64; 8] = values
        .try_into()
        .map_err(|_| "invalid calibrated conditional rates")?;
    if q.iter().any(|x| !x.is_finite() || *x <= 0.0 || *x >= 1.0) {
        return Err("invalid calibrated conditional rates".into());
    }
    Ok(q)
}
impl Ratings {
    pub fn read_tsv(
        text: &str,
        counts: &BTreeMap<String, [u64; 9]>,
        base: &preview::Baseline,
        required: &BTreeSet<String>,
    ) -> Result<Self, String> {
        ratings::Ratings::from_counts(counts.clone(), base, required)?;
        if text.len() > 65536
            || text
                .chars()
                .any(|c| c.is_control() && c != '\t' && c != '\n')
        {
            return Err("invalid calibrated artifact controls/size".into());
        }
        let mut lines = text.lines();
        if lines.next() != Some("key\tvalue") {
            return Err("invalid calibrated artifact header".into());
        }
        let mut fields = BTreeMap::new();
        for line in lines {
            let cells: Vec<_> = line.split('\t').collect();
            if cells.len() != 2
                || cells.iter().any(|x| x.is_empty())
                || fields.insert(cells[0], cells[1]).is_some()
            {
                return Err("invalid/duplicate calibrated artifact field".into());
            }
        }
        for (key, value) in [
            ("format", "closed-opponent-tree-ratings-v2"),
            ("model", MODEL_VERSION),
            ("opportunity_report_sha256", ratings::REPORT_SHA),
            ("snapshot_sha256", real::SNAPSHOT_SHA),
            ("supplement_sha256", real::SUPPLEMENT_SHA),
            ("cast_sha256", CAST_SHA),
            ("reused_test_benchmark", "true"),
            ("strengths", STRENGTHS),
            ("attribution", joint::ATTRIBUTION),
        ] {
            if fields.remove(key) != Some(value) {
                return Err(format!("calibrated artifact metadata mismatch: {key}"));
            }
        }
        // These are provenance declarations, not signatures or independent-validation certificates.
        for key in ["audit_sha256", "selection_sha256"] {
            let hash = fields
                .remove(key)
                .ok_or("missing calibrated evidence digest")?;
            if hash.len() != 64
                || !hash
                    .bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
            {
                return Err("invalid calibrated evidence digest".into());
            }
        }
        let league = vector(
            fields
                .remove("league_nodes")
                .ok_or("missing calibrated league")?,
        )?;
        let expected = tree::league_nodes(*base.probabilities())?;
        if league
            .iter()
            .zip(expected)
            .any(|(a, b)| (*a - b).abs() > 1e-14)
        {
            return Err("calibrated league differs from source cohort".into());
        }
        let mut profiles = BTreeMap::new();
        for key in required {
            let source = counts.get(key).ok_or("missing calibrated source counts")?;
            if fields.remove(format!("counts:{key}").as_str())
                != Some(
                    source
                        .iter()
                        .map(u64::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                        .as_str(),
                )
            {
                return Err("calibrated source count mismatch".into());
            }
            profiles.insert(
                key.clone(),
                vector(
                    fields
                        .remove(key.as_str())
                        .ok_or("missing calibrated role profile")?,
                )?,
            );
        }
        if !fields.is_empty() {
            return Err("unexpected calibrated artifact fields/roles".into());
        }
        let mut batters = BTreeMap::new();
        let mut pitchers = BTreeMap::new();
        for key in required {
            if let Some(id) = key.strip_prefix("batter:") {
                batters.insert(id.to_owned(), batters.len());
            } else if let Some(id) = key.strip_prefix("pitcher:") {
                pitchers.insert(id.to_owned(), pitchers.len());
            } else {
                return Err("invalid calibrated role".into());
            }
        }
        let mut model = tree::Tree::neutral(league, batters.len(), pitchers.len())?;
        for (prefix, ids) in [("batter", &batters), ("pitcher", &pitchers)] {
            for (id, index) in ids {
                let q = profiles
                    .get(&format!("{prefix}:{id}"))
                    .ok_or("missing calibrated role profile")?;
                for (node, rate) in q.iter().enumerate() {
                    let roles = if prefix == "batter" {
                        &mut model.nodes[node].batters
                    } else {
                        &mut model.nodes[node].pitchers
                    };
                    roles[*index] = [*rate, 1.0 - rate];
                }
            }
        }
        for b in 0..batters.len() {
            for p in 0..pitchers.len() {
                let q = model.probabilities(b, p);
                if q.iter().any(|x| !x.is_finite() || *x <= 0.0)
                    || (q.iter().sum::<f64>() - 1.0).abs() > 1e-12
                {
                    return Err("invalid calibrated complete distribution".into());
                }
            }
        }
        Ok(Self {
            model,
            batters,
            pitchers,
        })
    }
    pub fn probabilities(&self, state: &GameState) -> Result<[f64; 9], String> {
        let offense = state.half.offense();
        let team = &state.teams[offense];
        let batter = team
            .batting_order
            .get(team.next_batter as usize)
            .ok_or("invalid calibrated batting cursor")?;
        let b = self
            .batters
            .get(batter)
            .ok_or("missing current calibrated batter rating")?;
        let pitcher = state.teams[1 - offense]
            .pitcher
            .active_id()
            .ok_or("no current calibrated pitcher")?;
        let p = self
            .pitchers
            .get(pitcher)
            .ok_or("missing current calibrated pitcher rating")?;
        Ok(self.model.probabilities(*b, *p))
    }
}
