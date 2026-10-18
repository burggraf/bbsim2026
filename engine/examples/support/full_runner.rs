//! Supplied runner-only decisions; sampling and eligibility policy stay with the caller.
use bbsim_engine::{
    apply, BalkCause, BalkDisposition, BalkFacts, BalkResolution, Base, Credit, CreditKind,
    DeliveryContext, Delta, Destination, Event, EventKind, EventTransition, FieldingReview,
    GameState, HandlingAdvance, HandlingAssessment, Participants, RecoveryReview, RunnerEvent,
    RunnerMove, SingleDeliveryFacts, ThirdOut, ThirdOutKind,
};

// Positional assignment order is supplied by the caller, not inferred here.
const PITCHER: usize = 0;
const CATCHER: usize = 1;
const FIRST_BASE: usize = 2;
const SECOND_BASE: usize = 3;
const THIRD_BASE: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunnerDecision {
    Steal {
        origin: Base,
        target: Destination,
        caught: bool,
    },
    Pickoff {
        origin: Base,
        caught: bool,
    },
    WildPitch,
    PassedBall,
    Balk,
}

fn base_index(base: Base) -> Option<usize> {
    match base {
        Base::First => Some(0),
        Base::Second => Some(1),
        Base::Third => Some(2),
        Base::Plate => None,
    }
}

fn base_at(index: usize) -> Base {
    [Base::First, Base::Second, Base::Third][index]
}

fn next_base(index: usize) -> Destination {
    [Destination::Second, Destination::Third, Destination::Home][index]
}

fn held_base(index: usize) -> Destination {
    [Destination::First, Destination::Second, Destination::Third][index]
}

fn credit(player_id: &str, kind: CreditKind) -> Credit {
    Credit {
        player_id: player_id.to_owned(),
        kind,
    }
}

fn base_moves(before: &GameState, changed: Option<(usize, Destination)>) -> Vec<RunnerMove> {
    before
        .bases
        .iter()
        .enumerate()
        .filter_map(|(index, runner)| {
            runner.as_ref().map(|runner| RunnerMove {
                runner: runner.clone(),
                from: base_at(index),
                to: changed
                    .filter(|(changed_index, _)| *changed_index == index)
                    .map_or_else(|| held_base(index), |(_, target)| target),
            })
        })
        .collect()
}

fn all_advances(before: &GameState) -> Vec<RunnerMove> {
    before
        .bases
        .iter()
        .enumerate()
        .filter_map(|(index, runner)| {
            runner.as_ref().map(|runner| RunnerMove {
                runner: runner.clone(),
                from: base_at(index),
                to: next_base(index),
            })
        })
        .collect()
}

fn add_run_credits(moves: &[RunnerMove], credits: &mut Vec<Credit>) -> u8 {
    for movement in moves
        .iter()
        .filter(|movement| movement.to == Destination::Home)
    {
        credits.push(credit(&movement.runner.player_id, CreditKind::Run));
    }
    u8::try_from(
        moves
            .iter()
            .filter(|movement| movement.to == Destination::Home)
            .count(),
    )
    .expect("at most three runners can score in a runner-only event")
}

fn third_out(before: &GameState, runner_id: &str, caught: bool) -> Option<ThirdOut> {
    (caught && before.outs == 2).then(|| ThirdOut {
        runner_id: runner_id.to_owned(),
        kind: ThirdOutKind::Timing,
        home_before_out: Vec::new(),
    })
}

fn delta(
    moves: Vec<RunnerMove>,
    outs: u8,
    mut credits: Vec<Credit>,
    third_out: Option<ThirdOut>,
) -> Delta {
    let runs = add_run_credits(&moves, &mut credits);
    Delta {
        moves,
        outs,
        runs,
        credits,
        changes: Vec::new(),
        third_out,
    }
}

fn event(
    before: &GameState,
    fielders: &[String; 9],
    kind: RunnerEvent,
    fielding_review: FieldingReview,
    delta: Delta,
    single_delivery: Option<SingleDeliveryFacts>,
    balk: Option<BalkFacts>,
) -> Result<Event, String> {
    let offense = before.half.offense();
    let batter = before.teams[offense]
        .batting_order
        .get(usize::from(before.teams[offense].next_batter))
        .ok_or("invalid runner-only batting cursor")?
        .clone();
    Ok(Event {
        kind: EventKind::RunnerOnly(kind),
        participants: Participants {
            batter: Some(batter),
            pitcher: Some(fielders[PITCHER].clone()),
        },
        fielding_review,
        delta,
        two_out_contact: None,
        single_delivery,
        balk,
        ordinary_ground_throw: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        muffed_ground_ball: None,
        ordinary_air_catch: None,
    })
}

fn apply_event(before: &GameState, action: Event) -> Result<EventTransition, String> {
    apply(before, action)
        .map_err(|error| format!("canonical runner-only projection refused: {error:?}"))
}

fn runner_at(before: &GameState, origin: Base) -> Result<(usize, &bbsim_engine::Runner), String> {
    let index = base_index(origin).ok_or("runner origin must be First, Second or Third")?;
    let runner = before.bases[index]
        .as_ref()
        .ok_or("selected runner origin is empty")?;
    Ok((index, runner))
}

