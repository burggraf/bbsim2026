use bbsim_engine::{
    accounting::derive_accounting, apply, Base, Credit, CreditKind, Delta, Destination, Event,
    EventKind, EventTransition, GameState, Half, Participants, PersonnelChange, Runner, RunnerMove,
    TeamSide, TeamState, TransitionError,
};

fn team(prefix: &str) -> TeamState {
    TeamState {
        batting_order: std::array::from_fn(|i| format!("{prefix}{i}")),
        next_batter: 0,
        pitcher: format!("{prefix}_pitcher"),
        dh_slot: Some(8),
        dh_must_bat: false,
        bench: vec!["sub".into(), "reliever".into(), "runner".into()],
        removed: vec![],
    }
}
fn state() -> GameState {
    GameState {
        inning: 5,
        half: Half::Bottom,
        outs: 0,
        score: [0, 0],
        teams: [team("away"), team("home")],
        bases: [None, None, None],
        over: false,
    }
}
fn substitution(before: &GameState, changes: Vec<PersonnelChange>) -> Event {
    let offense = before.half.offense();
    Event {
        kind: EventKind::Substitution,
        participants: Participants {
            batter: Some(
                before.teams[offense].batting_order[before.teams[offense].next_batter as usize]
                    .clone(),
            ),
            pitcher: before.teams[1 - offense].pitcher.clone(),
        },
        delta: Delta {
            moves: vec![],
            outs: 0,
            runs: 0,
            credits: vec![],
            changes,
            third_out: None,
        },
        fielding_review: bbsim_engine::FieldingReview::Unknown,
        two_out_contact: None,
        single_delivery: None,
        balk: None,
    }
}
fn batter_out(before: &GameState) -> Event {
    let id = before.teams[1].batting_order[before.teams[1].next_batter as usize].clone();
    Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(id.clone()),
            pitcher: before.teams[0].pitcher.clone(),
        },
        delta: Delta {
            moves: vec![RunnerMove {
                runner: Runner {
                    player_id: id.clone(),
                    responsible_pitcher: before.teams[0].pitcher.clone(),
                },
                from: Base::Plate,
                to: Destination::Out,
            }],
            outs: 1,
            runs: 0,
            credits: vec![Credit {
                player_id: id,
                kind: CreditKind::PlateAppearance,
            }],
            changes: vec![],
            third_out: None,
        },
        fielding_review: bbsim_engine::FieldingReview::Unknown,
        two_out_contact: None,
        single_delivery: None,
        balk: None,
    }
}

#[test]
fn no_dh_pitcher_must_bat_and_advances_the_order() {
    let mut before = state();
    before.teams[1].dh_slot = None;
    before.teams[1].batting_order[8] = "home_pitcher".into();
    before.teams[1].next_batter = 8;
    let next = apply(&before, batter_out(&before)).unwrap();
    assert_eq!(next.after().teams[1].next_batter, 0);
    assert_eq!(
        next.event().participants.batter.as_deref(),
        Some("home_pitcher")
    );
}

#[test]
fn pitcher_replacement_without_dh_requires_matching_batting_change() {
    let mut before = state();
    before.teams[1].dh_slot = None;
    before.teams[1].batting_order[8] = "home_pitcher".into();
    let pitching = PersonnelChange::Pitcher {
        team: TeamSide::Home,
        outgoing: "home_pitcher".into(),
        incoming: "reliever".into(),
    };
    assert!(apply(&before, substitution(&before, vec![pitching.clone()])).is_err());
    let lineup = PersonnelChange::BattingSlot {
        team: TeamSide::Home,
        slot: 8,
        outgoing: "home_pitcher".into(),
        incoming: "reliever".into(),
    };
    let next = apply(&before, substitution(&before, vec![pitching, lineup])).unwrap();
    assert_eq!(next.after().teams[1].pitcher, "reliever");
    assert_eq!(next.after().teams[1].batting_order[8], "reliever");
    assert_eq!(next.after().teams[1].removed, ["home_pitcher"]);
}

