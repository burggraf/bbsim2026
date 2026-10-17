//! Supplied uncaught-third-strike leaves; canonical `apply` owns every transition.
use bbsim_engine::{
    apply, Base, Credit, CreditKind, DeliveryContext, Delta, Destination, Event, EventKind,
    EventTransition, FieldingReview, GameState, HandlingAdvance, HandlingAssessment, Participants,
    RecoveryReview, Runner, RunnerMove, SingleDeliveryFacts, ThirdOut, ThirdOutKind,
};
use std::collections::BTreeSet;

const PITCHER: usize = 0;
const CATCHER: usize = 1;
const FIRST_BASE: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrikeoutDecision {
    /// Batter is ruled out; before-runners hold and no handling is supplied.
    Automatic,
    /// Batter reaches first; each before-runner advances one base jointly.
    SafeReach { handling: HandlingAssessment },
    /// Catcher throws the recovered ball to first before any runner advances;
    /// before-runners hold, so no WP/PB unit is credited.
    RecoveredBeforeFirst { handling: HandlingAssessment },
    /// All before-runners advance one base before the catcher-to-first out.
    RecoveredBeforeFirstAfterAdvances { handling: HandlingAssessment },
}

fn credit(player_id: &str, kind: CreditKind) -> Credit {
    Credit {
        player_id: player_id.to_owned(),
        kind,
    }
}

fn validate_fielders(before: &GameState, fielders: &[String; 9]) -> Result<(), String> {
    let defense = &before.teams[1 - before.half.offense()];
    if fielders[PITCHER] != defense.pitcher {
        return Err("fielding pitcher must match the current defensive pitcher".into());
    }
    let unique = fielders.iter().collect::<BTreeSet<_>>();
    if unique.len() != fielders.len() || fielders.iter().any(|id| id.is_empty()) {
        return Err("fielders must be nine distinct nonempty players".into());
    }
    if fielders[1..].iter().any(|fielder| {
        !defense
            .batting_order
            .iter()
            .enumerate()
            .any(|(slot, player)| Some(slot as u8) != defense.dh_slot && player == fielder)
    }) {
        return Err("non-pitcher fielders must be active defensive lineup players".into());
    }
    Ok(())
}

fn next_base(index: usize) -> Destination {
    [Destination::Second, Destination::Third, Destination::Home][index]
}

fn base(index: usize) -> Base {
    [Base::First, Base::Second, Base::Third][index]
}

fn one_base_advances(before: &GameState) -> Vec<RunnerMove> {
    before
        .bases
        .iter()
        .enumerate()
        .filter_map(|(index, runner)| {
            runner.as_ref().map(|runner| RunnerMove {
                runner: runner.clone(),
                from: base(index),
                to: next_base(index),
            })
        })
        .collect()
}

