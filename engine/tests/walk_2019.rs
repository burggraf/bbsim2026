use bbsim_engine::accounting::derive_accounting;
use bbsim_engine::{
    apply, Base, Credit, CreditKind, Delta, Destination, Event, EventKind, EventTransition,
    FieldingReview, GameState, Half, Participants, Runner, RunnerMove, TeamState, TransitionError,
};

fn state(mask: u8, outs: u8, half: Half) -> GameState {
    let teams = ["away", "home"].map(|prefix| TeamState {
        batting_order: std::array::from_fn(|slot| format!("{prefix}{slot}")),
        next_batter: 0,
        pitcher: format!("{prefix}_pitcher"),
        dh_slot: Some(8),
        dh_must_bat: false,
        bench: vec![format!("{prefix}_relief")],
        removed: vec![],
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
                responsible_pitcher: format!("prior_pitcher_{base}"),
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

fn walk(before: &GameState, classification: Option<CreditKind>, reverse: bool) -> Event {
    let offense = before.half.offense();
    let batter =
        before.teams[offense].batting_order[before.teams[offense].next_batter as usize].clone();
    let pitcher = before.teams[1 - offense].pitcher.clone();
    let mut moves = vec![RunnerMove {
        runner: Runner {
            player_id: batter.clone(),
            responsible_pitcher: pitcher.clone(),
        },
        from: Base::Plate,
        to: Destination::First,
    }];
    let mut forced = before.bases[0].is_some();
    for base in 0..3 {
        if !forced {
            break;
        }
        let Some(runner) = before.bases[base].as_ref() else {
            break;
        };
        moves.push(RunnerMove {
            runner: runner.clone(),
            from: [Base::First, Base::Second, Base::Third][base],
            to: if base == 2 {
                Destination::Home
            } else {
                [Destination::Second, Destination::Third][base]
            },
        });
        forced = base < 2 && before.bases[base + 1].is_some();
    }
    let mut credits = vec![
        credit(&batter, CreditKind::PlateAppearance),
        credit(&batter, CreditKind::Walk),
    ];
    if let Some(classification) = classification {
        credits.push(credit(&batter, classification));
    }
    for movement in &moves {
        if movement.to == Destination::Home {
            credits.push(credit(&movement.runner.player_id, CreditKind::Run));
        }
    }
    if reverse {
        moves.reverse();
        credits.reverse();
    }
    let runs = credits.iter().filter(|c| c.kind == CreditKind::Run).count() as u8;
    Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(batter),
            pitcher,
        },
        fielding_review: if reverse {
            FieldingReview::NoMisplay
        } else {
            FieldingReview::Unknown
        },
        two_out_contact: None,
        delta: Delta {
            moves,
            outs: 0,
            runs,
            credits,
            changes: vec![],
            third_out: None,
        },
    }
}

fn assert_walk_line(transition: &EventTransition, classification: Option<CreditKind>) {
    let lines = derive_accounting(std::slice::from_ref(transition)).unwrap();
    let before = transition.before();
    let offense = before.half.offense();
    let defense = 1 - offense;
    let batter = transition.event().participants.batter.as_ref().unwrap();
    let pitcher = &transition.event().participants.pitcher;
    let batting = &lines.batters[offense][batter];
    let pitching = &lines.pitchers[defense][pitcher];
    assert_eq!(batting.plate_appearances, 1);
    assert_eq!(batting.credited_walks, 1);
    assert_eq!(batting.at_bats, Some(0));
    assert_eq!(pitching.batters_faced, 1);
    assert_eq!(pitching.credited_walks_allowed, 1);
    assert_eq!(pitching.at_bats_against, Some(0));
    let expected = match classification {
        Some(CreditKind::OrdinaryWalk) => Some(0),
        Some(CreditKind::IntentionalWalk) => Some(1),
        _ => None,
    };
    assert_eq!(batting.credited_intentional_walks, expected);
    assert_eq!(pitching.credited_intentional_walks_allowed, expected);
    assert_eq!(
        lines.runs[offense],
        u32::from(transition.event().delta.runs)
    );
}

#[test]
fn classified_and_unclassified_walks_conserve_counts_for_all_masks_outs_and_halves() {
    for half in [Half::Top, Half::Bottom] {
        for mask in 0..8 {
            for outs in 0..3 {
                for classification in [
                    None,
                    Some(CreditKind::OrdinaryWalk),
                    Some(CreditKind::IntentionalWalk),
                ] {
                    for reverse in [false, true] {
                        let before = state(mask, outs, half);
                        let event = walk(&before, classification, reverse);
                        let transition = apply(&before, event.clone()).unwrap();
                        assert_eq!(
                            EventTransition::try_new(
                                before.clone(),
                                event,
                                transition.after().clone()
                            )
                            .unwrap(),
                            transition
                        );
                        assert_eq!(
                            EventTransition::from_bytes(&transition.to_bytes()).unwrap(),
                            transition
                        );
                        assert_walk_line(&transition, classification);
                        assert_eq!(transition.after().outs, outs);
                        assert_eq!(
                            transition.after().score[half.offense()],
                            u16::from(mask == 7)
                        );
                    }
                }
            }
        }
    }
}

fn rejected(before: &GameState, event: Event) {
    assert_eq!(
        apply(before, event.clone()),
        Err(TransitionError::IllegalCredit)
    );
    assert_eq!(
        EventTransition::try_new(before.clone(), event, before.clone()),
        Err(TransitionError::IllegalCredit)
    );
}

#[test]
fn classification_requires_one_generic_walk_by_the_current_batter() {
    let before = state(0, 0, Half::Top);
    let ordinary = walk(&before, Some(CreditKind::OrdinaryWalk), false);
    let intentional = walk(&before, Some(CreditKind::IntentionalWalk), false);

    let mut missing_walk = ordinary.clone();
    missing_walk
        .delta
        .credits
        .retain(|c| c.kind != CreditKind::Walk);
    rejected(&before, missing_walk);

    let mut duplicate = ordinary.clone();
    duplicate.delta.credits.push(credit(
        duplicate.participants.batter.as_ref().unwrap(),
        CreditKind::OrdinaryWalk,
    ));
    rejected(&before, duplicate);

    let mut conflicting = intentional.clone();
    conflicting.delta.credits.push(credit(
        conflicting.participants.batter.as_ref().unwrap(),
        CreditKind::OrdinaryWalk,
    ));
    rejected(&before, conflicting);

    let mut wrong_actor = ordinary.clone();
    wrong_actor
        .delta
        .credits
        .iter_mut()
        .find(|c| c.kind == CreditKind::OrdinaryWalk)
        .unwrap()
        .player_id = "away1".into();
    rejected(&before, wrong_actor);

    for incompatible in [
        CreditKind::Hit,
        CreditKind::Strikeout,
        CreditKind::AtBat,
        CreditKind::HitByPitch,
        CreditKind::SacrificeBunt,
        CreditKind::SacrificeFly,
        CreditKind::InterferenceAward,
    ] {
        let mut event = ordinary.clone();
        event.delta.credits.push(credit(
            event.participants.batter.as_ref().unwrap(),
            incompatible,
        ));
        rejected(&before, event);
    }

    let mut non_pa = ordinary.clone();
    non_pa.kind = EventKind::RunnerOnly(bbsim_engine::RunnerEvent::StealAttempt);
    rejected(&before, non_pa);

    for kind in [EventKind::OutOfParkHomeRun, EventKind::FieldersChoice] {
        let mut incompatible = ordinary.clone();
        incompatible.kind = kind;
        rejected(&before, incompatible);
    }
}

#[test]
fn review_never_infers_walk_classification_and_known_zero_is_qualified() {
    for review in [FieldingReview::Unknown, FieldingReview::NoMisplay] {
        let mut event = walk(&state(0, 0, Half::Top), None, false);
        event.fielding_review = review;
        let transition = apply(&state(0, 0, Half::Top), event).unwrap();
        assert_walk_line(&transition, None);
    }

    let before = state(0, 0, Half::Top);
    let event = Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some("away0".into()),
            pitcher: "home_pitcher".into(),
        },
        fielding_review: FieldingReview::Unknown,
        two_out_contact: None,
        delta: Delta {
            moves: vec![RunnerMove {
                runner: Runner {
                    player_id: "away0".into(),
                    responsible_pitcher: "home_pitcher".into(),
                },
                from: Base::Plate,
                to: Destination::Out,
            }],
            outs: 1,
            runs: 0,
            credits: vec![
                credit("away0", CreditKind::PlateAppearance),
                credit("away0", CreditKind::Strikeout),
            ],
            changes: vec![],
            third_out: None,
        },
    };
    let transition = apply(&before, event).unwrap();
    let lines = derive_accounting(&[transition]).unwrap();
    assert_eq!(
        lines.batters[0]["away0"].credited_intentional_walks,
        Some(0)
    );
    assert_eq!(
        lines.pitchers[1]["home_pitcher"].credited_intentional_walks_allowed,
        Some(0)
    );
}