#[test]
fn substitute_must_be_on_bench_and_removed_players_cannot_reenter() {
    let before = state();
    let change = PersonnelChange::BattingSlot {
        team: TeamSide::Home,
        slot: 3,
        outgoing: "home3".into(),
        incoming: "phantom".into(),
    };
    assert!(apply(&before, substitution(&before, vec![change])).is_err());
    let change = PersonnelChange::BattingSlot {
        team: TeamSide::Home,
        slot: 3,
        outgoing: "home3".into(),
        incoming: "sub".into(),
    };
    let next = apply(&before, substitution(&before, vec![change])).unwrap();
    assert_eq!(next.after().teams[1].batting_order[3], "sub");
    assert!(!next.after().teams[1].bench.contains(&"sub".into()));
    assert!(next.after().teams[1].removed.contains(&"home3".into()));
    let reentry = PersonnelChange::BattingSlot {
        team: TeamSide::Home,
        slot: 4,
        outgoing: "home4".into(),
        incoming: "home3".into(),
    };
    assert!(apply(next.after(), substitution(next.after(), vec![reentry])).is_err());
}

#[test]
fn dh_ends_irreversibly_when_pitcher_bats_for_him() {
    let before = state();
    let change = PersonnelChange::BattingSlot {
        team: TeamSide::Home,
        slot: 8,
        outgoing: "home8".into(),
        incoming: "home_pitcher".into(),
    };
    let next = apply(&before, substitution(&before, vec![change])).unwrap();
    assert_eq!(next.after().teams[1].dh_slot, None);
    assert_eq!(next.after().teams[1].batting_order[8], "home_pitcher");
    assert!(next.after().teams[1].removed.contains(&"home8".into()));
    let attempt = PersonnelChange::BattingSlot {
        team: TeamSide::Home,
        slot: 8,
        outgoing: "home_pitcher".into(),
        incoming: "sub".into(),
    };
    assert!(apply(next.after(), substitution(next.after(), vec![attempt])).is_err());
}

#[test]
fn pinch_runner_for_dh_takes_his_slot_without_changing_pitcher_responsibility() {
    let mut before = state();
    let outgoing = Runner {
        player_id: "home8".into(),
        responsible_pitcher: "away_pitcher".into(),
    };
    before.bases[0] = Some(outgoing.clone());
    let incoming = Runner {
        player_id: "runner".into(),
        responsible_pitcher: "away_pitcher".into(),
    };
    let change = PersonnelChange::PinchRunner {
        base: Base::First,
        outgoing,
        incoming: incoming.clone(),
    };
    let next = apply(&before, substitution(&before, vec![change])).unwrap();
    assert_eq!(next.after().bases[0], Some(incoming));
    assert_eq!(next.after().teams[1].batting_order[8], "runner");
    assert_eq!(next.after().teams[1].dh_slot, Some(8));
    assert!(next.after().teams[1].removed.contains(&"home8".into()));
    assert_eq!(
        bbsim_engine::EventTransition::from_bytes(&next.to_bytes()).unwrap(),
        next
    );
}

#[test]
fn duplicate_lineup_or_bench_player_is_an_invalid_state() {
    let mut before = state();
    before.teams[1].batting_order[5] = "home4".into();
    assert!(matches!(
        apply(&before, batter_out(&before)),
        Err(TransitionError::InvalidState)
    ));
    let mut before = state();
    before.teams[1].bench.push("home0".into());
    assert!(matches!(
        apply(&before, batter_out(&before)),
        Err(TransitionError::InvalidState)
    ));
}

#[test]
fn occupied_base_requires_an_active_batting_order_player() {
    let mut before = state();
    before.bases[0] = Some(Runner {
        player_id: "ghost".into(),
        responsible_pitcher: "away_pitcher".into(),
    });
    assert!(matches!(
        apply(&before, batter_out(&before)),
        Err(TransitionError::InvalidState)
    ));
}