/// Fixed joint policy: a safe handling-caused reach advances every before-runner
/// exactly one base. A pure automatic out holds runners because no handling was
/// supplied. RecoveredBeforeFirst holds; its AfterAdvances counterpart completes
/// every before-runner's advance before retiring the batter before First.
pub fn project(
    before: &GameState,
    fielders: &[String; 9],
    decision: &StrikeoutDecision,
) -> Result<EventTransition, String> {
    if before.over || before.inning == 0 || before.outs > 2 {
        return Err("invalid or ended game state".into());
    }
    validate_fielders(before, fielders)?;
    let offense = before.half.offense();
    let batter = before.teams[offense]
        .batting_order
        .get(usize::from(before.teams[offense].next_batter))
        .ok_or("invalid batting cursor")?
        .clone();
    let eligible = before.bases[0].is_none() || before.outs == 2;

    let (handling, batter_destination, recovered, advance_runners) = match *decision {
        StrikeoutDecision::Automatic => {
            if before.bases[0].is_none() || before.outs >= 2 {
                return Err(
                    "automatic uncaught strikeout requires occupied first and fewer than two outs"
                        .into(),
                );
            }
            // No handling cause was supplied for this rule-created retirement, so
            // before-runners hold and no WP/PB is inferred.
            (None, Destination::Out, false, false)
        }
        StrikeoutDecision::SafeReach { handling } => {
            if !eligible {
                return Err(
                    "safe reach is ineligible with first occupied and fewer than two outs".into(),
                );
            }
            (Some(handling), Destination::First, false, true)
        }
        StrikeoutDecision::RecoveredBeforeFirstAfterAdvances { handling } => {
            if !eligible || before.bases.iter().all(Option::is_none) {
                return Err(
                    "recovered advance requires an eligible batter and before-runner".into(),
                );
            }
            (Some(handling), Destination::Out, true, true)
        }
        StrikeoutDecision::RecoveredBeforeFirst { handling } => {
            if !eligible {
                return Err(
                    "recovered batter is ineligible with first occupied and fewer than two outs"
                        .into(),
                );
            }
            (Some(handling), Destination::Out, true, false)
        }
    };

    let mut moves = if advance_runners {
        one_base_advances(before)
    } else {
        Vec::new()
    };
    let batter_runner = Runner {
        player_id: batter.clone(),
        responsible_pitcher: fielders[PITCHER].clone(),
    };
    moves.push(RunnerMove {
        runner: batter_runner,
        from: Base::Plate,
        to: batter_destination,
    });

    let outs = u8::from(batter_destination == Destination::Out);
    let third_out = (outs == 1 && before.outs == 2).then(|| ThirdOut {
        runner_id: batter.clone(),
        kind: ThirdOutKind::BatterBeforeFirst,
        home_before_out: moves
            .iter()
            .filter(|movement| movement.to == Destination::Home)
            .map(|movement| movement.runner.player_id.clone())
            .collect(),
    });
    let runs = if third_out.is_some() {
        0
    } else {
        u8::try_from(
            moves
                .iter()
                .filter(|movement| movement.to == Destination::Home)
                .count(),
        )
        .map_err(|_| "too many runners to score")?
    };

    let mut credits = vec![
        credit(&batter, CreditKind::PlateAppearance),
        credit(&batter, CreditKind::Strikeout),
    ];
    credits.extend(
        moves
            .iter()
            .filter(|movement| movement.to == Destination::Home && third_out.is_none())
            .map(|movement| credit(&movement.runner.player_id, CreditKind::Run)),
    );

    let mut advances = moves
        .iter()
        .filter(|movement| movement.from != Base::Plate)
        .map(|movement| HandlingAdvance {
            runner_id: movement.runner.player_id.clone(),
            reached: movement.to,
        })
        .collect::<Vec<_>>();
    if batter_destination == Destination::First {
        advances.push(HandlingAdvance {
            runner_id: batter.clone(),
            reached: Destination::First,
        });
    }

    if let Some(handling) = handling {
        let has_qualifying_advance = !advances.is_empty();
        if has_qualifying_advance {
            match handling {
                HandlingAssessment::PitcherWild => {
                    credits.push(credit(&fielders[PITCHER], CreditKind::WildPitch));
                }
                HandlingAssessment::CatcherPassed => {
                    credits.push(credit(&fielders[CATCHER], CreditKind::PassedBall));
                }
            }
        }
    }

    if recovered {
        credits.push(credit(&fielders[FIRST_BASE], CreditKind::Putout));
        credits.push(credit(&fielders[CATCHER], CreditKind::Assist));
    } else if batter_destination == Destination::Out {
        // The rule-created automatic batter retirement is credited to the catcher.
        credits.push(credit(&fielders[CATCHER], CreditKind::Putout));
    }

    let recovery = if recovered && advance_runners {
        RecoveryReview::RecoveredThirdStrikeBatterBeforeFirstAfterAdvances {
            putout_fielder_id: fielders[FIRST_BASE].clone(),
        }
    } else if recovered {
        RecoveryReview::BeforeAnyAdvance {
            runner_id: batter.clone(),
        }
    } else {
        RecoveryReview::NoRecoveryPutout
    };
    let single_delivery = handling.map(|handling| SingleDeliveryFacts {
        context: DeliveryContext::UncaughtThirdStrike,
        handling,
        catcher_id: fielders[CATCHER].clone(),
        advances,
        pre_delivery_steals: Some(Vec::new()),
        recovery,
    });
    let fielding_review = match handling {
        Some(HandlingAssessment::CatcherPassed) | None => FieldingReview::Unknown,
        Some(HandlingAssessment::PitcherWild) => FieldingReview::NoMisplay,
    };

    let event = Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(batter),
            pitcher: fielders[PITCHER].clone(),
        },
        fielding_review,
        delta: Delta {
            moves,
            outs,
            runs,
            credits,
            changes: Vec::new(),
            third_out,
        },
        two_out_contact: None,
        single_delivery,
        balk: None,
        ordinary_ground_throw: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        muffed_ground_ball: None,
        ordinary_air_catch: None,
    };
    apply(before, event)
        .map_err(|error| format!("canonical uncaught-strikeout projection refused: {error:?}"))
}
