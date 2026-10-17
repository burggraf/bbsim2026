#[allow(dead_code)]
#[path = "../examples/support/full_game.rs"]
mod full_game;
#[allow(dead_code)]
#[path = "../examples/support/full_manager.rs"]
mod full_manager;
#[allow(dead_code)]
#[path = "../examples/support/full_simulation.rs"]
mod full_simulation;
#[path = "../examples/support/full_transcript.rs"]
mod full_transcript;
#[allow(dead_code)]
#[path = "../examples/support/preview.rs"]
mod preview;
#[path = "../examples/support/transcript.rs"]
mod transcript;
use bbsim_engine::*;
use std::collections::BTreeMap;
fn fixture() -> (GameState, [String; 9], BTreeMap<String, String>) {
    let before = GameState {
        inning: 3,
        half: Half::Top,
        outs: 1,
        score: [2, 3],
        teams: ["a", "h"].map(|p| TeamState {
            batting_order: std::array::from_fn(|i| format!("{p}{i}")),
            next_batter: 0,
            pitcher: format!("{p}p"),
            dh_slot: Some(8),
            dh_must_bat: false,
            bench: vec![],
            removed: vec![],
        }),
        bases: [
            Some(Runner {
                player_id: "a5".into(),
                responsible_pitcher: "hp".into(),
            }),
            None,
            None,
        ],
        over: false,
    };
    let fielders = std::array::from_fn(|i| {
        if i == 0 {
            "hp".into()
        } else {
            format!("h{}", i - 1)
        }
    });
    let names = before
        .teams
        .iter()
        .flat_map(|t| t.batting_order.iter().chain(std::iter::once(&t.pitcher)))
        .map(|id| {
            (
                id.clone(),
                format!("Name {}", id.replace('a', "Away ").replace('h', "Home ")),
            )
        })
        .collect();
    (before, fielders, names)
}
fn render(
    step: &full_simulation::Step,
    names: &BTreeMap<String, String>,
) -> std::io::Result<String> {
    let mut text = Vec::new();
    full_transcript::write_step(
        &mut text,
        1,
        step,
        &["Seattle".into(), "Oakland".into()],
        names,
    )?;
    let text = String::from_utf8(text).unwrap();
    if let Some(directory) = std::env::var_os("BBSIM_TEXT_GAME_CASES") {
        use std::io::Write;
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT_CASE: AtomicUsize = AtomicUsize::new(1);
        std::fs::create_dir_all(&directory)?;
        let path = std::path::PathBuf::from(directory).join(format!(
            "case-{:03}.txt",
            NEXT_CASE.fetch_add(1, Ordering::Relaxed)
        ));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        writeln!(file, "CONSTRUCTED SYNTHETIC-IDENTITY REGRESSION EXAMPLE — NOT AN ORDINARY GAME OR HISTORICAL REPLAY")?;
        writeln!(
            file,
            "Test: {}",
            std::thread::current()
                .name()
                .unwrap_or("constructed regression")
        )?;
        file.write_all(text.as_bytes())?;
    }
    Ok(text)
}
#[test]
fn joint_hit_narration_names_direction_throw_tag_and_winning_official_credit() {
    let (mut before, defense, names) = fixture();
    let decision = full_game::HitDecision {
        value: 2,
        position: 9,
        runners: [Some(Destination::Out), None, None],
        tag: Some(full_game::TagAttempt {
            origin: Base::First,
            attempted_base: Destination::Home,
            home_before_tag: vec![],
        }),
    };
    let step = full_simulation::Step::Play(full_game::hit(&before, &defense, decision).unwrap());
    let text = render(&step, &names).unwrap();
    for required in [
        "Double",
        "right fielder Name Home 7",
        "catcher Name Home 0",
        "tag at home",
        "Name Away 5 is out",
        "Name Away 0 reaches second",
    ] {
        assert!(text.contains(required), "missing {required}: {text}");
    }
    before.half = Half::Bottom;
    before.teams.swap(0, 1);
    before.score = [0, 0];
    before.inning = 9;
    before.bases[2] = before.bases[0].take();
    let decision = full_game::HitDecision {
        value: 3,
        position: 7,
        runners: [None, None, Some(Destination::Home)],
        tag: None,
    };
    let step = full_simulation::Step::Play(full_game::hit(&before, &defense, decision).unwrap());
    let text = render(&step, &names).unwrap();
    for required in [
        "Single",
        "left fielder Name Home 5",
        "winning run",
        "Name Away 5 scores",
        "game over",
    ] {
        assert!(text.contains(required), "missing {required}: {text}");
    }
    assert!(
        !text.contains("Triple"),
        "official walkoff credit must not retain sampled value: {text}"
    );
}

