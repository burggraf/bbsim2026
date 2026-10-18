use bbsim_engine::{
    accounting::derive_accounting, apply, Base, Credit, CreditKind, DefenseState, Delta,
    Destination, Event, EventKind, EventTransition, FieldingReview, GameState, Half, Participants,
    PersonnelChange, PitcherObligation, PitcherRole, Runner, RunnerEvent, RunnerMove, TeamSide,
    TeamState,
};

fn team(prefix: &str, dh: bool) -> TeamState {
    let pitcher = format!("{prefix}p");
    let order: [String; 9] = std::array::from_fn(|i| {
        if i == 8 && !dh {
            pitcher.clone()
        } else {
            format!("{prefix}{i}")
        }
    });
    let defense = std::array::from_fn(|i| {
        Some(if i == 0 {
            pitcher.clone()
        } else {
            order[i - 1].clone()
        })
    });
    TeamState {
        batting_order: order,
        next_batter: 8,
        pitcher: PitcherRole::Active {
            player_id: pitcher,
            obligation: PitcherObligation::Satisfied,
        },
        defense: DefenseState::Assigned(Box::new(defense)),
        dh_slot: dh.then_some(8),
        dh_must_bat: false,
        bench: vec![
            format!("{prefix}ph"),
            format!("{prefix}relief"),
            format!("{prefix}fresh"),
        ],
        removed: vec![],
    }
}
fn state(dh: bool) -> GameState {
    GameState {
        inning: 2,
        half: Half::Top,
        outs: 2,
        score: [0, 0],
        teams: [team("a", dh), team("h", dh)],
        bases: [None, None, None],
        over: false,
    }
}
fn event(before: &GameState, kind: EventKind, delta: Delta) -> Event {
    let offense = before.half.offense();
    let personnel = kind == EventKind::Substitution;
    Event {
        kind,
        participants: Participants {
            batter: (!personnel).then(|| {
                before.teams[offense].batting_order[before.teams[offense].next_batter as usize]
                    .clone()
            }),
            pitcher: if personnel {
                None
            } else {
                before.teams[1 - offense]
                    .pitcher
                    .active_id()
                    .map(str::to_owned)
            },
        },
        delta,
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
    }
}
fn changes(before: &GameState, changes: Vec<PersonnelChange>) -> Event {
    event(
        before,
        EventKind::Substitution,
        Delta {
            changes,
            moves: vec![],
            outs: 0,
            runs: 0,
            credits: vec![],
            third_out: None,
        },
    )
}
fn ph(before: &GameState) -> Event {
    let offense = before.half.offense();
    changes(
        before,
        vec![PersonnelChange::PinchHitter {
            outgoing: before.teams[offense].batting_order
                [before.teams[offense].next_batter as usize]
                .clone(),
            incoming: before.teams[offense].bench[0].clone(),
        }],
    )
}
fn out(before: &GameState) -> Event {
    let offense = before.half.offense();
    let id =
        before.teams[offense].batting_order[before.teams[offense].next_batter as usize].clone();
    let pitcher = before.teams[1 - offense]
        .pitcher
        .active_id()
        .unwrap_or("")
        .to_owned();
    event(
        before,
        EventKind::PlateAppearance,
        Delta {
            changes: vec![],
            outs: 1,
            runs: 0,
            third_out: None,
            moves: vec![RunnerMove {
                runner: Runner {
                    player_id: id.clone(),
                    responsible_pitcher: pitcher,
                },
                from: Base::Plate,
                to: Destination::Out,
            }],
            credits: vec![
                Credit {
                    player_id: id.clone(),
                    kind: CreditKind::PlateAppearance,
                },
                Credit {
                    player_id: id,
                    kind: CreditKind::AtBat,
                },
            ],
        },
    )
}
fn wire(transition: &EventTransition) {
    let bytes = transition.to_bytes();
    assert!(bytes.starts_with(b"BBE33"));
    assert_eq!(EventTransition::from_bytes(&bytes).unwrap(), *transition);
    for version in [
        b"BBE25", b"BBE26", b"BBE27", b"BBE28", b"BBE29", b"BBE30", b"BBE31", b"BBE32",
    ] {
        let mut corrupt = bytes.clone();
        corrupt[..5].copy_from_slice(version);
        assert!(EventTransition::from_bytes(&corrupt).is_err());
    }
    let mut trailing = bytes;
    trailing.push(0);
    assert!(EventTransition::from_bytes(&trailing).is_err());
}

