//! Explicit modeled availability, not reconstructed daily roster eligibility.
pub const DEFENSE_BASIS: &str =
    "supplied observed positions assumed usable; not verified defensive eligibility";
pub const POLICY: &str = "season-pool-soft-usage-v1";
pub const AVAILABILITY: &str =
    "observed season club players assumed available; no daily injury/minor-league reconstruction";
pub const MANAGER: &str = "season-pool-workload-context-2019-v1";
pub const PINCH_HITTING: &str = "late-close-expected-bases-ph-usage-v1";
pub const WORKLOAD_BASIS: &str =
    "source empirical role outings; explicitly marked pooled role priors when unavailable";

pub fn pace(value: &str) -> Result<i8, String> {
    match value {
        "-1" => Ok(-1),
        "0" => Ok(0),
        "1" => Ok(1),
        _ => Err("invalid season-pool pace".into()),
    }
}
