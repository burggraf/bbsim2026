//! Bounded, explicitly ordered two-out contact facts. No classification is
//! inferred from movement order, absent credits, or whole-play review.
use super::*;

pub(crate) fn validate(before: &GameState, event: &Event) -> Result<(), TransitionError> {
    let count = |kind| {
        event
            .delta
            .credits
            .iter()
            .filter(|c| c.kind == kind)
            .count()
    };
    let gidp = count(CreditKind::GroundedIntoDoublePlay);
    let incompatible = |kind| {
        matches!(
            kind,
            CreditKind::Hit
                | CreditKind::Single
                | CreditKind::Double
                | CreditKind::Triple
                | CreditKind::Strikeout
                | CreditKind::Walk
                | CreditKind::OrdinaryWalk
                | CreditKind::IntentionalWalk
                | CreditKind::HitByPitch
                | CreditKind::SacrificeBunt
                | CreditKind::SacrificeFly
                | CreditKind::InterferenceAward
        )
    };
    let batter = &before.teams[before.half.offense()].batting_order
        [before.teams[before.half.offense()].next_batter as usize];
    if gidp > 1
        || (gidp == 1
            && (event.kind != EventKind::PlateAppearance
                || event.participants.batter.as_ref() != Some(batter)
                || count(CreditKind::PlateAppearance) != 1
                || count(CreditKind::AtBat) > 1
                || event.delta.credits.iter().any(|c| {
                    (matches!(
                        c.kind,
                        CreditKind::GroundedIntoDoublePlay | CreditKind::PlateAppearance
                    ) && &c.player_id != batter)
                        || incompatible(c.kind)
                })))
    {
        return Err(TransitionError::IllegalCredit);
    }
    if (gidp > 0 || event.two_out_contact.is_some())
        && event.fielding_review == FieldingReview::NoMisplay
        && event.delta.credits.iter().any(|c| {
            matches!(
                c.kind,
                CreditKind::Error | CreditKind::PassedBall | CreditKind::InterferenceAward
            )
        })
    {
        return Err(TransitionError::IllegalCredit);
    }
    let Some(facts) = &event.two_out_contact else {
        return if gidp == 0 {
            Ok(())
        } else {
            Err(TransitionError::UnsupportedRule)
        };
    };
    let interference = [&facts.first_out.cause, &facts.second_out.cause]
        .iter()
        .any(|cause| {
            matches!(
                cause,
                RetirementCause::RunnerInterference
                    | RetirementCause::BatterByPrecedingRunnerInterference { .. }
            )
        });
    // Represented scorer contradictions precede missing review/charge history.
    if (interference && event.fielding_review == FieldingReview::NoMisplay)
        || (gidp == 1
            && (facts.contact != ContactClass::Grounded
                || facts.continuity == ContactContinuity::Discontinuous
                || interference
                || facts
                    .error_placement
                    .as_ref()
                    .is_some_and(|review| !review.between_outs.is_empty())))
        || (facts.contact == ContactClass::Grounded
            && event.delta.credits.iter().any(|c| {
                incompatible(c.kind)
                    && !matches!(
                        c.kind,
                        CreditKind::Hit
                            | CreditKind::Single
                            | CreditKind::Double
                            | CreditKind::Triple
                            | CreditKind::SacrificeBunt
                    )
            }))
    {
        return Err(TransitionError::IllegalCredit);
    }
    if event.kind != EventKind::PlateAppearance {
        return Err(TransitionError::IllegalStateChange);
    }
    let retired: Vec<_> = event
        .delta
        .moves
        .iter()
        .filter(|m| m.to == Destination::Out)
        .collect();
    if before.outs > 1
        || event.delta.outs != 2
        || retired.len() != 2
        || facts.first_out.runner_id.is_empty()
        || facts.second_out.runner_id.is_empty()
        || facts.first_out.runner_id == facts.second_out.runner_id
    {
        return Err(TransitionError::DeltaMismatch);
    }
    let find = |fact: &RetirementFact| {
        retired
            .iter()
            .find(|m| m.runner.player_id == fact.runner_id)
            .copied()
            .ok_or(TransitionError::DeltaMismatch)
    };
    let first = find(&facts.first_out)?;
    let second = find(&facts.second_out)?;
    for (movement, cause, earlier) in [
        (first, &facts.first_out.cause, None),
        (second, &facts.second_out.cause, Some(first.from)),
    ] {
        if movement.from == Base::Plate {
            if &movement.runner.player_id != batter {
                return Err(TransitionError::DeltaMismatch);
            }
        } else if before.bases[rank(movement.from) - 1].as_ref() != Some(&movement.runner) {
            return Err(TransitionError::DeltaMismatch);
        }
        validate_cause(before, movement.from, cause, earlier)?;
    }
    if let Some(review) = &facts.error_placement {
        let mut placed: Vec<_> = review
            .between_outs
            .iter()
            .chain(&review.outside_interval)
            .map(String::as_str)
            .collect();
        let mut supplied: Vec<_> = event
            .delta
            .credits
            .iter()
            .filter(|c| c.kind == CreditKind::Error)
            .map(|c| c.player_id.as_str())
            .collect();
        placed.sort_unstable();
        supplied.sort_unstable();
        if placed != supplied {
            return Err(TransitionError::DeltaMismatch);
        }
    }
    if interference {
        // The simple two-penalty exclusion is represented, but its supplied
        // order is not defensive timing for third outs/home crossings.
        if !matches!(&facts.first_out.cause, RetirementCause::RunnerInterference)
            || !matches!(&facts.second_out.cause, RetirementCause::BatterByPrecedingRunnerInterference { interferer_id } if *interferer_id == facts.first_out.runner_id)
        {
            return Err(TransitionError::DeltaMismatch);
        }
        if before.outs == 1
            || event.delta.moves.iter().any(|m| {
                !matches!(
                    (m.from, m.to),
                    (_, Destination::Out)
                        | (Base::First, Destination::First)
                        | (Base::Second, Destination::Second)
                        | (Base::Third, Destination::Third)
                )
            })
        {
            // The penalty cannot grant bases. Earlier legal advancement/last
            // touches need chronology; only held survivors are supported here.
            return Err(TransitionError::UnsupportedRule);
        }
        return Ok(());
    }
    let first_force = force_like(&facts.first_out.cause);
    let second_force = force_like(&facts.second_out.cause);
    let reverse = matches!(
        facts.second_out.cause,
        RetirementCause::Tag {
            force_loss: ForceLoss::RemovedByFirstOut,
            ..
        }
    );
    let unknown_mechanics = first_force
        && forced(before, second.from)
        && rank(first.from) < rank(second.from)
        && matches!(
            facts.second_out.cause,
            RetirementCause::Tag {
                force_loss: ForceLoss::Unknown,
                ..
            }
        );
    let possible = facts.contact == ContactClass::Grounded
        && facts.continuity != ContactContinuity::Discontinuous
        && first_force
        && (second_force || reverse || unknown_mechanics)
        && facts
            .error_placement
            .as_ref()
            .is_none_or(|r| r.between_outs.is_empty());
    let qualified = possible
        && !unknown_mechanics
        && facts.continuity == ContactContinuity::Continuous
        && facts.error_placement.is_some();
    if (gidp == 1 && !possible)
        || (qualified
            && (count(CreditKind::Rbi) > 0
                || event.delta.credits.iter().any(|c| incompatible(c.kind))))
        || (count(CreditKind::Hit) > 0 && (first_force || second_force))
    {
        return Err(TransitionError::IllegalCredit);
    }
    if before.outs == 1 {
        let third = event
            .delta
            .third_out
            .as_ref()
            .ok_or(TransitionError::UnsupportedRule)?;
        let expected = match facts.second_out.cause {
            RetirementCause::BatterPutoutBeforeFirstTouch => ThirdOutKind::BatterBeforeFirst,
            RetirementCause::Force { .. } => ThirdOutKind::Force,
            RetirementCause::Tag { .. } => ThirdOutKind::Timing,
            RetirementCause::BatterAfterFirstTag { .. } => ThirdOutKind::BatterAfterFirstTiming,
            _ => unreachable!("interference handled above"),
        };
        if third.runner_id != facts.second_out.runner_id || third.kind != expected {
            return Err(TransitionError::DeltaMismatch);
        }
    } else if event.delta.third_out.is_some() {
        return Err(TransitionError::DeltaMismatch);
    }
    if (gidp == 1 || count(CreditKind::Rbi) > 0) && possible && !qualified {
        return Err(TransitionError::UnsupportedRule);
    }
    if before.inning >= 9
        && before.half == Half::Bottom
        && before.score[1] <= before.score[0]
        && u32::from(before.score[1]) + u32::from(event.delta.runs) > u32::from(before.score[0])
    {
        // Home-before-third-out does not order the winning crossing against
        // the first retirement or certify both actions before game end.
        return Err(TransitionError::UnsupportedRule);
    }
    Ok(())
}