fn handling_event(
    before: &GameState,
    fielders: &[String; 9],
    handling: HandlingAssessment,
) -> Result<EventTransition, String> {
    let moves = all_advances(before);
    if moves.is_empty() {
        return Err("runner-only pitch event requires an occupied base".into());
    }
    let mut credits = Vec::new();
    let (kind, fielding_review) = match handling {
        HandlingAssessment::PitcherWild => {
            credits.push(credit(&fielders[PITCHER], CreditKind::WildPitch));
            (RunnerEvent::WildPitch, FieldingReview::NoMisplay)
        }
        HandlingAssessment::CatcherPassed => {
            credits.push(credit(&fielders[CATCHER], CreditKind::PassedBall));
            (RunnerEvent::PassedBall, FieldingReview::Unknown)
        }
    };
    let facts = SingleDeliveryFacts {
        context: DeliveryContext::RunnerOnly,
        handling,
        catcher_id: fielders[CATCHER].clone(),
        advances: moves
            .iter()
            .map(|movement| HandlingAdvance {
                runner_id: movement.runner.player_id.clone(),
                reached: movement.to,
            })
            .collect(),
        pre_delivery_steals: Some(Vec::new()),
        recovery: RecoveryReview::NoRecoveryPutout,
    };
    let action = event(
        before,
        fielders,
        kind,
        fielding_review,
        delta(moves, 0, credits, None),
        Some(facts),
        None,
    )?;
    apply_event(before, action)
}

/// Project one supplied decision; the canonical reducer validates and derives the after-state.
/// `fielders` is ordered P/C/1B/2B/3B/SS/LF/CF/RF; selected actors are
/// checked by the canonical transition validator.
pub fn project(
    before: &GameState,
    fielders: &[String; 9],
    decision: &RunnerDecision,
) -> Result<EventTransition, String> {
    match *decision {
        RunnerDecision::Steal {
            origin,
            target,
            caught,
        } => {
            let (index, runner) = runner_at(before, origin)?;
            if target != next_base(index) {
                return Err("steal target must be the next base".into());
            }
            if target != Destination::Home
                && before.bases[match target {
                    Destination::First => 0,
                    Destination::Second => 1,
                    Destination::Third => 2,
                    Destination::Home | Destination::Out => unreachable!(),
                }]
                .is_some()
            {
                return Err("steal target is occupied".into());
            }
            let destination = if caught { Destination::Out } else { target };
            let moves = base_moves(before, Some((index, destination)));
            let mut credits = vec![credit(
                &runner.player_id,
                if caught {
                    CreditKind::CaughtStealing
                } else {
                    CreditKind::StolenBase
                },
            )];
            if caught {
                let putout = match target {
                    Destination::Second => SECOND_BASE,
                    Destination::Third => THIRD_BASE,
                    Destination::Home => CATCHER,
                    Destination::First | Destination::Out => unreachable!(),
                };
                credits.push(credit(&fielders[putout], CreditKind::Putout));
                if target != Destination::Home {
                    credits.push(credit(&fielders[CATCHER], CreditKind::Assist));
                }
            }
            let action = event(
                before,
                fielders,
                RunnerEvent::StealAttempt,
                FieldingReview::NoMisplay,
                delta(
                    moves,
                    u8::from(caught),
                    credits,
                    third_out(before, &runner.player_id, caught),
                ),
                None,
                None,
            )?;
            apply_event(before, action)
        }
        RunnerDecision::Pickoff { origin, caught } => {
            let (index, runner) = runner_at(before, origin)?;
            let moves = base_moves(
                before,
                Some((
                    index,
                    if caught {
                        Destination::Out
                    } else {
                        held_base(index)
                    },
                )),
            );
            let mut credits = Vec::new();
            if caught {
                let putout = [FIRST_BASE, SECOND_BASE, THIRD_BASE][index];
                credits.push(credit(&fielders[putout], CreditKind::Putout));
                credits.push(credit(&fielders[PITCHER], CreditKind::Assist));
            }
            let action = event(
                before,
                fielders,
                RunnerEvent::Pickoff,
                FieldingReview::NoMisplay,
                delta(
                    moves,
                    u8::from(caught),
                    credits,
                    third_out(before, &runner.player_id, caught),
                ),
                None,
                None,
            )?;
            apply_event(before, action)
        }
        RunnerDecision::WildPitch => {
            handling_event(before, fielders, HandlingAssessment::PitcherWild)
        }
        RunnerDecision::PassedBall => {
            handling_event(before, fielders, HandlingAssessment::CatcherPassed)
        }
        RunnerDecision::Balk => {
            let moves = all_advances(before);
            if moves.is_empty() {
                return Err("runner-only balk requires an occupied base".into());
            }
            let facts = BalkFacts {
                cause: BalkCause::PitcherAction602a,
                disposition: BalkDisposition::PenaltyApplied,
                resolution: BalkResolution::ExactProtectedAwards,
            };
            let action = event(
                before,
                fielders,
                RunnerEvent::Balk,
                FieldingReview::NoMisplay,
                delta(
                    moves,
                    0,
                    vec![credit(&fielders[PITCHER], CreditKind::Balk)],
                    None,
                ),
                None,
                Some(facts),
            )?;
            apply_event(before, action)
        }
    }
}
