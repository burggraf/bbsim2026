#[path = "../examples/support/full_strikeout.rs"]
mod full_strikeout;

use bbsim_engine::accounting::derive_accounting;
use bbsim_engine::{
    apply, Base, Credit, CreditKind, Destination, Event, EventTransition, GameState, Half,
    HandlingAdvance, HandlingAssessment, RecoveryReview, Runner, RunnerMove, TeamState,
    ThirdOutKind, TransitionError,
};
use full_strikeout::{project, StrikeoutDecision};

fn state(half: Half, outs: u8, mask: u8) -> GameState {
    let teams = ["away", "home"].map(|prefix| TeamState {
        defense: bbsim_engine::DefenseState::LegacyUnassigned,
        batting_order: std::array::from_fn(|i| format!("{prefix}{i}")),
        next_batter: 0,
        pitcher: (format!("{prefix}_pitcher")).into(),
        dh_slot: Some(8),
        dh_must_bat: false,
        bench: vec![],
        removed: vec![],
    });
    let offense = half.offense();
    let pitcher = teams[1 - offense]
        .pitcher
        .active_id()
        .expect("legacy fixture pitcher")
        .to_owned();
    GameState {
        inning: 2,
        half,
        outs,
        score: [0, 0],
        bases: std::array::from_fn(|index| {
            (mask & (1 << index) != 0).then(|| Runner {
                player_id: teams[offense].batting_order[5 + index].clone(),
                responsible_pitcher: pitcher.clone(),
            })
        }),
        teams,
        over: false,
    }
}

fn fielders(before: &GameState) -> [String; 9] {
    let defense = &before.teams[1 - before.half.offense()];
    [
        defense
            .pitcher
            .active_id()
            .expect("legacy fixture pitcher")
            .to_owned(),
        defense.batting_order[0].clone(),
        defense.batting_order[1].clone(),
        defense.batting_order[2].clone(),
        defense.batting_order[3].clone(),
        defense.batting_order[4].clone(),
        defense.batting_order[5].clone(),
        defense.batting_order[6].clone(),
        defense.batting_order[7].clone(),
    ]
}

fn apply_and_round_trip(before: &GameState, transition: &EventTransition) {
    assert_eq!(
        apply(before, transition.event().clone()).unwrap(),
        *transition
    );
    assert_eq!(
        EventTransition::try_new(
            before.clone(),
            transition.event().clone(),
            transition.after().clone(),
        )
        .unwrap(),
        *transition
    );
    assert_eq!(
        EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
        *transition
    );
}

fn accepted(
    before: &GameState,
    fielders: &[String; 9],
    decision: StrikeoutDecision,
) -> EventTransition {
    let saved = before.clone();
    let transition = project(before, fielders, &decision).unwrap();
    assert_eq!(*before, saved, "projection must not mutate its input state");
    apply_and_round_trip(before, &transition);
    transition
}

fn assert_credits(event: &Event, kind: CreditKind, expected: &[&str]) {
    let mut actual = event
        .delta
        .credits
        .iter()
        .filter(|credit| credit.kind == kind)
        .map(|credit| credit.player_id.as_str())
        .collect::<Vec<_>>();
    actual.sort_unstable();
    let mut expected = expected.to_vec();
    expected.sort_unstable();
    assert_eq!(actual, expected, "unexpected {kind:?} credits");
}

