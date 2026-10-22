#[path = "../examples/support/pitcher_entry_v2.rs"]
mod entry;
#[path = "../examples/support/pitcher_policy_v2.rs"]
mod pitcher_policy_v2;

use bbsim_engine::{
    apply, DefenseState, Delta, Event, EventKind, EventTransition, FieldingReview, GameState, Half,
    Participants, PersonnelChange, PitcherObligation, PitcherRole, TeamSide, TeamState,
};
use entry::{Budget, Candidate, Reason};
use pitcher_policy_v2::{Clock, ReliefUsage};

const GAME: &str = "OAK201903200";
const SEED: u64 = 17;

fn complete_trace(log: &[EventTransition], records: &[String]) -> String {
    let mut text = entry::header(GAME, SEED, log.len() as u32, records.len() as u32).unwrap();
    for record in records {
        text.push_str(record);
    }
    text
}

fn state() -> GameState {
    let team = |prefix: &str, pitcher: &str, bench: &[&str]| {
        let batting_order = std::array::from_fn(|i| format!("{prefix}{i}"));
        let positions = std::array::from_fn(|i| {
            Some(if i == 0 {
                pitcher.into()
            } else {
                format!("{prefix}{}", i - 1)
            })
        });
        TeamState {
            batting_order,
            next_batter: 0,
            pitcher: PitcherRole::Active {
                player_id: pitcher.into(),
                obligation: PitcherObligation::Satisfied,
            },
            defense: DefenseState::Assigned(Box::new(positions)),
            dh_slot: Some(8),
            dh_must_bat: false,
            bench: bench.iter().map(|s| (*s).into()).collect(),
            removed: vec![],
        }
    };
    GameState {
        inning: 6,
        half: Half::Bottom,
        outs: 0,
        score: [0, 0],
        teams: [team("a", "ap", &["a", "b"]), team("h", "hp", &["hr"])],
        bases: [None, None, None],
        over: false,
    }
}

fn candidates() -> Vec<Candidate> {
    [
        ("ap", true, "0,?", 0, 0),
        ("a", false, "60,60", 6, 1),
        ("b", false, "30,60", 5, 1),
    ]
    .into_iter()
    .map(|(id, starter, usage, pa_sum, outings)| Candidate {
        id: id.into(),
        starter,
        usage: ReliefUsage::read(usage).unwrap(),
        pa_sum,
        outings,
    })
    .collect()
}

fn selected(before: &GameState) -> entry::Selection {
    entry::select(before, 0, Clock::read("80,162").unwrap(), &candidates()).unwrap()
}

fn transition(before: &GameState, side: usize, incoming: &str) -> EventTransition {
    let team = if side == 0 {
        TeamSide::Away
    } else {
        TeamSide::Home
    };
    let changes = match &before.teams[side].pitcher {
        PitcherRole::Active { player_id, .. } => vec![PersonnelChange::Pitcher {
            team,
            outgoing: player_id.clone(),
            incoming: incoming.into(),
        }],
        PitcherRole::Vacant { batting_slot } => vec![
            PersonnelChange::AssignPitcher {
                team,
                incoming: incoming.into(),
            },
            PersonnelChange::BattingSlot {
                team,
                slot: *batting_slot,
                outgoing: before.teams[side].batting_order[*batting_slot as usize].clone(),
                incoming: incoming.into(),
            },
        ],
    };
    apply(
        before,
        Event {
            kind: EventKind::Substitution,
            participants: Participants {
                batter: Some(before.teams[before.half.offense()].batting_order[0].clone()),
                pitcher: before.teams[1 - before.half.offense()]
                    .pitcher
                    .active_id()
                    .map(str::to_owned),
            },
            fielding_review: FieldingReview::Unknown,
            delta: Delta {
                moves: Vec::new(),
                outs: 0,
                runs: 0,
                credits: Vec::new(),
                changes,
                third_out: None,
            },
            two_out_contact: None,
            single_delivery: None,
            balk: None,
            caught_strikeout: None,
            protected_award: None,
            sole_home_run: None,
            sole_safe_hit: None,
            muffed_ground_ball: None,
            ordinary_air_catch: None,
            ordinary_ground_throw: None,
        },
    )
    .unwrap()
}

