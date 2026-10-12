use bbsim_engine::{
    accounting::derive_accounting, apply, Base, Credit, CreditKind, Delta, Destination, Event,
    EventKind, EventTransition, FieldingReview, GameState, Half, Participants, Runner, RunnerMove,
    TeamState, TransitionError,
};

fn team(prefix: &str, pitcher: &str) -> TeamState {
    TeamState {
        batting_order: std::array::from_fn(|i| format!("{prefix}{i}")),
        next_batter: 0,
        pitcher: pitcher.into(),
        dh_slot: Some(8),
        dh_must_bat: false,
        bench: vec![],
        removed: vec![],
    }
}
fn state(base: usize, outs: u8) -> GameState {
    let mut bases = [None, None, None];
    bases[base] = Some(Runner {
        player_id: "away7".into(),
        responsible_pitcher: "home_starter".into(),
    });
    GameState {
        inning: 2,
        half: Half::Top,
        outs,
        score: [0, 0],
        teams: [team("away", "away_pitcher"), team("home", "home_reliever")],
        bases,
        over: false,
    }
}
fn action(
    before: &GameState,
    kind: EventKind,
    moves: Vec<RunnerMove>,
    outs: u8,
    runs: u8,
    mut credits: Vec<Credit>,
) -> Event {
    let offense = before.half.offense();
    let batter =
        before.teams[offense].batting_order[before.teams[offense].next_batter as usize].clone();
    credits.push(Credit {
        player_id: batter.clone(),
        kind: CreditKind::PlateAppearance,
    });
    Event {
        kind,
        participants: Participants {
            batter: Some(batter),
            pitcher: before.teams[1 - offense].pitcher.clone(),
        },
        fielding_review: FieldingReview::NoMisplay,
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        caught_strikeout: None,
        protected_award: None,
        ordinary_ground_throw: None,
        delta: Delta {
            moves,
            outs,
            runs,
            credits,
            changes: vec![],
            third_out: None,
        },
    }
}
fn from_base(index: usize) -> Base {
    [Base::First, Base::Second, Base::Third][index]
}
fn replacement(before: &GameState) -> Event {
    let (base, runner) = before
        .bases
        .iter()
        .enumerate()
        .find_map(|(i, r)| r.as_ref().map(|r| (i, r)))
        .unwrap();
    let offense = before.half.offense();
    action(
        before,
        EventKind::FieldersChoice,
        vec![
            RunnerMove {
                runner: runner.clone(),
                from: from_base(base),
                to: Destination::Out,
            },
            RunnerMove {
                runner: Runner {
                    player_id: before.teams[offense].batting_order
                        [before.teams[offense].next_batter as usize]
                        .clone(),
                    responsible_pitcher: runner.responsible_pitcher.clone(),
                },
                from: Base::Plate,
                to: Destination::First,
            },
        ],
        1,
        0,
        vec![],
    )
}
fn homer(before: &GameState) -> Event {
    let offense = before.half.offense();
    let batter = &before.teams[offense].batting_order[before.teams[offense].next_batter as usize];
    let mut moves = vec![RunnerMove {
        runner: Runner {
            player_id: batter.clone(),
            responsible_pitcher: before.teams[1 - offense].pitcher.clone(),
        },
        from: Base::Plate,
        to: Destination::Home,
    }];
    let mut credits = vec![
        Credit {
            player_id: batter.clone(),
            kind: CreditKind::Hit,
        },
        Credit {
            player_id: batter.clone(),
            kind: CreditKind::Run,
        },
    ];
    for (base, runner) in before
        .bases
        .iter()
        .enumerate()
        .filter_map(|(i, r)| r.as_ref().map(|r| (i, r)))
    {
        moves.push(RunnerMove {
            runner: runner.clone(),
            from: from_base(base),
            to: Destination::Home,
        });
        credits.push(Credit {
            player_id: runner.player_id.clone(),
            kind: CreditKind::Run,
        });
    }
    action(
        before,
        EventKind::OutOfParkHomeRun,
        moves,
        0,
        (credits.len() - 1) as u8,
        credits,
    )
}

