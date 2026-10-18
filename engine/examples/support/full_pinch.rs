//! Deterministic PH decision over an explicitly assumed closed bench, not roster inference.
use bbsim_engine::{
    apply, DefenseState, Delta, Event, EventKind, EventTransition, FieldingReview, GameState,
    Participants, PersonnelChange, TeamSide, TransitionError,
};
use std::collections::BTreeSet;

pub const POLICY_VERSION: &str = "late-close-expected-bases-ph-v1";

#[derive(Clone, Debug)]
pub struct Candidate {
    pub id: String,
    pub positions: Vec<u8>,
}

fn valid_id(id: &str) -> bool {
    !id.is_empty() && !id.chars().any(|c| c.is_whitespace() || c.is_control())
}

fn score(q: [f64; 9]) -> Result<f64, String> {
    if q.iter().any(|p| !p.is_finite() || *p <= 0.0) || (q.iter().sum::<f64>() - 1.0).abs() > 1e-12
    {
        return Err("PH requires positive normalized current-matchup probabilities".into());
    }
    Ok(q[1] + q[2] + q[3] + 2.0 * q[4] + 3.0 * q[5] + 4.0 * q[6] + q[7])
}

pub fn decide(
    before: &GameState,
    pool: &[Candidate],
    reserve_pitcher: Option<&str>,
    mut probabilities: impl FnMut(&str, &str) -> Result<[f64; 9], String>,
) -> Result<Option<EventTransition>, String> {
    decide_states(before, pool, reserve_pitcher, |state| {
        let offense = state.half.offense();
        let team = &state.teams[offense];
        probabilities(
            &team.batting_order[team.next_batter as usize],
            state.teams[1 - offense]
                .pitcher
                .active_id()
                .ok_or("PH missing opposing pitcher")?,
        )
    })
}

/// Query only the current frame or a legally projected candidate, never a simulated outcome.
pub fn decide_states(
    before: &GameState,
    pool: &[Candidate],
    reserve_pitcher: Option<&str>,
    mut probabilities: impl FnMut(&GameState) -> Result<[f64; 9], String>,
) -> Result<Option<EventTransition>, String> {
    if before.over {
        return Ok(None);
    }
    before
        .validate_play_ready()
        .map_err(|e| format!("PH unready frame: {e:?}"))?;
    let side = before.half.offense();
    let team = &before.teams[side];
    let opponent = &before.teams[1 - side];
    let DefenseState::Assigned(positions) = &team.defense else {
        return Err("PH requires explicit offensive defense".into());
    };
    if !matches!(opponent.defense, DefenseState::Assigned(_)) {
        return Err("PH requires explicit opposing defense".into());
    }
    let foreign = |id: &str| {
        opponent
            .batting_order
            .iter()
            .chain(&opponent.bench)
            .chain(&opponent.removed)
            .any(|p| p == id)
            || opponent.pitcher.active_id() == Some(id)
    };
    let mut ids = BTreeSet::new();
    for c in pool {
        if !valid_id(&c.id)
            || !ids.insert(c.id.as_str())
            || foreign(&c.id)
            || c.positions.iter().any(|p| !(2..=9).contains(p))
            || c.positions.iter().collect::<BTreeSet<_>>().len() != c.positions.len()
            || !(team.bench.contains(&c.id)
                || team.batting_order.contains(&c.id)
                || team.removed.contains(&c.id)
                || team.pitcher.active_id() == Some(c.id.as_str()))
        {
            return Err("invalid assumed PH pool identity/position/membership".into());
        }
    }
    if reserve_pitcher.is_some_and(|id| {
        !valid_id(id) || !team.bench.iter().any(|p| p == id) || ids.contains(id) || foreign(id)
    }) {
        return Err("invalid assumed reserve pitcher".into());
    }
    if before.inning < 7
        || before.score[0].abs_diff(before.score[1]) > 3
        || positions.iter().any(Option::is_none)
    {
        return Ok(None);
    }
    let outgoing = &team.batting_order[team.next_batter as usize];
    let position = positions.iter().position(|p| p.as_ref() == Some(outgoing));
    if position == Some(0) && reserve_pitcher.is_none() {
        return Ok(None);
    }
    let mut candidates: Vec<_> = pool.iter().filter(|c| team.bench.contains(&c.id)).collect();
    candidates.sort_by(|a, b| a.id.cmp(&b.id));
    let mut legal = Vec::new();
    for c in candidates {
        let mut changes = vec![PersonnelChange::PinchHitter {
            outgoing: outgoing.clone(),
            incoming: c.id.clone(),
        }];
        if let Some(index) = position.filter(|i| *i != 0) {
            if !c.positions.contains(&(index as u8 + 1)) {
                continue;
            }
            changes.push(PersonnelChange::AssignDefense {
                team: if side == 0 {
                    TeamSide::Away
                } else {
                    TeamSide::Home
                },
                position: index as u8 + 1,
                outgoing: Some(outgoing.clone()),
                incoming: c.id.clone(),
            });
        }
        let event = Event {
            kind: EventKind::Substitution,
            participants: Participants {
                batter: None,
                pitcher: None,
            },
            delta: Delta {
                changes,
                moves: vec![],
                outs: 0,
                runs: 0,
                credits: vec![],
                third_out: None,
            },
            fielding_review: FieldingReview::Unknown,
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
        };
        match apply(before, event) {
            Ok(t) => legal.push((&c.id, t)),
            Err(TransitionError::IllegalStateChange) => continue,
            Err(e) => return Err(format!("PH canonical candidate refused: {e:?}")),
        }
    }
    if legal.is_empty() {
        return Ok(None);
    }
    let current = score(probabilities(before)?)?;
    let mut best = None;
    let mut best_score = current;
    for (_, transition) in legal {
        let candidate_score = score(probabilities(transition.after())?)?;
        // Stable-ID order resolves equal scores without an outcome or manager RNG draw.
        if candidate_score - current > 0.10 && candidate_score > best_score {
            best_score = candidate_score;
            best = Some(transition);
        }
    }
    Ok(best)
}