fn pinch_transition(before: &GameState, incoming: &str) -> EventTransition {
    let side = before.half.offense();
    let team = if side == 0 {
        TeamSide::Away
    } else {
        TeamSide::Home
    };
    let outgoing = before.teams[side].batting_order[0].clone();
    apply(
        before,
        Event {
            kind: EventKind::Substitution,
            participants: Participants {
                batter: None,
                pitcher: None,
            },
            fielding_review: FieldingReview::Unknown,
            delta: Delta {
                moves: Vec::new(),
                outs: 0,
                runs: 0,
                credits: Vec::new(),
                changes: vec![
                    PersonnelChange::PinchHitter {
                        outgoing: outgoing.clone(),
                        incoming: incoming.into(),
                    },
                    PersonnelChange::AssignDefense {
                        team,
                        position: 2,
                        outgoing: Some(outgoing),
                        incoming: incoming.into(),
                    },
                ],
                third_out: None,
            },
            two_out_contact: None,
            single_delivery: None,
            balk: None,
            caught_strikeout: None,
            protected_award: None,
            sole_home_run: None,
            sole_safe_hit: None,
            muffed_ground_ball: None,
            ordinary_air_catch: None,
            ordinary_ground_throw: None,
        },
    )
    .unwrap()
}

fn pitcher_pinch_transition(before: &GameState, incoming: &str) -> EventTransition {
    let side = before.half.offense();
    let outgoing = before.teams[side].pitcher.active_id().unwrap().to_owned();
    apply(
        before,
        Event {
            kind: EventKind::Substitution,
            participants: Participants {
                batter: None,
                pitcher: None,
            },
            fielding_review: FieldingReview::Unknown,
            delta: Delta {
                moves: Vec::new(),
                outs: 0,
                runs: 0,
                credits: Vec::new(),
                changes: vec![PersonnelChange::PinchHitter {
                    outgoing,
                    incoming: incoming.into(),
                }],
                third_out: None,
            },
            two_out_contact: None,
            single_delivery: None,
            balk: None,
            caught_strikeout: None,
            protected_award: None,
            sole_home_run: None,
            sole_safe_hit: None,
            muffed_ground_ball: None,
            ordinary_air_catch: None,
            ordinary_ground_throw: None,
        },
    )
    .unwrap()
}

fn budget() -> Budget {
    Budget {
        word: 7,
        sample_index: 0,
        base: 20,
        scale: 80,
        lead_extension: 0,
        no_hit_floor: None,
        final_budget: 16,
    }
}

fn hex(t: &EventTransition) -> String {
    t.to_bytes().iter().map(|b| format!("{b:02x}")).collect()
}

#[test]
fn header_freezes_versions_and_canonical_metadata() {
    assert_eq!(entry::header("OAK201903200",u64::MAX,99,2).unwrap(),format!("format\tpitcher-entry-decisions-v2\ngame\tOAK201903200\nseed\t{}\nseason_policy\tseason-pool-pitcher-usage-v2\nmanager\tseason-pool-workload-context-2019-v2\nevents\t99\ndecisions\t2\n",u64::MAX));
    for bad in [
        "",
        "OAK201902290",
        "OAK201904310",
        "OAK201913010",
        "OAK201900010",
        "OAK201903000",
        "OAK201903203",
        "oak201903200",
        "OAK201803200",
        "OAK201903200\n",
    ] {
        assert!(entry::header(bad, 1, 1, 1).is_err(), "accepted {bad:?}");
    }
    assert!(entry::header("OAK201903200", 1, 0, 1).is_err());
}

