//! Concrete modeled mechanisms projected through the canonical Rust reducer.
use bbsim_engine::{EventTransition, GameState};
// Shared projectors expose additional legacy-preview entries not used by every caller.
#[allow(dead_code)]
#[path = "contact_consequence.rs"]
mod contact_projection;
#[allow(dead_code)]
#[path = "full_hit.rs"]
mod hit_projection;
#[allow(dead_code)]
#[path = "full_runner.rs"]
mod runner_projection;
#[allow(dead_code)]
#[path = "full_strikeout.rs"]
mod strikeout_projection;
pub use contact_projection::ContactDecision;
pub use hit_projection::HitDecision;
// Shared test consumers need different subsets of the supplied decision API.
#[allow(unused_imports)]
pub use hit_projection::TagAttempt;
pub use runner_projection::RunnerDecision;
pub use strikeout_projection::StrikeoutDecision;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mechanism {
    Hit {
        decision: HitDecision,
        fielders: [String; 9],
    },
    UncaughtStrikeout {
        decision: StrikeoutDecision,
        fielders: [String; 9],
    },
    GroundOut {
        fielder: String,
        receiver: String,
        position: u8,
    },
    Ordinary {
        choice: Ordinary,
        fielders: [String; 9],
    },
    Contact {
        decision: ContactDecision,
        fielders: [String; 9],
    },
    AirAdvance {
        contact: bbsim_engine::AirContact,
        position: u8,
        origin: bbsim_engine::Base,
        caught: bool,
        fielders: [String; 9],
    },
    ForceChoice {
        fielder: String,
        receiver: String,
        position: u8,
        pivot_position: u8,
        failed_bunt: bool,
    },
    Runner {
        decision: RunnerDecision,
        fielders: [String; 9],
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ordinary {
    CalledStrikeout,
    SwingingStrikeout,
    Walk,
    IntentionalWalk,
    Hbp,
    // Supplied station-to-station fixtures; ordinary games use joint HitDecision.
    #[allow(dead_code)]
    Single(u8),
    #[allow(dead_code)]
    Double(u8),
    #[allow(dead_code)]
    Triple(u8),
    HomeRun(u8),
    FlyOut(u8),
    LineOut(u8),
    PopOut(u8),
    FieldingError(u8),
    ThrowingError(u8),
    CatcherInterference,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedPlay {
    transition: EventTransition,
    mechanism: Mechanism,
}
impl ResolvedPlay {
    pub fn transition(&self) -> &EventTransition {
        &self.transition
    }
    pub fn mechanism(&self) -> &Mechanism {
        &self.mechanism
    }
}
pub(crate) fn validate_defense(before: &GameState, fielders: &[String; 9]) -> Result<(), String> {
    before
        .validate_play_ready()
        .map_err(|e| format!("unready personnel: {e:?}"))?;
    if let bbsim_engine::DefenseState::Assigned(positions) =
        &before.teams[1 - before.half.offense()].defense
    {
        if positions
            .iter()
            .zip(fielders)
            .any(|(id, supplied)| id.as_ref() != Some(supplied))
        {
            return Err("positional defense differs from canonical assignments".into());
        }
    }
    use std::collections::BTreeSet;
    let defense = &before.teams[1 - before.half.offense()];
    let expected: BTreeSet<_> = defense
        .batting_order
        .iter()
        .enumerate()
        .filter(|(i, _)| defense.dh_slot != Some(*i as u8))
        .map(|(_, id)| id.as_str())
        .chain(defense.pitcher.active_id())
        .collect();
    let supplied: BTreeSet<_> = fielders.iter().map(String::as_str).collect();
    if Some(fielders[0].as_str()) != defense.pitcher.active_id()
        || supplied.len() != 9
        || expected != supplied
    {
        return Err(
            "positional defense must name exactly the active nine with current pitcher at P".into(),
        );
    }
    Ok(())
}

pub fn hit(
    before: &GameState,
    fielders: &[String; 9],
    decision: HitDecision,
) -> Result<ResolvedPlay, String> {
    validate_defense(before, fielders)?;
    let transition = hit_projection::project(before, fielders, &decision)?;
    Ok(ResolvedPlay {
        transition,
        mechanism: Mechanism::Hit {
            decision,
            fielders: fielders.clone(),
        },
    })
}
pub fn uncaught_strikeout(
    before: &GameState,
    fielders: &[String; 9],
    decision: StrikeoutDecision,
) -> Result<ResolvedPlay, String> {
    validate_defense(before, fielders)?;
    let transition = strikeout_projection::project(before, fielders, &decision)?;
    Ok(ResolvedPlay {
        transition,
        mechanism: Mechanism::UncaughtStrikeout {
            decision,
            fielders: fielders.clone(),
        },
    })
}

pub fn ordinary(
    before: &GameState,
    fielders: &[String; 9],
    choice: Ordinary,
) -> Result<ResolvedPlay, String> {
    use super::preview::Outcome;
    use bbsim_engine::{AirContact, Credit, CreditKind, FieldingReview, OrdinaryAirCatchFacts};
    validate_defense(before, fielders)?;
    let position = match choice {
        Ordinary::Single(p)
        | Ordinary::Double(p)
        | Ordinary::Triple(p)
        | Ordinary::HomeRun(p)
        | Ordinary::FlyOut(p)
        | Ordinary::LineOut(p)
        | Ordinary::PopOut(p)
        | Ordinary::FieldingError(p)
        | Ordinary::ThrowingError(p) => {
            if !(1..=9).contains(&p) {
                return Err("contact requires an assigned defensive position".into());
            }
            p as usize - 1
        }
        _ => 5,
    };
    let outcome = match choice {
        Ordinary::CalledStrikeout | Ordinary::SwingingStrikeout => Outcome::Strikeout,
        Ordinary::Walk | Ordinary::IntentionalWalk | Ordinary::CatcherInterference => Outcome::Walk,
        Ordinary::Hbp => Outcome::Hbp,
        Ordinary::Single(_) => Outcome::Single,
        Ordinary::Double(_) => Outcome::Double,
        Ordinary::Triple(_) => Outcome::Triple,
        Ordinary::HomeRun(_) => Outcome::HomeRun,
        Ordinary::FieldingError(_) | Ordinary::ThrowingError(_) => Outcome::Roe,
        Ordinary::FlyOut(_) | Ordinary::LineOut(_) | Ordinary::PopOut(_) => Outcome::ContactOut,
    };
    let mut event = super::preview::event_for_with_fielders(
        before,
        outcome,
        [&fielders[1], &fielders[position], &fielders[2]],
    )?;
    if matches!(choice, Ordinary::ThrowingError(_)) {
        // A wild throw is not the canonical sole-ground-muff proof.
        event.muffed_ground_ball = None;
        event.fielding_review = FieldingReview::Unknown;
    }
    if matches!(choice, Ordinary::IntentionalWalk) {
        for credit in &mut event.delta.credits {
            if credit.kind == CreditKind::OrdinaryWalk {
                credit.kind = CreditKind::IntentionalWalk;
            }
        }
    }
    if matches!(choice, Ordinary::CatcherInterference) {
        event.protected_award = None;
        event.fielding_review = FieldingReview::Unknown;
        event
            .delta
            .credits
            .retain(|c| !matches!(c.kind, CreditKind::Walk | CreditKind::OrdinaryWalk));
        event.delta.credits.extend([
            Credit {
                player_id: event.participants.batter.clone().ok_or("missing batter")?,
                kind: CreditKind::InterferenceAward,
            },
            Credit {
                player_id: fielders[1].clone(),
                kind: CreditKind::Error,
            },
        ]);
    }
    let contact = match choice {
        Ordinary::FlyOut(_) => Some(AirContact::Fly),
        Ordinary::LineOut(_) => Some(AirContact::LineDrive),
        Ordinary::PopOut(_) => Some(AirContact::PopUp),
        _ => None,
    };
    if let Some(contact) = contact {
        event.ordinary_ground_throw = None;
        event
            .delta
            .credits
            .retain(|c| !matches!(c.kind, CreditKind::Assist | CreditKind::Putout));
        event.delta.credits.push(Credit {
            player_id: fielders[position].clone(),
            kind: CreditKind::Putout,
        });
        event.ordinary_air_catch = Some(OrdinaryAirCatchFacts {
            batter_id: event.participants.batter.clone().ok_or("missing batter")?,
            fielder_id: fielders[position].clone(),
            contact,
        });
    }
    let transition = bbsim_engine::apply(before, event)
        .map_err(|error| format!("ordinary {choice:?} refused by canonical reducer: {error:?}"))?;
    Ok(ResolvedPlay {
        transition,
        mechanism: Mechanism::Ordinary {
            choice,
            fielders: fielders.clone(),
        },
    })
}

pub fn air_advance(
    before: &GameState,
    fielders: &[String; 9],
    air: bbsim_engine::AirContact,
    position: u8,
    origin: bbsim_engine::Base,
    caught: bool,
) -> Result<ResolvedPlay, String> {
    use bbsim_engine::*;
    validate_defense(before, fielders)?;
    let i = match origin {
        Base::First => 0,
        Base::Second => 1,
        Base::Third => 2,
        Base::Plate => return Err("air attempt requires a preexisting runner".into()),
    };
    if !(1..=9).contains(&position)
        || before.outs >= 2
        || before.bases[i].is_none()
        || (i < 2 && before.bases[i + 1].is_some())
    {
        return Err(
            "air attempt requires a runner, an open next base and fewer than two outs".into(),
        );
    }
    let batter = before.teams[before.half.offense()].batting_order
        [usize::from(before.teams[before.half.offense()].next_batter)]
    .clone();
    let selected = before.bases[i].as_ref().unwrap();
    if i == 2 && !caught {
        if position < 7 {
            return Err("modeled scoring retouch requires an outfielder".into());
        }
        return contact(
            before,
            fielders,
            ContactDecision::CaughtSacrificeFly {
                catch: OrdinaryAirCatchFacts {
                    batter_id: batter,
                    fielder_id: fielders[usize::from(position) - 1].clone(),
                    contact: air,
                },
                facts: CaughtSacrificeFlyFacts {
                    scoring_runner_id: selected.player_id.clone(),
                    handling: SacrificeFlyHandling::QualifyingOutfielderOrInfielderInOutfield,
                    retouch: SacrificeFlyRetouch::Legal,
                    timing: SacrificeFlyRunTiming::AfterCatch,
                },
            },
        );
    }
    let receiver = [3, 4, 1][i];
    let fielder = usize::from(position) - 1;
    let mut event = super::preview::event_for_with_fielders(
        before,
        super::preview::Outcome::ContactOut,
        [&fielders[1], &fielders[fielder], &fielders[fielder]],
    )?;
    // Held-only air/ground proofs cannot certify this modeled retouch-and-tag episode.
    event.ordinary_ground_throw = None;
    event.delta.moves.push(RunnerMove {
        runner: selected.clone(),
        from: origin,
        to: if caught {
            Destination::Out
        } else {
            [Destination::Second, Destination::Third, Destination::Home][i]
        },
    });
    event.delta.credits.retain(|c| c.kind != CreditKind::Assist);
    if caught {
        event.delta.outs = 2;
        if receiver != fielder {
            event.delta.credits.push(Credit {
                player_id: fielders[fielder].clone(),
                kind: CreditKind::Assist,
            });
        }
        event.delta.credits.push(Credit {
            player_id: fielders[receiver].clone(),
            kind: CreditKind::Putout,
        });
        event.delta.third_out = (before.outs == 1).then(|| ThirdOut {
            runner_id: selected.player_id.clone(),
            kind: ThirdOutKind::Timing,
            home_before_out: vec![],
        });
    }
    let transition =
        apply(before, event).map_err(|e| format!("modeled air advance refused: {e:?}"))?;
    Ok(ResolvedPlay {
        transition,
        mechanism: Mechanism::AirAdvance {
            contact: air,
            position,
            origin,
            caught,
            fielders: fielders.clone(),
        },
    })
}

pub fn force_choice(
    before: &GameState,
    fielders: &[String; 9],
    position: u8,
    failed_bunt: bool,
) -> Result<ResolvedPlay, String> {
    use bbsim_engine::{
        Base, Credit, CreditKind, Destination, EventKind, RunnerMove, ThirdOut, ThirdOutKind,
    };
    validate_defense(before, fielders)?;
    if !(1..=6).contains(&position) || before.bases[0].is_none() {
        return Err("force choice requires a runner on first and an infielder".into());
    }
    let pivot = if position == 4 { 5 } else { 3 };
    let mut event = super::preview::event_for_with_fielders(
        before,
        super::preview::Outcome::ContactOut,
        [
            &fielders[1],
            &fielders[usize::from(position) - 1],
            &fielders[pivot],
        ],
    )?;
    event.kind = EventKind::FieldersChoice;
    event.ordinary_ground_throw = None;
    let retired = before.bases[0].as_ref().unwrap();
    event.delta.moves[0].to = Destination::First;
    event.delta.moves[0].runner.responsible_pitcher = retired.responsible_pitcher.clone();
    event.delta.moves.push(RunnerMove {
        runner: retired.clone(),
        from: Base::First,
        to: Destination::Out,
    });
    event
        .delta
        .credits
        .retain(|c| !matches!(c.kind, CreditKind::Putout | CreditKind::Assist));
    event.delta.credits.extend([
        Credit {
            player_id: fielders[usize::from(position) - 1].clone(),
            kind: CreditKind::Assist,
        },
        Credit {
            player_id: fielders[pivot].clone(),
            kind: CreditKind::Putout,
        },
    ]);
    event.delta.third_out = (before.outs == 2).then(|| ThirdOut {
        runner_id: retired.player_id.clone(),
        kind: ThirdOutKind::Force,
        home_before_out: vec![],
    });
    let transition =
        bbsim_engine::apply(before, event).map_err(|e| format!("force choice refused: {e:?}"))?;
    Ok(ResolvedPlay {
        transition,
        mechanism: Mechanism::ForceChoice {
            fielder: fielders[usize::from(position) - 1].clone(),
            receiver: fielders[pivot].clone(),
            position,
            pivot_position: pivot as u8 + 1,
            failed_bunt,
        },
    })
}

pub fn contact(
    before: &GameState,
    fielders: &[String; 9],
    decision: ContactDecision,
) -> Result<ResolvedPlay, String> {
    validate_defense(before, fielders)?;
    let transition = contact_projection::project(before, &decision)?;
    Ok(ResolvedPlay {
        transition,
        mechanism: Mechanism::Contact {
            decision,
            fielders: fielders.clone(),
        },
    })
}
pub fn runner(
    before: &GameState,
    fielders: &[String; 9],
    decision: RunnerDecision,
) -> Result<ResolvedPlay, String> {
    validate_defense(before, fielders)?;
    let transition = runner_projection::project(before, fielders, &decision)?;
    Ok(ResolvedPlay {
        transition,
        mechanism: Mechanism::Runner {
            decision,
            fielders: fielders.clone(),
        },
    })
}

pub fn ground_out(
    before: &GameState,
    fielders: &[String; 9],
    position: u8,
) -> Result<ResolvedPlay, String> {
    if !(1..=6).contains(&position) {
        return Err("ordinary infield grounder requires assigned P/C/1B/2B/3B/SS".into());
    }
    validate_defense(before, fielders)?;
    let fielder = &fielders[position as usize - 1];
    let receiver = &fielders[2];
    let mut event = super::preview::event_for_with_fielders(
        before,
        super::preview::Outcome::ContactOut,
        [&fielders[1], fielder, receiver],
    )?;
    if position == 3 {
        // An unassisted out is not the existing two-actor direct-throw proof.
        event.ordinary_ground_throw = None;
        event
            .delta
            .credits
            .retain(|credit| credit.kind != bbsim_engine::CreditKind::Assist);
    }
    let transition = bbsim_engine::apply(before, event)
        .map_err(|error| format!("ground mechanism refused by canonical reducer: {error:?}"))?;
    Ok(ResolvedPlay {
        transition,
        mechanism: Mechanism::GroundOut {
            fielder: fielder.clone(),
            receiver: receiver.clone(),
            position,
        },
    })
}
