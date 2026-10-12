use bbsim_engine::accounting::{derive_accounting, HitBreakdown};
use bbsim_engine::{
    apply, Base, Credit, CreditKind, Delta, Destination, Event, EventKind, EventTransition,
    FieldingReview, GameState, Half, Participants, PersonnelChange, Runner, RunnerEvent,
    RunnerMove, TeamSide, TeamState, ThirdOut, ThirdOutKind, TransitionError,
};

fn state(mask: u8, outs: u8, half: Half) -> GameState {
    let teams = std::array::from_fn(|side| {
        let prefix = ["away", "home"][side];
        TeamState {
            batting_order: std::array::from_fn(|slot| format!("{prefix}{slot}")),
            next_batter: 0,
            pitcher: format!("{prefix}_pitcher"),
            dh_slot: Some(8),
            dh_must_bat: false,
            bench: vec![format!("{prefix}_relief")],
            removed: vec![],
        }
    });
    let offense = half.offense();
    GameState {
        inning: 2,
        half,
        outs,
        score: [0, 0],
        bases: std::array::from_fn(|base| {
            (mask & (1 << base) != 0).then(|| Runner {
                player_id: teams[offense].batting_order[5 + base].clone(),
                responsible_pitcher: teams[1 - offense].pitcher.clone(),
            })
        }),
        teams,
        over: false,
    }
}

fn credit(id: &str, kind: CreditKind) -> Credit {
    Credit {
        player_id: id.into(),
        kind,
    }
}

fn hit(before: &GameState, to: Destination, class: Option<CreditKind>) -> Event {
    let offense = before.half.offense();
    let batter = &before.teams[offense].batting_order[before.teams[offense].next_batter as usize];
    let pitcher = &before.teams[1 - offense].pitcher;
    let mut moves = vec![RunnerMove {
        runner: Runner {
            player_id: batter.clone(),
            responsible_pitcher: pitcher.clone(),
        },
        from: Base::Plate,
        to,
    }];
    let mut credits = vec![
        credit(batter, CreditKind::PlateAppearance),
        credit(batter, CreditKind::Hit),
    ];
    for (base, runner) in before.bases.iter().enumerate() {
        if let Some(runner) = runner {
            moves.push(RunnerMove {
                runner: runner.clone(),
                from: [Base::First, Base::Second, Base::Third][base],
                to: Destination::Home,
            });
            credits.push(credit(&runner.player_id, CreditKind::Run));
        }
    }
    if to == Destination::Home {
        credits.push(credit(batter, CreditKind::Run));
    }
    if let Some(class) = class {
        credits.push(credit(batter, class));
    }
    Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(batter.clone()),
            pitcher: pitcher.clone(),
        },
        fielding_review: FieldingReview::NoMisplay,
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        ordinary_ground_throw: None,
        delta: Delta {
            runs: credits.iter().filter(|c| c.kind == CreditKind::Run).count() as u8,
            moves,
            outs: 0,
            credits,
            changes: vec![],
            third_out: None,
        },
    }
}

fn breakdown(kind: CreditKind) -> HitBreakdown {
    HitBreakdown {
        singles: u32::from(kind == CreditKind::Single),
        doubles: u32::from(kind == CreditKind::Double),
        triples: u32::from(kind == CreditKind::Triple),
        home_runs: 0,
    }
}

fn strip_types(mut event: Event) -> Event {
    event.delta.credits.retain(|c| {
        !matches!(
            c.kind,
            CreditKind::Single | CreditKind::Double | CreditKind::Triple
        )
    });
    event
}

fn rejected(before: &GameState, event: Event, error: TransitionError) {
    let unchanged = before.clone();
    let valid = apply(before, strip_types(event.clone())).unwrap();
    assert_eq!(apply(before, event.clone()), Err(error.clone()));
    assert_eq!(*before, unchanged);
    assert_eq!(
        EventTransition::try_new(before.clone(), event, valid.after().clone()),
        Err(error)
    );
}