#[test]
fn shared_selection_is_debt_first_deterministic_and_immutable() {
    let before = state();
    let copy = before.clone();
    let mut profiles = candidates();
    let original = format!("{profiles:?}");
    let first = entry::select(&before, 0, Clock::read("80,162").unwrap(), &profiles).unwrap();
    assert_eq!(first.chosen(), Some("b"));
    assert_eq!(format!("{profiles:?}"), original);
    assert_eq!(before, copy);
    profiles.reverse();
    let second = entry::select(&before, 0, Clock::read("80,162").unwrap(), &profiles).unwrap();
    assert_eq!(
        first.exhaustion_key().unwrap(),
        second.exhaustion_key().unwrap()
    );
    let log = vec![transition(&before, 0, "b")];
    assert_eq!(
        entry::encode_record(0, 0, Reason::Hook, &first, Some(&budget()), &log).unwrap(),
        entry::encode_record(0, 0, Reason::Hook, &second, Some(&budget()), &log).unwrap()
    );
}

#[test]
fn hook_serialization_binds_raw_inputs_exact_ranks_snapshot_budget_and_actual_entry() {
    let before = state();
    let choice = selected(&before);
    let t = transition(&before, 0, "b");
    let witness = hex(&t);
    assert_eq!(
        EventTransition::from_bytes(&t.to_bytes()).unwrap().before(),
        &before
    );
    let actual = entry::encode_record(0, 0, Reason::Hook, &choice, Some(&budget()), &[t]).unwrap();
    let expected=format!("decision\t0\t0\tAway\thook\tb\t80\t162\t{witness}\ncandidate\t0\ta\teligible\t60\t60\t6\t1\t-24300\t4860\t-4860\t4860\t1\t0\t-19440\t19440\t2\ncandidate\t0\tap\tstarter\t0\t?\t0\t0\t0\t1\t-\t-\t-\t-\t-\t-\t-\ncandidate\t0\tb\teligible\t30\t60\t5\t1\t0\t4860\t0\t4860\t0\t1\t4860\t19440\t1\nbudget\t0\tap\tstart\t7\t0\t20\t80\t0\t-\t16\napplied\t0\t{witness}\nenddecision\t0\t3\n");
    assert_eq!(actual, expected);
}

#[test]
fn removed_active_and_absent_candidates_never_reenter() {
    let mut before = state();
    before.teams[0].bench = vec!["a".into()];
    before.teams[0].removed = vec!["b".into()];
    assert_eq!(selected(&before).chosen(), Some("a"));
    before.teams[0].bench.clear();
    assert_eq!(selected(&before).chosen(), None);
    let mut profiles = candidates();
    profiles.push(Candidate {
        id: "hp".into(),
        starter: false,
        usage: ReliefUsage::read("0,60").unwrap(),
        pa_sum: 6,
        outings: 1,
    });
    assert_eq!(
        entry::select(&before, 1, Clock::read("80,162").unwrap(), &profiles)
            .unwrap()
            .chosen(),
        None
    );
}

#[test]
fn unknown_zero_targets_and_stable_id_ties_remain_distinct() {
    let before = state();
    let mut profiles = candidates();
    profiles[1].usage = ReliefUsage::read("0,?").unwrap();
    profiles[1].pa_sum = 5;
    profiles[2].usage = ReliefUsage::read("0,0").unwrap();
    profiles[2].pa_sum = 5;
    assert_eq!(
        entry::select(&before, 0, Clock::read("80,162").unwrap(), &profiles)
            .unwrap()
            .chosen(),
        Some("a")
    );
    profiles[1].usage = ReliefUsage::read("17,0").unwrap();
    assert_eq!(
        entry::select(&before, 0, Clock::read("80,162").unwrap(), &profiles)
            .unwrap()
            .chosen(),
        Some("b")
    );
}