#[test]
fn uncaught_k_narration_distinguishes_automatic_safe_hold_and_advance_before_recovery() {
    use full_game::StrikeoutDecision::*;
    for (decision, outs, first_empty, expected) in [
        (
            Automatic,
            1,
            false,
            ["automatically out", "catcher Name Home 0"],
        ),
        (
            SafeReach {
                handling: HandlingAssessment::PitcherWild,
            },
            2,
            false,
            ["wild pitch", "reaches first safely"],
        ),
        (
            RecoveredBeforeFirst {
                handling: HandlingAssessment::CatcherPassed,
            },
            1,
            true,
            ["before first", "runners hold"],
        ),
        (
            RecoveredBeforeFirstAfterAdvances {
                handling: HandlingAssessment::PitcherWild,
            },
            1,
            true,
            ["wild pitch", "advances before the recovery throw"],
        ),
        (
            RecoveredBeforeFirstAfterAdvances {
                handling: HandlingAssessment::CatcherPassed,
            },
            2,
            true,
            ["passed ball", "crosses home; run does not count"],
        ),
    ] {
        let (mut before, defense, names) = fixture();
        before.outs = outs;
        if first_empty {
            before.bases.swap(0, 1);
        }
        if outs == 2 && matches!(decision, RecoveredBeforeFirstAfterAdvances { .. }) {
            before.bases[2] = before.bases[1].take();
        }
        let step = full_simulation::Step::Play(
            full_game::uncaught_strikeout(&before, &defense, decision).unwrap(),
        );
        let text = render(&step, &names).unwrap();
        assert!(text.contains("Strike three is not caught"), "{text}");
        for required in expected {
            assert!(text.contains(required), "missing {required}: {text}");
        }
        if matches!(decision, RecoveredBeforeFirst { .. } | Automatic) {
            assert!(
                !text.contains("passed ball") && !text.contains("wild pitch"),
                "unearned handling unit: {text}"
            );
        }
    }
}

#[test]
fn connected_rendering_names_matchup_ground_mechanism_every_runner_and_continuous_snapshots() {
    let (before, fielders, names) = fixture();
    let first = full_simulation::Step::Play(full_game::ground_out(&before, &fielders, 6).unwrap());
    let text = render(&first, &names).unwrap();
    for required in [
        "Before: Seattle 2 — Oakland 3",
        "Top 3 | 1 out",
        "Batter: Name Away 0",
        "Pitcher: Name Home p",
        "shortstop",
        "Name Home 4",
        "first baseman",
        "Name Home 1",
        "Name Away 5 holds at first",
        "After:",
    ] {
        assert!(text.contains(required), "missing {required}: {text}");
    }
    let second = full_simulation::Step::Play(
        full_game::ground_out(first.transition().after(), &fielders, 3).unwrap(),
    );
    let next = render(&second, &names).unwrap();
    assert!(next.contains("Name Away 5 is stranded at first"), "{next}");
    let after = text
        .lines()
        .find_map(|l| l.strip_prefix("After: "))
        .unwrap();
    let before = next
        .lines()
        .find_map(|l| l.strip_prefix("Before: "))
        .unwrap();
    assert_eq!(after, before);
    assert!(!text.contains("recorded play"));
}
#[test]
fn throwing_error_and_failed_bunt_force_choice_are_concrete_not_fielding_muff_or_sacrifice() {
    let (before, fielders, names) = fixture();
    let step = full_simulation::Step::Play(
        full_game::ordinary(&before, &fielders, full_game::Ordinary::ThrowingError(6)).unwrap(),
    );
    let text = render(&step, &names).unwrap();
    assert!(text.contains("wild throw"), "{text}");
    assert!(!text.contains("muffed"), "{text}");
    let step =
        full_simulation::Step::Play(full_game::force_choice(&before, &fielders, 6, true).unwrap());
    let text = render(&step, &names).unwrap();
    assert!(text.contains("Failed sacrifice bunt"), "{text}");
    assert!(text.contains("force at second"), "{text}");
    assert!(text.contains("Name Away 5 is out"), "{text}");
    assert!(text.contains("Name Away 0 reaches first"), "{text}");
}

#[test]
fn caught_ball_tag_double_play_names_the_catch_retouch_throw_or_unassisted_tag() {
    let (before, fielders, names) = fixture();
    for p in [4, 8] {
        let step = full_simulation::Step::Play(
            full_game::air_advance(
                &before,
                &fielders,
                AirContact::LineDrive,
                p,
                Base::First,
                true,
            )
            .unwrap(),
        );
        let text = render(&step, &names).unwrap();
        assert!(text.contains("Line drive"), "{text}");
        assert!(text.contains("legal retouch"), "{text}");
        assert!(text.contains("Double play"), "{text}");
        if p == 4 {
            assert!(text.contains("keeps the ball"), "{text}");
            assert!(!text.contains("throws"), "{text}");
        } else {
            assert!(
                text.contains("throws to second baseman Name Home 2"),
                "{text}"
            );
        }
    }
}