// Fixed projection semantics: Automatic holds; SafeReach advances all existing
// runners one base; RecoveredBeforeFirst retires the batter before any runner moves.
#[test]
fn all_base_masks_outs_and_halves_accept_only_the_legal_decision_leaves() {
    let mut accepted = 0;
    let mut refused = 0;
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..=2 {
            for mask in 0..=7 {
                let before = state(half, outs, mask);
                let fielders = fielders(&before);
                let eligible = mask & 1 == 0 || outs == 2;
                for handling in [
                    HandlingAssessment::PitcherWild,
                    HandlingAssessment::CatcherPassed,
                ] {
                    for (decision, should_accept) in [
                        (StrikeoutDecision::Automatic, mask & 1 != 0 && outs < 2),
                        (StrikeoutDecision::SafeReach { handling }, eligible),
                        (
                            StrikeoutDecision::RecoveredBeforeFirst { handling },
                            eligible,
                        ),
                    ] {
                        let saved = before.clone();
                        let projected = project(&before, &fielders, &decision);
                        assert_eq!(before, saved, "even a refusal must preserve input");
                        if should_accept {
                            let transition = projected.unwrap_or_else(|error| {
                                panic!("{half:?} outs={outs} mask={mask} {decision:?}: {error}")
                            });
                            apply_and_round_trip(&before, &transition);
                            assert_eq!(
                                transition.event().kind,
                                bbsim_engine::EventKind::PlateAppearance
                            );
                            assert!(transition.event().caught_strikeout.is_none());
                            assert_eq!(
                                transition
                                    .event()
                                    .delta
                                    .credits
                                    .iter()
                                    .filter(|credit| credit.kind == CreditKind::Strikeout)
                                    .count(),
                                1
                            );
                            accepted += 1;
                        } else {
                            assert!(projected.is_err(), "illegal leaf accepted: {decision:?}");
                            refused += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(accepted, 160);
    assert_eq!(refused, 128);
}

#[test]
fn automatic_k_is_a_first_ineligible_retirement_with_runners_held() {
    let before = state(Half::Top, 1, 1);
    let fielders = fielders(&before);
    let transition = accepted(&before, &fielders, StrikeoutDecision::Automatic);
    let batter = &before.teams[0].batting_order[0];
    let pitcher = before.teams[1].pitcher.active_id().unwrap();
    let catcher = &fielders[1];

    assert_eq!(
        transition.event().participants.batter.as_deref(),
        Some(batter.as_str())
    );
    assert_eq!(
        transition.event().participants.pitcher.as_deref(),
        Some(pitcher)
    );
    assert!(transition.event().single_delivery.is_none());
    assert!(transition.event().caught_strikeout.is_none());
    assert_eq!(transition.event().delta.outs, 1);
    assert_eq!(transition.after().outs, 2);
    assert_eq!(transition.after().bases, before.bases);
    assert_eq!(transition.event().delta.runs, 0);
    assert_credits(transition.event(), CreditKind::PlateAppearance, &[batter]);
    assert_credits(transition.event(), CreditKind::Strikeout, &[batter]);
    assert_credits(transition.event(), CreditKind::AtBat, &[]);
    assert_credits(transition.event(), CreditKind::WildPitch, &[]);
    assert_credits(transition.event(), CreditKind::PassedBall, &[]);
    assert_credits(transition.event(), CreditKind::Putout, &[catcher]);
    assert_credits(transition.event(), CreditKind::Assist, &[]);

    let box_score = derive_accounting(std::slice::from_ref(&transition)).unwrap();
    let batter_line = &box_score.batters[0][batter];
    assert_eq!(batter_line.plate_appearances, 1);
    assert_eq!(batter_line.credited_strikeouts, 1);
    assert_eq!(batter_line.at_bats, Some(1));
    let pitcher_line = &box_score.pitchers[1][pitcher];
    assert_eq!(pitcher_line.batters_faced, 1);
    assert_eq!(pitcher_line.outs_recorded, 1);
    assert_eq!(pitcher_line.credited_strikeouts, 1);
    assert_eq!(pitcher_line.credited_wild_pitches, 0);
    assert_eq!(box_score.fielders[1][catcher].credited_putouts, 1);
}

#[test]
fn safe_reach_advances_the_joint_runner_vector_and_uses_handling_units() {
    for handling in [
        HandlingAssessment::PitcherWild,
        HandlingAssessment::CatcherPassed,
    ] {
        let before = state(Half::Bottom, 0, 6);
        let fielders = fielders(&before);
        let transition = accepted(
            &before,
            &fielders,
            StrikeoutDecision::SafeReach { handling },
        );
        let batter = &before.teams[1].batting_order[0];
        let catcher = &fielders[1];
        let pitcher = &fielders[0];
        assert_eq!(transition.after().outs, 0);
        assert_eq!(
            transition.after().bases[0].as_ref().unwrap().player_id,
            *batter
        );
        assert_eq!(transition.event().delta.runs, 1);
        assert!(transition.event().delta.third_out.is_none());
        assert_eq!(
            transition
                .event()
                .single_delivery
                .as_ref()
                .unwrap()
                .handling,
            handling
        );
        assert_credits(transition.event(), CreditKind::Strikeout, &[batter]);
        let wild_credits = if handling == HandlingAssessment::PitcherWild {
            vec![pitcher.as_str()]
        } else {
            vec![]
        };
        let passed_credits = if handling == HandlingAssessment::CatcherPassed {
            vec![catcher.as_str()]
        } else {
            vec![]
        };
        assert_credits(transition.event(), CreditKind::WildPitch, &wild_credits);
        assert_credits(transition.event(), CreditKind::PassedBall, &passed_credits);
        assert_credits(transition.event(), CreditKind::Putout, &[]);
        assert_credits(transition.event(), CreditKind::Assist, &[]);
    }
}

#[test]
fn recovered_before_first_is_a_catcher_to_first_out_with_distinct_fielding_credits() {
    let before = state(Half::Top, 0, 4);
    let fielders = fielders(&before);
    let transition = accepted(
        &before,
        &fielders,
        StrikeoutDecision::RecoveredBeforeFirst {
            handling: HandlingAssessment::CatcherPassed,
        },
    );
    let batter = &before.teams[0].batting_order[0];
    let catcher = &fielders[1];
    let first_baseman = &fielders[2];
    let event = transition.event();
    let facts = event.single_delivery.as_ref().unwrap();
    assert_eq!(
        facts.context,
        bbsim_engine::DeliveryContext::UncaughtThirdStrike
    );
    assert_eq!(facts.handling, HandlingAssessment::CatcherPassed);
    assert!(facts.advances.is_empty());
    assert_eq!(event.delta.outs, 1);
    assert_eq!(event.delta.runs, 0);
    assert_eq!(transition.after().outs, 1);
    assert_eq!(transition.after().bases, before.bases);
    assert_eq!(
        event.single_delivery.as_ref().unwrap().recovery,
        bbsim_engine::RecoveryReview::BeforeAnyAdvance {
            runner_id: batter.clone(),
        }
    );
    assert_eq!(
        event
            .delta
            .moves
            .iter()
            .find(|movement| movement.from == Base::Plate)
            .unwrap()
            .to,
        Destination::Out
    );
    assert_credits(event, CreditKind::Putout, &[first_baseman]);
    assert_credits(event, CreditKind::Assist, &[catcher]);
    assert_credits(event, CreditKind::PassedBall, &[]);
    assert_credits(event, CreditKind::WildPitch, &[]);
    assert!(event
        .delta
        .credits
        .iter()
        .filter(|c| c.kind == CreditKind::Putout)
        .all(|c| {
            event
                .delta
                .credits
                .iter()
                .filter(|assist| assist.kind == CreditKind::Assist)
                .all(|assist| assist.player_id != c.player_id)
        }));

    let box_score = derive_accounting(std::slice::from_ref(&transition)).unwrap();
    assert_eq!(box_score.fielders[1][first_baseman].credited_putouts, 1);
    assert_eq!(box_score.fielders[1][catcher].credited_assists, 1);
    assert_eq!(box_score.fielders[1][catcher].credited_passed_balls, 0);
}

#[test]
fn canonical_rejects_a_recovered_before_first_k_with_a_handling_caused_runner_advance() {
    let before = state(Half::Top, 0, 2);
    let saved = before.clone();
    let fielders = fielders(&before);
    let held = project(
        &before,
        &fielders,
        &StrikeoutDecision::RecoveredBeforeFirst {
            handling: HandlingAssessment::PitcherWild,
        },
    )
    .unwrap();
    assert!(held
        .event()
        .single_delivery
        .as_ref()
        .unwrap()
        .advances
        .is_empty());

    let runner = before.bases[1].as_ref().unwrap().clone();
    let batter = before.teams[0].batting_order[0].clone();
    let mut event = held.event().clone();
    event.delta.moves.push(RunnerMove {
        runner: runner.clone(),
        from: Base::Second,
        to: Destination::Third,
    });
    event.delta.credits.push(Credit {
        player_id: fielders[0].clone(),
        kind: CreditKind::WildPitch,
    });
    let facts = event.single_delivery.as_mut().unwrap();
    facts.advances.push(HandlingAdvance {
        runner_id: runner.player_id.clone(),
        reached: Destination::Third,
    });
    facts.recovery = RecoveryReview::AfterAdvance {
        runner_id: batter,
        advancer_id: runner.player_id,
    };

    assert_eq!(
        apply(&before, event).map(|_| ()),
        Err(TransitionError::UnsupportedRule)
    );
    assert_eq!(before, saved);
}

#[test]
fn recovered_third_out_clears_bases_without_scoring() {
    let mut before = state(Half::Top, 2, 4);
    before.inning = 9;
    before.score = [0, 1];
    let runner = before.bases[2].as_ref().unwrap().player_id.clone();
    let fielders = fielders(&before);
    let transition = accepted(
        &before,
        &fielders,
        StrikeoutDecision::RecoveredBeforeFirst {
            handling: HandlingAssessment::PitcherWild,
        },
    );
    let third = transition.event().delta.third_out.as_ref().unwrap();
    assert_eq!(third.kind, ThirdOutKind::BatterBeforeFirst);
    assert!(third.home_before_out.is_empty());
    assert_eq!(transition.event().delta.runs, 0);
    assert_credits(transition.event(), CreditKind::Run, &[]);
    assert!(!transition.event().delta.moves.iter().any(|movement| {
        movement.runner.player_id == runner && movement.to == Destination::Home
    }));
    assert!(transition.after().over);
    assert_eq!(transition.after().score, [0, 1]);
    assert_eq!(transition.after().outs, 0);
    assert_eq!(transition.after().bases, [None, None, None]);
    let box_score = derive_accounting(std::slice::from_ref(&transition)).unwrap();
    assert_eq!(box_score.runs, [0, 0]);
    let batter = &before.teams[0].batting_order[0];
    let pitcher = before.teams[1].pitcher.active_id().unwrap();
    assert_eq!(box_score.batters[0][batter].plate_appearances, 1);
    assert_eq!(box_score.batters[0][batter].credited_strikeouts, 1);
    assert_eq!(box_score.batters[0][batter].at_bats, Some(1));
    assert_eq!(box_score.pitchers[1][pitcher].batters_faced, 1);
    assert_eq!(box_score.pitchers[1][pitcher].outs_recorded, 1);
}

#[test]
fn late_recovered_third_out_keeps_bottom_ninth_tie_alive() {
    let mut before = state(Half::Bottom, 2, 4);
    before.inning = 9;
    let fielders = fielders(&before);
    let transition = accepted(
        &before,
        &fielders,
        StrikeoutDecision::RecoveredBeforeFirst {
            handling: HandlingAssessment::CatcherPassed,
        },
    );
    assert_eq!(transition.event().delta.runs, 0);
    assert_eq!(transition.after().inning, 10);
    assert_eq!(transition.after().half, Half::Top);
    assert_eq!(transition.after().outs, 0);
    assert!(!transition.after().over);
}

#[test]
fn bottom_tenth_safe_reach_can_end_on_the_one_legal_winning_run() {
    let mut before = state(Half::Bottom, 1, 4);
    before.inning = 10;
    let fielders = fielders(&before);
    let transition = accepted(
        &before,
        &fielders,
        StrikeoutDecision::SafeReach {
            handling: HandlingAssessment::PitcherWild,
        },
    );
    assert_eq!(transition.event().delta.runs, 1);
    assert_eq!(transition.after().score, [0, 1]);
    assert!(transition.after().over);
    assert_eq!(transition.after().outs, 1);
}

#[test]
fn ineligible_variants_and_misassigned_fielders_are_refused() {
    let first_occupied = state(Half::Top, 1, 1);
    let fielders = fielders(&first_occupied);
    assert!(project(&first_occupied, &fielders, &StrikeoutDecision::Automatic).is_ok());
    assert!(project(
        &first_occupied,
        &fielders,
        &StrikeoutDecision::SafeReach {
            handling: HandlingAssessment::PitcherWild,
        }
    )
    .is_err());
    assert!(project(
        &first_occupied,
        &fielders,
        &StrikeoutDecision::RecoveredBeforeFirst {
            handling: HandlingAssessment::PitcherWild,
        }
    )
    .is_err());

    let first_empty = state(Half::Top, 0, 0);
    let mut wrong = self::fielders(&first_empty);
    wrong[0] = "not-the-current-pitcher".into();
    assert!(project(
        &first_empty,
        &wrong,
        &StrikeoutDecision::SafeReach {
            handling: HandlingAssessment::PitcherWild,
        }
    )
    .is_err());
}

#[test]
fn legacy_wire_receipt() {
    let before = state(Half::Top, 1, 3);
    let fielders = fielders(&before);
    let automatic = project(&before, &fielders, &StrikeoutDecision::Automatic).unwrap();
    let before = state(Half::Top, 1, 6);
    let fielders = self::fielders(&before);
    let safe = project(
        &before,
        &fielders,
        &StrikeoutDecision::SafeReach {
            handling: HandlingAssessment::PitcherWild,
        },
    )
    .unwrap();
    let held = project(
        &before,
        &fielders,
        &StrikeoutDecision::RecoveredBeforeFirst {
            handling: HandlingAssessment::CatcherPassed,
        },
    )
    .unwrap();
    let expected = [
        "42424532350200000100000000050000000000000061776179300500000000000000617761793105000000000000006177617932050000000000000061776179330500000000000000617761793405000000000000006177617935050000000000000061776179360500000000000000617761793705000000000000006177617938000c00000000000000617761795f70697463686572010800000000000000000000000000000000000500000000000000686f6d65300500000000000000686f6d65310500000000000000686f6d65320500000000000000686f6d65330500000000000000686f6d65340500000000000000686f6d65350500000000000000686f6d65360500000000000000686f6d65370500000000000000686f6d6538000c00000000000000686f6d655f706974636865720108000000000000000000000000000000000001050000000000000061776179350c00000000000000686f6d655f7069746368657201050000000000000061776179360c00000000000000686f6d655f7069746368657200000001050000000000000061776179300c00000000000000686f6d655f70697463686572000100000000000000050000000000000061776179300c00000000000000686f6d655f70697463686572000401000300000000000000050000000000000061776179300005000000000000006177617930010500000000000000686f6d6530080000000000000000000000000000000000000200000200000000050000000000000061776179300500000000000000617761793105000000000000006177617932050000000000000061776179330500000000000000617761793405000000000000006177617935050000000000000061776179360500000000000000617761793705000000000000006177617938010c00000000000000617761795f70697463686572010800000000000000000000000000000000000500000000000000686f6d65300500000000000000686f6d65310500000000000000686f6d65320500000000000000686f6d65330500000000000000686f6d65340500000000000000686f6d65350500000000000000686f6d65360500000000000000686f6d65370500000000000000686f6d6538000c00000000000000686f6d655f706974636865720108000000000000000000000000000000000001050000000000000061776179350c00000000000000686f6d655f7069746368657201050000000000000061776179360c00000000000000686f6d655f706974636865720000",
        "42424532350200000100000000050000000000000061776179300500000000000000617761793105000000000000006177617932050000000000000061776179330500000000000000617761793405000000000000006177617935050000000000000061776179360500000000000000617761793705000000000000006177617938000c00000000000000617761795f70697463686572010800000000000000000000000000000000000500000000000000686f6d65300500000000000000686f6d65310500000000000000686f6d65320500000000000000686f6d65330500000000000000686f6d65340500000000000000686f6d65350500000000000000686f6d65360500000000000000686f6d65370500000000000000686f6d6538000c00000000000000686f6d655f70697463686572010800000000000000000000000000000000000001050000000000000061776179360c00000000000000686f6d655f7069746368657201050000000000000061776179370c00000000000000686f6d655f70697463686572000001050000000000000061776179300c00000000000000686f6d655f70697463686572010300000000000000050000000000000061776179360c00000000000000686f6d655f706974636865720202050000000000000061776179370c00000000000000686f6d655f706974636865720303050000000000000061776179300c00000000000000686f6d655f706974636865720000000104000000000000000500000000000000617761793000050000000000000061776179300105000000000000006177617937040c00000000000000686f6d655f7069746368657217000000000000000000000101000500000000000000686f6d6530030000000000000005000000000000006177617936020500000000000000617761793703050000000000000061776179300001000000000000000001000000000000000200000101000000050000000000000061776179300500000000000000617761793105000000000000006177617932050000000000000061776179330500000000000000617761793405000000000000006177617935050000000000000061776179360500000000000000617761793705000000000000006177617938010c00000000000000617761795f70697463686572010800000000000000000000000000000000000500000000000000686f6d65300500000000000000686f6d65310500000000000000686f6d65320500000000000000686f6d65330500000000000000686f6d65340500000000000000686f6d65350500000000000000686f6d65360500000000000000686f6d65370500000000000000686f6d6538000c00000000000000686f6d655f706974636865720108000000000000000000000000000000000001050000000000000061776179300c00000000000000686f6d655f706974636865720001050000000000000061776179360c00000000000000686f6d655f7069746368657200",
        "42424532350200000100000000050000000000000061776179300500000000000000617761793105000000000000006177617932050000000000000061776179330500000000000000617761793405000000000000006177617935050000000000000061776179360500000000000000617761793705000000000000006177617938000c00000000000000617761795f70697463686572010800000000000000000000000000000000000500000000000000686f6d65300500000000000000686f6d65310500000000000000686f6d65320500000000000000686f6d65330500000000000000686f6d65340500000000000000686f6d65350500000000000000686f6d65360500000000000000686f6d65370500000000000000686f6d6538000c00000000000000686f6d655f70697463686572010800000000000000000000000000000000000001050000000000000061776179360c00000000000000686f6d655f7069746368657201050000000000000061776179370c00000000000000686f6d655f70697463686572000001050000000000000061776179300c00000000000000686f6d655f70697463686572000100000000000000050000000000000061776179300c00000000000000686f6d655f70697463686572000401000400000000000000050000000000000061776179300005000000000000006177617930010500000000000000686f6d6531080500000000000000686f6d653009000000000000000000000101010500000000000000686f6d653000000000000000000100000000000000000205000000000000006177617930000000000000000200000200000000050000000000000061776179300500000000000000617761793105000000000000006177617932050000000000000061776179330500000000000000617761793405000000000000006177617935050000000000000061776179360500000000000000617761793705000000000000006177617938010c00000000000000617761795f70697463686572010800000000000000000000000000000000000500000000000000686f6d65300500000000000000686f6d65310500000000000000686f6d65320500000000000000686f6d65330500000000000000686f6d65340500000000000000686f6d65350500000000000000686f6d65360500000000000000686f6d65370500000000000000686f6d6538000c00000000000000686f6d655f70697463686572010800000000000000000000000000000000000001050000000000000061776179360c00000000000000686f6d655f7069746368657201050000000000000061776179370c00000000000000686f6d655f7069746368657200",
    ];
    for (transition, expected) in [automatic, safe, held].into_iter().zip(expected) {
        let hex: String = transition
            .to_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(hex, expected);
    }
}

fn recovered_advances(before: &GameState, handling: HandlingAssessment) -> EventTransition {
    accepted(
        before,
        &fielders(before),
        StrikeoutDecision::RecoveredBeforeFirstAfterAdvances { handling },
    )
}

#[test]
fn recovered_advances_exhaust_masks_outs_halves_handling_and_accounting() {
    let mut successes = 0;
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..=2 {
            for mask in 0..=7 {
                for handling in [
                    HandlingAssessment::PitcherWild,
                    HandlingAssessment::CatcherPassed,
                ] {
                    let mut before = state(half, outs, mask);
                    for runner in before.bases.iter_mut().flatten() {
                        runner.responsible_pitcher = "inherited-pitcher".into();
                    }
                    let saved = before.clone();
                    let result = project(
                        &before,
                        &fielders(&before),
                        &StrikeoutDecision::RecoveredBeforeFirstAfterAdvances { handling },
                    );
                    assert_eq!(before, saved);
                    if mask == 0 || (mask & 1 != 0 && outs < 2) {
                        assert!(result.is_err(), "ineligible/empty {half:?} {outs} {mask}");
                        continue;
                    }
                    let transition = result.unwrap_or_else(|e| {
                        panic!("{half:?} outs={outs} mask={mask} {handling:?}: {e}")
                    });
                    apply_and_round_trip(&before, &transition);
                    successes += 1;
                    let event = transition.event();
                    let f = fielders(&before);
                    let batter = &before.teams[half.offense()].batting_order[0];
                    let facts = event.single_delivery.as_ref().unwrap();
                    assert_eq!(
                        facts.recovery,
                        RecoveryReview::RecoveredThirdStrikeBatterBeforeFirstAfterAdvances {
                            putout_fielder_id: f[2].clone()
                        }
                    );
                    assert_eq!(facts.advances.len(), mask.count_ones() as usize);
                    assert_eq!(facts.pre_delivery_steals, Some(vec![]));
                    assert_eq!(event.delta.outs, 1);
                    assert_eq!(event.delta.moves.len(), mask.count_ones() as usize + 1);
                    for (index, runner) in before
                        .bases
                        .iter()
                        .enumerate()
                        .filter_map(|(i, r)| r.as_ref().map(|r| (i, r)))
                    {
                        assert!(event.delta.moves.iter().any(|m| m.runner == *runner
                            && m.to
                                == [Destination::Second, Destination::Third, Destination::Home]
                                    [index]));
                    }
                    let home = before.bases[2]
                        .as_ref()
                        .map(|r| r.player_id.clone())
                        .into_iter()
                        .collect::<Vec<_>>();
                    if outs == 2 {
                        let third = event.delta.third_out.as_ref().unwrap();
                        assert_eq!(third.runner_id, *batter);
                        assert_eq!(third.kind, ThirdOutKind::BatterBeforeFirst);
                        assert_eq!(third.home_before_out, home);
                        assert_eq!(event.delta.runs, 0);
                        assert_credits(event, CreditKind::Run, &[]);
                        assert_eq!(transition.after().bases, [None, None, None]);
                        assert_eq!(transition.after().outs, 0);
                        assert_ne!(transition.after().half, half);
                    } else {
                        assert_eq!(event.delta.runs, u8::from(mask & 4 != 0));
                        assert_eq!(transition.after().outs, outs + 1);
                        assert_eq!(transition.after().half, half);
                    }
                    assert_eq!(transition.after().teams[half.offense()].next_batter, 1);
                    assert_eq!(transition.after().teams[1 - half.offense()].next_batter, 0);
                    assert_credits(event, CreditKind::Putout, &[&f[2]]);
                    assert_credits(event, CreditKind::Assist, &[&f[1]]);
                    assert_credits(event, CreditKind::StolenBase, &[]);
                    assert_credits(event, CreditKind::Rbi, &[]);
                    assert_credits(
                        event,
                        CreditKind::WildPitch,
                        &if handling == HandlingAssessment::PitcherWild {
                            vec![f[0].as_str()]
                        } else {
                            vec![]
                        },
                    );
                    assert_credits(
                        event,
                        CreditKind::PassedBall,
                        &if handling == HandlingAssessment::CatcherPassed {
                            vec![f[1].as_str()]
                        } else {
                            vec![]
                        },
                    );
                    let accounting = derive_accounting(std::slice::from_ref(&transition)).unwrap();
                    let batting = &accounting.batters[half.offense()][batter];
                    assert_eq!(batting.plate_appearances, 1);
                    assert_eq!(batting.at_bats, Some(1));
                    assert_eq!(batting.credited_strikeouts, 1);
                    let pitching = &accounting.pitchers[1 - half.offense()][&f[0]];
                    assert_eq!(pitching.batters_faced, 1);
                    assert_eq!(pitching.outs_recorded, 1);
                    assert_eq!(pitching.credited_strikeouts, 1);
                    assert_eq!(&transition.to_bytes()[..5], b"BBE32");
                }
            }
        }
    }
    assert_eq!(successes, 52);
}

#[test]
fn recovered_advance_endings_and_winning_stop_refusal() {
    for inning in [9, 10, 12] {
        for handling in [
            HandlingAssessment::PitcherWild,
            HandlingAssessment::CatcherPassed,
        ] {
            let mut before = state(Half::Top, 2, 4);
            before.inning = inning;
            before.score = [0, 1];
            let t = recovered_advances(&before, handling);
            assert!(t.after().over);
            assert_eq!(t.after().score, [0, 1]);
            before = GameState {
                inning,
                score: [0, 0],
                ..state(Half::Bottom, 2, 4)
            };
            let t = recovered_advances(&before, handling);
            assert!(!t.after().over);
            assert_eq!(t.after().inning, inning + 1);
            assert_eq!(t.after().half, Half::Top);
            for outs in [0, 1] {
                let before = GameState {
                    inning,
                    ..state(Half::Bottom, outs, 4)
                };
                let saved = before.clone();
                assert!(project(
                    &before,
                    &fielders(&before),
                    &StrikeoutDecision::RecoveredBeforeFirstAfterAdvances { handling }
                )
                .is_err());
                assert_eq!(before, saved);
                // One run only ties, or leaves the home side behind: recovery remains legal.
                for score in [[1, 0], [2, 0]] {
                    let before = GameState {
                        score,
                        ..before.clone()
                    };
                    let t = recovered_advances(&before, handling);
                    assert!(!t.after().over);
                    assert_eq!(t.after().outs, outs + 1);
                }
                // With no Home crossing a late-inning recovery remains legal.
                let before = GameState {
                    inning,
                    ..state(Half::Bottom, outs, 2)
                };
                assert!(!recovered_advances(&before, handling).after().over);
            }
        }
    }
}

fn refuse_event(before: &GameState, event: Event, after: &GameState) -> TransitionError {
    let saved = before.clone();
    let error = apply(before, event.clone()).unwrap_err();
    assert!(EventTransition::try_new(before.clone(), event, after.clone()).is_err());
    assert_eq!(*before, saved);
    error
}

#[test]
fn recovered_advance_rejects_malformed_actors_moves_credits_and_chronology() {
    for outs in [0, 2] {
        for handling in [
            HandlingAssessment::PitcherWild,
            HandlingAssessment::CatcherPassed,
        ] {
            let before = state(Half::Top, outs, 6);
            let t = recovered_advances(&before, handling);
            let f = fielders(&before);
            for mutation in 0..24 {
                let mut e = t.event().clone();
                match mutation {
                    0 => e.single_delivery.as_mut().unwrap().advances.clear(),
                    1 => {
                        let a = e.single_delivery.as_ref().unwrap().advances[0].clone();
                        e.single_delivery.as_mut().unwrap().advances.push(a);
                    }
                    2 => {
                        e.single_delivery.as_mut().unwrap().advances[0].runner_id =
                            "invented".into()
                    }
                    3 => {
                        e.single_delivery.as_mut().unwrap().advances[0].reached = Destination::Home
                    }
                    4 => {
                        e.single_delivery.as_mut().unwrap().advances[0].runner_id =
                            e.participants.batter.clone().unwrap()
                    }
                    5 => e.single_delivery.as_mut().unwrap().catcher_id = f[0].clone(),
                    6 => e.single_delivery.as_mut().unwrap().catcher_id = "outsider".into(),
                    7 => {
                        e.single_delivery.as_mut().unwrap().recovery =
                            RecoveryReview::RecoveredThirdStrikeBatterBeforeFirstAfterAdvances {
                                putout_fielder_id: f[1].clone(),
                            }
                    }
                    8 => {
                        e.single_delivery.as_mut().unwrap().recovery =
                            RecoveryReview::RecoveredThirdStrikeBatterBeforeFirstAfterAdvances {
                                putout_fielder_id: "outsider".into(),
                            }
                    }
                    9 => e.delta.credits.retain(|c| c.kind != CreditKind::Assist),
                    10 => e.delta.credits.retain(|c| c.kind != CreditKind::Putout),
                    11 => {
                        e.delta
                            .credits
                            .iter_mut()
                            .find(|c| c.kind == CreditKind::Assist)
                            .unwrap()
                            .player_id = f[2].clone()
                    }
                    12 => e.delta.credits.push(Credit {
                        player_id: f[3].clone(),
                        kind: CreditKind::Putout,
                    }),
                    13 => e.delta.credits.retain(|c| {
                        !matches!(c.kind, CreditKind::WildPitch | CreditKind::PassedBall)
                    }),
                    14 => e.delta.credits.push(Credit {
                        player_id: before.bases[1].as_ref().unwrap().player_id.clone(),
                        kind: CreditKind::StolenBase,
                    }),
                    15 => e.delta.moves[0].runner.responsible_pitcher = "forged-charge".into(),
                    16 => e.delta.moves[0].to = Destination::Out,
                    17 => e.delta.moves[0].to = Destination::Second,
                    18 => e
                        .single_delivery
                        .as_mut()
                        .unwrap()
                        .advances
                        .remove(0)
                        .runner_id
                        .clear(),
                    19 => {
                        e.delta
                            .moves
                            .iter_mut()
                            .find(|m| m.from == Base::Plate)
                            .unwrap()
                            .to = Destination::First
                    }
                    20 => e.participants.batter = Some(before.teams[0].batting_order[1].clone()),
                    21 => {
                        e.single_delivery.as_mut().unwrap().context =
                            bbsim_engine::DeliveryContext::RunnerOnly
                    }
                    22 => {
                        if outs == 2 {
                            e.delta.third_out.as_mut().unwrap().home_before_out.clear();
                        } else {
                            e.delta.outs = 2;
                        }
                    }
                    23 => {
                        if outs == 2 {
                            e.delta.third_out.as_mut().unwrap().kind =
                                ThirdOutKind::BatterAfterFirstTiming;
                        } else {
                            e.delta.credits.push(Credit {
                                player_id: e.participants.batter.clone().unwrap(),
                                kind: CreditKind::Rbi,
                            });
                        }
                    }
                    _ => unreachable!(),
                }
                let saved = before.clone();
                assert!(
                    apply(&before, e.clone()).is_err(),
                    "accepted mutation {mutation}, outs={outs}, {handling:?}"
                );
                assert!(EventTransition::try_new(before.clone(), e, t.after().clone()).is_err());
                assert_eq!(before, saved);
            }
        }
    }
}

#[test]
fn recovered_advance_known_errors_precede_missing_steal_or_winning_history() {
    let before = state(Half::Top, 0, 2);
    let t = recovered_advances(&before, HandlingAssessment::PitcherWild);
    let mut missing = t.event().clone();
    missing
        .single_delivery
        .as_mut()
        .unwrap()
        .pre_delivery_steals = None;
    assert_eq!(
        refuse_event(&before, missing.clone(), t.after()),
        TransitionError::UnsupportedRule
    );
    missing.single_delivery.as_mut().unwrap().catcher_id = "foreign".into();
    assert_eq!(
        refuse_event(&before, missing, t.after()),
        TransitionError::IllegalCredit
    );
    let before = GameState {
        inning: 9,
        ..state(Half::Bottom, 0, 4)
    };
    // Obtain the event from a nonterminal inning rather than asking the helper to repair it.
    let earlier = GameState {
        inning: 2,
        ..before.clone()
    };
    let t = recovered_advances(&earlier, HandlingAssessment::PitcherWild);
    assert_eq!(
        refuse_event(&before, t.event().clone(), t.after()),
        TransitionError::UnsupportedRule
    );
    let mut e = t.event().clone();
    e.delta.credits.retain(|c| c.kind != CreditKind::WildPitch);
    assert_eq!(
        refuse_event(&before, e, t.after()),
        TransitionError::IllegalCredit
    );
}

#[test]
fn recovered_advance_codec_rejects_header_forgeries() {
    let before = state(Half::Top, 0, 2);
    let t = recovered_advances(&before, HandlingAssessment::PitcherWild);
    for version in 25..=31 {
        let mut bytes = t.to_bytes();
        bytes[..5].copy_from_slice(format!("BBE{version}").as_bytes());
        assert_eq!(
            EventTransition::from_bytes(&bytes),
            Err(TransitionError::InvalidEncoding)
        );
    }
    let old = project(
        &before,
        &fielders(&before),
        &StrikeoutDecision::RecoveredBeforeFirst {
            handling: HandlingAssessment::PitcherWild,
        },
    )
    .unwrap();
    let mut bytes = old.to_bytes();
    bytes[..5].copy_from_slice(b"BBE32");
    assert_eq!(
        EventTransition::from_bytes(&bytes),
        Err(TransitionError::InvalidEncoding)
    );
}

fn supplied_recovered_advance_event(before: &GameState) -> Event {
    let f = fielders(before);
    let mut e = project(
        before,
        &f,
        &StrikeoutDecision::RecoveredBeforeFirst {
            handling: HandlingAssessment::PitcherWild,
        },
    )
    .unwrap()
    .event()
    .clone();
    let runner = before.bases[1].as_ref().unwrap().clone();
    e.delta.moves.push(RunnerMove {
        runner: runner.clone(),
        from: Base::Second,
        to: Destination::Third,
    });
    e.delta.credits.push(Credit {
        player_id: f[0].clone(),
        kind: CreditKind::WildPitch,
    });
    let facts = e.single_delivery.as_mut().unwrap();
    facts.advances = vec![HandlingAdvance {
        runner_id: runner.player_id,
        reached: Destination::Third,
    }];
    facts.recovery = RecoveryReview::RecoveredThirdStrikeBatterBeforeFirstAfterAdvances {
        putout_fielder_id: f[2].clone(),
    };
    e
}

#[test]
fn canonical_recovered_advance_leaf_supplies_the_previously_missing_before_first_proof() {
    let before = state(Half::Top, 0, 2);
    let t = apply(&before, supplied_recovered_advance_event(&before)).unwrap();
    apply_and_round_trip(&before, &t);
    assert_eq!(t.after().outs, 1);
    assert_eq!(t.after().bases[2], before.bases[1]);
}

#[test]
fn canonical_recovered_advance_leaf_refuses_empty_and_held_only_claims() {
    let before = state(Half::Top, 0, 2);
    let mut e = supplied_recovered_advance_event(&before);
    e.single_delivery.as_mut().unwrap().advances.clear();
    e.delta.moves.retain(|m| m.from == Base::Plate);
    e.delta.credits.retain(|c| c.kind != CreditKind::WildPitch);
    assert!(apply(&before, e.clone()).is_err());
    e.single_delivery
        .as_mut()
        .unwrap()
        .advances
        .push(HandlingAdvance {
            runner_id: before.bases[1].as_ref().unwrap().player_id.clone(),
            reached: Destination::Third,
        });
    assert!(apply(&before, e).is_err());
}

#[test]
fn recovered_advance_partial_runner_vector_and_explicit_steal_review_use_native_qualification() {
    let before = state(Half::Top, 0, 6);
    let t = recovered_advances(&before, HandlingAssessment::PitcherWild);
    // Only Third scores; Second holds. No implied movement or inferred steal.
    let mut e = t.event().clone();
    let held = before.bases[1].as_ref().unwrap();
    e.delta
        .moves
        .retain(|m| m.runner.player_id != held.player_id);
    e.single_delivery
        .as_mut()
        .unwrap()
        .advances
        .retain(|a| a.runner_id != held.player_id);
    let t = apply(&before, e).unwrap();
    apply_and_round_trip(&before, &t);
    assert_eq!(t.after().bases[1], before.bases[1]);
    assert_eq!(t.event().delta.runs, 1);

    let before = state(Half::Top, 0, 2);
    let mut e = supplied_recovered_advance_event(&before);
    let id = before.bases[1].as_ref().unwrap().player_id.clone();
    e.single_delivery.as_mut().unwrap().pre_delivery_steals =
        Some(vec![bbsim_engine::PreDeliverySteal {
            runner_id: id,
            target: Destination::Third,
        }]);
    // Reaching only the pre-delivery steal target does not qualify a WP unit.
    assert_eq!(
        apply(&before, e.clone()),
        Err(TransitionError::IllegalCredit)
    );
    e.delta.credits.retain(|c| c.kind != CreditKind::WildPitch);
    let t = apply(&before, e).unwrap();
    apply_and_round_trip(&before, &t);
    assert_credits(t.event(), CreditKind::StolenBase, &[]);

    // A longer explicit reach qualifies even without steal-start review.
    let mut e = supplied_recovered_advance_event(&before);
    e.delta
        .moves
        .iter_mut()
        .find(|m| m.from == Base::Second)
        .unwrap()
        .to = Destination::Home;
    e.single_delivery.as_mut().unwrap().advances[0].reached = Destination::Home;
    e.single_delivery.as_mut().unwrap().pre_delivery_steals = None;
    e.delta.runs = 1;
    e.delta.credits.push(Credit {
        player_id: before.bases[1].as_ref().unwrap().player_id.clone(),
        kind: CreditKind::Run,
    });
    let t = apply(&before, e).unwrap();
    apply_and_round_trip(&before, &t);
}

#[test]
fn recovered_advance_cannot_score_or_cross_home_past_an_omitted_held_leader() {
    fn string_bytes(text: &str) -> Vec<u8> {
        let mut b = (text.len() as u64).to_le_bytes().to_vec();
        b.extend_from_slice(text.as_bytes());
        b
    }
    fn replace_once(bytes: &mut [u8], old: &[u8], new: &[u8]) {
        assert_eq!(old.len(), new.len());
        let offsets: Vec<_> = bytes
            .windows(old.len())
            .enumerate()
            .filter_map(|(i, b)| (b == old).then_some(i))
            .collect();
        assert_eq!(offsets.len(), 1);
        bytes[offsets[0]..offsets[0] + old.len()].copy_from_slice(new);
    }
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            for reverse in [false, true] {
                let before = state(half, outs, 6);
                let saved = before.clone();
                let second = before.bases[1].as_ref().unwrap();
                let third = before.bases[2].as_ref().unwrap();
                let mut legal = recovered_advances(&before, HandlingAssessment::PitcherWild)
                    .event()
                    .clone();
                legal.delta.moves.retain(|m| m.from != Base::Second);
                legal
                    .single_delivery
                    .as_mut()
                    .unwrap()
                    .advances
                    .retain(|a| a.runner_id != second.player_id);
                if reverse {
                    legal.delta.moves.reverse();
                    legal.delta.credits.reverse();
                }
                let legal = apply(&before, legal).unwrap();
                let mut illegal = legal.event().clone();
                let movement = illegal
                    .delta
                    .moves
                    .iter_mut()
                    .find(|m| m.from == Base::Third)
                    .unwrap();
                movement.from = Base::Second;
                movement.runner = second.clone();
                illegal.single_delivery.as_mut().unwrap().advances[0].runner_id =
                    second.player_id.clone();
                for c in &mut illegal.delta.credits {
                    if c.kind == CreditKind::Run {
                        c.player_id = second.player_id.clone();
                    }
                }
                if let Some(t) = &mut illegal.delta.third_out {
                    t.home_before_out = vec![second.player_id.clone()];
                }
                let mut after = legal.after().clone();
                if outs < 2 {
                    after.bases[1] = None;
                    after.bases[2] = Some(third.clone());
                }
                assert_eq!(
                    apply(&before, illegal.clone()),
                    Err(TransitionError::UnsupportedRule)
                );
                assert_eq!(
                    EventTransition::try_new(before.clone(), illegal, after),
                    Err(TransitionError::UnsupportedRule)
                );
                assert_eq!(before, saved);
                if outs == 2 {
                    let mut bytes = legal.to_bytes();
                    assert_eq!(&bytes[..5], b"BBE32");
                    let mut old = string_bytes(&third.player_id);
                    old.extend(string_bytes(&third.responsible_pitcher));
                    old.extend([3, 3]);
                    let mut new = string_bytes(&second.player_id);
                    new.extend(string_bytes(&second.responsible_pitcher));
                    new.extend([2, 3]);
                    replace_once(&mut bytes, &old, &new);
                    let mut old = string_bytes(&third.player_id);
                    old.push(3);
                    let mut new = string_bytes(&second.player_id);
                    new.push(3);
                    replace_once(&mut bytes, &old, &new);
                    let batter = legal.event().participants.batter.as_ref().unwrap();
                    let mut prefix = vec![1];
                    prefix.extend(string_bytes(batter));
                    prefix.push(0);
                    prefix.extend(1u64.to_le_bytes());
                    let mut old = prefix.clone();
                    old.extend(string_bytes(&third.player_id));
                    let mut new = prefix;
                    new.extend(string_bytes(&second.player_id));
                    replace_once(&mut bytes, &old, &new);
                    // Before-First third out suppresses both scores and clears after-bases;
                    // only the supplied passing chronology is now false.
                    assert_eq!(
                        EventTransition::from_bytes(&bytes),
                        Err(TransitionError::UnsupportedRule)
                    );
                }
            }
        }
    }
}

#[test]
fn recovered_advance_preserves_inherited_run_charge_without_new_er_rbi_or_fielding_certainty() {
    for handling in [
        HandlingAssessment::PitcherWild,
        HandlingAssessment::CatcherPassed,
    ] {
        let mut before = state(Half::Top, 0, 4);
        before.bases[2].as_mut().unwrap().responsible_pitcher = "previous-pitcher".into();
        let t = recovered_advances(&before, handling);
        let a = derive_accounting(&[t]).unwrap();
        let current = before.teams[1].pitcher.active_id().unwrap();
        assert_eq!(a.pitchers[1][current].runs_allowed, 0);
        assert_eq!(a.pitchers[1][current].batters_faced, 1);
        assert_eq!(a.pitchers[1]["previous-pitcher"].runs_allowed, 1);
        assert_eq!(a.pitchers[1]["previous-pitcher"].earned_runs, None);
        assert_eq!(
            a.batters[0][&before.teams[0].batting_order[0]].classified_rbi,
            None
        );
        let f = fielders(&before);
        assert_eq!(a.classified_fielding[1][&f[2]].putouts, None);
        assert_eq!(a.classified_fielding[1][&f[1]].assists, None);
    }
}

#[test]
fn recovered_advance_rejects_unrelated_causes_duplicate_moves_and_false_third_out_runs() {
    for outs in [0, 2] {
        let before = state(Half::Top, outs, 6);
        let t = recovered_advances(&before, HandlingAssessment::PitcherWild);
        for mutation in 0..8 {
            let mut e = t.event().clone();
            match mutation {
                0 => {
                    let m = e.delta.moves[0].clone();
                    e.delta.moves.push(m);
                }
                1 => e.delta.moves[0].runner.player_id = "not-a-before-runner".into(),
                2 => e.delta.credits.push(Credit {
                    player_id: e.participants.batter.clone().unwrap(),
                    kind: CreditKind::Walk,
                }),
                3 => e.delta.credits.push(Credit {
                    player_id: fielders(&before)[1].clone(),
                    kind: CreditKind::Error,
                }),
                4 => {
                    e.delta.credits.push(Credit {
                        player_id: before.bases[2].as_ref().unwrap().player_id.clone(),
                        kind: CreditKind::Run,
                    });
                    e.delta.runs += 1;
                }
                5 => {
                    e.balk = Some(bbsim_engine::BalkFacts {
                        cause: bbsim_engine::BalkCause::PitcherAction602a,
                        disposition: bbsim_engine::BalkDisposition::Unknown,
                        resolution: bbsim_engine::BalkResolution::Unknown,
                    })
                }
                6 => {
                    if let Some(third) = e.delta.third_out.as_mut() {
                        third.home_before_out.push("foreign".into());
                    } else {
                        e.single_delivery.as_mut().unwrap().pre_delivery_steals = None;
                        e.single_delivery.as_mut().unwrap().advances.clear();
                    }
                }
                7 => {
                    if let Some(third) = e.delta.third_out.as_mut() {
                        third.runner_id = before.bases[2].as_ref().unwrap().player_id.clone();
                    } else {
                        e.single_delivery.as_mut().unwrap().pre_delivery_steals = None;
                        e.delta.credits.push(Credit {
                            player_id: fielders(&before)[2].clone(),
                            kind: CreditKind::Assist,
                        });
                    }
                }
                _ => unreachable!(),
            }
            assert!(
                apply(&before, e.clone()).is_err(),
                "accepted unrelated mutation {mutation} outs={outs}"
            );
            assert!(EventTransition::try_new(before.clone(), e, t.after().clone()).is_err());
        }
    }
}

#[test]
fn recovered_advance_codec_revalidates_named_receiver_and_absent_leaf_headers() {
    let before = state(Half::Top, 0, 2);
    let t = recovered_advances(&before, HandlingAssessment::PitcherWild);
    let receiver = fielders(&before)[2].clone();
    let mut needle = vec![6]; // Recovery tag, then the actual length-prefixed receiver.
    needle.extend((receiver.len() as u64).to_le_bytes());
    needle.extend(receiver.as_bytes());
    let mut bytes = t.to_bytes();
    let matches = bytes
        .windows(needle.len())
        .enumerate()
        .filter_map(|(i, b)| (b == needle).then_some(i))
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1);
    let index = matches[0] + 9;
    bytes[index..index + receiver.len()].copy_from_slice(fielders(&before)[1].as_bytes());
    assert_eq!(
        EventTransition::from_bytes(&bytes),
        Err(TransitionError::IllegalCredit)
    );
    let before = state(Half::Top, 0, 1);
    let automatic = project(&before, &fielders(&before), &StrikeoutDecision::Automatic).unwrap();
    let mut bytes = automatic.to_bytes();
    bytes[..5].copy_from_slice(b"BBE32");
    assert_eq!(
        EventTransition::from_bytes(&bytes),
        Err(TransitionError::InvalidEncoding)
    );
}

#[test]
fn canonical_recovered_advance_eligibility_uses_before_first_for_every_mask_and_half() {
    for half in [Half::Top, Half::Bottom] {
        for mask in 1..=7 {
            for handling in [
                HandlingAssessment::PitcherWild,
                HandlingAssessment::CatcherPassed,
            ] {
                let template = state(half, 2, mask);
                let t = recovered_advances(&template, handling);
                for outs in 0..=2 {
                    let before = GameState {
                        outs,
                        ..template.clone()
                    };
                    let mut e = t.event().clone();
                    if outs < 2 {
                        e.delta.third_out = None;
                        if let Some(runner) = &before.bases[2] {
                            e.delta.runs = 1;
                            e.delta.credits.push(Credit {
                                player_id: runner.player_id.clone(),
                                kind: CreditKind::Run,
                            });
                        }
                    }
                    let saved = before.clone();
                    let result = apply(&before, e);
                    assert_eq!(before, saved);
                    if mask & 1 != 0 && outs < 2 {
                        assert_eq!(
                            result,
                            Err(TransitionError::IllegalCredit),
                            "{half:?} mask={mask} outs={outs}"
                        );
                    } else {
                        apply_and_round_trip(&before, &result.unwrap());
                    }
                }
            }
        }
    }
}