fn rank(base: Base) -> usize {
    match base {
        Base::Plate => 0,
        Base::First => 1,
        Base::Second => 2,
        Base::Third => 3,
    }
}
fn forced(before: &GameState, base: Base) -> bool {
    rank(base) > 0 && before.bases[..rank(base)].iter().all(Option::is_some)
}
fn force_like(cause: &RetirementCause) -> bool {
    matches!(
        cause,
        RetirementCause::BatterPutoutBeforeFirstTouch | RetirementCause::Force { .. }
    )
}
fn validate_cause(
    before: &GameState,
    origin: Base,
    cause: &RetirementCause,
    earlier: Option<Base>,
) -> Result<(), TransitionError> {
    let location = match cause {
        RetirementCause::Force { at }
        | RetirementCause::Tag { at, .. }
        | RetirementCause::BatterAfterFirstTag { at } => Some(*at),
        _ => None,
    };
    if location == Some(Destination::Out) {
        return Err(TransitionError::DeltaMismatch);
    }
    match cause {
        RetirementCause::BatterPutoutBeforeFirstTouch
        | RetirementCause::BatterAfterFirstTag { .. }
        | RetirementCause::BatterByPrecedingRunnerInterference { .. } => {
            if origin != Base::Plate {
                return Err(TransitionError::DeltaMismatch);
            }
        }
        RetirementCause::Force { at } => {
            let next = match origin {
                Base::First => Destination::Second,
                Base::Second => Destination::Third,
                Base::Third => Destination::Home,
                Base::Plate => return Err(TransitionError::DeltaMismatch),
            };
            if *at != next
                || !forced(before, origin)
                || earlier.is_some_and(|base| rank(base) < rank(origin))
            {
                return Err(TransitionError::DeltaMismatch);
            }
        }
        RetirementCause::Tag { force_loss, .. } => {
            if origin == Base::Plate {
                return Err(TransitionError::DeltaMismatch);
            }
            match force_loss {
                ForceLoss::RemovedByFirstOut
                    if !forced(before, origin)
                        || earlier.is_none_or(|base| rank(base) >= rank(origin)) =>
                {
                    return Err(TransitionError::DeltaMismatch)
                }
                ForceLoss::PreviouslyRemoved if !forced(before, origin) => {
                    return Err(TransitionError::DeltaMismatch)
                }
                ForceLoss::NeverForced if forced(before, origin) => {
                    return Err(TransitionError::DeltaMismatch)
                }
                _ => {}
            }
        }
        RetirementCause::RunnerInterference => {
            if origin == Base::Plate {
                return Err(TransitionError::DeltaMismatch);
            }
        }
    }
    Ok(())
}