#[test]
fn excluded_profiles_retain_real_debt_without_inventing_a_relief_class() {
    let before = state();
    let clock = Clock::read("80,162").unwrap();
    let mut profiles = candidates();
    profiles[0].usage = ReliefUsage::read("17,0").unwrap();
    assert_eq!(
        pitcher_policy_v2::relief_debt(clock, profiles[0].usage),
        (-13770, 4860)
    );
    let choice = entry::select(&before, 0, clock, &profiles).unwrap();
    let log = vec![transition(&before, 0, "b")];
    let wire = entry::encode_record(0, 0, Reason::Hook, &choice, Some(&budget()), &log).unwrap();
    assert!(wire.contains("\tap\tstarter\t17\t0\t0\t0\t-13770\t4860\t-\t-\t-\t-\t-\t-\t-\n"));
    let mut excluded = before.clone();
    excluded.teams[0].removed.push("a".into());
    excluded.teams[0].bench.retain(|id| id != "a");
    let choice = selected(&excluded);
    let log = vec![transition(&excluded, 0, "b")];
    let wire = entry::encode_record(0, 0, Reason::Hook, &choice, Some(&budget()), &log).unwrap();
    assert!(wire.contains(
        "\ta\tremoved\t60\t60\t6\t1\t-24300\t4860\t-4860\t4860\t1\t0\t-19440\t19440\t-\n"
    ));
}

#[test]
fn exhaustion_and_probes_cannot_hide_an_actual_entry() {
    let mut before = state();
    before.teams[0].bench = vec!["c".into()];
    let choice = selected(&before);
    assert_eq!(choice.chosen(), None);
    let log = vec![transition(&before, 0, "c")];
    assert!(entry::encode_record(0, 0, Reason::Hook, &choice, Some(&budget()), &log).is_err());
    let mut before = state();
    before.half = Half::Top;
    let choice = selected(&before);
    let log = vec![transition(&before, 0, "a")];
    assert!(entry::encode_record(0, 0, Reason::PhProbe, &choice, None, &log).is_err());
}

#[test]
fn invalid_profiles_fail_even_when_the_candidate_would_be_excluded() {
    let before = state();
    for id in ["", "A", "a\tb", "a b", "é", &"a".repeat(65)] {
        let mut p = candidates();
        p[1].id = id.into();
        assert!(entry::select(&before, 0, Clock::read("0,162").unwrap(), &p).is_err());
    }
    let mut p = candidates();
    p[1].id = "b".into();
    assert!(entry::select(&before, 0, Clock::read("0,162").unwrap(), &p).is_err());
    let mut p = candidates();
    p[1].outings = 0;
    assert!(entry::select(&before, 1, Clock::read("0,162").unwrap(), &p).is_err());
    for count in [0, 2] {
        let mut p = candidates();
        p[0].starter = count == 2;
        p[1].starter = count == 2;
        assert!(entry::select(&before, 0, Clock::read("0,162").unwrap(), &p).is_err());
    }
    assert!(entry::select(&before, 2, Clock::read("0,162").unwrap(), &candidates()).is_err());
    let mut invalid = before;
    invalid.inning = 0;
    assert!(entry::select(&invalid, 0, Clock::read("0,162").unwrap(), &candidates()).is_err());
}