#[test]
fn lone_runner_fc_preserves_charge_at_each_base_and_out_state() {
    for base in 0..3 {
        for outs in 0..3 {
            for reversed in [false, true] {
                let before = state(base, outs);
                let original = before.clone();
                let mut event = replacement(&before);
                if reversed {
                    event.delta.moves.reverse();
                }
                let transition = apply(&before, event).unwrap();
                assert_eq!(before, original);
                assert_eq!(transition.after().teams[0].next_batter, 1);
                assert_eq!(transition.event().delta.outs, 1);
                if outs == 2 {
                    assert_eq!(transition.after().half, Half::Bottom);
                    assert!(transition.after().bases.iter().all(Option::is_none));
                } else {
                    assert_eq!(transition.after().outs, outs + 1);
                    assert_eq!(
                        transition.after().bases[0],
                        Some(Runner {
                            player_id: "away0".into(),
                            responsible_pitcher: "home_starter".into()
                        })
                    );
                }
                let line = derive_accounting(std::slice::from_ref(&transition)).unwrap();
                assert_eq!(line.batters[0]["away0"].plate_appearances, 1);
                assert_eq!(line.batters[0]["away0"].at_bats, Some(1));
                assert_eq!(line.pitchers[1]["home_reliever"].batters_faced, 1);
                assert_eq!(line.pitchers[1]["home_reliever"].outs_recorded, 1);
                assert_eq!(line.runs, [0, 0]);
                assert_eq!(
                    EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
                    transition
                );
            }
        }
    }
}

#[test]
fn repeated_fc_then_homer_charges_original_pitcher_not_reliever() {
    let before = state(0, 0);
    let first = apply(&before, replacement(&before)).unwrap();
    let s = first.after();
    let second = apply(s, replacement(s)).unwrap();
    assert!(second.follows(&first).is_ok());
    let s = second.after();
    let third = apply(s, homer(s)).unwrap();
    assert!(third.follows(&second).is_ok());
    let line = derive_accounting(&[first, second, third]).unwrap();
    assert_eq!(line.runs, [2, 0]);
    assert_eq!(line.pitchers[1]["home_starter"].runs_allowed, 1);
    assert_eq!(line.pitchers[1]["home_reliever"].runs_allowed, 1);
    assert_eq!(line.pitchers[1]["home_reliever"].outs_recorded, 2);
    assert_eq!(line.pitchers[1]["home_starter"].earned_runs, None);
    assert_eq!(line.pitchers[1]["home_reliever"].earned_runs, None);
}

#[test]
fn fc_rejects_wrong_charge_in_event_or_after_state() {
    let before = state(0, 0);
    let event = replacement(&before);
    let correct = apply(&before, event.clone()).unwrap();
    let mut forged = event.clone();
    forged
        .delta
        .moves
        .iter_mut()
        .find(|m| m.from == Base::Plate)
        .unwrap()
        .runner
        .responsible_pitcher = "home_reliever".into();
    assert_eq!(
        apply(&before, forged.clone()),
        Err(TransitionError::DeltaMismatch)
    );
    let mut after = correct.after().clone();
    after.bases[0].as_mut().unwrap().responsible_pitcher = "home_reliever".into();
    assert!(EventTransition::try_new(before.clone(), event, after.clone()).is_err());
    assert!(EventTransition::try_new(before, forged, after).is_err());
}

#[test]
fn fc_label_cannot_coexist_with_hit_walk_k_or_pa_exclusions() {
    let before = state(0, 0);
    for kind in [
        CreditKind::Hit,
        CreditKind::Walk,
        CreditKind::Strikeout,
        CreditKind::HitByPitch,
        CreditKind::SacrificeBunt,
        CreditKind::SacrificeFly,
        CreditKind::InterferenceAward,
    ] {
        let mut event = replacement(&before);
        event.delta.credits.push(Credit {
            player_id: "away0".into(),
            kind,
        });
        assert!(apply(&before, event).is_err(), "{kind:?}");
    }
}