#[test]
fn starting_dh_must_bat_once_before_being_replaced() {
    let mut before = state();
    before.teams[1].dh_must_bat = true;
    let change = PersonnelChange::BattingSlot {
        team: TeamSide::Home,
        slot: 8,
        outgoing: "home8".into(),
        incoming: "sub".into(),
    };
    assert!(apply(&before, substitution(&before, vec![change.clone()])).is_err());
    // A change by the opposing pitcher lifts the starting-DH obligation.
    let pitching = PersonnelChange::Pitcher {
        team: TeamSide::Away,
        outgoing: "away_pitcher".into(),
        incoming: "reliever".into(),
    };
    let next = apply(&before, substitution(&before, vec![pitching, change])).unwrap();
    assert!(!next.after().teams[1].dh_must_bat);
    assert_eq!(next.after().teams[1].dh_slot, Some(8));
}

#[test]
fn starting_dh_obligation_clears_when_he_bats() {
    let mut before = state();
    before.teams[1].next_batter = 8;
    before.teams[1].dh_must_bat = true;
    let next = apply(&before, batter_out(&before)).unwrap();
    assert!(!next.after().teams[1].dh_must_bat);
}

fn configuration(half: Half, dh: bool) -> GameState {
    let mut before = state();
    before.half = half;
    if !dh {
        for team in &mut before.teams {
            team.dh_slot = None;
            team.batting_order[8] = team.pitcher.clone();
        }
    }
    before
}

fn retained_swap_cases() -> Vec<(GameState, Event, GameState)> {
    let mut cases = vec![];
    for half in [Half::Top, Half::Bottom] {
        for dh in [false, true] {
            for side in 0..2 {
                for first in 0..9 {
                    for second in first + 1..9 {
                        for reverse in [false, true] {
                            let before = configuration(half, dh);
                            let team = [TeamSide::Away, TeamSide::Home][side];
                            let order = &before.teams[side].batting_order;
                            let mut changes = vec![
                                PersonnelChange::BattingSlot {
                                    team,
                                    slot: first as u8,
                                    outgoing: order[first].clone(),
                                    incoming: order[second].clone(),
                                },
                                PersonnelChange::BattingSlot {
                                    team,
                                    slot: second as u8,
                                    outgoing: order[second].clone(),
                                    incoming: order[first].clone(),
                                },
                            ];
                            if reverse {
                                changes.reverse();
                            }
                            let action = substitution(&before, changes);
                            let mut after = before.clone();
                            after.teams[side].batting_order.swap(first, second);
                            cases.push((before, action, after));
                        }
                    }
                }
            }
        }
    }
    assert_eq!(cases.len(), 576);
    cases
}

#[test]
fn reducer_rejects_retained_hitter_slot_permutations_without_mutating_caller() {
    for (before, action, _) in retained_swap_cases() {
        let original = before.clone();
        assert_eq!(
            apply(&before, action),
            Err(TransitionError::IllegalStateChange)
        );
        assert_eq!(before, original);
    }
}

#[test]
fn constructor_rejects_retained_hitter_slot_permutations() {
    for (before, action, after) in retained_swap_cases() {
        assert_eq!(
            EventTransition::try_new(before, action, after),
            Err(TransitionError::IllegalStateChange)
        );
    }
}

fn encoded_name(id: &str) -> Vec<u8> {
    let mut encoded = (id.len() as u64).to_le_bytes().to_vec();
    encoded.extend_from_slice(id.as_bytes());
    encoded
}

fn unique_wire_offset(bytes: &[u8], marker: &[u8]) -> usize {
    let offsets: Vec<_> = bytes
        .windows(marker.len())
        .enumerate()
        .filter_map(|(offset, window)| (window == marker).then_some(offset))
        .collect();
    assert_eq!(offsets.len(), 1);
    offsets[0]
}