#[test]
fn ph_probe_has_no_applied_entry_or_fabricated_budget() {
    let mut before = state();
    before.half = Half::Top;
    let choice = selected(&before);
    let log = vec![pinch_transition(&before, "a")];
    let text = entry::encode_record(1, 0, Reason::PhProbe, &choice, None, &log).unwrap();
    assert!(text.starts_with("decision\t1\t0\tAway\tph_probe\tb\t"));
    assert!(!text.contains("\napplied\t") && !text.contains("\nbudget\t"));
    assert!(entry::encode_record(1, 0, Reason::PhProbe, &choice, Some(&budget()), &log).is_err());
    let unrelated = vec![transition(&before, 1, "hr")];
    assert!(entry::encode_record(1, 0, Reason::PhProbe, &choice, None, &unrelated).is_err());

    let mut pitcher_at_bat = before;
    pitcher_at_bat.teams[0].dh_slot = None;
    pitcher_at_bat.teams[0].batting_order[8] = "ap".into();
    pitcher_at_bat.teams[0].next_batter = 8;
    pitcher_at_bat.teams[0].bench = vec!["ph".into()];
    let exhausted = entry::select(
        &pitcher_at_bat,
        0,
        Clock::read("80,162").unwrap(),
        &candidates(),
    )
    .unwrap();
    assert_eq!(exhausted.chosen(), None);
    let vacancy = vec![pitcher_pinch_transition(&pitcher_at_bat, "ph")];
    assert!(entry::encode_record(1, 0, Reason::PhProbe, &exhausted, None, &vacancy).is_err());

    let mut available = pitcher_at_bat;
    available.teams[0].bench = vec!["a".into(), "b".into(), "ph".into()];
    let reserve =
        entry::select(&available, 0, Clock::read("80,162").unwrap(), &candidates()).unwrap();
    assert!(reserve.chosen().is_some());
    let vacancy = vec![pitcher_pinch_transition(&available, "ph")];
    let text = entry::encode_record(1, 0, Reason::PhProbe, &reserve, None, &vacancy).unwrap();
    assert!(!text.contains("\napplied\t"));
}

#[test]
fn restoration_binds_the_real_vacancy_transition_without_a_manager_draw() {
    let mut before = state();
    before.teams[0].dh_slot = None;
    before.teams[0].batting_order[8] = "ph".into();
    before.teams[0].pitcher = PitcherRole::Vacant { batting_slot: 8 };
    before.teams[0].removed = vec!["ap".into()];
    let DefenseState::Assigned(ref mut positions) = before.teams[0].defense else {
        panic!()
    };
    positions[0] = None;
    let choice = selected(&before);
    let t = transition(&before, 0, "b");
    let text = entry::encode_record(2, 0, Reason::PhRestore, &choice, None, &[t]).unwrap();
    assert!(text.contains("\tAway\tph_restore\tb\t"));
    assert!(text.contains("\napplied\t2\t"));
    assert!(!text.contains("\nbudget\t"));
}

#[test]
fn exhaustion_is_explicit_and_does_not_forge_an_applied_change() {
    let mut before = state();
    before.teams[0].bench.clear();
    let choice = selected(&before);
    let t = transition(&before, 1, "hr");
    let text = entry::encode_record(0, 0, Reason::Hook, &choice, Some(&budget()), &[t]).unwrap();
    assert!(text.contains("\tAway\thook\t-\t"));
    assert!(!text.contains("\napplied\t"));
    assert!(text.contains("\tb\tnot_benched\t"));
}

#[test]
fn relief_budget_cannot_apply_starter_extensions_or_scale() {
    let mut before = state();
    before.teams[0].pitcher = PitcherRole::Active {
        player_id: "a".into(),
        obligation: PitcherObligation::Satisfied,
    };
    before.teams[0].bench = vec!["b".into()];
    before.teams[0].removed = vec!["ap".into()];
    let DefenseState::Assigned(ref mut positions) = before.teams[0].defense else {
        panic!()
    };
    positions[0] = Some("a".into());
    let choice = entry::select(&before, 0, Clock::read("80,162").unwrap(), &candidates()).unwrap();
    let log = vec![transition(&before, 0, "b")];
    for budget in [
        Budget {
            scale: 80,
            final_budget: 16,
            ..budget()
        },
        Budget {
            lead_extension: 3,
            final_budget: 19,
            ..budget()
        },
        Budget {
            no_hit_floor: Some(17),
            final_budget: 17,
            ..budget()
        },
    ] {
        assert!(entry::encode_record(0, 0, Reason::Hook, &choice, Some(&budget), &log).is_err());
    }
}

