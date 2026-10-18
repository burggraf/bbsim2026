//! Model-supplied legal retouch/throw/tag chronology is distinct from held-air proof.
#[allow(dead_code)]
#[path = "../examples/support/full_game.rs"]
mod full_game;
#[allow(dead_code)]
#[path = "../examples/support/preview.rs"]
mod preview;
use bbsim_engine::*;
fn state(mask: u8, outs: u8, half: Half) -> GameState {
    let prefix = if half == Half::Top { "a" } else { "h" };
    GameState {
        inning: 3,
        half,
        outs,
        score: [0, 0],
        teams: ["a", "h"].map(|p| TeamState {
            defense: bbsim_engine::DefenseState::LegacyUnassigned,
            batting_order: std::array::from_fn(|i| format!("{p}{i}")),
            next_batter: 0,
            pitcher: (format!("{p}p")).into(),
            dh_slot: Some(8),
            dh_must_bat: false,
            bench: vec![],
            removed: vec![],
        }),
        bases: std::array::from_fn(|i| {
            (mask & (1 << i) != 0).then(|| Runner {
                player_id: format!("{prefix}{}", i + 5),
                responsible_pitcher: format!("{}p", if half == Half::Top { "h" } else { "a" }),
            })
        }),
        over: false,
    }
}
fn fielders(before: &GameState) -> [String; 9] {
    let p = if before.half == Half::Top { "h" } else { "a" };
    std::array::from_fn(|i| {
        if i == 0 {
            format!("{p}p")
        } else {
            format!("{p}{}", i - 1)
        }
    })
}
#[test]
fn legally_retouching_runner_advances_or_is_tagged_after_the_catch_without_false_ground_dp_credit()
{
    for half in [Half::Top, Half::Bottom] {
        for mask in 1..8 {
            for outs in 0..3 {
                for i in 0..3 {
                    for caught in [false, true] {
                        let before = state(mask, outs, half);
                        let origin = [Base::First, Base::Second, Base::Third][i];
                        let p = if i == 0 { 4 } else { 8 };
                        let play = full_game::air_advance(
                            &before,
                            &fielders(&before),
                            AirContact::Fly,
                            p,
                            origin,
                            caught,
                        );
                        let eligible = outs < 2
                            && before.bases[i].is_some()
                            && (i == 2 || before.bases[i + 1].is_none());
                        if !eligible {
                            assert!(play.is_err());
                            continue;
                        }
                        let play = play.unwrap();
                        let t = play.transition();
                        assert_eq!(t.before(), &before);
                        assert_eq!(t.event().delta.outs, 1 + u8::from(caught));
                        assert!(!t
                            .event()
                            .delta
                            .credits
                            .iter()
                            .any(|c| c.kind == CreditKind::GroundedIntoDoublePlay));
                        assert_eq!(
                            t.event()
                                .delta
                                .credits
                                .iter()
                                .filter(|c| c.kind == CreditKind::Putout)
                                .count(),
                            usize::from(t.event().delta.outs)
                        );
                        assert_eq!(
                            t.event()
                                .delta
                                .credits
                                .iter()
                                .filter(|c| c.kind == CreditKind::Assist)
                                .count(),
                            usize::from(caught && p != 4)
                        );
                        if caught {
                            assert!(t.after().bases.iter().flatten().all(
                                |r| r.player_id != before.bases[i].as_ref().unwrap().player_id
                            ));
                        } else if i == 2 {
                            assert_eq!(t.after().score[half.offense()], 1);
                            assert!(t.caught_sacrifice_fly_facts().is_some());
                        } else {
                            assert_eq!(t.after().bases[i + 1], before.bases[i]);
                        }
                        assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), *t);
                        if t.caught_sacrifice_fly_facts().is_none() {
                            assert_eq!(apply(&before, t.event().clone()).unwrap(), *t);
                            assert!(
                                t.event().ordinary_air_catch.is_none(),
                                "old held-only proof must not certify a compound leaf"
                            );
                        }
                        assert_eq!(before, state(mask, outs, half));
                    }
                }
            }
        }
    }
}
#[test]
fn outfield_retouch_can_win_a_late_game_but_two_out_catch_cannot_launch_an_attempt() {
    let mut before = state(4, 1, Half::Bottom);
    before.inning = 9;
    before.score = [2, 2];
    let play = full_game::air_advance(
        &before,
        &fielders(&before),
        AirContact::Fly,
        8,
        Base::Third,
        false,
    )
    .unwrap();
    assert!(play.transition().after().over);
    assert_eq!(play.transition().after().score, [2, 3]);
    before.outs = 2;
    assert!(full_game::air_advance(
        &before,
        &fielders(&before),
        AirContact::Fly,
        8,
        Base::Third,
        false
    )
    .is_err());
}
