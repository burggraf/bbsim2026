//! Experimental pooled baseline and coarse policy, deliberately outside the pure engine.
use bbsim_engine::{
    apply, Base, Credit, CreditKind, Delta, Destination, Event, EventKind, EventTransition,
    FieldingReview, GameState, Half, Participants, Runner, RunnerMove, TeamState, ThirdOut,
    ThirdOutKind,
};
use std::collections::BTreeMap;

pub const MODEL_VERSION: &str = "pooled-jeffreys-preview-v1";
pub const POLICY_VERSION: &str = "station-ground-contact-v1";
pub const RNG_VERSION: &str = "splitmix64-v1";
const KEYS: [&str; 9] = [
    "strikeout",
    "walk",
    "hbp",
    "single",
    "double",
    "triple",
    "home_run",
    "roe",
    "contact_out",
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Strikeout,
    Walk,
    Hbp,
    Single,
    Double,
    Triple,
    HomeRun,
    Roe,
    ContactOut,
}
pub const ALL: [Outcome; 9] = [
    Outcome::Strikeout,
    Outcome::Walk,
    Outcome::Hbp,
    Outcome::Single,
    Outcome::Double,
    Outcome::Triple,
    Outcome::HomeRun,
    Outcome::Roe,
    Outcome::ContactOut,
];

pub struct Baseline {
    pub counts: [u64; 9],
    probabilities: [f64; 9],
}
impl Baseline {
    pub fn read_csv(text: &str) -> Result<Self, String> {
        let mut lines = text.lines();
        if lines.next() != Some("key,value") {
            return Err("expected key,value header".into());
        }
        let mut fields = BTreeMap::new();
        for line in lines {
            let (key, value) = line.split_once(',').ok_or("invalid count row")?;
            if value.contains(',') || fields.insert(key, value).is_some() {
                return Err("duplicate/invalid count row".into());
            }
        }
        let required = [
            "format",
            "catalog_sha256",
            "query_version",
            "selected_pa",
            "excluded_ibb",
            "excluded_interference",
        ];
        if fields.len() != KEYS.len() + required.len()
            || KEYS
                .iter()
                .chain(required.iter())
                .any(|k| !fields.contains_key(k))
        {
            return Err("missing/unknown baseline fields".into());
        }
        if fields["format"] != "preview-counts-v1"
            || fields["query_version"] != "preview-baseline-v1"
            || fields["catalog_sha256"]
                != "2a3362ba0bc970515048c87c5bf5bcaab1bd093593b7bafc1e50e2ebaa241583"
        {
            return Err("unsupported baseline provenance/version".into());
        }
        let number = |key: &str| {
            fields[key]
                .parse::<u64>()
                .map_err(|_| format!("invalid count for {key}"))
        };
        let mut counts = [0; 9];
        for (i, key) in KEYS.iter().enumerate() {
            counts[i] = number(key)?;
        }
        let included = counts
            .iter()
            .try_fold(0u64, |sum, n| sum.checked_add(*n))
            .ok_or("count overflow")?;
        let total = included
            .checked_add(number("excluded_ibb")?)
            .and_then(|n| n.checked_add(number("excluded_interference").ok()?))
            .ok_or("count overflow/invalid exclusion")?;
        if total != number("selected_pa")? {
            return Err("PA/exclusion reconciliation failed".into());
        }
        Self::from_counts(counts)
    }
    pub fn from_counts(counts: [u64; 9]) -> Result<Self, String> {
        let n = counts
            .iter()
            .try_fold(0u64, |sum, n| sum.checked_add(*n))
            .ok_or("count overflow")?;
        if n == 0 || n > (1u64 << 52) {
            return Err("empty or numerically unsupported baseline".into());
        }
        let probabilities = counts.map(|n_i| (n_i as f64 + 0.5) / (n as f64 + 4.5));
        Ok(Self {
            counts,
            probabilities,
        })
    }
    pub fn probabilities(&self) -> &[f64; 9] {
        &self.probabilities
    }
    pub fn sample(&self, rng: &mut Rng) -> Outcome {
        let u = (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        let mut cumulative = 0.0;
        for (i, p) in self.probabilities.iter().enumerate() {
            cumulative += p;
            if u < cumulative {
                return ALL[i];
            }
        }
        // Floating-point cumulative rounding only; no extra roll or category rejection.
        Outcome::ContactOut
    }
}

pub struct Rng {
    state: u64,
}
impl Rng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}
fn credit(id: &str, kind: CreditKind) -> Credit {
    Credit {
        player_id: id.into(),
        kind,
    }
}