#[test]
fn intentional_walkoff_and_retired_batter_touch_uncertainty_are_preserved() {
    let mut before = state(7, 0, Half::Bottom);
    before.inning = 9;
    let transition = apply(
        &before,
        walk(&before, Some(CreditKind::IntentionalWalk), false),
    )
    .unwrap();
    assert!(transition.after().over);
    assert_eq!(transition.after().score, [0, 1]);
    assert_walk_line(&transition, Some(CreditKind::IntentionalWalk));

    let before = state(0, 0, Half::Top);
    let mut retired = walk(&before, Some(CreditKind::IntentionalWalk), false);
    retired.delta.moves[0].to = Destination::Out;
    retired.delta.outs = 1;
    assert_eq!(
        apply(&before, retired.clone()),
        Err(TransitionError::UnsupportedRule)
    );
    let mut after = before.clone();
    after.teams[0].next_batter = 1;
    after.outs = 1;
    assert_eq!(
        EventTransition::try_new(before, retired, after),
        Err(TransitionError::UnsupportedRule)
    );
}

fn out_pa(before: &GameState) -> Event {
    let offense = before.half.offense();
    let batter =
        before.teams[offense].batting_order[before.teams[offense].next_batter as usize].clone();
    let pitcher = before.teams[1 - offense].pitcher.clone();
    let third_out = (before.outs == 2).then(|| bbsim_engine::ThirdOut {
        runner_id: batter.clone(),
        kind: bbsim_engine::ThirdOutKind::BatterBeforeFirst,
        home_before_out: vec![],
    });
    Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(batter.clone()),
            pitcher,
        },
        fielding_review: FieldingReview::Unknown,
        two_out_contact: None,
        delta: Delta {
            moves: vec![RunnerMove {
                runner: Runner {
                    player_id: batter.clone(),
                    responsible_pitcher: before.teams[1 - offense].pitcher.clone(),
                },
                from: Base::Plate,
                to: Destination::Out,
            }],
            outs: 1,
            runs: 0,
            credits: vec![
                credit(&batter, CreditKind::PlateAppearance),
                credit(&batter, CreditKind::Strikeout),
            ],
            changes: vec![],
            third_out,
        },
    }
}