#[test]
fn whole_trace_accepts_ordered_hook_exhaustion_and_successful_ph_probe() {
    let mut before = state();
    before.teams[0].bench.clear();
    let away = candidates();
    let home = vec![
        Candidate {
            id: "hp".into(),
            starter: true,
            usage: ReliefUsage::read("0,?").unwrap(),
            pa_sum: 0,
            outings: 0,
        },
        Candidate {
            id: "hr".into(),
            starter: false,
            usage: ReliefUsage::read("30,60").unwrap(),
            pa_sum: 6,
            outings: 1,
        },
    ];
    let clocks = [
        Clock::read("80,162").unwrap(),
        Clock::read("80,162").unwrap(),
    ];
    let log = vec![pinch_transition(&before, "hr")];
    let hook = entry::select(&before, 0, clocks[0], &away).unwrap();
    let probe = entry::select(&before, 1, clocks[1], &home).unwrap();
    assert_eq!(hook.chosen(), None);
    assert_eq!(probe.chosen(), Some("hr"));
    let records = [
        entry::encode_record(0, 0, Reason::Hook, &hook, Some(&budget()), &log).unwrap(),
        entry::encode_record(1, 0, Reason::PhProbe, &probe, None, &log).unwrap(),
    ];
    let text = complete_trace(&log, &records);

    let decisions = entry::validate_trace(
        &text,
        GAME,
        SEED,
        &log,
        &[away.clone(), home.clone()],
        &clocks,
    )
    .unwrap();
    assert_eq!(decisions.len(), 2);
    assert_eq!(decisions[0].chosen, None);
    assert_eq!(decisions[0].budget, Some(budget()));
    assert_eq!(decisions[1].chosen.as_deref(), Some("hr"));
    assert_eq!(decisions[1].budget, None);
    let altered_candidate = text.replace("\teligible\t30", "\tremoved\t30");
    assert!(entry::validate_trace(
        &altered_candidate,
        GAME,
        SEED,
        &log,
        &[away.clone(), home.clone()],
        &clocks
    )
    .is_err());
    let extra_row = format!("{text}unexpected\n");
    assert!(entry::validate_trace(&extra_row, GAME, SEED, &log, &[away, home], &clocks).is_err());
}

#[test]
fn whole_trace_requires_actual_entries_restores_and_contiguous_history() {
    let before = state();
    let home = vec![
        Candidate {
            id: "hp".into(),
            starter: true,
            usage: ReliefUsage::read("0,?").unwrap(),
            pa_sum: 0,
            outings: 0,
        },
        Candidate {
            id: "hr".into(),
            starter: false,
            usage: ReliefUsage::read("30,60").unwrap(),
            pa_sum: 6,
            outings: 1,
        },
    ];
    let away = candidates();
    let clocks = [
        Clock::read("80,162").unwrap(),
        Clock::read("80,162").unwrap(),
    ];
    let log = vec![transition(&before, 0, "b")];
    let away_choice = entry::select(&before, 0, clocks[0], &away).unwrap();
    let record =
        entry::encode_record(0, 0, Reason::Hook, &away_choice, Some(&budget()), &log).unwrap();
    let text = complete_trace(&log, std::slice::from_ref(&record));
    assert!(entry::validate_trace(
        &text,
        GAME,
        SEED,
        &log,
        &[away.clone(), home.clone()],
        &clocks
    )
    .is_ok());

    let missing_entry = complete_trace(&log, &[]);
    assert!(entry::validate_trace(
        &missing_entry,
        GAME,
        SEED,
        &log,
        &[away.clone(), home.clone()],
        &clocks
    )
    .is_err());
    let wrong_count = text.replacen("events\t1\n", "events\t2\n", 1);
    assert!(entry::validate_trace(
        &wrong_count,
        GAME,
        SEED,
        &log,
        &[away.clone(), home.clone()],
        &clocks
    )
    .is_err());

    let mut broken = before.clone();
    broken.outs = 1;
    let discontinuous = vec![log[0].clone(), transition(&broken, 0, "b")];
    let invalid_history = complete_trace(&discontinuous, &[record]);
    assert!(entry::validate_trace(
        &invalid_history,
        GAME,
        SEED,
        &discontinuous,
        &[away.clone(), home.clone()],
        &clocks
    )
    .is_err());

    let mut vacant = before;
    vacant.teams[0].dh_slot = None;
    vacant.teams[0].batting_order[8] = "ph".into();
    vacant.teams[0].pitcher = PitcherRole::Vacant { batting_slot: 8 };
    vacant.teams[0].removed = vec!["ap".into()];
    vacant.teams[0].bench = vec!["b".into()];
    let DefenseState::Assigned(ref mut positions) = vacant.teams[0].defense else {
        panic!()
    };
    positions[0] = None;
    let restore_log = vec![transition(&vacant, 0, "b")];
    let restore = entry::select(&vacant, 0, clocks[0], &away).unwrap();
    let restore_record =
        entry::encode_record(0, 0, Reason::PhRestore, &restore, None, &restore_log).unwrap();
    let restore_trace = complete_trace(&restore_log, &[restore_record]);
    assert!(entry::validate_trace(
        &restore_trace,
        GAME,
        SEED,
        &restore_log,
        &[away, home],
        &clocks
    )
    .is_ok());
}

