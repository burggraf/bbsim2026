//! Supplied modeled joint hits, not source-observed causal facts, RNG or attempt policy.
use bbsim_engine::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HitDecision {
    /// Selected safe hit value, 1..=3; only winning-run official scoring can reduce it.
    pub value: u8,
    /// Actual supplied outfielder position, 7..=9; never a second fielding roll.
    pub position: u8,
    /// Exact before occupancy; complete joint endpoints in First/Second/Third order.
    pub runners: [Option<Destination>; 3],
    pub tag: Option<TagAttempt>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagAttempt {
    pub origin: Base,
    pub attempted_base: Destination,
    /// Exact set of before-origin runners physically crossing Home before the tag.
    /// Must be a leading prefix among Home endpoints (Third/Second/First), in any Vec order.
    /// At two outs only this set scores; otherwise all Home endpoints score.
    /// The hitter has attained his credited base before this separate tag.
    pub home_before_tag: Vec<Base>,
}
const BASES: [Base; 3] = [Base::First, Base::Second, Base::Third];
const SAFE: [Destination; 4] = [
    Destination::First,
    Destination::Second,
    Destination::Third,
    Destination::Home,
];
fn target(d: Destination) -> u8 {
    match d {
        Destination::Out => 0,
        Destination::First => 1,
        Destination::Second => 2,
        Destination::Third => 3,
        Destination::Home => 4,
    }
}
fn credit(id: &str, kind: CreditKind) -> Credit {
    Credit {
        player_id: id.into(),
        kind,
    }
}

/// Project exactly once through the native reducer. Ordinary invalid vectors are never repaired.
/// Safe winners cross in Third/Second/First order, stopping at exactly the needed run.
/// Only then clip trailing endpoints, lead-to-trail, below their leader (uncounted Home at
/// most Third), and reduce hit value to the actual winner's advance under Rule9.06.
/// The official batter base is a generated natural-course AfterWinningRun touch.
/// A supplied winning tag is refused, never converted to safe. Ordinary tags retain
/// native forced-next-base, target<=hitter and no-passing refusals. The selected OF throws
/// to actual 3B at Third, C at Home, or SS (LF throw) / 2B (CF/RF) at Second.
/// Safe transitions reapply with `apply`; winning safe hits and tags require their retained
/// `walkoff_hit_facts` / `hit_runner_tag_facts` supplemental apply and constructor APIs.
pub fn project(
    before: &GameState,
    fielders: &[String; 9],
    decision: &HitDecision,
) -> Result<EventTransition, String> {
    if before.over || before.inning == 0 || before.outs > 2 {
        return Err("invalid or ended hit state".into());
    }
    if !(1..=3).contains(&decision.value) || !(7..=9).contains(&decision.position) {
        return Err("hit value must be 1..=3 and actual outfielder 7..=9".into());
    }
    let offense = &before.teams[before.half.offense()];
    let defense = &before.teams[1 - before.half.offense()];
    let batter = offense
        .batting_order
        .get(usize::from(offense.next_batter))
        .ok_or("invalid batting cursor")?;
    if fielders[0] != defense.pitcher
        || fielders.iter().enumerate().any(|(i, id)| {
            id.is_empty()
                || fielders[..i].contains(id)
                || !(id == &defense.pitcher
                    || defense
                        .batting_order
                        .iter()
                        .enumerate()
                        .any(|(slot, actor)| {
                            actor == id && defense.dh_slot.map(usize::from) != Some(slot)
                        }))
                || offense.batting_order.contains(id)
                || &offense.pitcher == id
                || offense.bench.contains(id)
                || offense.removed.contains(id)
                || defense.bench.contains(id)
                || defense.removed.contains(id)
        })
    {
        return Err(
            "fielders must be nine distinct active defenders with current pitcher at P".into(),
        );
    }
    let mut moves = vec![RunnerMove {
        runner: Runner {
            player_id: batter.clone(),
            responsible_pitcher: defense.pitcher.clone(),
        },
        from: Base::Plate,
        to: SAFE[usize::from(decision.value - 1)],
    }];
    for (i, runner) in before.bases.iter().enumerate() {
        match (runner, decision.runners[i]) {
            (Some(runner), Some(to)) => {
                if to != Destination::Out && usize::from(target(to)) < i + 1 {
                    return Err("backward runner endpoint".into());
                }
                moves.push(RunnerMove {
                    runner: runner.clone(),
                    from: BASES[i],
                    to,
                });
            }
            (None, None) => {}
            _ => return Err("joint endpoints must exactly match before occupancy".into()),
        }
    }
    let tag_runner = if let Some(tag) = &decision.tag {
        let index = BASES
            .iter()
            .position(|base| *base == tag.origin)
            .ok_or("tag origin must be a before base")?;
        let runner = before.bases[index].as_ref().ok_or("empty tag origin")?;
        if decision.runners[index] != Some(Destination::Out) {
            return Err("tagged runner must have Out endpoint".into());
        }
        Some(runner)
    } else {
        None
    };
    if moves.iter().filter(|m| m.to == Destination::Out).count()
        != usize::from(tag_runner.is_some())
    {
        return Err("exactly the supplied runner tag, and no other out, is supported".into());
    }
    // Check the supplied safe joint vector BEFORE any winning-stop normalization.
    let safe: Vec<_> = moves.iter().filter(|m| m.to != Destination::Out).collect();
    if safe.windows(2).any(|pair| {
        target(pair[0].to) > target(pair[1].to)
            || pair[0].to == pair[1].to && pair[0].to != Destination::Home
    }) {
        return Err("colliding or passing joint endpoints".into());
    }
    let mut tag_facts = None;
    if let Some(tag) = &decision.tag {
        let mut home_before_tag = Vec::new();
        for (i, base) in tag.home_before_tag.iter().enumerate() {
            let index = BASES
                .iter()
                .position(|b| b == base)
                .ok_or("pre-tag crossing must name a before base")?;
            if tag.home_before_tag[..i].contains(base)
                || decision.runners[index] != Some(Destination::Home)
            {
                return Err("pre-tag crossings must be distinct occupied Home endpoints".into());
            }
            home_before_tag.push(
                before.bases[index]
                    .as_ref()
                    .ok_or("empty pre-tag origin")?
                    .player_id
                    .clone(),
            );
        }
        // Native subset membership alone does not prove this continuous no-passing clock.
        let mut omitted_leader = false;
        for i in (0..3).rev() {
            if decision.runners[i] == Some(Destination::Home) {
                if tag.home_before_tag.contains(&BASES[i]) {
                    if omitted_leader {
                        return Err(
                            "pre-tag Home crossings cannot precede an omitted leading runner"
                                .into(),
                        );
                    }
                } else {
                    omitted_leader = true;
                }
            }
        }
        let tagger = match tag.attempted_base {
            Destination::Third => 4, // actual 3B
            Destination::Home => 1,  // actual C
            Destination::Second => {
                if decision.position == 7 {
                    5
                } else {
                    3
                }
            } // SS / 2B
            _ => return Err("unsupported runner tag target".into()),
        };
        tag_facts = Some(HitRunnerTagFacts {
            batter_id: batter.clone(),
            runner_id: tag_runner.expect("validated tag runner").player_id.clone(),
            attempted_base: tag.attempted_base,
            thrower_id: fielders[usize::from(decision.position - 1)].clone(),
            tagger_id: fielders[tagger].clone(),
            order: HitRunnerTagOrder::BatterAttainedCreditedBaseBeforeTag,
            home_before_tag,
        });
    }
    let scores = |m: &RunnerMove| {
        m.to == Destination::Home
            && tag_facts
                .as_ref()
                .is_none_or(|f| before.outs < 2 || f.home_before_tag.contains(&m.runner.player_id))
    };
    let runs = moves.iter().filter(|m| scores(m)).count();
    let needed = if before.half == Half::Bottom && before.inning >= 9 {
        Some(
            u32::from(before.score[0])
                .checked_sub(u32::from(before.score[1]))
                .ok_or("home already leads in late bottom half")?
                + 1,
        )
    } else {
        None
    };
    let mut value = decision.value;
    let mut walkoff = None;
    if let Some(needed) = needed.filter(|needed| runs as u32 >= *needed) {
        if tag_facts.is_some() {
            return Err("winning supplied runner tag is unsupported; select stopped safe opportunity before any tag".into());
        }
        let winner_index = (0..3)
            .rev()
            .filter(|i| decision.runners[*i] == Some(Destination::Home))
            .nth(needed as usize - 1)
            .expect("enough crossings");
        let winner_id = before.bases[winner_index]
            .as_ref()
            .expect("validated Home origin")
            .player_id
            .clone();
        value = value.min(3 - winner_index as u8);
        let mut remaining = needed;
        let mut ceiling = 3;
        for origin in BASES.iter().rev() {
            if let Some(m) = moves.iter_mut().find(|m| m.from == *origin) {
                if m.to == Destination::Home && remaining > 0 {
                    remaining -= 1;
                } else {
                    let endpoint = target(m.to).min(ceiling);
                    m.to = SAFE[usize::from(endpoint - 1)];
                    ceiling = endpoint - 1;
                }
            }
        }
        moves[0].to = SAFE[usize::from(value - 1)];
        walkoff = Some(WalkoffHitFacts {
            winning_runner_id: winner_id,
            batter_touch: NamedBaseTouch {
                runner_id: batter.clone(),
                base: moves[0].to,
            },
            timing: NaturalCourseTouchTiming::AfterWinningRun,
        });
    }
    let mut credits = vec![
        credit(batter, CreditKind::PlateAppearance),
        credit(batter, CreditKind::AtBat),
        credit(batter, CreditKind::Hit),
        credit(
            batter,
            [CreditKind::Single, CreditKind::Double, CreditKind::Triple][usize::from(value - 1)],
        ),
    ];
    let mut runs = 0;
    for m in moves.iter().filter(|m| scores(m)) {
        runs += 1;
        credits.extend([
            credit(&m.runner.player_id, CreditKind::Run),
            credit(batter, CreditKind::Rbi),
        ]);
    }
    if let Some(f) = &tag_facts {
        credits.extend([
            credit(&f.thrower_id, CreditKind::Assist),
            credit(&f.tagger_id, CreditKind::Putout),
        ]);
    }
    let event = Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(batter.clone()),
            pitcher: defense.pitcher.clone(),
        },
        fielding_review: FieldingReview::NoMisplay,
        delta: Delta {
            moves,
            outs: u8::from(tag_facts.is_some()),
            runs,
            credits,
            changes: vec![],
            third_out: tag_facts
                .as_ref()
                .filter(|_| before.outs == 2)
                .map(|f| ThirdOut {
                    runner_id: f.runner_id.clone(),
                    kind: ThirdOutKind::Timing,
                    home_before_out: f.home_before_tag.clone(),
                }),
        },
        sole_safe_hit: tag_facts.is_none().then(|| SoleSafeHitFacts {
            batter_id: batter.clone(),
        }),
        ordinary_ground_throw: None,
        ordinary_air_catch: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        muffed_ground_ball: None,
        two_out_contact: None,
        single_delivery: None,
        balk: None,
    };
    let result = if let Some(facts) = tag_facts {
        apply_with_hit_runner_tag_facts(before, event, facts)
    } else if let Some(facts) = walkoff {
        apply_with_walkoff_hit_facts(before, event, facts)
    } else {
        apply(before, event)
    };
    result.map_err(|e| format!("canonical full-hit projection refused: {e:?}"))
}