fn inherited_replacement_cases() -> Vec<(GameState, Event)> {
    let mut cases = vec![];
    for half in [Half::Top, Half::Bottom] {
        for base in 0..3 {
            for outs in 0..3 {
                for reverse in [false, true] {
                    let mut before = state(1 << base, outs, half);
                    before.bases[base].as_mut().unwrap().responsible_pitcher =
                        "synthetic_prior_pitcher".into();
                    let mut event = hit(&before, Destination::First, None);
                    event.delta.moves[1].to = Destination::Out;
                    event.delta.outs = 1;
                    event.delta.runs = 0;
                    event.delta.credits.retain(|c| c.kind != CreditKind::Run);
                    event.delta.credits.push(credit(
                        event.participants.batter.as_ref().unwrap(),
                        CreditKind::AtBat,
                    ));
                    if outs == 2 {
                        event.delta.third_out = Some(ThirdOut {
                            runner_id: before.bases[base].as_ref().unwrap().player_id.clone(),
                            kind: ThirdOutKind::Timing,
                            home_before_out: vec![],
                        });
                    }
                    if reverse {
                        event.delta.moves.reverse();
                        event.delta.credits.reverse();
                    }
                    cases.push((before, event));
                }
            }
        }
    }
    assert_eq!(cases.len(), 36);
    cases
}

fn missing_hit(mut event: Event) -> Event {
    let hit = event
        .delta
        .credits
        .iter_mut()
        .find(|c| c.kind == CreditKind::Hit)
        .unwrap();
    hit.kind = CreditKind::Single;
    event
}

#[test]
fn reducer_reports_missing_hit_before_inherited_responsibility_ambiguity() {
    for (before, event) in inherited_replacement_cases() {
        let unchanged = before.clone();
        let reference = apply(&before, event.clone()).unwrap();
        assert_eq!(
            apply(&before, missing_hit(event.clone())),
            Err(TransitionError::IllegalCredit)
        );
        assert_eq!(before, unchanged);
        let mut unclassified = event.clone();
        unclassified
            .delta
            .credits
            .retain(|c| c.kind != CreditKind::Hit);
        if before.outs < 2 {
            assert_eq!(
                apply(&before, unclassified),
                Err(TransitionError::UnsupportedRule)
            );
        } else {
            assert!(apply(&before, unclassified).is_ok());
        }
        let mut typed_retirement = event;
        typed_retirement.delta.credits.push(credit(
            typed_retirement.participants.batter.as_ref().unwrap(),
            CreditKind::Single,
        ));
        assert_eq!(
            apply(&before, typed_retirement),
            Err(TransitionError::UnsupportedRule)
        );
        let lines = derive_accounting(std::slice::from_ref(&reference)).unwrap();
        let side = before.half.offense();
        assert_eq!(
            lines.batters[side][reference.event().participants.batter.as_ref().unwrap()]
                .credited_hits,
            1
        );
        assert_eq!(
            lines.pitchers[1 - side][&reference.event().participants.pitcher]
                .credited_hit_breakdown_allowed,
            None
        );
    }
}

#[test]
fn constructor_reports_missing_hit_before_inherited_responsibility_ambiguity() {
    for (before, event) in inherited_replacement_cases() {
        let valid = apply(&before, event.clone()).unwrap();
        assert_eq!(
            EventTransition::try_new(before, missing_hit(event), valid.after().clone()),
            Err(TransitionError::IllegalCredit)
        );
    }
}

#[test]
fn codec_reports_parsed_missing_hit_before_inherited_responsibility_ambiguity() {
    for (before, event) in inherited_replacement_cases() {
        let reference = apply(&before, event).unwrap();
        let original = reference.to_bytes();
        assert_eq!(EventTransition::from_bytes(&original).unwrap(), reference);
        let batter = reference.event().participants.batter.as_ref().unwrap();
        let mut marker = (batter.len() as u64).to_le_bytes().to_vec();
        marker.extend_from_slice(batter.as_bytes());
        marker.push(3); // Unique Hit credit unit; change only its kind to Single.
        let offset = unique_offset(&original, &marker) + 8 + batter.len();
        let mut bytes = original.clone();
        bytes[offset] = 17;
        assert_eq!(
            EventTransition::from_bytes(&bytes),
            Err(TransitionError::IllegalCredit)
        );
        assert_eq!(&bytes[..offset], &original[..offset]);
        assert_eq!(&bytes[offset + 1..], &original[offset + 1..]);
    }
}