#[test]
fn unknown_intentionality_is_irreversible_per_batter_and_pitcher_segment() {
    let before = state(0, 0, Half::Top);
    let mut log = Vec::new();
    let mut current = before;
    let generic = apply(&current, walk(&current, None, false)).unwrap();
    current = generic.after().clone();
    log.push(generic);

    // A generic walk's uncertainty is local: the next batter and a reliever
    // still have a known zero IBB line until their own unknown walk.
    let pitcher_change = Event {
        kind: EventKind::Substitution,
        participants: Participants {
            batter: Some("away1".into()),
            pitcher: "home_pitcher".into(),
        },
        fielding_review: FieldingReview::Unknown,
        two_out_contact: None,
        delta: Delta {
            moves: vec![],
            outs: 0,
            runs: 0,
            credits: vec![],
            changes: vec![bbsim_engine::PersonnelChange::Pitcher {
                team: bbsim_engine::TeamSide::Home,
                outgoing: "home_pitcher".into(),
                incoming: "home_relief".into(),
            }],
            third_out: None,
        },
    };
    let changed = apply(&current, pitcher_change).unwrap();
    current = changed.after().clone();
    log.push(changed);

    let ordinary = apply(
        &current,
        walk(&current, Some(CreditKind::OrdinaryWalk), false),
    )
    .unwrap();
    current = ordinary.after().clone();
    log.push(ordinary);

    // Finish the batting order with outs. Third outs and half changes do not
    // restore either the first batter's or the original pitcher's unknown IBB.
    while current.teams[current.half.offense()].next_batter != 0
        || current.half != Half::Top
        || current.inning == 2
    {
        let event = out_pa(&current);
        let transition = apply(&current, event).unwrap();
        current = transition.after().clone();
        log.push(transition);
    }
    let repeated = apply(
        &current,
        walk(&current, Some(CreditKind::IntentionalWalk), false),
    )
    .unwrap();
    log.push(repeated);

    let lines = derive_accounting(&log).unwrap();
    assert_eq!(lines.batters[0]["away0"].credited_walks, 2);
    assert_eq!(lines.batters[0]["away0"].credited_intentional_walks, None);
    assert_eq!(
        lines.batters[0]["away1"].credited_intentional_walks,
        Some(0)
    );
    assert_eq!(lines.pitchers[1]["home_pitcher"].credited_walks_allowed, 1);
    assert_eq!(
        lines.pitchers[1]["home_pitcher"].credited_intentional_walks_allowed,
        None
    );
    assert_eq!(
        lines.pitchers[1]["home_relief"].credited_intentional_walks_allowed,
        Some(1)
    );
    assert_eq!(lines.pitchers[1]["home_relief"].credited_walks_allowed, 2);
    assert_eq!(lines.pitchers[1]["home_pitcher"].batters_faced, 1);
}