#[test]
fn non_dh_ph_vacates_pitching_without_a_fake_appearance_then_relief_restores_defense() {
    let before = state(false);
    let original = before.clone();
    let first = apply(&before, ph(&before)).unwrap();
    assert_eq!(before, original);
    assert_eq!(
        first.after().teams[0].pitcher,
        PitcherRole::Vacant { batting_slot: 8 }
    );
    assert_eq!(first.after().teams[0].removed, ["ap"]);
    assert_eq!(first.after().teams[0].next_batter, 8);
    assert_eq!(first.after().score, before.score);
    assert_eq!(first.after().outs, before.outs);
    assert_eq!(first.after().bases, before.bases);
    wire(&first);
    let second = apply(first.after(), out(first.after())).unwrap();
    assert_eq!(second.after().half, Half::Bottom);
    assert_eq!(second.after().teams[0].next_batter, 0);
    assert!(apply(second.after(), out(second.after())).is_err());
    let mut resolved = Vec::new();
    for reverse in [false, true] {
        let mut assignments = vec![
            PersonnelChange::BattingSlot {
                team: TeamSide::Away,
                slot: 8,
                outgoing: "aph".into(),
                incoming: "arelief".into(),
            },
            PersonnelChange::AssignPitcher {
                team: TeamSide::Away,
                incoming: "arelief".into(),
            },
        ];
        if reverse {
            assignments.reverse();
        }
        let third = apply(second.after(), changes(second.after(), assignments)).unwrap();
        assert_eq!(third.after().teams[0].pitcher.active_id(), Some("arelief"));
        assert_eq!(
            third.after().teams[0].pitcher,
            PitcherRole::Active {
                player_id: "arelief".into(),
                obligation: PitcherObligation::ReliefBatter,
            }
        );
        assert_eq!(third.after().teams[0].removed, ["ap", "aph"]);
        wire(&third);
        resolved.push(third);
    }
    assert_eq!(resolved[0].after(), resolved[1].after());
    let fourth = apply(resolved[0].after(), out(resolved[0].after())).unwrap();
    wire(&fourth);
    let log = [first, second, resolved.remove(0), fourth];
    for pair in log.windows(2) {
        pair[1].follows(&pair[0]).unwrap();
    }
    let box_score = derive_accounting(&log).unwrap();
    assert!(!box_score.pitchers[0].contains_key("aph"));
    assert!(!box_score.fielders[0].contains_key("aph"));
    assert_eq!(box_score.pitchers[0]["arelief"].batters_faced, 1);
    assert_eq!(box_score.batters[0]["aph"].plate_appearances, 1);
}

#[test]
fn already_active_player_pitching_role_change_remains_explicitly_unsupported() {
    let before = state(false);
    let action = changes(
        &before,
        vec![
            PersonnelChange::Pitcher {
                team: TeamSide::Away,
                outgoing: "ap".into(),
                incoming: "a0".into(),
            },
            PersonnelChange::BattingSlot {
                team: TeamSide::Away,
                slot: 8,
                outgoing: "ap".into(),
                incoming: "aph".into(),
            },
            PersonnelChange::AssignDefense {
                team: TeamSide::Away,
                position: 2,
                outgoing: Some("a0".into()),
                incoming: "aph".into(),
            },
        ],
    );
    assert!(apply(&before, action).is_err());
}
#[test]
fn nonpitcher_ph_requires_explicit_defensive_assignment_not_a_stale_starter() {
    let mut before = state(false);
    before.teams[0].next_batter = 0;
    let first = apply(&before, ph(&before)).unwrap();
    assert_eq!(first.after().teams[0].pitcher.active_id(), Some("ap"));
    let second = apply(first.after(), out(first.after())).unwrap();
    assert!(apply(second.after(), out(second.after())).is_err());
    for incoming in ["a0", "h0", "afresh", "a1"] {
        let assignment = PersonnelChange::AssignDefense {
            team: TeamSide::Away,
            position: 2,
            outgoing: None,
            incoming: incoming.into(),
        };
        assert!(apply(second.after(), changes(second.after(), vec![assignment])).is_err());
    }
    let third = apply(
        second.after(),
        changes(
            second.after(),
            vec![PersonnelChange::AssignDefense {
                team: TeamSide::Away,
                position: 2,
                outgoing: None,
                incoming: "aph".into(),
            }],
        ),
    )
    .unwrap();
    assert_eq!(third.after().teams[0].bench, second.after().teams[0].bench);
    assert_eq!(
        third.after().teams[0].removed,
        second.after().teams[0].removed
    );
    let fourth = apply(third.after(), out(third.after())).unwrap();
    wire(&fourth);
    assert_eq!(first.after().teams[0].removed, ["a0"]);
}

