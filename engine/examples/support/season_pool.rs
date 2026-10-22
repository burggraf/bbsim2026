//! Explicit modeled availability, not reconstructed daily roster eligibility.
pub const DEFENSE_BASIS: &str =
    "supplied observed positions assumed usable; not verified defensive eligibility";
pub const POLICY: &str = "season-pool-soft-usage-v1";
pub const PITCHER_USAGE_POLICY_V2: &str = "season-pool-pitcher-usage-v2";
pub const AVAILABILITY: &str =
    "observed season club players assumed available; no daily injury/minor-league reconstruction";
pub const MANAGER: &str = "season-pool-workload-context-2019-v1";
pub const MANAGER_V2: &str = "season-pool-workload-context-2019-v2";
pub const PINCH_HITTING: &str = "late-close-expected-bases-ph-usage-v1";
pub const WORKLOAD_BASIS: &str =
    "source empirical role outings; explicitly marked pooled role priors when unavailable";

pub fn scheduled_games(team_id: &str) -> Result<u32, String> {
    match team_id {
        "CHA" | "DET" => Ok(161),
        "ANA" | "ARI" | "ATL" | "BAL" | "BOS" | "CHN" | "CIN" | "CLE" | "COL" | "HOU" | "KCA"
        | "LAN" | "MIA" | "MIL" | "MIN" | "NYA" | "NYN" | "OAK" | "PHI" | "PIT" | "SDN" | "SEA"
        | "SFN" | "SLN" | "TBA" | "TEX" | "TOR" | "WAS" => Ok(162),
        _ => Err("unsupported 2019 club clock".into()),
    }
}

pub fn pace(value: &str) -> Result<i8, String> {
    match value {
        "-1" => Ok(-1),
        "0" => Ok(0),
        "1" => Ok(1),
        _ => Err("invalid season-pool pace".into()),
    }
}