#[test]
fn whole_trace_rejects_pitcher_replacing_ph_without_a_successful_probe() {
    let mut before = state();
    before.half = Half::Top;
    before.teams[0].dh_slot = None;
    before.teams[0].batting_order[8] = "ap".into();
    before.teams[0].next_batter = 8;
    before.teams[0].bench = vec!["ph".into()];
    let log = vec![pitcher_pinch_transition(&before, "ph")];
    let text = complete_trace(&log, &[]);
    let profiles = [
        candidates(),
        vec![
            Candidate {
                id: "hp".into(),
                starter: true,
                usage: ReliefUsage::read("0,?").unwrap(),
                pa_sum: 0,
                outings: 0,
            },
            Candidate {
                id: "hr".into(),
                starter: false,
                usage: ReliefUsage::read("30,60").unwrap(),
                pa_sum: 6,
                outings: 1,
            },
        ],
    ];
    let clocks = [
        Clock::read("80,162").unwrap(),
        Clock::read("80,162").unwrap(),
    ];
    assert!(entry::validate_trace(&text, GAME, SEED, &log, &profiles, &clocks).is_err());
}

#[test]
fn wrong_event_index_snapshot_applied_actor_reason_and_budget_fail_closed() {
    let before = state();
    let choice = selected(&before);
    let log = vec![transition(&before, 0, "b")];
    assert!(entry::encode_record(0, 1, Reason::Hook, &choice, Some(&budget()), &log).is_err());
    let mut changed = before.clone();
    changed.outs = 1;
    let wrong = vec![transition(&changed, 0, "b")];
    assert!(entry::encode_record(0, 0, Reason::Hook, &choice, Some(&budget()), &wrong).is_err());
    let wrong = vec![transition(&before, 0, "a")];
    assert!(entry::encode_record(0, 0, Reason::Hook, &choice, Some(&budget()), &wrong).is_err());
    assert!(entry::encode_record(0, 0, Reason::PhProbe, &choice, None, &log).is_err());
    assert!(entry::encode_record(0, 0, Reason::PhRestore, &choice, None, &log).is_err());
    assert!(entry::encode_record(0, 0, Reason::Hook, &choice, None, &log).is_err());
    for b in [
        Budget {
            final_budget: 17,
            ..budget()
        },
        Budget {
            base: 0,
            ..budget()
        },
        Budget {
            scale: 90,
            ..budget()
        },
        Budget {
            lead_extension: 1,
            ..budget()
        },
        Budget {
            no_hit_floor: Some(0),
            ..budget()
        },
    ] {
        assert!(entry::encode_record(0, 0, Reason::Hook, &choice, Some(&b), &log).is_err());
    }
}