#[test]
fn later_generic_walk_makes_previously_known_classification_unknown() {
    let mut current = state(0, 0, Half::Top);
    let mut log = vec![];
    let classified = apply(
        &current,
        walk(&current, Some(CreditKind::OrdinaryWalk), false),
    )
    .unwrap();
    current = classified.after().clone();
    log.push(classified);
    while current.teams[0].next_batter != 0 {
        let transition = apply(&current, out_pa(&current)).unwrap();
        current = transition.after().clone();
        log.push(transition);
    }
    let later_generic = apply(&current, walk(&current, None, false)).unwrap();
    log.push(later_generic);

    let lines = derive_accounting(&log).unwrap();
    assert_eq!(lines.batters[0]["away0"].credited_walks, 2);
    assert_eq!(lines.batters[0]["away0"].credited_intentional_walks, None);
    assert_eq!(lines.pitchers[1]["home_pitcher"].credited_walks_allowed, 2);
    assert_eq!(
        lines.pitchers[1]["home_pitcher"].credited_intentional_walks_allowed,
        None
    );
}

#[test]
fn inherited_run_charge_does_not_change_current_pitcher_bb_ibb_or_bf() {
    let mut before = state(7, 0, Half::Bottom);
    before.bases[2].as_mut().unwrap().responsible_pitcher = "old_pitcher".into();
    let event = walk(&before, Some(CreditKind::IntentionalWalk), false);
    let transition = apply(&before, event).unwrap();
    let lines = derive_accounting(&[transition]).unwrap();
    assert_eq!(lines.runs[1], 1);
    assert_eq!(lines.pitchers[0]["old_pitcher"].runs_allowed, 1);
    assert_eq!(lines.pitchers[0]["away_pitcher"].runs_allowed, 0);
    assert_eq!(lines.pitchers[0]["away_pitcher"].credited_walks_allowed, 1);
    assert_eq!(
        lines.pitchers[0]["away_pitcher"].credited_intentional_walks_allowed,
        Some(1)
    );
    assert_eq!(lines.pitchers[0]["away_pitcher"].batters_faced, 1);
    assert_eq!(lines.batters[1]["home0"].credited_walks, 1);
    assert_eq!(
        lines.batters[1]["home0"].credited_intentional_walks,
        Some(1)
    );
}

fn unique_tag(bytes: &[u8], id: &str, tag: u8) -> usize {
    let mut marker = (id.len() as u64).to_le_bytes().to_vec();
    marker.extend_from_slice(id.as_bytes());
    marker.push(tag);
    let offsets: Vec<_> = bytes
        .windows(marker.len())
        .enumerate()
        .filter_map(|(offset, window)| (window == marker).then_some(offset))
        .collect();
    assert_eq!(offsets.len(), 1);
    offsets[0] + 8 + id.len()
}

