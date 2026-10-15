//! Standalone supplied clean contact decisions; no RNG or producer integration.
use bbsim_engine::{
    apply_with_caught_sacrifice_fly_facts, apply_with_ground_force_double_play_facts,
    apply_with_sacrifice_bunt_throw_facts, Base, CaughtSacrificeFlyFacts, ContactClass,
    ContactContinuity, Credit, CreditKind, Delta, Destination, Event, EventKind, EventTransition,
    FieldingReview, GameState, GroundForceDoublePlayFacts, OrdinaryAirCatchFacts, Participants,
    RetirementCause, RetirementFact, ReviewedErrorPlacement, Runner, RunnerMove,
    SacrificeBuntThrowFacts, ThirdOut, ThirdOutKind, TwoOutContactFacts,
};

pub const POLICY_VERSION: &str = "supplied-contact-consequence-projection-v1";

/// Explicit modeled assertions of the documented clean episode, not source labels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContactDecision {
    CaughtSacrificeFly {
        catch: OrdinaryAirCatchFacts,
        facts: CaughtSacrificeFlyFacts,
    },
    SacrificeBunt(SacrificeBuntThrowFacts),
    GroundForceDoublePlay(GroundForceDoublePlayFacts),
}

fn credit(actor: &str, kind: CreditKind) -> Credit {
    Credit {
        player_id: actor.into(),
        kind,
    }
}

/// Construct one supplied episode; canonical APIs validate and own the after-state.
pub fn project(before: &GameState, decision: &ContactDecision) -> Result<EventTransition, String> {
    let offense = &before.teams[before.half.offense()];
    let batter = offense
        .batting_order
        .get(usize::from(offense.next_batter))
        .ok_or("invalid supplied-contact batting cursor")?
        .clone();
    let pitcher = before.teams[1 - before.half.offense()].pitcher.clone();
    let sf = matches!(decision, ContactDecision::CaughtSacrificeFly { .. });
    let dp = matches!(decision, ContactDecision::GroundForceDoublePlay(_));
    let mut credits = vec![
        credit(&batter, CreditKind::PlateAppearance),
        credit(&batter, CreditKind::NoHit),
    ];
    let mut air = None;
    let mut contact = None;
    match decision {
        ContactDecision::CaughtSacrificeFly { catch, .. } => {
            let scorer = before.bases[2]
                .as_ref()
                .ok_or("missing before-Third scorer for supplied caught-SF")?;
            air = Some(catch.clone());
            credits.extend([
                credit(&batter, CreditKind::SacrificeFly),
                credit(&batter, CreditKind::Rbi),
                credit(&scorer.player_id, CreditKind::Run),
                credit(&catch.fielder_id, CreditKind::Putout),
            ]);
        }
        ContactDecision::SacrificeBunt(facts) => credits.extend([
            credit(&batter, CreditKind::SacrificeBunt),
            credit(&facts.receiver_id, CreditKind::Putout),
            credit(&facts.thrower_id, CreditKind::Assist),
        ]),
        ContactDecision::GroundForceDoublePlay(facts) => {
            let forced = before.bases[0]
                .as_ref()
                .ok_or("missing before-First runner for supplied force/relay")?;
            credits.extend([
                credit(&batter, CreditKind::AtBat),
                credit(&batter, CreditKind::GroundedIntoDoublePlay),
                credit(&facts.pivot_id, CreditKind::Putout),
                credit(&facts.first_receiver_id, CreditKind::Putout),
                credit(&facts.initial_fielder_id, CreditKind::Assist),
                credit(&facts.pivot_id, CreditKind::Assist),
            ]);
            contact = Some(TwoOutContactFacts {
                contact: ContactClass::Grounded,
                continuity: ContactContinuity::Continuous,
                first_out: RetirementFact {
                    runner_id: forced.player_id.clone(),
                    cause: RetirementCause::Force {
                        at: Destination::Second,
                    },
                },
                second_out: RetirementFact {
                    runner_id: batter.clone(),
                    cause: RetirementCause::BatterPutoutBeforeFirstTouch,
                },
                error_placement: Some(ReviewedErrorPlacement {
                    between_outs: vec![],
                    outside_interval: vec![],
                }),
            });
        }
    }
    let mut moves = vec![RunnerMove {
        runner: Runner {
            player_id: batter.clone(),
            responsible_pitcher: pitcher.clone(),
        },
        from: Base::Plate,
        to: Destination::Out,
    }];
    for (i, runner) in before.bases.iter().enumerate() {
        if let Some(runner) = runner {
            let target = match decision {
                ContactDecision::CaughtSacrificeFly { .. } if i == 2 => Destination::Home,
                ContactDecision::SacrificeBunt(_) if i == 0 => Destination::Second,
                ContactDecision::GroundForceDoublePlay(_) if i == 0 => Destination::Out,
                _ => [Destination::First, Destination::Second, Destination::Third][i],
            };
            moves.push(RunnerMove {
                runner: runner.clone(),
                from: [Base::First, Base::Second, Base::Third][i],
                to: target,
            });
        }
    }
    let event = Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(batter.clone()),
            pitcher,
        },
        fielding_review: FieldingReview::NoMisplay,
        delta: Delta {
            moves,
            outs: if dp { 2 } else { 1 },
            runs: u8::from(sf),
            credits,
            changes: vec![],
            third_out: (dp && before.outs == 1).then(|| ThirdOut {
                runner_id: batter,
                kind: ThirdOutKind::BatterBeforeFirst,
                home_before_out: vec![],
            }),
        },
        two_out_contact: contact,
        ordinary_air_catch: air,
        single_delivery: None,
        balk: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        muffed_ground_ball: None,
        ordinary_ground_throw: None,
    };
    match decision {
        ContactDecision::CaughtSacrificeFly { facts, .. } => {
            apply_with_caught_sacrifice_fly_facts(before, event, facts.clone())
        }
        ContactDecision::SacrificeBunt(facts) => {
            apply_with_sacrifice_bunt_throw_facts(before, event, facts.clone())
        }
        ContactDecision::GroundForceDoublePlay(facts) => {
            apply_with_ground_force_double_play_facts(before, event, facts.clone())
        }
    }
    .map_err(|error| format!("canonical supplied-contact projection refused: {error:?}"))
}
