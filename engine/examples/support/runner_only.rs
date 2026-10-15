//! Standalone supplied-decision projection; no RNG, attempt policy or integration.
use bbsim_engine::{
    apply, Base, Credit, CreditKind, Delta, Destination, Event, EventKind, EventTransition,
    FieldingReview, GameState, Half, Participants, RunnerEvent, RunnerMove, ThirdOut, ThirdOutKind,
};

pub const POLICY_VERSION: &str = "ordinary-runner-only-projection-v1";

/// A supplied modeled assertion, never inferred from historical labels/endpoints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StealCause {
    OrdinaryReviewed,
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StealResult {
    Safe,
    Caught {
        putout: String,
        assists: Vec<String>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StealDecision {
    pub runner_id: String,
    pub target: Destination,
    pub cause: StealCause,
    pub result: StealResult,
}

/// Project one supplied decision; canonical apply validates and derives after-state.
pub fn project(before: &GameState, decision: &StealDecision) -> Result<EventTransition, String> {
    if before.half == Half::Bottom && before.inning >= 9 {
        return Err("terminal bottom-half runner-only projection unsupported".into());
    }
    if before.over || before.inning == 0 || before.outs > 2 {
        return Err("invalid or ended runner-only state".into());
    }
    if decision.cause != StealCause::OrdinaryReviewed {
        return Err("missing explicit ordinary reviewed steal cause".into());
    }
    let (origin, runner) = before
        .bases
        .iter()
        .enumerate()
        .find_map(|(index, runner)| {
            runner
                .as_ref()
                .filter(|runner| runner.player_id == decision.runner_id)
                .map(|runner| (index, runner))
        })
        .ok_or("named steal runner not on a before base")?;
    let target = match (origin, decision.target) {
        (0, Destination::Second) | (1, Destination::Third) => origin + 1,
        _ => return Err("only the next Second/Third steal target is supported".into()),
    };
    if before.bases[target].is_some() {
        return Err("steal target is occupied".into());
    }
    let mut credits = Vec::new();
    let caught = match &decision.result {
        StealResult::Safe => {
            credits.push(Credit {
                player_id: runner.player_id.clone(),
                kind: CreditKind::StolenBase,
            });
            false
        }
        StealResult::Caught { putout, assists } => {
            if assists
                .iter()
                .enumerate()
                .any(|(index, actor)| assists[index + 1..].contains(actor))
            {
                return Err("repeated assist units need unsupported credit chronology".into());
            }
            credits.push(Credit {
                player_id: runner.player_id.clone(),
                kind: CreditKind::CaughtStealing,
            });
            credits.push(Credit {
                player_id: putout.clone(),
                kind: CreditKind::Putout,
            });
            credits.extend(assists.iter().map(|actor| Credit {
                player_id: actor.clone(),
                kind: CreditKind::Assist,
            }));
            true
        }
    };
    let offense = before.half.offense();
    let batter = before.teams[offense]
        .batting_order
        .get(usize::from(before.teams[offense].next_batter))
        .ok_or("invalid runner-only batting cursor")?;
    let moves = before
        .bases
        .iter()
        .enumerate()
        .filter_map(|(index, runner)| {
            runner.as_ref().map(|runner| RunnerMove {
                runner: runner.clone(),
                from: [Base::First, Base::Second, Base::Third][index],
                to: if index == origin {
                    if caught {
                        Destination::Out
                    } else {
                        decision.target
                    }
                } else {
                    [Destination::First, Destination::Second, Destination::Third][index]
                },
            })
        })
        .collect();
    let event = Event {
        kind: EventKind::RunnerOnly(RunnerEvent::StealAttempt),
        participants: Participants {
            batter: Some(batter.clone()),
            pitcher: before.teams[1 - offense].pitcher.clone(),
        },
        fielding_review: FieldingReview::NoMisplay,
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        ordinary_air_catch: None,
        muffed_ground_ball: None,
        ordinary_ground_throw: None,
        delta: Delta {
            moves,
            outs: u8::from(caught),
            runs: 0,
            credits,
            changes: vec![],
            third_out: (caught && before.outs == 2).then(|| ThirdOut {
                runner_id: runner.player_id.clone(),
                kind: ThirdOutKind::Timing,
                home_before_out: vec![],
            }),
        },
    };
    apply(before, event)
        .map_err(|error| format!("canonical runner-only projection refused: {error:?}"))
}