#[test]
fn codec_revalidates_walk_classification_forgeries_and_roundtrips_controls() {
    let before = state(0, 0, Half::Top);
    let ordinary = apply(
        &before,
        walk(&before, Some(CreditKind::OrdinaryWalk), false),
    )
    .unwrap();
    let intentional = apply(
        &before,
        walk(&before, Some(CreditKind::IntentionalWalk), false),
    )
    .unwrap();
    assert_eq!(
        EventTransition::from_bytes(&ordinary.to_bytes()).unwrap(),
        ordinary
    );
    assert_eq!(
        EventTransition::from_bytes(&intentional.to_bytes()).unwrap(),
        intentional
    );
    assert!(ordinary.to_bytes().starts_with(b"BBE14"));
    let batter = ordinary.event().participants.batter.as_ref().unwrap();

    // Replace the sole Walk on a generic record with an IntentionalWalk:
    // structurally intact but missing the required BB credit.
    let generic = apply(&before, walk(&before, None, false)).unwrap();
    let mut missing = generic.to_bytes();
    let walk_tag = unique_tag(&missing, batter, 2);
    missing[walk_tag] = 21;
    assert_eq!(
        EventTransition::from_bytes(&missing),
        Err(TransitionError::IllegalCredit)
    );

    let pa_tag = unique_tag(&ordinary.to_bytes(), batter, 0);
    let mut duplicate = ordinary.to_bytes();
    duplicate[pa_tag] = 20;
    assert_eq!(
        EventTransition::from_bytes(&duplicate),
        Err(TransitionError::IllegalCredit)
    );
    let pa_tag = unique_tag(&intentional.to_bytes(), batter, 0);
    let mut conflicting = intentional.to_bytes();
    conflicting[pa_tag] = 20;
    assert_eq!(
        EventTransition::from_bytes(&conflicting),
        Err(TransitionError::IllegalCredit)
    );

    // A well-formed actor forgery retains the same string length and all bytes
    // other than the classification credit's player ID.
    let mut wrong_actor = ordinary.to_bytes();
    let modifier = unique_tag(&wrong_actor, batter, 20);
    wrong_actor[modifier - batter.len()..modifier].copy_from_slice(b"away1");
    assert_eq!(
        EventTransition::from_bytes(&wrong_actor),
        Err(TransitionError::IllegalCredit)
    );

    // A scorer's ordinary-vs-intentional distinction can be changed without
    // changing record layout; both tags are appended after the old prefix.
    let mut intentional_forgery = ordinary.to_bytes();
    let modifier = unique_tag(&intentional_forgery, batter, 20);
    intentional_forgery[modifier] = 21;
    let reclassified = EventTransition::from_bytes(&intentional_forgery).unwrap();
    assert_eq!(
        reclassified.event().delta.credits.len(),
        ordinary.event().delta.credits.len()
    );
    assert_walk_line(&reclassified, Some(CreditKind::IntentionalWalk));
}

#[test]
fn missing_walk_modifier_precedes_inherited_runner_charge_ambiguity() {
    let mut before = state(1, 0, Half::Top);
    before.bases[0].as_mut().unwrap().responsible_pitcher = "inherited_pitcher".into();
    let batter = before.teams[0].batting_order[0].clone();
    let pitcher = before.teams[1].pitcher.clone();
    let event = Event {
        kind: EventKind::PlateAppearance,
        participants: Participants {
            batter: Some(batter.clone()),
            pitcher: pitcher.clone(),
        },
        fielding_review: FieldingReview::Unknown,
        two_out_contact: None,
        delta: Delta {
            moves: vec![
                RunnerMove {
                    runner: Runner {
                        player_id: batter.clone(),
                        responsible_pitcher: pitcher,
                    },
                    from: Base::Plate,
                    to: Destination::First,
                },
                RunnerMove {
                    runner: before.bases[0].as_ref().unwrap().clone(),
                    from: Base::First,
                    to: Destination::Out,
                },
            ],
            outs: 1,
            runs: 0,
            credits: vec![
                credit(&batter, CreditKind::PlateAppearance),
                credit(&batter, CreditKind::Hit),
            ],
            changes: vec![],
            third_out: None,
        },
    };
    let valid = apply(&before, event.clone()).unwrap();
    let mut forged = event;
    forged
        .delta
        .credits
        .iter_mut()
        .find(|c| c.kind == CreditKind::Hit)
        .unwrap()
        .kind = CreditKind::IntentionalWalk;
    assert_eq!(
        apply(&before, forged.clone()),
        Err(TransitionError::IllegalCredit)
    );
    assert_eq!(
        EventTransition::try_new(before.clone(), forged, valid.after().clone()),
        Err(TransitionError::IllegalCredit)
    );

    let mut bytes = valid.to_bytes();
    let hit_tag = unique_tag(&bytes, &batter, 3);
    bytes[hit_tag] = 21;
    assert_eq!(
        EventTransition::from_bytes(&bytes),
        Err(TransitionError::IllegalCredit)
    );
}
