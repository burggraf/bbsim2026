//! Scripted presentation only: all state changes and statistics belong to the engine.
use bbsim_engine::{
    apply, Base, Credit, CreditKind, Delta, Destination, Event, EventKind, EventTransition,
    FieldingReview, GameState, Half, Participants, Runner, RunnerEvent, RunnerMove, TeamState,
    TransitionError,
};
use std::io::{self, BufRead, Write};
#[path = "presentation.rs"]
mod presentation;

fn team(name: &str) -> TeamState {
    TeamState {
        batting_order: std::array::from_fn(|i| format!("{name}-{}", i + 1)),
        next_batter: 0,
        pitcher: format!("{name}-pitcher"),
        dh_slot: Some(8),
        dh_must_bat: true,
        bench: vec![],
        removed: vec![],
    }
}
fn credit(id: &str, kind: CreditKind) -> Credit {
    Credit {
        player_id: id.into(),
        kind,
    }
}
pub fn scripted_game() -> Result<Vec<EventTransition>, TransitionError> {
    let mut state = GameState {
        inning: 1,
        half: Half::Top,
        outs: 0,
        score: [0, 0],
        teams: [team("Away"), team("Home")],
        bases: [None, None, None],
        over: false,
    };
    let mut log = Vec::new();
    while !state.over {
        // A fixture guard, not a rule or simulated game-length limit.
        if log.len() >= 80 {
            return Err(TransitionError::UnsupportedRule);
        }
        let offense = state.half.offense();
        let batter =
            state.teams[offense].batting_order[state.teams[offense].next_batter as usize].clone();
        let pitcher = state.teams[1 - offense].pitcher.clone();
        let mut kind = EventKind::PlateAppearance;
        let mut moves = vec![RunnerMove {
            runner: Runner {
                player_id: batter.clone(),
                responsible_pitcher: pitcher.clone(),
            },
            from: Base::Plate,
            to: Destination::Out,
        }];
        let mut credits = vec![credit(&batter, CreditKind::PlateAppearance)];
        let mut outs = 0;
        let mut runs = 0;
        match log.len() {
            0 => {
                moves[0].to = Destination::First;
                credits.extend([
                    credit(&batter, CreditKind::Hit),
                    credit(&batter, CreditKind::Single),
                ]);
            }
            1 => {
                kind = EventKind::RunnerOnly(RunnerEvent::StealAttempt);
                let runner = state.bases[0]
                    .clone()
                    .expect("script single precedes steal");
                credits = vec![credit(&runner.player_id, CreditKind::StolenBase)];
                moves = vec![RunnerMove {
                    runner,
                    from: Base::First,
                    to: Destination::Second,
                }];
            }
            2 => {
                moves[0].to = Destination::First;
                credits.extend([
                    credit(&batter, CreditKind::Walk),
                    credit(&batter, CreditKind::OrdinaryWalk),
                ]);
            }
            _ if log.len() == 3
                || (state.inning == 1
                    && state.half == Half::Bottom
                    && state.outs == 0
                    && state.score[1] == 0) =>
            {
                kind = EventKind::OutOfParkHomeRun;
                moves[0].to = Destination::Home;
                credits.extend([
                    credit(&batter, CreditKind::Hit),
                    credit(&batter, CreditKind::Run),
                ]);
                runs = 1;
                for (base, runner) in [Base::First, Base::Second, Base::Third]
                    .into_iter()
                    .zip(&state.bases)
                {
                    if let Some(runner) = runner {
                        moves.push(RunnerMove {
                            runner: runner.clone(),
                            from: base,
                            to: Destination::Home,
                        });
                        credits.push(credit(&runner.player_id, CreditKind::Run));
                        runs += 1;
                    }
                }
            }
            _ => {
                outs = 1;
                credits.push(credit(&batter, CreditKind::Strikeout));
            }
        }
        let event = Event {
            kind,
            participants: Participants {
                batter: Some(batter),
                pitcher,
            },
            delta: Delta {
                moves,
                outs,
                runs,
                credits,
                changes: vec![],
                third_out: None,
            },
            fielding_review: FieldingReview::NoMisplay,
            two_out_contact: None,
            single_delivery: None,
            balk: None,
            caught_strikeout: None,
            protected_award: None,
            sole_home_run: None,
            sole_safe_hit: None,
            ordinary_ground_throw: None,
        };
        let transition = apply(&state, event)?;
        state = transition.after().clone();
        log.push(transition);
    }
    Ok(log)
}
pub fn run(input: &mut dyn BufRead, output: &mut dyn Write, autoplay: bool) -> io::Result<()> {
    let log = scripted_game().map_err(|error| io::Error::other(format!("script: {error:?}")))?;
    writeln!(
        output,
        "SCRIPTED ENGINE DEMO — fictional teams, not a probability model."
    )?;
    writeln!(
        output,
        "Opening: single, steal, walk, three-run HR; Home answers with a solo HR."
    )?;
    writeln!(
        output,
        "Remaining outs are scripted strikeouts. Same game on every restart.\n"
    )?;
    presentation::present(&log, input, output, autoplay)
}