fn strikeout(before: &GameState) -> Event {
    let offense = before.half.offense();
    let batter = &before.teams[offense].batting_order[before.teams[offense].next_batter as usize];
    let pitcher = &before.teams[1 - offense].pitcher;
    Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(batter.clone()),
            pitcher: pitcher.clone(),
        },
        fielding_review: FieldingReview::NoMisplay,
        two_out_contact: None,
        single_delivery: None,
        balk: None,
        caught_strikeout: None,
        protected_award: None,
        sole_home_run: None,
        sole_safe_hit: None,
        ordinary_ground_throw: None,
        delta: Delta {
            moves: vec![RunnerMove {
                runner: Runner {
                    player_id: batter.clone(),
                    responsible_pitcher: pitcher.clone(),
                },
                from: Base::Plate,
                to: Destination::Out,
            }],
            outs: 1,
            runs: 0,
            credits: vec![
                credit(batter, CreditKind::PlateAppearance),
                credit(batter, CreditKind::Strikeout),
                credit(
                    &before.teams[1 - offense].batting_order[3],
                    CreditKind::Putout,
                ),
            ],
            changes: vec![],
            third_out: (before.outs == 2).then(|| ThirdOut {
                runner_id: batter.clone(),
                kind: ThirdOutKind::BatterBeforeFirst,
                home_before_out: vec![],
            }),
        },
    }
}

#[test]
fn runner_only_events_cannot_receive_batting_hit_modifiers() {
    for half in [Half::Top, Half::Bottom] {
        for kind in [
            RunnerEvent::WildPitch,
            RunnerEvent::PassedBall,
            RunnerEvent::StealAttempt,
            RunnerEvent::Pickoff,
            RunnerEvent::Balk,
        ] {
            let before = state(1, 0, half);
            let mut event = hit(&before, Destination::First, None);
            event.kind = EventKind::RunnerOnly(kind);
            event.fielding_review = FieldingReview::Unknown;
            event.delta.moves = vec![RunnerMove {
                runner: before.bases[0].clone().unwrap(),
                from: Base::First,
                to: Destination::Second,
            }];
            event.delta.credits.clear();
            event.delta.runs = 0;
            for class in [CreditKind::Single, CreditKind::Double, CreditKind::Triple] {
                let mut invalid = event.clone();
                invalid
                    .delta
                    .credits
                    .push(credit(invalid.participants.batter.as_ref().unwrap(), class));
                rejected(&before, invalid, TransitionError::IllegalCredit);
            }
        }
    }
}

#[test]
fn physical_complete_game_and_no_misplay_do_not_classify_an_omitted_hit_value() {
    let mut before = state(0, 0, Half::Top);
    before.inning = 1;
    let mut log = vec![];
    let first = apply(&before, hit(&before, Destination::First, None)).unwrap();
    before = first.after().clone();
    log.push(first);
    let mut home_hr = false;
    while !before.over {
        let event = if before.inning == 1 && before.half == Half::Bottom && !home_hr {
            home_hr = true;
            let mut event = hit(&before, Destination::Home, None);
            event.kind = EventKind::OutOfParkHomeRun;
            event
        } else {
            strikeout(&before)
        };
        let transition = apply(&before, event).unwrap();
        before = transition.after().clone();
        log.push(transition);
        assert!(log.len() <= 53);
    }
    assert_eq!(log.len(), 53);
    let lines = derive_accounting(&log).unwrap();
    assert!(lines.complete_game);
    assert_eq!(lines.innings[&9], [Some(0), None]);
    assert_eq!(lines.runs, [0, 1]);
    assert_eq!(lines.batters[0]["away0"].credited_hit_breakdown, None);
    assert_eq!(
        lines.batters[0]["away1"].credited_hit_breakdown,
        Some(HitBreakdown::default())
    );
    assert_eq!(
        lines.pitchers[1]["home_pitcher"].credited_hit_breakdown_allowed,
        None
    );
    assert_eq!(
        lines.batters[1]["home0"].credited_hit_breakdown,
        Some(HitBreakdown {
            home_runs: 1,
            ..HitBreakdown::default()
        })
    );
    assert_eq!(
        lines.pitchers[0]["away_pitcher"].credited_hit_breakdown_allowed,
        Some(HitBreakdown {
            home_runs: 1,
            ..HitBreakdown::default()
        })
    );
    // Unknown survives later PAs, both halves and completed/terminal boundaries.
    assert_eq!(lines.pitchers[0]["away_pitcher"].outs_recorded, 24);
    assert_eq!(lines.pitchers[1]["home_pitcher"].outs_recorded, 27);
    assert_eq!(
        lines
            .fielders
            .iter()
            .flat_map(|side| side.values())
            .map(|f| f.credited_putouts)
            .sum::<u32>(),
        51
    );
}