#[test]
fn safe_hit_endpoints_keep_their_advances_and_stranding_at_the_third_out() {
    use full_game::{HitDecision, TagAttempt};
    for outs in [1, 2] {
        let (mut before, fielders, names) = fixture();
        before.outs = outs;
        before.bases[1] = Some(Runner {
            player_id: "a6".into(),
            responsible_pitcher: "hp".into(),
        });
        let step = full_simulation::Step::Play(
            full_game::hit(
                &before,
                &fielders,
                HitDecision {
                    value: 1,
                    position: 8,
                    runners: [Some(Destination::Second), Some(Destination::Out), None],
                    tag: Some(TagAttempt {
                        origin: Base::Second,
                        attempted_base: Destination::Home,
                        home_before_tag: vec![],
                    }),
                },
            )
            .unwrap(),
        );
        let text = render(&step, &names).unwrap();
        assert!(text.contains("Name Away 0 reaches first"), "{text}");
        assert!(text.contains("Name Away 5 advances to second"), "{text}");
        assert!(text.contains("Name Away 6 is out (from second)"), "{text}");
        if outs == 2 {
            assert!(
                text.contains("Name Away 0 reaches first; is stranded at first."),
                "{text}"
            );
            assert!(
                text.contains("Name Away 5 advances to second; is stranded at second."),
                "{text}"
            );
            assert!(step.transition().after().bases.iter().all(Option::is_none));
        } else {
            assert!(!text.contains("stranded"), "{text}");
            assert!(step.transition().after().bases[0].is_some());
            assert!(step.transition().after().bases[1].is_some());
        }
    }
}

#[test]
fn recovered_k_advances_are_stranded_but_a_suppressed_home_crossing_is_not() {
    let (mut before, fielders, names) = fixture();
    before.outs = 2;
    before.bases[1] = Some(Runner {
        player_id: "a6".into(),
        responsible_pitcher: "hp".into(),
    });
    before.bases[2] = Some(Runner {
        player_id: "a7".into(),
        responsible_pitcher: "hp".into(),
    });
    let step = full_simulation::Step::Play(
        full_game::uncaught_strikeout(
            &before,
            &fielders,
            full_game::StrikeoutDecision::RecoveredBeforeFirstAfterAdvances {
                handling: bbsim_engine::HandlingAssessment::PitcherWild,
            },
        )
        .unwrap(),
    );
    let text = render(&step, &names).unwrap();
    assert!(
        text.contains("Name Away 5 advances to second; is stranded at second."),
        "{text}"
    );
    assert!(
        text.contains("Name Away 6 advances to third; is stranded at third."),
        "{text}"
    );
    assert!(
        text.contains("Name Away 7 crosses home; run does not count."),
        "{text}"
    );
    assert!(!text.contains("Name Away 7 is stranded"), "{text}");
    assert_eq!(step.transition().after().score, before.score);
    assert!(step.transition().after().bases.iter().all(Option::is_none));
}

#[test]
fn missing_actor_names_refuse_without_partial_or_id_fallback_output() {
    let (before, fielders, mut names) = fixture();
    let step = full_simulation::Step::Play(full_game::ground_out(&before, &fielders, 6).unwrap());
    names.remove("h4");
    let mut output = Vec::new();
    let error = full_transcript::write_step(
        &mut output,
        1,
        &step,
        &["Seattle".into(), "Oakland".into()],
        &names,
    )
    .unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(output.is_empty());
}

#[test]
fn home_steal_tag_has_no_catcher_self_throw_and_explicit_holds_never_become_advances() {
    let (mut before, fielders, names) = fixture();
    before.bases.swap(0, 2);
    let step = full_simulation::Step::Play(
        full_game::runner(
            &before,
            &fielders,
            full_game::RunnerDecision::Steal {
                origin: Base::Third,
                target: Destination::Home,
                caught: true,
            },
        )
        .unwrap(),
    );
    let text = render(&step, &names).unwrap();
    assert!(text.contains("tags the runner at home"), "{text}");
    assert!(!text.contains("throws to"), "{text}");
}

#[test]
fn failed_pickoff_explicit_holds_never_become_advances() {
    let (mut before, fielders, names) = fixture();
    before.bases.swap(0, 2);
    let step = full_simulation::Step::Play(
        full_game::runner(
            &before,
            &fielders,
            full_game::RunnerDecision::Pickoff {
                origin: Base::Third,
                caught: false,
            },
        )
        .unwrap(),
    );
    let text = render(&step, &names).unwrap();
    assert!(text.contains("Name Away 5 holds at third"), "{text}");
    assert!(!text.contains("advances to"), "{text}");
}
