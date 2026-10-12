use bbsim_engine::{
    accounting::derive_accounting, apply, Base, Credit, CreditKind, Delta, Destination, Event,
    EventKind, EventTransition, FieldingReview, GameState, Half, Participants, Runner, RunnerEvent,
    RunnerMove, TeamState, TransitionError,
};

fn state(half: Half, defense_dh: bool) -> GameState {
    let mut teams = ["away", "home"].map(|prefix| TeamState {
        batting_order: std::array::from_fn(|i| format!("{prefix}{i}")),
        next_batter: 0,
        pitcher: format!("{prefix}_pitcher"),
        dh_slot: Some(8),
        dh_must_bat: false,
        bench: vec![],
        removed: vec![],
    });
    let offense = half.offense();
    if !defense_dh {
        teams[1 - offense].dh_slot = None;
        teams[1 - offense].batting_order[0] = teams[1 - offense].pitcher.clone();
    }
    let runner = Runner {
        player_id: teams[offense].batting_order[7].clone(),
        responsible_pitcher: teams[1 - offense].pitcher.clone(),
    };
    GameState {
        inning: 2,
        half,
        outs: 0,
        score: [0, 0],
        teams,
        bases: [Some(runner), None, None],
        over: false,
    }
}

fn play(before: &GameState, kind: CreditKind, fielder: &str) -> Event {
    let offense = before.half.offense();
    let batter = before.teams[offense].batting_order[0].clone();
    let pitcher = before.teams[1 - offense].pitcher.clone();
    let pb = kind == CreditKind::PassedBall;
    let movement = if pb {
        RunnerMove {
            runner: before.bases[0].clone().unwrap(),
            from: Base::First,
            to: Destination::Second,
        }
    } else {
        RunnerMove {
            runner: Runner {
                player_id: batter.clone(),
                responsible_pitcher: pitcher.clone(),
            },
            from: Base::Plate,
            to: Destination::Out,
        }
    };
    let mut credits = vec![Credit {
        player_id: fielder.into(),
        kind,
    }];
    if !pb {
        credits.extend([
            Credit {
                player_id: batter.clone(),
                kind: CreditKind::PlateAppearance,
            },
            Credit {
                player_id: batter.clone(),
                kind: CreditKind::Strikeout,
            },
        ]);
    }
    Event {
        kind: if pb {
            EventKind::RunnerOnly(RunnerEvent::PassedBall)
        } else {
            EventKind::PlateAppearance
        },
        participants: Participants {
            batter: Some(batter),
            pitcher,
        },
        fielding_review: FieldingReview::Unknown,
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        muffed_ground_ball: None,
        ordinary_ground_throw: None,
        delta: Delta {
            moves: vec![movement],
            outs: u8::from(!pb),
            runs: 0,
            credits,
            changes: vec![],
            third_out: None,
        },
    }
}

fn rejects_known_role(before: &GameState, kind: CreditKind, tag: u8, ineligible: &str) {
    let defense = 1 - before.half.offense();
    let eligible = &before.teams[defense].batting_order[1];
    let valid = apply(before, play(before, kind, eligible)).unwrap();
    assert_eq!(
        EventTransition::from_bytes(&valid.to_bytes()).unwrap(),
        valid
    );
    let mut forged = valid.event().clone();
    forged
        .delta
        .credits
        .iter_mut()
        .find(|credit| credit.kind == kind)
        .unwrap()
        .player_id = ineligible.into();
    assert_eq!(
        apply(before, forged.clone()),
        Err(TransitionError::IllegalCredit)
    );
    assert_eq!(
        EventTransition::try_new(before.clone(), forged, valid.after().clone()),
        Err(TransitionError::IllegalCredit)
    );

    // Change only a unique serialized credit actor, keeping valid lengths and
    // kind tag. Before/after batting-order strings do not have this suffix.
    let mut needle = (eligible.len() as u64).to_le_bytes().to_vec();
    needle.extend_from_slice(eligible.as_bytes());
    needle.push(tag);
    let mut bytes = valid.to_bytes();
    let offsets: Vec<_> = bytes
        .windows(needle.len())
        .enumerate()
        .filter_map(|(i, part)| (part == needle).then_some(i))
        .collect();
    assert_eq!(offsets.len(), 1);
    let mut replacement = (ineligible.len() as u64).to_le_bytes().to_vec();
    replacement.extend_from_slice(ineligible.as_bytes());
    bytes.splice(offsets[0]..offsets[0] + needle.len() - 1, replacement);
    assert_eq!(
        EventTransition::from_bytes(&bytes),
        Err(TransitionError::IllegalCredit)
    );
}

#[test]
fn active_dh_cannot_receive_fielding_credits_without_losing_the_role() {
    for half in [Half::Top, Half::Bottom] {
        let before = state(half, true);
        let dh = &before.teams[1 - half.offense()].batting_order[8];
        for (kind, tag) in [
            (CreditKind::Putout, 8),
            (CreditKind::Assist, 9),
            (CreditKind::Error, 10),
            (CreditKind::PassedBall, 11),
        ] {
            rejects_known_role(&before, kind, tag, dh);
        }
    }
}

#[test]
fn current_pitcher_cannot_be_charged_a_passed_ball_with_or_without_dh() {
    for half in [Half::Top, Half::Bottom] {
        for dh in [false, true] {
            let before = state(half, dh);
            rejects_known_role(
                &before,
                CreditKind::PassedBall,
                11,
                &before.teams[1 - half.offense()].pitcher,
            );
        }
    }
}

#[test]
fn known_role_restrictions_preserve_eligible_fielder_and_pitcher_credits() {
    for half in [Half::Top, Half::Bottom] {
        for dh in [false, true] {
            let before = state(half, dh);
            let defense = 1 - half.offense();
            for kind in [
                CreditKind::Putout,
                CreditKind::Assist,
                CreditKind::Error,
                CreditKind::PassedBall,
            ] {
                let mut actors = vec![before.teams[defense].batting_order[1].as_str()];
                if kind != CreditKind::PassedBall {
                    actors.push(&before.teams[defense].pitcher);
                }
                if !dh {
                    actors.push(&before.teams[defense].batting_order[8]);
                }
                for actor in actors {
                    let transition = apply(&before, play(&before, kind, actor)).unwrap();
                    let box_score = derive_accounting(std::slice::from_ref(&transition)).unwrap();
                    assert_eq!(box_score.fielders[defense].len(), 1);
                    assert!(box_score.fielders[half.offense()].is_empty());
                    let line = &box_score.fielders[defense][actor];
                    let count = match kind {
                        CreditKind::Putout => line.credited_putouts,
                        CreditKind::Assist => line.credited_assists,
                        CreditKind::Error => line.credited_errors,
                        CreditKind::PassedBall => line.credited_passed_balls,
                        _ => unreachable!(),
                    };
                    assert_eq!(count, 1);
                }
            }
        }
    }
}
