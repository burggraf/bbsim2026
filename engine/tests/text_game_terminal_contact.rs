//! Late-inning eligibility for the already-bound simple SF/bunt/direct-force families.
#[allow(dead_code)]
#[path = "../examples/support/contact_consequence.rs"]
mod contact;
use bbsim_engine::*;

fn state(mask: u8, outs: u8, inning: u16, score: [u16; 2]) -> GameState {
    GameState {
        inning,
        half: Half::Bottom,
        outs,
        score,
        teams: ["a", "h"].map(|p| TeamState {
            batting_order: std::array::from_fn(|i| format!("{p}{i}")),
            next_batter: 0,
            pitcher: format!("{p}p"),
            dh_slot: Some(8),
            dh_must_bat: false,
            bench: vec![],
            removed: vec![],
        }),
        bases: std::array::from_fn(|i| {
            (mask & (1 << i) != 0).then(|| Runner {
                player_id: format!("h{}", i + 5),
                responsible_pitcher: "ap".into(),
            })
        }),
        over: false,
    }
}
fn checked(before: &GameState, decision: contact::ContactDecision) -> EventTransition {
    let original = before.clone();
    let t = contact::project(before, &decision)
        .unwrap_or_else(|error| panic!("{error}; {decision:?}; {before:?}"));
    assert_eq!(before, &original);
    assert_eq!(t.before(), before);
    assert_eq!(EventTransition::from_bytes(&t.to_bytes()).unwrap(), t);
    // Event alone omits these supplemental facts; reapply the full canonical input.
    let independently_applied = match decision {
        contact::ContactDecision::CaughtSacrificeFly { facts, .. } => {
            apply_with_caught_sacrifice_fly_facts(before, t.event().clone(), facts)
        }
        contact::ContactDecision::SacrificeBunt(facts) => {
            apply_with_sacrifice_bunt_throw_facts(before, t.event().clone(), facts)
        }
        contact::ContactDecision::GroundForceDoublePlay(facts) => {
            apply_with_ground_force_double_play_facts(before, t.event().clone(), facts)
        }
    }
    .unwrap();
    assert_eq!(independently_applied, t);
    t
}
#[test]
fn explicitly_timed_single_runner_sacrifice_fly_supports_walkoff_and_nonwinning_late_runs() {
    for inning in [9, 10] {
        for outs in [0, 1] {
            for score in [[1, 1], [2, 0], [1, 0]] {
                for mask in [4, 5, 6, 7] {
                    let before = state(mask, outs, inning, score);
                    let t = checked(
                        &before,
                        contact::ContactDecision::CaughtSacrificeFly {
                            catch: OrdinaryAirCatchFacts {
                                batter_id: "h0".into(),
                                fielder_id: "a6".into(),
                                contact: AirContact::Fly,
                            },
                            facts: CaughtSacrificeFlyFacts {
                                scoring_runner_id: "h7".into(),
                                handling:
                                    SacrificeFlyHandling::QualifyingOutfielderOrInfielderInOutfield,
                                retouch: SacrificeFlyRetouch::Legal,
                                timing: SacrificeFlyRunTiming::AfterCatch,
                            },
                        },
                    );
                    assert_eq!(t.after().score, [score[0], score[1] + 1]);
                    assert_eq!(t.event().delta.outs, 1);
                    assert_eq!(t.after().outs, outs + 1);
                    assert_eq!(t.after().over, score[1] + 1 > score[0]);
                    assert_eq!(t.after().teams[1].next_batter, 1);
                    assert_eq!(t.after().bases[0], before.bases[0]);
                    assert_eq!(t.after().bases[1], before.bases[1]);
                    assert!(t.after().bases[2].is_none());
                    assert_eq!(
                        t.rbi_contribution().unwrap().runs_batted_in.get("h0"),
                        Some(&1)
                    );
                }
            }
        }
    }
}
#[test]
fn direct_force_relay_can_end_top_ninth_when_home_already_leads() {
    let mut before = state(1, 1, 9, [0, 1]);
    before.half = Half::Top;
    before.bases[0] = Some(Runner {
        player_id: "a5".into(),
        responsible_pitcher: "hp".into(),
    });
    let t = checked(
        &before,
        contact::ContactDecision::GroundForceDoublePlay(GroundForceDoublePlayFacts {
            batter_id: "a0".into(),
            forced_runner_id: "a5".into(),
            initial_fielder_id: "h4".into(),
            pivot_id: "h2".into(),
            first_receiver_id: "h1".into(),
        }),
    );
    assert!(t.after().over);
    assert_eq!(
        (t.after().inning, t.after().half, t.after().outs),
        (9, Half::Top, 0)
    );
    assert_eq!(t.after().score, before.score);
    assert!(t.after().bases.iter().all(Option::is_none));
}
#[test]
fn non_scoring_bunt_and_direct_force_relay_have_no_blanket_bottom_ninth_exclusion() {
    for inning in [9, 10] {
        for outs in [0, 1] {
            for score in [[0, 0], [1, 0]] {
                for mask in [1, 5] {
                    let before = state(mask, outs, inning, score);
                    let bunt = checked(
                        &before,
                        contact::ContactDecision::SacrificeBunt(SacrificeBuntThrowFacts {
                            batter_id: "h0".into(),
                            advancing_runner_id: "h5".into(),
                            thrower_id: "a3".into(),
                            receiver_id: "a1".into(),
                            judgment: SacrificeBuntJudgment::SacrificingOwnChance,
                        }),
                    );
                    assert_eq!(bunt.after().score, score);
                    assert_eq!(bunt.after().outs, outs + 1);
                    assert_eq!(bunt.after().inning, inning);
                    assert_eq!(bunt.after().half, Half::Bottom);
                    assert!(!bunt.after().over);
                    assert_eq!(bunt.after().bases[1], before.bases[0]);
                    let dp = checked(
                        &before,
                        contact::ContactDecision::GroundForceDoublePlay(
                            GroundForceDoublePlayFacts {
                                batter_id: "h0".into(),
                                forced_runner_id: "h5".into(),
                                initial_fielder_id: "a4".into(),
                                pivot_id: "a2".into(),
                                first_receiver_id: "a1".into(),
                            },
                        ),
                    );
                    assert_eq!(dp.after().score, score);
                    assert_eq!(dp.event().delta.outs, 2);
                    assert_eq!(dp.after().over, outs == 1 && score[1] < score[0]);
                    if outs == 1 {
                        let expected_frame = if dp.after().over {
                            (inning, Half::Bottom, 0)
                        } else {
                            (inning + 1, Half::Top, 0)
                        };
                        assert_eq!(
                            (dp.after().inning, dp.after().half, dp.after().outs),
                            expected_frame
                        );
                        assert!(dp.after().bases.iter().all(Option::is_none));
                    } else {
                        assert_eq!(dp.after().outs, 2);
                        assert!(dp.after().bases[0].is_none());
                        assert_eq!(dp.after().bases[2], before.bases[2]);
                    }
                }
            }
        }
    }
}