#[test]
fn fc_without_a_lone_runner_or_with_unmodeled_moves_is_unsupported() {
    let mut before = state(0, 0);
    let valid = replacement(&before);
    before.bases[1] = Some(Runner {
        player_id: "away6".into(),
        responsible_pitcher: "home_reliever".into(),
    });
    assert_eq!(
        apply(&before, valid.clone()),
        Err(TransitionError::UnsupportedRule)
    );
    before.bases = [None, None, None];
    assert_eq!(apply(&before, valid), Err(TransitionError::UnsupportedRule));
    let before = state(0, 0);
    for destination in [
        Destination::Second,
        Destination::Third,
        Destination::Home,
        Destination::Out,
    ] {
        let mut event = replacement(&before);
        event
            .delta
            .moves
            .iter_mut()
            .find(|m| m.from == Base::Plate)
            .unwrap()
            .to = destination;
        assert_eq!(apply(&before, event), Err(TransitionError::UnsupportedRule));
    }
}

#[test]
fn unclassified_inherited_runner_replacement_cannot_silently_change_charge() {
    let before = state(0, 0);
    let mut event = replacement(&before);
    event.kind = EventKind::PlateAppearance;
    event
        .delta
        .moves
        .iter_mut()
        .find(|m| m.from == Base::Plate)
        .unwrap()
        .runner
        .responsible_pitcher = "home_reliever".into();
    assert_eq!(apply(&before, event), Err(TransitionError::UnsupportedRule));
}

#[test]
fn fc_cannot_claim_caught_stealing_through_reducer_or_constructor() {
    let before = state(0, 0);
    let event = replacement(&before);
    let correct = apply(&before, event.clone()).unwrap();
    let mut forged = event;
    forged.delta.credits.push(Credit {
        player_id: "away7".into(),
        kind: CreditKind::CaughtStealing,
    });
    assert_eq!(
        apply(&before, forged.clone()),
        Err(TransitionError::IllegalCredit)
    );
    assert_eq!(
        EventTransition::try_new(before, forged, correct.after().clone()),
        Err(TransitionError::IllegalCredit)
    );
}

#[test]
fn codec_revalidation_rejects_fc_with_a_caught_stealing_retirement() {
    let mut before = state(0, 0);
    before.teams[1].pitcher = "home_starter".into();
    // Generic PA can contain a separate runner retirement. There is no
    // inherited charge to erase here, so the generic record is valid.
    let mut event = replacement(&before);
    event.kind = EventKind::PlateAppearance;
    event.delta.credits.push(Credit {
        player_id: "away7".into(),
        kind: CreditKind::CaughtStealing,
    });
    let valid = apply(&before, event).unwrap();
    let mut bytes = valid.to_bytes();
    assert_eq!(EventTransition::from_bytes(&bytes).unwrap(), valid);
    // Locate the unique serialized kind + participants + review sequence;
    // change only generic PA tag 0 to FC tag 4, without corrupting the record.
    let mut needle = vec![0u8, 1u8];
    for id in ["away0", "home_starter"] {
        needle.extend_from_slice(&(id.len() as u64).to_le_bytes());
        needle.extend_from_slice(id.as_bytes());
    }
    needle.push(1); // NoMisplay
    let offsets: Vec<_> = bytes
        .windows(needle.len())
        .enumerate()
        .filter_map(|(i, part)| (part == needle).then_some(i))
        .collect();
    assert_eq!(offsets.len(), 1);
    bytes[offsets[0]] = 4;
    assert_eq!(
        EventTransition::from_bytes(&bytes),
        Err(TransitionError::IllegalCredit)
    );
}

#[test]
fn hit_with_runner_out_is_not_fc_and_keeps_batter_on_current_pitcher() {
    let before = state(0, 0);
    let mut event = replacement(&before);
    event.kind = EventKind::PlateAppearance;
    event.delta.credits.push(Credit {
        player_id: "away0".into(),
        kind: CreditKind::Hit,
    });
    event
        .delta
        .moves
        .iter_mut()
        .find(|m| m.from == Base::Plate)
        .unwrap()
        .runner
        .responsible_pitcher = "home_reliever".into();
    let first = apply(&before, event).unwrap();
    assert_eq!(
        first.after().bases[0].as_ref().unwrap().responsible_pitcher,
        "home_reliever"
    );
    let s = first.after();
    let second = apply(s, homer(s)).unwrap();
    let line = derive_accounting(&[first, second]).unwrap();
    assert_eq!(line.pitchers[1]["home_reliever"].runs_allowed, 2);
    assert!(!line.pitchers[1].contains_key("home_starter"));
}
