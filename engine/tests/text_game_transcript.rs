//! Human-readable state snapshots; no outcome generation in the transcript layer.
#[path = "../examples/support/transcript.rs"]
mod transcript;
use bbsim_engine::{GameState, Half, Runner, TeamState};
use std::collections::BTreeMap;

fn state(mask: u8) -> (GameState, BTreeMap<String, String>) {
    let teams = ["a", "h"].map(|p| TeamState {
        defense: bbsim_engine::DefenseState::LegacyUnassigned,
        batting_order: std::array::from_fn(|i| format!("{p}{i}")),
        next_batter: 0,
        pitcher: (format!("{p}p")).into(),
        dh_slot: Some(8),
        dh_must_bat: false,
        bench: vec![],
        removed: vec![],
    });
    let names = [("a3", "Alice"), ("a4", "Beth"), ("a5", "Casey")]
        .map(|(id, name)| (id.to_string(), name.to_string()))
        .into_iter()
        .collect();
    (
        GameState {
            inning: 4,
            half: Half::Top,
            outs: 1,
            score: [2, 3],
            teams,
            bases: std::array::from_fn(|i| {
                (mask & (1 << i) != 0).then(|| Runner {
                    player_id: format!("a{}", i + 3),
                    responsible_pitcher: "hp".into(),
                })
            }),
            over: false,
        },
        names,
    )
}

#[test]
fn snapshots_spell_out_every_base_configuration_with_names_and_complete_state() {
    let descriptions = [
        "bases empty",
        "runner on first",
        "runner on second",
        "runners on first and second",
        "runner on third",
        "runners on first and third",
        "runners on second and third",
        "bases loaded",
    ];
    for (mask, description) in descriptions.into_iter().enumerate() {
        let (state, names) = state(mask as u8);
        let frozen = state.clone();
        let mut text = Vec::new();
        transcript::write_state(
            &mut text,
            "Before",
            &state,
            &["Seattle".into(), "Oakland".into()],
            &names,
        )
        .unwrap();
        let text = String::from_utf8(text).unwrap();
        assert!(text.contains("Before: Seattle 2 — Oakland 3"), "{text}");
        assert!(text.contains("Top 4 | 1 out"), "{text}");
        assert!(text.contains(description), "{text}");
        for (i, name) in ["Alice", "Beth", "Casey"].into_iter().enumerate() {
            assert_eq!(text.contains(name), mask & (1 << i) != 0, "{text}");
        }
        assert_eq!(state, frozen);
    }
}

#[test]
fn bottom_final_and_missing_runner_names_are_explicit_without_printing_source_ids() {
    let (mut state, names) = state(1);
    state.half = Half::Bottom;
    let mut text = Vec::new();
    transcript::write_state(
        &mut text,
        "After",
        &state,
        &["Away".into(), "Home".into()],
        &names,
    )
    .unwrap();
    assert!(String::from_utf8(text).unwrap().contains("Bottom 4"));
    state.over = true;
    let mut text = Vec::new();
    transcript::write_state(
        &mut text,
        "After",
        &state,
        &["Away".into(), "Home".into()],
        &names,
    )
    .unwrap();
    assert!(String::from_utf8(text).unwrap().contains("FINAL"));
    let mut text = Vec::new();
    assert!(transcript::write_state(
        &mut text,
        "After",
        &state,
        &["Away".into(), "Home".into()],
        &BTreeMap::new()
    )
    .is_err());
    assert!(
        text.is_empty(),
        "refuse missing identities before writing a misleading partial snapshot"
    );
}
