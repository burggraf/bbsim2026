//! Reconstruct the five bounded F-02 transition shapes from the published
//! fixture report. Historical runner-to-pitcher responsibility is not present
//! in that audit; the explicit placeholder below tests preservation only.
use bbsim_engine::{
    accounting::derive_accounting, apply, Base, Credit, CreditKind, Delta, Destination, Event,
    EventKind, GameState, Half, Participants, Runner, RunnerEvent, RunnerMove, TeamState,
};

fn state(
    half: Half,
    inning: u16,
    outs: u8,
    score: [u16; 2],
    batter: &str,
    pitcher: &str,
    occupants: [Option<&str>; 3],
) -> GameState {
    let mut teams = [
        TeamState {
            defense: bbsim_engine::DefenseState::LegacyUnassigned,
            batting_order: std::array::from_fn(|i| format!("away_{i}")),
            next_batter: 0,
            pitcher: "other_away_pitcher".into(),
            dh_slot: Some(8),
            dh_must_bat: false,
            bench: vec![],
            removed: vec![],
        },
        TeamState {
            defense: bbsim_engine::DefenseState::LegacyUnassigned,
            batting_order: std::array::from_fn(|i| format!("home_{i}")),
            next_batter: 0,
            pitcher: "other_home_pitcher".into(),
            dh_slot: Some(8),
            dh_must_bat: false,
            bench: vec![],
            removed: vec![],
        },
    ];
    let offense = half.offense();
    teams[offense].batting_order[0] = batter.into();
    for (index, id) in occupants.iter().enumerate() {
        if let Some(id) = id {
            teams[offense].batting_order[5 + index] = (*id).into();
        }
    }
    teams[1 - offense].pitcher = pitcher.into();
    GameState {
        inning,
        half,
        outs,
        score,
        teams,
        bases: occupants.map(|id| {
            id.map(|player_id| Runner {
                player_id: player_id.into(),
                responsible_pitcher: "unverified_prior_pitcher".into(),
            })
        }),
        over: false,
    }
}
fn move_base(before: &GameState, index: usize, from: Base, to: Destination) -> RunnerMove {
    RunnerMove {
        runner: before.bases[index].clone().unwrap(),
        from,
        to,
    }
}
fn plate(batter: &str, pitcher: &str, to: Destination) -> RunnerMove {
    RunnerMove {
        runner: Runner {
            player_id: batter.into(),
            responsible_pitcher: pitcher.into(),
        },
        from: Base::Plate,
        to,
    }
}
fn event(
    before: &GameState,
    kind: EventKind,
    moves: Vec<RunnerMove>,
    outs: u8,
    runs: u8,
    mut credits: Vec<Credit>,
) -> Event {
    let offense = before.half.offense();
    let batter = before.teams[offense].batting_order[0].clone();
    if kind == EventKind::PlateAppearance {
        credits.push(Credit {
            player_id: batter.clone(),
            kind: CreditKind::PlateAppearance,
        });
    }
    Event {
        kind,
        participants: Participants {
            batter: Some(batter),
            pitcher: Some(
                before.teams[1 - offense]
                    .pitcher
                    .active_id()
                    .expect("legacy fixture pitcher")
                    .to_owned(),
            ),
        },
        delta: Delta {
            moves,
            outs,
            runs,
            credits,
            changes: vec![],
            third_out: None,
        },
        fielding_review: bbsim_engine::FieldingReview::Unknown,
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

#[test]
fn ana201904040_36_wild_pitch_moves_two_runners_without_pa() {
    let before = state(
        Half::Bottom,
        3,
        2,
        [6, 2],
        "lastt001",
        "volqe001",
        [Some("pujoa001"), Some("fletd002"), None],
    );
    let action = event(
        &before,
        EventKind::RunnerOnly(RunnerEvent::WildPitch),
        vec![
            move_base(&before, 0, Base::First, Destination::Second),
            move_base(&before, 1, Base::Second, Destination::Third),
        ],
        0,
        0,
        vec![],
    );
    let next = apply(&before, action).unwrap();
    assert_eq!(
        (
            next.after().inning,
            next.after().half,
            next.after().outs,
            next.after().score
        ),
        (3, Half::Bottom, 2, [6, 2])
    );
    assert_eq!(next.after().bases[0], None);
    assert_eq!(
        next.after().bases[1].as_ref().unwrap().player_id,
        "pujoa001"
    );
    assert_eq!(
        next.after().bases[2].as_ref().unwrap().player_id,
        "fletd002"
    );
    assert_eq!(next.after().teams[1].next_batter, 0);
}

#[test]
fn ana201904040_96_strikeout_passed_ball_batter_reaches() {
    let before = state(
        Half::Bottom,
        9,
        0,
        [11, 3],
        "bourp001",
        "dowdk001",
        [None, None, None],
    );
    // The bounded F-02 row does not identify the catcher, so this checks K/PA
    // and movement, not who receives the passed-ball statistical credit.
    let action = event(
        &before,
        EventKind::PlateAppearance,
        vec![plate("bourp001", "dowdk001", Destination::First)],
        0,
        0,
        vec![Credit {
            player_id: "bourp001".into(),
            kind: CreditKind::Strikeout,
        }],
    );
    let next = apply(&before, action).unwrap();
    assert_eq!((next.after().outs, next.after().score), (0, [11, 3]));
    assert_eq!(
        next.after().bases[0].as_ref().unwrap().player_id,
        "bourp001"
    );
    assert_eq!(next.after().teams[1].next_batter, 1);
}

#[test]
fn ana201904040_99_batter_out_and_runner_scores() {
    let before = state(
        Half::Bottom,
        9,
        1,
        [11, 3],
        "pujoa001",
        "dowdk001",
        [None, Some("bourj002"), Some("bourp001")],
    );
    let action = event(
        &before,
        EventKind::PlateAppearance,
        vec![
            plate("pujoa001", "dowdk001", Destination::Out),
            move_base(&before, 2, Base::Third, Destination::Home),
        ],
        1,
        1,
        vec![Credit {
            player_id: "bourp001".into(),
            kind: CreditKind::Run,
        }],
    );
    let next = apply(&before, action).unwrap();
    assert_eq!((next.after().outs, next.after().score), (2, [11, 4]));
    assert_eq!(
        next.after().bases[1].as_ref().unwrap().player_id,
        "bourj002"
    );
    assert_eq!(next.after().bases[2], None);
    let accounted = derive_accounting(&[next]).unwrap();
    assert_eq!(accounted.innings[&9], [None, Some(1)]);
    assert_eq!(
        accounted.pitchers[0]["unverified_prior_pitcher"].runs_allowed,
        1
    );
    assert_eq!(accounted.pitchers[0]["dowdk001"].outs_recorded, 1);
}

#[test]
fn ana201904050_4_strikeout_and_steal() {
    let before = state(
        Half::Top,
        1,
        1,
        [0, 0],
        "mazan001",
        "penaf002",
        [Some("andre001"), None, Some("odorr001")],
    );
    let action = event(
        &before,
        EventKind::PlateAppearance,
        vec![
            plate("mazan001", "penaf002", Destination::Out),
            move_base(&before, 0, Base::First, Destination::Second),
        ],
        1,
        0,
        vec![
            Credit {
                player_id: "mazan001".into(),
                kind: CreditKind::Strikeout,
            },
            Credit {
                player_id: "andre001".into(),
                kind: CreditKind::StolenBase,
            },
        ],
    );
    let next = apply(&before, action).unwrap();
    assert_eq!((next.after().outs, next.after().score), (2, [0, 0]));
    assert_eq!(
        next.after().bases[1].as_ref().unwrap().player_id,
        "andre001"
    );
    assert_eq!(
        next.after().bases[2].as_ref().unwrap().player_id,
        "odorr001"
    );
    assert_eq!(next.after().teams[0].next_batter, 1);
    let accounted = derive_accounting(&[next]).unwrap();
    assert_eq!(accounted.batters[0]["mazan001"].plate_appearances, 1);
    assert_eq!(accounted.batters[0]["mazan001"].credited_strikeouts, 1);
    assert_eq!(accounted.batters[0]["andre001"].credited_steals, 1);
    assert_eq!(accounted.pitchers[1]["penaf002"].outs_recorded, 1);
}

#[test]
fn ana201904070_3_caught_stealing_keeps_batter() {
    let before = state(
        Half::Top,
        1,
        1,
        [0, 0],
        "andre001",
        "strac001",
        [Some("odorr001"), None, None],
    );
    let action = event(
        &before,
        EventKind::RunnerOnly(RunnerEvent::StealAttempt),
        vec![move_base(&before, 0, Base::First, Destination::Out)],
        1,
        0,
        vec![Credit {
            player_id: "odorr001".into(),
            kind: CreditKind::CaughtStealing,
        }],
    );
    let next = apply(&before, action).unwrap();
    assert_eq!(
        (
            next.after().inning,
            next.after().half,
            next.after().outs,
            next.after().score
        ),
        (1, Half::Top, 2, [0, 0])
    );
    assert_eq!(next.after().bases, [None, None, None]);
    assert_eq!(next.after().teams[0].next_batter, 0);
    let accounted = derive_accounting(&[next]).unwrap();
    assert_eq!(accounted.batters[0]["odorr001"].credited_caught_stealing, 1);
    assert_eq!(
        accounted.batters[0]
            .get("andre001")
            .map(|b| b.plate_appearances),
        None
    );
    assert_eq!(accounted.pitchers[1]["strac001"].outs_recorded, 1);
}