#[test]
fn codec_rejects_retained_hitter_move_hidden_among_a_real_bench_replacement() {
    for half in [Half::Top, Half::Bottom] {
        for side in 0..2 {
            let before = configuration(half, true);
            let team = [TeamSide::Away, TeamSide::Home][side];
            let old = &before.teams[side].batting_order[0];
            let retained = &before.teams[side].batting_order[1];
            let action = substitution(
                &before,
                vec![PersonnelChange::BattingSlot {
                    team,
                    slot: 0,
                    outgoing: old.clone(),
                    incoming: "sub".into(),
                }],
            );
            let valid = apply(&before, action).unwrap();
            let mut bytes = valid.to_bytes();
            assert_eq!(EventTransition::from_bytes(&bytes).unwrap(), valid);
            // BBE11: replace one real substitution with two slot changes.
            // Preserve the final active membership/bench/removed transfers,
            // but put the retained slot-1 hitter into slot 0.
            let mut change = vec![1, side as u8, 0];
            change.extend(encoded_name(old));
            change.extend(encoded_name("sub"));
            let mut marker = 1u64.to_le_bytes().to_vec();
            marker.extend(&change);
            let offset = unique_wire_offset(&bytes, &marker);
            bytes[offset..offset + 8].copy_from_slice(&2u64.to_le_bytes());
            let incoming = offset + 8 + 3 + encoded_name(old).len();
            bytes.splice(
                incoming..incoming + encoded_name("sub").len(),
                encoded_name(retained),
            );
            let mut second = vec![1, side as u8, 1];
            second.extend(encoded_name(retained));
            second.extend(encoded_name("sub"));
            let end = incoming + encoded_name(retained).len();
            bytes.splice(end..end, second);
            let mut after_order = encoded_name("sub");
            after_order.extend(encoded_name(retained));
            let offset = unique_wire_offset(&bytes, &after_order);
            let mut swapped = encoded_name(retained);
            swapped.extend(encoded_name("sub"));
            bytes.splice(offset..offset + after_order.len(), swapped);
            assert_eq!(
                EventTransition::from_bytes(&bytes),
                Err(TransitionError::IllegalStateChange)
            );
        }
    }
}

#[test]
fn bench_replacements_preserve_retained_slots_and_do_not_add_pa_or_bf() {
    for half in [Half::Top, Half::Bottom] {
        for dh in [false, true] {
            for side in 0..2 {
                let before = configuration(half, dh);
                let team = [TeamSide::Away, TeamSide::Home][side];
                let changes = ["sub", "runner"]
                    .iter()
                    .enumerate()
                    .map(|(slot, id)| PersonnelChange::BattingSlot {
                        team,
                        slot: slot as u8,
                        outgoing: before.teams[side].batting_order[slot].clone(),
                        incoming: (*id).into(),
                    })
                    .collect();
                let valid = apply(&before, substitution(&before, changes)).unwrap();
                assert_eq!(
                    &valid.after().teams[side].batting_order[2..],
                    &before.teams[side].batting_order[2..]
                );
                assert_eq!(
                    valid.after().teams[side].next_batter,
                    before.teams[side].next_batter
                );
                assert_eq!(
                    valid.after().teams[side].removed,
                    before.teams[side].batting_order[..2]
                );
                assert_eq!(
                    EventTransition::from_bytes(&valid.to_bytes()).unwrap(),
                    valid
                );
                assert_eq!(
                    EventTransition::try_new(before, valid.event().clone(), valid.after().clone())
                        .unwrap(),
                    valid
                );
                let line = derive_accounting(std::slice::from_ref(&valid)).unwrap();
                assert_eq!(line.runs, [0, 0]);
                assert_eq!(
                    line.batters
                        .iter()
                        .flat_map(|team| team.values())
                        .map(|b| b.plate_appearances)
                        .sum::<u32>(),
                    0
                );
                assert_eq!(
                    line.pitchers
                        .iter()
                        .flat_map(|team| team.values())
                        .map(|p| p.batters_faced)
                        .sum::<u32>(),
                    0
                );
            }
        }
    }
}

#[test]
fn next_batter_cannot_already_be_on_base() {
    let mut before = state();
    before.bases[0] = Some(Runner {
        player_id: "home0".into(),
        responsible_pitcher: "away_pitcher".into(),
    });
    assert!(matches!(
        apply(&before, batter_out(&before)),
        Err(TransitionError::InvalidState)
    ));
}