#[test]
fn dh_ph_preserves_the_role_and_the_mandatory_starting_turn() {
    let before = state(true);
    let next = apply(&before, ph(&before)).unwrap();
    assert_eq!(next.after().teams[0].dh_slot, Some(8));
    assert_eq!(next.after().teams[0].pitcher, before.teams[0].pitcher);
    assert_eq!(next.after().teams[0].defense, before.teams[0].defense);
    let mut obligated = before;
    obligated.teams[0].dh_must_bat = true;
    assert!(apply(&obligated, ph(&obligated)).is_err());
}

#[test]
fn unsatisfied_or_untracked_pitcher_cannot_be_pinch_hit_for() {
    for obligation in [
        PitcherObligation::StartingBatter,
        PitcherObligation::ReliefBatter,
        PitcherObligation::UntrackedLegacy,
    ] {
        let mut before = state(false);
        before.teams[0].pitcher = PitcherRole::Active {
            player_id: "ap".into(),
            obligation,
        };
        assert!(apply(&before, ph(&before)).is_err());
    }
}

fn walk(before: &GameState) -> Event {
    let offense = before.half.offense();
    let id =
        before.teams[offense].batting_order[before.teams[offense].next_batter as usize].clone();
    let mut moves = vec![RunnerMove {
        runner: Runner {
            player_id: id.clone(),
            responsible_pitcher: before.teams[1 - offense]
                .pitcher
                .active_id()
                .unwrap()
                .into(),
        },
        from: Base::Plate,
        to: Destination::First,
    }];
    let mut credits: Vec<_> = [
        CreditKind::PlateAppearance,
        CreditKind::Walk,
        CreditKind::OrdinaryWalk,
    ]
    .into_iter()
    .map(|kind| Credit {
        player_id: id.clone(),
        kind,
    })
    .collect();
    let mut runs = 0;
    for index in 0..3 {
        if before.bases[..=index].iter().all(Option::is_some) {
            let runner = before.bases[index].as_ref().unwrap().clone();
            let from = [Base::First, Base::Second, Base::Third][index];
            let to = [Destination::Second, Destination::Third, Destination::Home][index];
            if to == Destination::Home {
                runs += 1;
                credits.push(Credit {
                    player_id: runner.player_id.clone(),
                    kind: CreditKind::Run,
                });
                credits.push(Credit {
                    player_id: id.clone(),
                    kind: CreditKind::Rbi,
                });
            }
            moves.push(RunnerMove { runner, from, to });
        }
    }
    event(
        before,
        EventKind::PlateAppearance,
        Delta {
            changes: vec![],
            moves,
            outs: 0,
            runs,
            credits,
            third_out: None,
        },
    )
}
#[test]
fn ph_walks_scores_and_bats_again_without_fabricating_pitching_or_losing_inherited_charges() {
    for half in [Half::Top, Half::Bottom] {
        let mut before = state(false);
        before.half = half;
        let offense = half.offense();
        let prefix = ["a", "h"][offense];
        let hitter = format!("{prefix}ph");
        before.bases[0] = Some(Runner {
            player_id: format!("{prefix}6"),
            responsible_pitcher: "prior-pitcher".into(),
        });
        before.bases[2] = Some(Runner {
            player_id: format!("{prefix}7"),
            responsible_pitcher: "prior-pitcher".into(),
        });
        let first = apply(&before, ph(&before)).unwrap();
        assert_eq!(first.after().bases, before.bases);
        let mut log = vec![first];
        for index in 0..10 {
            let state = log.last().unwrap().after();
            let action = walk(state);
            if index == 0 || index == 9 {
                assert_eq!(action.participants.batter.as_deref(), Some(hitter.as_str()));
            }
            log.push(apply(state, action).unwrap());
        }
        for pair in log.windows(2) {
            pair[1].follows(&pair[0]).unwrap();
        }
        for transition in &log {
            wire(transition);
        }
        let after = log.last().unwrap().after();
        assert_eq!(
            after.teams[offense].pitcher,
            PitcherRole::Vacant { batting_slot: 8 }
        );
        assert_eq!(after.teams[offense].next_batter, 0);
        assert_eq!(after.half, half);
        assert_eq!(after.outs, 2);
        assert_eq!(after.score[offense], 9);
        let box_score = derive_accounting(&log).unwrap();
        assert_eq!(box_score.batters[offense][&hitter].plate_appearances, 2);
        assert_eq!(box_score.batters[offense][&hitter].credited_runs, 1);
        assert_eq!(
            box_score.pitchers[1 - offense]["prior-pitcher"].runs_allowed,
            2
        );
        assert!(!box_score.pitchers[offense].contains_key(&hitter));
        assert!(!box_score.fielders[offense].contains_key(&hitter));
    }
}
#[test]
fn walkoff_ph_can_end_the_game_with_a_pitching_vacancy() {
    let mut before = state(false);
    before.inning = 9;
    before.half = Half::Bottom;
    let first = apply(&before, ph(&before)).unwrap();
    let id = "hph".to_owned();
    let action = event(
        first.after(),
        EventKind::OutOfParkHomeRun,
        Delta {
            changes: vec![],
            outs: 0,
            runs: 1,
            third_out: None,
            moves: vec![RunnerMove {
                runner: Runner {
                    player_id: id.clone(),
                    responsible_pitcher: "ap".into(),
                },
                from: Base::Plate,
                to: Destination::Home,
            }],
            credits: [
                CreditKind::PlateAppearance,
                CreditKind::AtBat,
                CreditKind::Hit,
                CreditKind::Run,
                CreditKind::Rbi,
            ]
            .into_iter()
            .map(|kind| Credit {
                player_id: id.clone(),
                kind,
            })
            .collect(),
        },
    );
    let last = apply(first.after(), action).unwrap();
    assert!(last.after().over);
    assert_eq!(last.after().score, [0, 1]);
    assert_eq!(
        last.after().teams[1].pitcher,
        PitcherRole::Vacant { batting_slot: 8 }
    );
    assert!(apply(last.after(), ph(last.after())).is_err());
    wire(&last);
    let box_score = derive_accounting(&[first, last]).unwrap();
    assert_eq!(box_score.batters[1]["hph"].plate_appearances, 1);
    assert_eq!(box_score.batters[1]["hph"].credited_runs, 1);
    assert!(!box_score.pitchers[1].contains_key("hph"));
}
#[test]
fn opposing_pitching_change_releases_the_dh_mandatory_turn_atomically() {
    let mut before = state(true);
    before.teams[0].dh_must_bat = true;
    let mut delta = ph(&before).delta.changes;
    delta.push(PersonnelChange::Pitcher {
        team: TeamSide::Home,
        outgoing: "hp".into(),
        incoming: "hrelief".into(),
    });
    let mut results = Vec::new();
    for reverse in [false, true] {
        let mut changes = delta.clone();
        if reverse {
            changes.reverse();
        }
        let t = apply(&before, self::changes(&before, changes)).unwrap();
        assert_eq!(t.after().teams[0].dh_slot, Some(8));
        assert!(!t.after().teams[0].dh_must_bat);
        assert_eq!(t.after().teams[0].batting_order[8], "aph");
        wire(&t);
        results.push(t);
    }
    assert_eq!(results[0].after(), results[1].after());
}
#[test]
fn dh_taking_defense_ends_dh_without_moving_any_retained_batting_slot() {
    let before = state(true);
    let delta = vec![
        PersonnelChange::BattingSlot {
            team: TeamSide::Away,
            slot: 0,
            outgoing: "a0".into(),
            incoming: "ap".into(),
        },
        PersonnelChange::AssignDefense {
            team: TeamSide::Away,
            position: 2,
            outgoing: Some("a0".into()),
            incoming: "a8".into(),
        },
    ];
    let mut results = Vec::new();
    for reverse in [false, true] {
        let mut delta = delta.clone();
        if reverse {
            delta.reverse();
        }
        let t = apply(&before, changes(&before, delta)).unwrap();
        assert_eq!(t.after().teams[0].dh_slot, None);
        assert_eq!(t.after().teams[0].batting_order[0], "ap");
        assert_eq!(t.after().teams[0].batting_order[8], "a8");
        assert_eq!(t.after().teams[0].removed, ["a0"]);
        assert_eq!(t.after().teams[0].bench, before.teams[0].bench);
        wire(&t);
        results.push(t);
    }
    assert_eq!(results[0].after(), results[1].after());
    let t = &results[0];
    let mut restored = t.after().clone();
    restored.teams[0].dh_slot = Some(8);
    assert!(EventTransition::try_new(t.before().clone(), t.event().clone(), restored).is_err());
}
#[test]
fn retained_defenders_can_swap_positions_in_either_assignment_order() {
    let before = state(false);
    let delta = vec![
        PersonnelChange::AssignDefense {
            team: TeamSide::Away,
            position: 2,
            outgoing: Some("a0".into()),
            incoming: "a7".into(),
        },
        PersonnelChange::AssignDefense {
            team: TeamSide::Away,
            position: 9,
            outgoing: Some("a7".into()),
            incoming: "a0".into(),
        },
    ];
    let mut results = Vec::new();
    for reverse in [false, true] {
        let mut delta = delta.clone();
        if reverse {
            delta.reverse();
        }
        let t = apply(&before, changes(&before, delta)).unwrap();
        assert_eq!(t.after().teams[0].bench, before.teams[0].bench);
        assert_eq!(t.after().teams[0].removed, before.teams[0].removed);
        assert_eq!(
            t.after().teams[0].batting_order,
            before.teams[0].batting_order
        );
        assert!(t.event().delta.credits.is_empty());
        wire(&t);
        results.push(t);
    }
    assert_eq!(results[0].after(), results[1].after());
}
fn encoded_name(id: &str) -> Vec<u8> {
    let mut bytes = (id.len() as u64).to_le_bytes().to_vec();
    bytes.extend(id.as_bytes());
    bytes
}
fn encoded_defense(positions: &[Option<String>; 9]) -> Vec<u8> {
    let mut bytes = vec![1];
    for position in positions {
        match position {
            Some(id) => {
                bytes.push(1);
                bytes.extend(encoded_name(id));
            }
            None => bytes.push(0),
        }
    }
    bytes
}
#[test]
fn forged_bbe33_position_maps_are_revalidated_not_just_parsed() {
    let mut before = state(false);
    before.teams[0].next_batter = 0;
    let first = apply(&before, ph(&before)).unwrap();
    let action = changes(
        first.after(),
        vec![PersonnelChange::AssignDefense {
            team: TeamSide::Away,
            position: 2,
            outgoing: None,
            incoming: "aph".into(),
        }],
    );
    let t = apply(first.after(), action).unwrap();
    wire(&t);
    let DefenseState::Assigned(original) = &t.after().teams[0].defense else {
        panic!("fixture defense");
    };
    let needle = encoded_defense(original);
    let bytes = t.to_bytes();
    let offsets: Vec<_> = bytes
        .windows(needle.len())
        .enumerate()
        .filter(|(_, v)| *v == needle)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(offsets.len(), 1);
    for invalid in ["a0", "h0", "afresh", "a1", ""] {
        let mut corrupt = original.clone();
        corrupt[1] = Some(invalid.into());
        let mut after = t.after().clone();
        after.teams[0].defense = DefenseState::Assigned(corrupt.clone());
        assert!(EventTransition::try_new(t.before().clone(), t.event().clone(), after).is_err());
        let mut forged = bytes.clone();
        forged.splice(
            offsets[0]..offsets[0] + needle.len(),
            encoded_defense(&corrupt),
        );
        assert!(
            EventTransition::from_bytes(&forged).is_err(),
            "accepted {invalid:?}"
        );
    }
    let mut repeated = t.event().clone();
    repeated
        .delta
        .changes
        .push(repeated.delta.changes[0].clone());
    assert!(apply(t.before(), repeated.clone()).is_err());
    assert!(EventTransition::try_new(t.before().clone(), repeated, t.after().clone()).is_err());
    // One complete encoded delta entry, not an incidental string occurrence.
    let mut entry = vec![5, 0, 2, 0];
    entry.extend(encoded_name("aph"));
    let mut marker = 1u64.to_le_bytes().to_vec();
    marker.extend(&entry);
    let offsets: Vec<_> = bytes
        .windows(marker.len())
        .enumerate()
        .filter(|(_, v)| *v == marker)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(offsets.len(), 1);
    let mut duplicate = 2u64.to_le_bytes().to_vec();
    duplicate.extend(&entry);
    duplicate.extend(&entry);
    let mut forged = bytes;
    forged.splice(offsets[0]..offsets[0] + marker.len(), duplicate);
    assert!(EventTransition::from_bytes(&forged).is_err());
}
#[test]
fn malformed_out_counts_cannot_overflow_first_batter_obligation_checks() {
    let mut before = state(false);
    before.teams[1].pitcher = PitcherRole::Active {
        player_id: "hp".into(),
        obligation: PitcherObligation::ReliefBatter,
    };
    let runner = Runner {
        player_id: "a1".into(),
        responsible_pitcher: "hp".into(),
    };
    before.bases[0] = Some(runner.clone());
    let legal = event(
        &before,
        EventKind::RunnerOnly(RunnerEvent::Pickoff),
        Delta {
            changes: vec![],
            moves: vec![RunnerMove {
                runner,
                from: Base::First,
                to: Destination::Out,
            }],
            outs: 1,
            runs: 0,
            credits: vec![],
            third_out: None,
        },
    );
    let control = apply(&before, legal.clone()).unwrap();
    wire(&control);
    let mut needle = 1u64.to_le_bytes().to_vec();
    needle.extend(encoded_name("a1"));
    needle.extend(encoded_name("hp"));
    needle.extend([1, 4]);
    let out_index = needle.len();
    needle.extend([1, 0]);
    needle.extend(0u64.to_le_bytes());
    needle.extend(0u64.to_le_bytes());
    needle.push(0);
    let bytes = control.to_bytes();
    let offsets: Vec<_> = bytes
        .windows(needle.len())
        .enumerate()
        .filter(|(_, v)| *v == needle)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(offsets.len(), 1);
    for outs in [4, 254, 255] {
        let mut action = legal.clone();
        action.delta.outs = outs;
        assert!(apply(&before, action.clone()).is_err());
        assert!(EventTransition::try_new(before.clone(), action, control.after().clone()).is_err());
        let mut forged = bytes.clone();
        forged[offsets[0] + out_index] = outs;
        assert!(EventTransition::from_bytes(&forged).is_err());
    }
}
#[test]
fn optional_actor_personnel_event_cannot_treat_untracked_legacy_usage_as_satisfied() {
    let mut before = state(true);
    for team in &mut before.teams {
        team.defense = DefenseState::LegacyUnassigned;
        team.pitcher = team.pitcher.active_id().unwrap().into();
    }
    let action = changes(
        &before,
        vec![PersonnelChange::Pitcher {
            team: TeamSide::Home,
            outgoing: "hp".into(),
            incoming: "hrelief".into(),
        }],
    );
    assert!(apply(&before, action).is_err());
}
#[test]
fn mixed_explicit_ph_cannot_evade_opposing_pitcher_usage_obligation() {
    let mut before = state(true);
    before.teams[0].dh_must_bat = true;
    before.teams[1].defense = DefenseState::LegacyUnassigned;
    before.teams[1].pitcher = "hp".into();
    let mut action = ph(&before);
    action.delta.changes.push(PersonnelChange::Pitcher {
        team: TeamSide::Home,
        outgoing: "hp".into(),
        incoming: "hrelief".into(),
    });
    assert!(apply(&before, action).is_err());
}
#[test]
fn one_completed_batter_is_enough_for_2019_relief_replacement() {
    let mut before = state(false);
    before.teams[1].pitcher = PitcherRole::Active {
        player_id: "hp".into(),
        obligation: PitcherObligation::StartingBatter,
    };
    let first = apply(&before, walk(&before)).unwrap();
    assert!(matches!(
        first.after().teams[1].pitcher,
        PitcherRole::Active {
            obligation: PitcherObligation::Satisfied,
            ..
        }
    ));
    let replacement = |before: &GameState, old: &str, new: &str| {
        changes(
            before,
            vec![
                PersonnelChange::Pitcher {
                    team: TeamSide::Home,
                    outgoing: old.into(),
                    incoming: new.into(),
                },
                PersonnelChange::BattingSlot {
                    team: TeamSide::Home,
                    slot: 8,
                    outgoing: old.into(),
                    incoming: new.into(),
                },
            ],
        )
    };
    let relief = apply(first.after(), replacement(first.after(), "hp", "hrelief")).unwrap();
    assert!(apply(
        relief.after(),
        replacement(relief.after(), "hrelief", "hfresh")
    )
    .is_err());
    let batter = apply(relief.after(), walk(relief.after())).unwrap();
    assert!(matches!(
        batter.after().teams[1].pitcher,
        PitcherRole::Active {
            obligation: PitcherObligation::Satisfied,
            ..
        }
    ));
    let next = apply(
        batter.after(),
        replacement(batter.after(), "hrelief", "hfresh"),
    )
    .unwrap();
    let log = [first, relief, batter, next];
    for t in &log {
        wire(t);
    }
    let box_score = derive_accounting(&log).unwrap();
    assert_eq!(box_score.pitchers[1]["hp"].batters_faced, 1);
    assert_eq!(box_score.pitchers[1]["hrelief"].batters_faced, 1);
    assert!(!box_score.pitchers[1].contains_key("hfresh"));
}
#[test]
fn a_completed_batter_does_not_retroactively_certify_untracked_legacy_usage() {
    let mut before = state(false);
    before.teams[1].pitcher = "hp".into();
    let t = apply(&before, out(&before)).unwrap();
    assert!(matches!(
        t.after().teams[1].pitcher,
        PitcherRole::Active {
            obligation: PitcherObligation::UntrackedLegacy,
            ..
        }
    ));
    assert!(apply(t.after(), ph(t.after())).is_err());
    wire(&t);
}
#[test]
fn assigned_caught_strikeout_requires_the_actual_catcher_in_all_canonical_paths() {
    for half in [Half::Top, Half::Bottom] {
        for dh in [false, true] {
            let mut before = state(dh);
            before.half = half;
            before.outs = 0;
            let prefix = if half == Half::Top { "h" } else { "a" };
            let catcher = format!("{prefix}0");
            let wrong = format!("{prefix}4");
            let mut correct = out(&before);
            let batter = correct.participants.batter.clone().unwrap();
            correct.fielding_review = FieldingReview::NoMisplay;
            correct.caught_strikeout = Some(bbsim_engine::CaughtStrikeoutFacts {
                batter_id: batter.clone(),
                catcher_id: catcher.clone(),
            });
            for kind in [CreditKind::Strikeout, CreditKind::NoHit] {
                correct.delta.credits.push(Credit {
                    player_id: batter.clone(),
                    kind,
                });
            }
            correct.delta.credits.push(Credit {
                player_id: catcher.clone(),
                kind: CreditKind::Putout,
            });
            let t = apply(&before, correct.clone()).unwrap();
            wire(&t);
            let mut bad = correct;
            bad.caught_strikeout.as_mut().unwrap().catcher_id = wrong.clone();
            bad.delta
                .credits
                .iter_mut()
                .find(|c| c.kind == CreditKind::Putout)
                .unwrap()
                .player_id = wrong.clone();
            assert!(apply(&before, bad.clone()).is_err());
            assert!(EventTransition::try_new(before.clone(), bad, t.after().clone()).is_err());
            let mut bytes = t.to_bytes();
            let mut pattern = vec![1];
            pattern.extend(encoded_name(&batter));
            pattern.extend(encoded_name(&catcher));
            let hits: Vec<_> = bytes
                .windows(pattern.len())
                .enumerate()
                .filter_map(|(i, s)| (s == pattern).then_some(i))
                .collect();
            assert_eq!(hits.len(), 1);
            let at = hits[0] + pattern.len() - catcher.len();
            bytes[at..at + wrong.len()].copy_from_slice(wrong.as_bytes());
            // Keep fact/credit consistent: Putout's frozen wire tag is8.
            let mut credit_pattern = encoded_name(&catcher);
            credit_pattern.push(8);
            let hits: Vec<_> = bytes
                .windows(credit_pattern.len())
                .enumerate()
                .filter_map(|(i, s)| (s == credit_pattern).then_some(i))
                .collect();
            assert_eq!(hits.len(), 1);
            let at = hits[0] + credit_pattern.len() - 1 - catcher.len();
            bytes[at..at + wrong.len()].copy_from_slice(wrong.as_bytes());
            assert!(EventTransition::from_bytes(&bytes).is_err());
        }
    }
}
#[test]
fn legacy_debug_text_stays_compatible_while_extended_roles_are_explicit() {
    let mut legacy = team("a", false);
    legacy.pitcher = "ap".into();
    legacy.defense = DefenseState::LegacyUnassigned;
    assert_eq!(
        format!("{legacy:?}"),
        r#"TeamState { batting_order: ["a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7", "ap"], next_batter: 8, pitcher: "ap", dh_slot: None, dh_must_bat: false, bench: ["aph", "arelief", "afresh"], removed: [] }"#
    );
    assert_eq!(
        format!(
            "{:?}",
            Participants {
                batter: Some("ap".into()),
                pitcher: Some("hp".into())
            }
        ),
        r#"Participants { batter: Some("ap"), pitcher: "hp" }"#
    );
    assert_eq!(
        format!(
            "{:?}",
            Participants {
                batter: None,
                pitcher: None
            }
        ),
        "Participants { batter: None, pitcher: None }"
    );
    let explicit = format!("{:?}", team("a", false));
    assert!(explicit.contains("obligation: Satisfied"));
    assert!(explicit.contains("defense: Assigned"));
    let before = state(false);
    let t = apply(&before, ph(&before)).unwrap();
    assert!(format!("{:?}", t.after()).contains("pitcher: Vacant { batting_slot: 8 }"));
}
#[test]
fn completed_batter_clears_the_starting_pitcher_obligation() {
    let mut before = state(false);
    before.half = Half::Bottom;
    before.outs = 0;
    before.teams[1].next_batter = 0;
    before.teams[0].pitcher = PitcherRole::Active {
        player_id: "ap".into(),
        obligation: PitcherObligation::StartingBatter,
    };
    let next = apply(&before, out(&before)).unwrap();
    assert_eq!(
        next.after().teams[0].pitcher,
        PitcherRole::Active {
            player_id: "ap".into(),
            obligation: PitcherObligation::Satisfied,
        }
    );
    wire(&next);
}

