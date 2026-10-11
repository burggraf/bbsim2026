#[path = "../examples/support/demo.rs"]
mod demo;

use bbsim_engine::{accounting::derive_accounting, EventTransition, Half};

#[test]
fn script_completes_a_repeatable_game_with_literal_totals() {
    let log = demo::scripted_game().unwrap();
    assert_eq!(log.len(), 59);
    assert_eq!(log, demo::scripted_game().unwrap());
    assert!(log.last().unwrap().after().over);
    assert_eq!(log.last().unwrap().after().score, [3, 1]);
    assert_eq!(log.first().unwrap().before().inning, 1);
    assert_eq!(log.first().unwrap().before().half, Half::Top);
    for pair in log.windows(2) {
        pair[1].follows(&pair[0]).unwrap();
    }
    for transition in &log {
        assert_eq!(
            EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
            *transition
        );
    }
    let stats = derive_accounting(&log).unwrap();
    assert!(stats.complete_game);
    assert_eq!(stats.runs, [3, 1]);
    assert_eq!(
        std::array::from_fn::<_, 2, _>(|side| stats.batters[side]
            .values()
            .map(|b| b.plate_appearances)
            .sum::<u32>()),
        [30, 28]
    );
    for (side, expected_hits) in [(0, 2), (1, 1)] {
        assert_eq!(
            stats.batters[side]
                .values()
                .map(|b| b.credited_hits)
                .sum::<u32>(),
            expected_hits
        );
        assert_eq!(
            stats.pitchers[side]
                .values()
                .map(|p| p.outs_recorded)
                .sum::<u32>(),
            27
        );
    }
}

#[test]
fn autoplay_shows_events_bases_final_lines_and_honest_labels() {
    let mut output = Vec::new();
    demo::run(&mut &b""[..], &mut output, true).unwrap();
    let text = String::from_utf8(output).unwrap();
    for expected in [
        "SCRIPTED",
        "not a probability model",
        "single",
        "steals second",
        "walk",
        "home run",
        "Bases: 1=- 2=Away-1 3=-",
        "FINAL: Away 3 - Home 1",
        "Line score",
        "PA",
        "not a complete official box",
    ] {
        assert!(text.contains(expected), "missing {expected:?} in {text}");
    }
    assert_eq!(text.matches("Play ").count(), 59);
}

#[test]
fn terminal_frame_is_game_over_not_an_active_zero_out_half() {
    let mut output = Vec::new();
    demo::run(&mut &b""[..], &mut output, true).unwrap();
    let text = String::from_utf8(output).unwrap();
    let terminal = text
        .split("Play 59")
        .nth(1)
        .unwrap()
        .split("FINAL:")
        .next()
        .unwrap();
    assert!(terminal.contains("GAME OVER"));
    assert!(!terminal.contains("Outs: 0"));
}

#[test]
fn stepping_reaches_the_same_game_and_eof_or_quit_stops_cleanly() {
    let mut stepped = Vec::new();
    demo::run(&mut "\n".repeat(59).as_bytes(), &mut stepped, false).unwrap();
    let text = String::from_utf8(stepped).unwrap();
    assert!(text.contains("FINAL: Away 3 - Home 1"));
    assert_eq!(text.matches("Play ").count(), 59);
    for input in [b"".as_slice(), b"q\n".as_slice()] {
        let mut output = Vec::new();
        demo::run(&mut &input[..], &mut output, false).unwrap();
        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("Stopped before play 1"));
        assert!(!text.contains("FINAL:"));
    }
}