#[test]
fn classified_hits_partition_credited_h_without_inferring_final_base_value() {
    for half in [Half::Top, Half::Bottom] {
        for mask in 0..8 {
            for outs in 0..3 {
                for (value, class) in [
                    (1, CreditKind::Single),
                    (2, CreditKind::Double),
                    (3, CreditKind::Triple),
                ] {
                    for destination in value..=3 {
                        for reverse in [false, true] {
                            let before = state(mask, outs, half);
                            let mut event = hit(
                                &before,
                                [Destination::First, Destination::Second, Destination::Third]
                                    [destination - 1],
                                Some(class),
                            );
                            if reverse {
                                event.delta.moves.reverse();
                                event.delta.credits.reverse();
                            }
                            let transition = apply(&before, event).unwrap();
                            assert_eq!(
                                EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
                                transition
                            );
                            let box_lines =
                                derive_accounting(std::slice::from_ref(&transition)).unwrap();
                            let offense = half.offense();
                            let b = &box_lines.batters[offense]
                                [&before.teams[offense].batting_order[0]];
                            let p = &box_lines.pitchers[1 - offense]
                                [&before.teams[1 - offense].pitcher];
                            let expected = breakdown(class);
                            assert_eq!(expected.hits(), 1);
                            assert_eq!(expected.total_bases(), value as u32);
                            assert_eq!(b.credited_hit_breakdown, Some(expected.clone()));
                            assert_eq!(p.credited_hit_breakdown_allowed, Some(expected));
                            assert_eq!(
                                (b.plate_appearances, b.at_bats, b.credited_hits),
                                (1, Some(1), 1)
                            );
                            assert_eq!(
                                (p.batters_faced, p.at_bats_against, p.credited_hits_allowed),
                                (1, Some(1), 1)
                            );
                            assert_eq!(p.outs_recorded, 0);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn known_type_contradictions_and_missing_context_use_distinct_errors() {
    for half in [Half::Top, Half::Bottom] {
        for outs in 0..3 {
            let before = state(0, outs, half);
            let good = hit(&before, Destination::First, Some(CreditKind::Single));
            let batter = good.participants.batter.clone().unwrap();
            let mut duplicate = good.clone();
            duplicate
                .delta
                .credits
                .push(credit(&batter, CreditKind::Single));
            rejected(&before, duplicate, TransitionError::IllegalCredit);
            let mut conflict = good.clone();
            conflict
                .delta
                .credits
                .push(credit(&batter, CreditKind::Double));
            rejected(&before, conflict, TransitionError::IllegalCredit);
            let mut no_hit = good.clone();
            no_hit.delta.credits.retain(|c| c.kind != CreditKind::Hit);
            rejected(&before, no_hit, TransitionError::IllegalCredit);
            let mut wrong_actor = good.clone();
            wrong_actor.delta.credits.last_mut().unwrap().player_id =
                before.teams[half.offense()].batting_order[1].clone();
            rejected(&before, wrong_actor, TransitionError::IllegalCredit);
            for class in [CreditKind::Double, CreditKind::Triple] {
                rejected(
                    &before,
                    hit(&before, Destination::First, Some(class)),
                    TransitionError::IllegalCredit,
                );
                let mut unknown = hit(&before, Destination::Third, Some(class));
                unknown.fielding_review = FieldingReview::Unknown;
                rejected(&before, unknown, TransitionError::UnsupportedRule);
            }
            let mut retired_batter = good;
            retired_batter.delta.moves[0].to = Destination::Out;
            retired_batter.delta.outs = 1;
            if outs == 2 {
                retired_batter.delta.third_out = Some(ThirdOut {
                    runner_id: batter,
                    kind: ThirdOutKind::BatterAfterFirstTiming,
                    home_before_out: vec![],
                });
            }
            rejected(&before, retired_batter, TransitionError::UnsupportedRule);
        }
        let before = state(1, 0, half);
        let mut retired_runner = hit(&before, Destination::First, Some(CreditKind::Single));
        retired_runner.delta.moves[1].to = Destination::Out;
        retired_runner.delta.outs = 1;
        retired_runner.delta.runs = 0;
        retired_runner
            .delta
            .credits
            .retain(|c| c.kind != CreditKind::Run);
        rejected(&before, retired_runner, TransitionError::UnsupportedRule);
    }
}

#[test]
fn single_on_throw_or_error_is_not_upgraded_and_double_can_leave_prior_runner_on_third() {
    for half in [Half::Top, Half::Bottom] {
        let before = state(0, 0, half);
        let mut event = hit(&before, Destination::Third, Some(CreditKind::Single));
        event.fielding_review = FieldingReview::Unknown;
        event.delta.credits.push(credit(
            &before.teams[1 - half.offense()].batting_order[4],
            CreditKind::Error,
        ));
        let transition = apply(&before, event).unwrap();
        let lines = derive_accounting(std::slice::from_ref(&transition)).unwrap();
        assert_eq!(
            lines.batters[half.offense()][&before.teams[half.offense()].batting_order[0]]
                .credited_hit_breakdown,
            Some(breakdown(CreditKind::Single))
        );
        let before = state(4, 0, half);
        let mut held = hit(&before, Destination::Second, Some(CreditKind::Double));
        held.delta.moves.retain(|m| m.from == Base::Plate);
        held.delta.credits.retain(|c| c.kind != CreditKind::Run);
        held.delta.runs = 0;
        let transition = apply(&before, held).unwrap();
        assert_eq!(transition.after().bases[2], before.bases[2]);
        let lines = derive_accounting(std::slice::from_ref(&transition)).unwrap();
        assert_eq!(
            lines.batters[half.offense()][&before.teams[half.offense()].batting_order[0]]
                .credited_hit_breakdown,
            Some(breakdown(CreditKind::Double))
        );
    }
}

#[test]
fn ordinary_walkoff_hit_ceiling_uses_winning_runner_origin_not_batter_destination() {
    for base in 0..3 {
        for (value, class) in [
            (1, CreditKind::Single),
            (2, CreditKind::Double),
            (3, CreditKind::Triple),
        ] {
            for destination in 1..=3 {
                for reverse in [false, true] {
                    let mut before = state(1 << base, 0, Half::Bottom);
                    before.inning = 9;
                    let mut event = hit(
                        &before,
                        [Destination::First, Destination::Second, Destination::Third]
                            [destination - 1],
                        Some(class),
                    );
                    if reverse {
                        event.delta.moves.reverse();
                        event.delta.credits.reverse();
                    }
                    if value > destination || value > 3 - base {
                        rejected(&before, event, TransitionError::IllegalCredit);
                    } else {
                        let transition = apply(&before, event).unwrap();
                        assert!(transition.after().over);
                        assert_eq!(
                            EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
                            transition
                        );
                        let lines = derive_accounting(std::slice::from_ref(&transition)).unwrap();
                        assert_eq!(
                            lines.batters[1]["home0"].credited_hit_breakdown,
                            Some(breakdown(class))
                        );
                    }
                }
            }
        }
    }
    let mut before = state(6, 0, Half::Bottom);
    before.inning = 9;
    before.score = [1, 0];
    rejected(
        &before,
        hit(&before, Destination::Third, Some(CreditKind::Double)),
        TransitionError::UnsupportedRule,
    );
    // Neither possible winner advances three bases: a definite contradiction.
    rejected(
        &before,
        hit(&before, Destination::Third, Some(CreditKind::Triple)),
        TransitionError::IllegalCredit,
    );
    let mut first_second = state(3, 0, Half::Bottom);
    first_second.inning = 9;
    first_second.score = [1, 0];
    rejected(
        &first_second,
        hit(&first_second, Destination::Third, Some(CreditKind::Triple)),
        TransitionError::UnsupportedRule,
    );
    // Both possible winners advance at least two bases: order is immaterial.
    assert!(
        apply(
            &first_second,
            hit(&first_second, Destination::Second, Some(CreditKind::Double))
        )
        .unwrap()
        .after()
        .over
    );
    assert!(
        apply(
            &before,
            hit(&before, Destination::Third, Some(CreditKind::Single))
        )
        .unwrap()
        .after()
        .over
    );
}

fn unique_offset(bytes: &[u8], marker: &[u8]) -> usize {
    let offsets: Vec<_> = bytes
        .windows(marker.len())
        .enumerate()
        .filter_map(|(i, w)| (w == marker).then_some(i))
        .collect();
    assert_eq!(offsets.len(), 1);
    offsets[0]
}

#[test]
fn well_formed_classification_forgeries_are_semantically_rejected() {
    for half in [Half::Top, Half::Bottom] {
        for (to, review, forged, error) in [
            (
                Destination::First,
                FieldingReview::NoMisplay,
                18,
                TransitionError::IllegalCredit,
            ),
            (
                Destination::Second,
                FieldingReview::NoMisplay,
                19,
                TransitionError::IllegalCredit,
            ),
            (
                Destination::Second,
                FieldingReview::Unknown,
                18,
                TransitionError::UnsupportedRule,
            ),
        ] {
            let before = state(0, 0, half);
            let mut event = hit(&before, to, Some(CreditKind::Single));
            event.fielding_review = review;
            let valid = apply(&before, event).unwrap();
            let original = valid.to_bytes();
            assert!(original.starts_with(b"BBE24"));
            assert_eq!(EventTransition::from_bytes(&original).unwrap(), valid);
            let batter = valid.event().participants.batter.as_ref().unwrap();
            let mut marker = (batter.len() as u64).to_le_bytes().to_vec();
            marker.extend_from_slice(batter.as_bytes());
            marker.push(17); // Single, followed by empty changes and absent third-out.
            marker.extend_from_slice(&[0; 9]);
            let offset = unique_offset(&original, &marker) + 8 + batter.len();
            let mut bytes = original.clone();
            bytes[offset] = forged;
            assert_eq!(EventTransition::from_bytes(&bytes), Err(error));
            assert_eq!(&bytes[..offset], &original[..offset]);
            assert_eq!(&bytes[offset + 1..], &original[offset + 1..]);
            bytes[offset] = 255;
            assert_eq!(
                EventTransition::from_bytes(&bytes),
                Err(TransitionError::InvalidEncoding)
            );
            for earlier_prefix in [b"BBE16", b"BBE15", b"BBE14", b"BBE13", b"BBE12", b"BBE11"] {
                let mut old_version = original.clone();
                old_version[..5].copy_from_slice(earlier_prefix);
                assert_eq!(
                    EventTransition::from_bytes(&old_version),
                    Err(TransitionError::InvalidEncoding)
                );
            }
        }
        // Same-length parsed positive control: a scorer-classified Double
        // can legitimately share a Single's final second-base destination.
        let before = state(0, 0, half);
        let valid = apply(
            &before,
            hit(&before, Destination::Second, Some(CreditKind::Single)),
        )
        .unwrap();
        let mut bytes = valid.to_bytes();
        let batter = valid.event().participants.batter.as_ref().unwrap();
        let mut marker = (batter.len() as u64).to_le_bytes().to_vec();
        marker.extend_from_slice(batter.as_bytes());
        marker.push(17);
        marker.extend_from_slice(&[0; 9]);
        let offset = unique_offset(&bytes, &marker) + 8 + batter.len();
        bytes[offset] = 18;
        let double = EventTransition::from_bytes(&bytes).unwrap();
        assert_eq!(
            derive_accounting(&[double]).unwrap().batters[half.offense()][batter]
                .credited_hit_breakdown,
            Some(breakdown(CreditKind::Double))
        );
    }
}

#[test]
fn explicit_hr_classifies_hits_with_unknown_review_and_preserves_walkoff_exception() {
    for half in [Half::Top, Half::Bottom] {
        for mask in 0..8 {
            let mut before = state(mask, 0, half);
            before.inning = 9;
            let mut event = hit(&before, Destination::Home, None);
            event.kind = EventKind::OutOfParkHomeRun;
            event.fielding_review = FieldingReview::Unknown;
            let transition = apply(&before, event.clone()).unwrap();
            let lines = derive_accounting(std::slice::from_ref(&transition)).unwrap();
            let expected = HitBreakdown {
                singles: 0,
                doubles: 0,
                triples: 0,
                home_runs: 1,
            };
            assert_eq!(expected.total_bases(), 4);
            assert_eq!(
                lines.batters[half.offense()][&before.teams[half.offense()].batting_order[0]]
                    .credited_hit_breakdown,
                Some(expected.clone())
            );
            assert_eq!(
                lines.pitchers[1 - half.offense()][&before.teams[1 - half.offense()].pitcher]
                    .credited_hit_breakdown_allowed,
                Some(expected)
            );
            if half == Half::Bottom {
                assert!(transition.after().over);
            }
            for class in [CreditKind::Single, CreditKind::Double, CreditKind::Triple] {
                let mut conflict = event.clone();
                conflict.delta.credits.push(credit(
                    conflict.participants.batter.as_ref().unwrap(),
                    class,
                ));
                rejected(&before, conflict, TransitionError::IllegalCredit);
            }
        }
    }
}

#[test]
fn unclassified_hit_unknown_is_irreversible_and_local_to_actor_and_actual_pitcher() {
    for half in [Half::Top, Half::Bottom] {
        for unknown_first in [false, true] {
            let mut before = state(0, 0, half);
            let mut log = vec![];
            for index in 0..10 {
                let class = if index == if unknown_first { 0 } else { 9 } {
                    None
                } else {
                    Some(CreditKind::Single)
                };
                let transition = apply(&before, hit(&before, Destination::First, class)).unwrap();
                before = transition.after().clone();
                log.push(transition);
            }
            let lines = derive_accounting(&log).unwrap();
            let side = half.offense();
            assert_eq!(
                lines.batters[side][&before.teams[side].batting_order[0]].credited_hit_breakdown,
                None
            );
            assert_eq!(
                lines.batters[side][&before.teams[side].batting_order[1]].credited_hit_breakdown,
                Some(breakdown(CreditKind::Single))
            );
            assert_eq!(
                lines.pitchers[1 - side][&before.teams[1 - side].pitcher]
                    .credited_hit_breakdown_allowed,
                None
            );
            assert_eq!(
                lines.pitchers[1 - side][&before.teams[1 - side].pitcher].credited_hits_allowed,
                10
            );
        }
        let mut before = state(0, 0, half);
        let old_pitcher = before.teams[1 - half.offense()].pitcher.clone();
        let reach = apply(&before, hit(&before, Destination::First, None)).unwrap();
        before = reach.after().clone();
        let relief = format!("{}_relief", ["away", "home"][1 - half.offense()]);
        let change = Event {
            kind: EventKind::Substitution,
            participants: Participants {
                batter: Some(before.teams[half.offense()].batting_order[1].clone()),
                pitcher: old_pitcher.clone(),
            },
            fielding_review: FieldingReview::NoMisplay,
            two_out_contact: None,
            single_delivery: None,
            balk: None,
            caught_strikeout: None,
            protected_award: None,
            sole_home_run: None,
            sole_safe_hit: None,
            ordinary_ground_throw: None,
            delta: Delta {
                moves: vec![],
                outs: 0,
                runs: 0,
                credits: vec![],
                third_out: None,
                changes: vec![PersonnelChange::Pitcher {
                    team: if half == Half::Top {
                        TeamSide::Home
                    } else {
                        TeamSide::Away
                    },
                    outgoing: old_pitcher.clone(),
                    incoming: relief.clone(),
                }],
            },
        };
        let substitution = apply(&before, change).unwrap();
        before = substitution.after().clone();
        let double = apply(
            &before,
            hit(&before, Destination::Second, Some(CreditKind::Double)),
        )
        .unwrap();
        let lines = derive_accounting(&[reach, substitution, double]).unwrap();
        let old = &lines.pitchers[1 - half.offense()][&old_pitcher];
        let new = &lines.pitchers[1 - half.offense()][&relief];
        assert_eq!(
            (
                old.credited_hits_allowed,
                old.runs_allowed,
                old.credited_hit_breakdown_allowed.clone()
            ),
            (1, 1, None)
        );
        assert_eq!(
            (
                new.credited_hits_allowed,
                new.runs_allowed,
                new.credited_hit_breakdown_allowed.clone()
            ),
            (1, 0, Some(breakdown(CreditKind::Double)))
        );
    }
}