#[test]
fn relief_obligation_but_not_starting_obligation_clears_on_a_runner_only_third_out() {
    for obligation in [
        PitcherObligation::StartingBatter,
        PitcherObligation::ReliefBatter,
    ] {
        let mut before = state(false);
        before.half = Half::Bottom;
        before.teams[1].next_batter = 0;
        before.teams[0].pitcher = PitcherRole::Active {
            player_id: "ap".into(),
            obligation,
        };
        let runner = Runner {
            player_id: "h1".into(),
            responsible_pitcher: "ap".into(),
        };
        before.bases[0] = Some(runner.clone());
        let action = event(
            &before,
            EventKind::RunnerOnly(RunnerEvent::Pickoff),
            Delta {
                changes: vec![],
                moves: vec![RunnerMove {
                    runner,
                    from: Base::First,
                    to: Destination::Out,
                }],
                outs: 1,
                runs: 0,
                credits: vec![],
                third_out: None,
            },
        );
        let next = apply(&before, action).unwrap();
        let expected = if obligation == PitcherObligation::ReliefBatter {
            PitcherObligation::Satisfied
        } else {
            obligation
        };
        assert_eq!(
            next.after().teams[0].pitcher,
            PitcherRole::Active {
                player_id: "ap".into(),
                obligation: expected
            }
        );
    }
}