pub fn event_for(state: &GameState, outcome: Outcome) -> Result<Event, String> {
    if state.over {
        return Err("cannot generate after game end".into());
    }
    let offense = state.half.offense();
    let attack = &state.teams[offense];
    let defense = &state.teams[1 - offense];
    let batter = attack
        .batting_order
        .get(usize::from(attack.next_batter))
        .ok_or("invalid batting cursor")?
        .clone();
    let mut kind = EventKind::PlateAppearance;
    let mut outs = 0;
    let mut runs = 0;
    let mut credits = vec![credit(&batter, CreditKind::PlateAppearance)];
    let mut moves = Vec::new();
    let mut hit_value = match outcome {
        Outcome::Single => 1,
        Outcome::Double => 2,
        Outcome::Triple => 3,
        Outcome::HomeRun => 4,
        _ => 0,
    };
    let advance = hit_value.max(1);
    let mut winning_targets = [None; 3];
    if hit_value > 0
        && hit_value < 4
        && state.half == Half::Bottom
        && state.inning >= 9
        && state.score[1] <= state.score[0]
    {
        let needed = usize::from(state.score[0] - state.score[1]) + 1;
        let scorers = (0..3)
            .rev()
            .filter(|i| state.bases[*i].is_some() && i + hit_value >= 3)
            .collect::<Vec<_>>();
        if scorers.len() >= needed {
            // Canonical records lack winning-run order. Use a value valid for
            // every supplied scorer, not just our policy's presumed winner.
            hit_value = hit_value.min(3 - scorers[0]);
            let mut available = 2;
            for i in (0..3).rev().filter(|i| state.bases[*i].is_some()) {
                if scorers[..needed].contains(&i) {
                    winning_targets[i] = Some(Destination::Home);
                } else {
                    let target = (i + advance).min(available);
                    winning_targets[i] = Some(match target {
                        0 => Destination::First,
                        1 => Destination::Second,
                        _ => Destination::Third,
                    });
                    available = target.saturating_sub(1);
                }
            }
        }
    }
    let destination = |index: usize| match index {
        0 => Destination::First,
        1 => Destination::Second,
        2 => Destination::Third,
        _ => Destination::Home,
    };
    let mut batter_destination = Destination::First;
    if matches!(outcome, Outcome::Strikeout | Outcome::ContactOut) {
        outs = 1;
        batter_destination = Destination::Out;
        if outcome == Outcome::Strikeout {
            credits.extend([
                credit(&batter, CreditKind::Strikeout),
                credit(&defense.batting_order[0], CreditKind::Putout),
            ]);
        } else {
            credits.extend([
                credit(&batter, CreditKind::AtBat),
                credit(&batter, CreditKind::NoHit),
                credit(&defense.batting_order[3], CreditKind::Assist),
                credit(&defense.batting_order[1], CreditKind::Putout),
            ]);
        }
    } else {
        match outcome {
            Outcome::Walk => credits.extend([
                credit(&batter, CreditKind::Walk),
                credit(&batter, CreditKind::OrdinaryWalk),
            ]),
            Outcome::Hbp => credits.push(credit(&batter, CreditKind::HitByPitch)),
            Outcome::Roe => credits.extend([
                credit(&batter, CreditKind::AtBat),
                credit(&batter, CreditKind::NoHit),
                credit(&defense.batting_order[3], CreditKind::Error),
            ]),
            _ => {
                credits.push(credit(&batter, CreditKind::Hit));
                match hit_value {
                    1 => credits.push(credit(&batter, CreditKind::Single)),
                    2 => credits.push(credit(&batter, CreditKind::Double)),
                    3 => credits.push(credit(&batter, CreditKind::Triple)),
                    4 => kind = EventKind::OutOfParkHomeRun,
                    _ => return Err("invalid hit policy".into()),
                }
                batter_destination = destination(hit_value - 1);
            }
        }
        let award = matches!(outcome, Outcome::Walk | Outcome::Hbp);
        let mut forced = true;
        for (i, from) in [Base::First, Base::Second, Base::Third]
            .into_iter()
            .enumerate()
        {
            if let Some(runner) = &state.bases[i] {
                if !award || forced {
                    let to = winning_targets[i].unwrap_or_else(|| destination(i + advance));
                    moves.push(RunnerMove {
                        runner: runner.clone(),
                        from,
                        to,
                    });
                    if to == Destination::Home {
                        runs += 1;
                        credits.push(credit(&runner.player_id, CreditKind::Run));
                    }
                }
            } else {
                forced = false;
            }
        }
        if batter_destination == Destination::Home {
            runs += 1;
            credits.push(credit(&batter, CreditKind::Run));
        }
        if outcome != Outcome::Roe {
            for _ in 0..runs {
                credits.push(credit(&batter, CreditKind::Rbi));
            }
        }
    }
    moves.push(RunnerMove {
        runner: Runner {
            player_id: batter.clone(),
            responsible_pitcher: defense.pitcher.clone(),
        },
        from: Base::Plate,
        to: batter_destination,
    });
    let third_out = if state.outs == 2 && outs == 1 {
        Some(ThirdOut {
            runner_id: batter.clone(),
            kind: ThirdOutKind::BatterBeforeFirst,
            home_before_out: vec![],
        })
    } else {
        None
    };
    let ordinary_ground_throw =
        (outcome == Outcome::ContactOut).then(|| bbsim_engine::OrdinaryGroundThrowFacts {
            batter_id: batter.clone(),
            thrower_id: defense.batting_order[3].clone(),
            receiver_id: defense.batting_order[1].clone(),
        });
    let caught_strikeout =
        (outcome == Outcome::Strikeout).then(|| bbsim_engine::CaughtStrikeoutFacts {
            batter_id: batter.clone(),
            catcher_id: defense.batting_order[0].clone(),
        });
    let protected_award = matches!(outcome, Outcome::Walk | Outcome::Hbp).then(|| {
        bbsim_engine::ProtectedAwardFacts {
            batter_id: batter.clone(),
        }
    });
    Ok(Event {
        kind,
        participants: Participants {
            batter: Some(batter),
            pitcher: defense.pitcher.clone(),
        },
        delta: Delta {
            moves,
            outs,
            runs,
            credits,
            changes: vec![],
            third_out,
        },
        fielding_review: if outcome == Outcome::Roe {
            FieldingReview::Unknown
        } else {
            FieldingReview::NoMisplay
        },
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        caught_strikeout,
        ordinary_ground_throw,
        protected_award,
    })
}

pub fn game(seed: u64, baseline: &Baseline) -> Result<Vec<EventTransition>, String> {
    let team = |name: &str| TeamState {
        batting_order: std::array::from_fn(|i| format!("{name}-{}", i + 1)),
        next_batter: 0,
        pitcher: format!("{name}-pitcher"),
        dh_slot: Some(8),
        dh_must_bat: true,
        bench: vec![],
        removed: vec![],
    };
    let mut state = GameState {
        inning: 1,
        half: Half::Top,
        outs: 0,
        score: [0, 0],
        teams: [team("Away"), team("Home")],
        bases: [None, None, None],
        over: false,
    };
    let mut rng = Rng::new(seed);
    let mut log = Vec::new();
    while !state.over {
        if log.len() >= 10_000 {
            return Err(format!(
                "seed {seed}: resource ceiling reached, no winner fabricated"
            ));
        }
        let outcome = baseline.sample(&mut rng);
        let event = event_for(&state, outcome)?;
        let transition = apply(&state, event)
            .map_err(|e| format!("seed {seed}, play {}, {outcome:?}: {e:?}", log.len() + 1))?;
        state = transition.after().clone();
        log.push(transition);
    }
    Ok(log)
}
