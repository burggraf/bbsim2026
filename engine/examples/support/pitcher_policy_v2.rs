//! Exact v2 workload values/ranks; live personnel selection is not wired here.
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug)]
pub struct Clock {
    completed: u32,
    scheduled: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct ReliefUsage {
    used: u32,
    target: Option<u32>,
}

#[derive(Clone, Copy, Debug)]
pub struct Rank {
    debt: (i128, i128),
    clipped: (i128, i128),
    score: (i128, i128),
    long: bool,
    matches: bool,
    target_known: bool,
}

fn count(text: &str) -> Result<u32, String> {
    let n = text.parse::<u32>().map_err(|_| "invalid v2 count")?;
    if n.to_string() != text {
        return Err("noncanonical v2 count".into());
    }
    Ok(n)
}

impl Clock {
    pub fn completed(&self) -> u32 {
        self.completed
    }
    pub fn scheduled(&self) -> u32 {
        self.scheduled
    }

    pub fn read(value: &str) -> Result<Self, String> {
        let (c, n) = value.split_once(',').ok_or("invalid v2 club clock")?;
        let (completed, scheduled) = (count(c)?, count(n)?);
        if completed >= scheduled || scheduled > 162 {
            return Err("invalid v2 club clock".into());
        }
        Ok(Self {
            completed,
            scheduled,
        })
    }
}

impl ReliefUsage {
    pub fn used(&self) -> u32 {
        self.used
    }
    pub fn target(&self) -> Option<u32> {
        self.target
    }

    pub fn read(value: &str) -> Result<Self, String> {
        let (u, t) = value.split_once(',').ok_or("invalid v2 relief usage")?;
        Ok(Self {
            used: count(u)?,
            target: if t == "?" { None } else { Some(count(t)?) },
        })
    }
}

impl Rank {
    pub fn debt(&self) -> (i128, i128) {
        self.debt
    }
    pub fn clipped(&self) -> (i128, i128) {
        self.clipped
    }
    pub fn score(&self) -> (i128, i128) {
        self.score
    }
    pub fn long(&self) -> bool {
        self.long
    }
    pub fn matches(&self) -> bool {
        self.matches
    }
    pub fn target_known(&self) -> bool {
        self.target_known
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        let compare = |a: (i128, i128), b: (i128, i128)| (a.0 * b.1).cmp(&(b.0 * a.1));
        compare(self.score, other.score).then_with(|| compare(self.debt, other.debt))
    }
}

/// Debt is defined even for an excluded starter with no supported relief outing.
pub fn relief_debt(clock: Clock, usage: ReliefUsage) -> (i128, i128) {
    match usage.target() {
        None => (0, 1),
        Some(target) => {
            let expected = i128::from(target) * i128::from(clock.completed() + 1);
            let n = i128::from(clock.scheduled());
            (
                5 * (expected - i128::from(usage.used()) * n),
                (30 * n).max(expected),
            )
        }
    }
}

pub fn relief_rank(
    clock: Clock,
    usage: ReliefUsage,
    pa_sum: u32,
    outings: u32,
    inning: u32,
    score_gap: i64,
) -> Result<Rank, String> {
    if outings == 0 || inning == 0 || score_gap.unsigned_abs() > u64::from(u32::MAX) {
        return Err("invalid v2 relief workload/situation".into());
    }
    let debt = relief_debt(clock, usage);
    let clipped = (debt.0.clamp(-debt.1, debt.1), debt.1);
    let long = u64::from(pa_sum) >= 6 * u64::from(outings);
    let wants_long = inning <= 5 || score_gap.unsigned_abs() >= 4;
    let matches = long == wants_long;
    let score = (
        4 * clipped.0 + i128::from(matches) * clipped.1,
        4 * clipped.1,
    );
    Ok(Rank {
        debt,
        clipped,
        score,
        long,
        matches,
        target_known: usage.target().is_some(),
    })
}
