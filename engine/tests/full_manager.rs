#[path = "../examples/support/full_manager.rs"]
mod full_manager;
#[allow(dead_code)]
#[path = "../examples/support/pitcher_entry_v2.rs"]
mod pitcher_entry_v2;
#[allow(dead_code)]
#[path = "../examples/support/pitcher_policy_v2.rs"]
mod pitcher_policy_v2;
#[allow(dead_code)]
#[path = "../examples/support/season_pool.rs"]
mod season_pool;

use bbsim_engine::{
    apply, BalkCause, BalkDisposition, BalkFacts, BalkResolution, Base, Credit, CreditKind, Delta,
    Destination, Event, EventKind, EventTransition, FieldingReview, GameState, Half, Participants,
    PersonnelChange, Runner, RunnerMove, TeamSide, TeamState,
};
use full_manager::Cast;
use std::collections::BTreeMap;

const META: &[(&str, &str)] = &[
    ("attribution", "The information used here was obtained free of charge from and is copyrighted by Retrosheet. Interested parties may contact Retrosheet at \"www.retrosheet.org\"."),
    ("availability_basis", "recorded nonstarters assumed available at start; not verified eligibility"),
    ("bullpen_report_sha256", "758186cceb67815573e7e109763f7be7d5e57b9ec9583bc8a3cca299d39edc88"),
    ("exhaustion", "last_pitcher_stays"),
    ("format", "closed-pitching-cast-v1"),
    ("league_counts", "42823,15142,1984,25947,8531,785,6776,1470,82245"),
    ("lineup_report_sha256", "5385231905a1f0e57b57a736e5229def6e94653c2894a093b4542b131b8df63b"),
    ("model", "dirichlet-log5-player-proxy-preview-v1"),
    ("opportunity_basis", "literal terminal-event participants; not official credits or PA/BF"),
    ("opportunity_report_sha256", "382be5a84b3e95687e6356b00d3b1b68522f41a5c2a6a065daf876a5a4c6881e"),
    ("policy", "pa-mean-closed-cast-v1"),
    ("snapshot_sha256", "d036b4664c8ce4cab51e86aacc075727558f9461c813b51f53d5974fc39a4528"),
    ("supplement_sha256", "554632467416eef202b7a55cadad2d09e5355ce98c83868b26b5267f98852b82"),
    ("workload_basis", "literal terminal-PA participants per pitcher-game appearance; not pitch counts, official BF or fatigue"),
    ("workload_report_sha256", "125768bd82ce54ead277d92301fbb7434e823a8fc4432777087c15b8e46ad2f9"),
    ("batter_kappa", "200"),
    ("pitcher_kappa", "200"),
];

fn tsv(away: &[(&str, &str)], home: &[(&str, &str)]) -> String {
    let mut rows = META
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect::<Vec<_>>();
    for (side, pitchers) in [("Away", away), ("Home", home)] {
        for (rank, (id, name)) in pitchers.iter().enumerate() {
            rows.push((format!("cast:{side}:{rank}:{id}"), (*name).to_owned()));
            rows.push((format!("pitcher:{id}"), "0,0,0,0,0,0,0,0,0".to_owned()));
            rows.push((format!("workload:start:{id}"), "27,1".to_owned()));
            rows.push((format!("workload:relief:{id}"), "6,1".to_owned()));
        }
    }
    rows.sort();
    let mut text = String::from("key\tvalue\n");
    for (key, value) in rows {
        text.push_str(&key);
        text.push('\t');
        text.push_str(&value);
        text.push('\n');
    }
    text
}

fn cast_text() -> String {
    tsv(
        &[
            ("gonzm005", "Marco Gonzales"),
            ("strih001", "Hunter Strickland"),
            ("gearc001", "Cory Gearrin"),
        ],
        &[
            ("fierm001", "Michael Fiers"),
            ("dullr001", "Ryan Dull"),
            ("wendj001", "J.B. Wendelken"),
        ],
    )
}

fn cast() -> Cast {
    Cast::read_tsv(&cast_text()).unwrap()
}

fn generic_cast() -> String {
    let text = tsv(
        &[("ap", "Starter A"), ("jay-j001", "Reliever A")],
        &[("hp", "Starter H")],
    );
    let mut rows = text
        .lines()
        .filter(|line| {
            !line.starts_with("bullpen_report_sha256\t")
                && !line.starts_with("workload_report_sha256\t")
        })
        .map(str::to_owned)
        .collect::<Vec<_>>();
    rows.extend([
        "game_id\tLAN201904010".into(),
        "cast_sizes\t2,1".into(),
        "workload_recipe_sha256\t512f542cee70c5824c6fff92f1278b51aad2be6a5a066db18b306091a3fae48f"
            .into(),
        "outcome_events_sha256\tea3fd660ac87deb02c61cddf28977db59f85f9a88f77aaabc0c07ba0f1831d1a"
            .into(),
    ]);
    for id in ["ap", "jay-j001", "hp"] {
        rows.push(format!("usage:start:{id}\t27"));
        rows.push(format!("usage:relief:{id}\t6"));
    }
    (rows.join("\n") + "\n")
        .replace("closed-pitching-cast-v1", "closed-pitching-cast-v3")
        .replace(
            "policy\tpa-mean-closed-cast-v1",
            "policy\tempirical-pa-closed-cast-v2",
        )
}
#[test]
fn generic_complete_cast_uses_declared_sizes_and_stable_ids_not_the_pinned_fixture() {
    let text = generic_cast();
    let cast = Cast::read_complete_tsv(&text).unwrap();
    assert_eq!(cast.game_id(), Some("LAN201904010"));
    let initial = GameState {
        teams: [team("a", "ap", false), team("h", "hp", false)],
        ..state(1, Half::Top, false)
    };
    let prepared = cast.prepare(&initial).unwrap();
    assert_eq!(prepared.teams[0].bench, ["jay-j001"]);
    assert!(prepared.teams[1].bench.is_empty());
    assert_eq!(cast.names().len(), 3);
    for bad in [
        text.replace("cast_sizes\t2,1", "cast_sizes\t1,1"),
        text.replace("cast:Away:1:jay-j001\tReliever A\n", ""),
        text.replace("game_id\tLAN201904010", "game_id\tinvalid"),
        text.replace("usage:start:ap\t27", "usage:start:ap\t26"),
        text.clone() + "unknown\tvalue\n",
    ] {
        assert!(Cast::read_complete_tsv(&bad).is_err(), "accepted {bad}");
    }
    assert_eq!(self::cast().game_id(), None);
}

fn team(prefix: &str, pitcher: &str, dh: bool) -> TeamState {
    let mut batting_order = std::array::from_fn(|slot| format!("{prefix}{slot}"));
    if !dh {
        batting_order[8] = pitcher.to_owned();
    }
    TeamState {
        defense: bbsim_engine::DefenseState::LegacyUnassigned,
        batting_order,
        next_batter: 0,
        pitcher: (pitcher.to_owned()).into(),
        dh_slot: dh.then_some(8),
        dh_must_bat: false,
        bench: Vec::new(),
        removed: Vec::new(),
    }
}

fn state(inning: u16, half: Half, dh: bool) -> GameState {
    GameState {
        inning,
        half,
        outs: 0,
        score: [0, 0],
        teams: [team("away", "gonzm005", dh), team("home", "fierm001", dh)],
        bases: [None, None, None],
        over: false,
    }
}

fn credit(player_id: &str, kind: CreditKind) -> Credit {
    Credit {
        player_id: player_id.to_owned(),
        kind,
    }
}

fn event(before: &GameState, kind: EventKind, delta: Delta) -> Event {
    let offense = before.half.offense();
    Event {
        kind,
        participants: Participants {
            batter: Some(
                before.teams[offense].batting_order[usize::from(before.teams[offense].next_batter)]
                    .clone(),
            ),
            pitcher: Some(
                before.teams[1 - offense]
                    .pitcher
                    .active_id()
                    .expect("legacy fixture pitcher")
                    .to_owned(),
            ),
        },
        fielding_review: FieldingReview::Unknown,
        delta,
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
    }
}

fn out_pa(before: &GameState) -> Event {
    let offense = before.half.offense();
    let batter =
        before.teams[offense].batting_order[usize::from(before.teams[offense].next_batter)].clone();
    let pitcher = before.teams[1 - offense]
        .pitcher
        .active_id()
        .expect("legacy fixture pitcher")
        .to_owned();
    event(
        before,
        EventKind::PlateAppearance,
        Delta {
            moves: vec![RunnerMove {
                runner: Runner {
                    player_id: batter.clone(),
                    responsible_pitcher: pitcher,
                },
                from: Base::Plate,
                to: Destination::Out,
            }],
            outs: 1,
            runs: 0,
            credits: vec![credit(&batter, CreditKind::PlateAppearance)],
            changes: vec![],
            third_out: None,
        },
    )
}

fn walk(before: &GameState) -> Event {
    let offense = before.half.offense();
    let batter =
        before.teams[offense].batting_order[usize::from(before.teams[offense].next_batter)].clone();
    let pitcher = before.teams[1 - offense]
        .pitcher
        .active_id()
        .expect("legacy fixture pitcher")
        .to_owned();
    let mut moves = vec![RunnerMove {
        runner: Runner {
            player_id: batter.clone(),
            responsible_pitcher: pitcher,
        },
        from: Base::Plate,
        to: Destination::First,
    }];
    let mut force = before.bases[0].is_some();
    for base in 0..3 {
        if !force {
            break;
        }
        let Some(runner) = before.bases[base].as_ref() else {
            break;
        };
        moves.push(RunnerMove {
            runner: runner.clone(),
            from: [Base::First, Base::Second, Base::Third][base],
            to: [Destination::Second, Destination::Third, Destination::Home][base],
        });
        force = base < 2 && before.bases[base + 1].is_some();
    }
    let mut credits = vec![
        credit(&batter, CreditKind::PlateAppearance),
        credit(&batter, CreditKind::Walk),
    ];
    for movement in &moves {
        if movement.to == Destination::Home {
            credits.push(credit(&movement.runner.player_id, CreditKind::Run));
        }
    }
    let runs = credits
        .iter()
        .filter(|credit| credit.kind == CreditKind::Run)
        .count() as u8;
    event(
        before,
        EventKind::PlateAppearance,
        Delta {
            moves,
            outs: 0,
            runs,
            credits,
            changes: vec![],
            third_out: None,
        },
    )
}

fn balk(before: &GameState) -> Event {
    let mut moves = Vec::new();
    let mut credits = vec![credit(
        before.teams[1 - before.half.offense()]
            .pitcher
            .active_id()
            .unwrap(),
        CreditKind::Balk,
    )];
    for (base, runner) in before.bases.iter().enumerate() {
        if let Some(runner) = runner {
            let to = [Destination::Second, Destination::Third, Destination::Home][base];
            moves.push(RunnerMove {
                runner: runner.clone(),
                from: [Base::First, Base::Second, Base::Third][base],
                to,
            });
            if to == Destination::Home {
                credits.push(credit(&runner.player_id, CreditKind::Run));
            }
        }
    }
    let runs = credits
        .iter()
        .filter(|credit| credit.kind == CreditKind::Run)
        .count() as u8;
    let mut event = event(
        before,
        EventKind::RunnerOnly(bbsim_engine::RunnerEvent::Balk),
        Delta {
            moves,
            outs: 0,
            runs,
            credits,
            changes: vec![],
            third_out: None,
        },
    );
    event.fielding_review = FieldingReview::NoMisplay;
    event.balk = Some(BalkFacts {
        cause: BalkCause::PitcherAction602a,
        disposition: BalkDisposition::PenaltyApplied,
        resolution: BalkResolution::ExactProtectedAwards,
    });
    event
}

fn advance(log: &mut Vec<EventTransition>, before: &mut GameState, make: fn(&GameState) -> Event) {
    let transition = apply(before, make(before)).unwrap();
    *before = transition.after().clone();
    log.push(transition);
}

fn strikeout(before: &GameState) -> Event {
    let mut action = out_pa(before);
    let batter = action.participants.batter.as_ref().unwrap().clone();
    action
        .delta
        .credits
        .push(credit(&batter, CreditKind::Strikeout));
    action
}

fn six_innings(cast: &Cast, known_hits: bool) -> (GameState, Vec<EventTransition>) {
    let mut before = cast.prepare(&state(1, Half::Top, true)).unwrap();
    let mut log = Vec::new();
    for _ in 0..36 {
        advance(
            &mut log,
            &mut before,
            if known_hits { strikeout } else { out_pa },
        );
    }
    assert_eq!((before.inning, before.half), (7, Half::Top));
    (before, log)
}

#[test]
fn individual_role_mean_controls_hooks_without_a_fixed_27_pa_target() {
    let low = Cast::read_tsv(&cast_text().replace(
        "workload:start:fierm001\t27,1",
        "workload:start:fierm001\t4,2",
    ))
    .unwrap();
    let high = cast();
    let mut before = low.prepare(&state(1, Half::Top, true)).unwrap();
    let mut log = Vec::new();
    advance(&mut log, &mut before, out_pa);
    assert_eq!(low.next_change(&before, &log).unwrap(), None);
    advance(&mut log, &mut before, out_pa);
    assert!(low.next_change(&before, &log).unwrap().is_some());
    assert_eq!(high.next_change(&before, &log).unwrap(), None);
}

#[test]
fn no_hit_extension_needs_known_hits_full_origin_and_stops_at_workload_limit() {
    let cast = Cast::read_tsv(&cast_text().replace(
        "workload:start:fierm001\t27,1",
        "workload:start:fierm001\t18,1",
    ))
    .unwrap();
    let (mut before, mut log) = six_innings(&cast, true);
    assert_eq!(cast.next_change(&before, &log).unwrap(), None);
    let (mut partial, mut segment) = (before.clone(), log[3..].to_vec());
    for _ in 0..6 {
        advance(&mut segment, &mut partial, strikeout);
    }
    assert!(cast.next_change(&partial, &segment).unwrap().is_some());
    let (unknown, unclassified) = six_innings(&cast, false);
    assert!(cast.next_change(&unknown, &unclassified).unwrap().is_some());
    for _ in 0..18 {
        advance(&mut log, &mut before, strikeout);
    }
    assert_eq!((before.inning, before.half), (10, Half::Top));
    assert!(cast.next_change(&before, &log).unwrap().is_some());
}

#[test]
fn loader_refuses_missing_selected_role_and_inconsistent_workload_denominators() {
    for value in ["0,0", "1,0", "1,2430", "4503599627370497,1"] {
        assert!(
            Cast::read_tsv(&cast_text().replace(
                "workload:start:fierm001\t27,1",
                &format!("workload:start:fierm001\t{value}")
            ))
            .is_err(),
            "{value}"
        );
    }
}

#[test]
fn individual_budget_is_not_shortened_again_by_performance_and_lead_extension_is_bounded() {
    let cast = cast();
    let mut before = cast.prepare(&state(1, Half::Top, true)).unwrap();
    let mut log = Vec::new();
    for _ in 0..7 {
        advance(&mut log, &mut before, walk);
    }
    assert_eq!(before.score, [4, 0]);
    advance(&mut log, &mut before, out_pa);
    assert_eq!(cast.next_change(&before, &log).unwrap(), None);
    advance(&mut log, &mut before, out_pa);
    assert_eq!(cast.next_change(&before, &log).unwrap(), None);
    // The same observed outing budget must not be shortened by a second removal process.
    let empirical = Cast::read_tsv(&empirical_text()).unwrap();
    assert_eq!(empirical.next_change(&before, &log).unwrap(), None);
    while log
        .iter()
        .filter(|t| t.event().participants.pitcher.as_deref() == Some("fierm001"))
        .count()
        < 26
    {
        advance(&mut log, &mut before, out_pa);
    }
    advance(&mut log, &mut before, walk);
    assert!(cast.next_change(&before, &log).unwrap().is_some());
    assert!(empirical.next_change(&before, &log).unwrap().is_some());
    let cast = Cast::read_tsv(&cast_text().replace(
        "workload:start:fierm001\t27,1",
        "workload:start:fierm001\t18,1",
    ))
    .unwrap();
    for lead in [3, 4] {
        let mut initial = state(1, Half::Top, true);
        initial.score[1] = lead;
        let mut before = cast.prepare(&initial).unwrap();
        let mut log = Vec::new();
        for _ in 0..36 {
            advance(&mut log, &mut before, out_pa);
        }
        assert_eq!(
            cast.next_change(&before, &log).unwrap().is_some(),
            lead == 3
        );
        if lead == 4 {
            for _ in 0..6 {
                advance(&mut log, &mut before, out_pa);
            }
            assert!(cast.next_change(&before, &log).unwrap().is_some());
        }
    }
}

fn empirical_text() -> String {
    let old = cast_text();
    let mut text = old
        .replace("closed-pitching-cast-v1", "closed-pitching-cast-v2")
        .replace("pa-mean-closed-cast-v1", "empirical-pa-closed-cast-v2");
    for line in old.lines().filter_map(|s| s.strip_prefix("workload:")) {
        let (key, value) = line.split_once('\t').unwrap();
        text.push_str(&format!(
            "usage:{key}\t{}\n",
            value.split(',').next().unwrap()
        ));
    }
    text
}

#[test]
fn empirical_sample_endpoints_reconcile_and_use_one_supplied_entry_word() {
    let text = empirical_text()
        .replace(
            "workload:start:fierm001\t27,1",
            "workload:start:fierm001\t6,3",
        )
        .replace(
            "usage:start:fierm001\t27\n",
            "usage:start:fierm001\t0,2,4\n",
        );
    let cast = Cast::read_tsv(&text).unwrap();
    assert_eq!(full_manager::POLICY_VERSION, "workload-context-2019-v3");
    assert!(cast.workload_mode().starts_with("empirical"));
    assert!(self::cast().workload_mode().contains("v1 fallback"));
    let mut before = cast.prepare(&state(1, Half::Top, true)).unwrap();
    let mut log = Vec::new();
    advance(&mut log, &mut before, out_pa);
    assert!(cast
        .next_change_with_word(&before, &log, 0)
        .unwrap()
        .is_some());
    assert_eq!(
        cast.next_change_with_word(&before, &log, u64::MAX).unwrap(),
        None
    );
    advance(&mut log, &mut before, out_pa);
    assert!(cast
        .next_change_with_word(&before, &log, 1 << 63)
        .unwrap()
        .is_some());
    assert_eq!(
        cast.next_change_with_word(&before, &log, u64::MAX).unwrap(),
        None
    );
    for bad in ["0,2", "0,2,5", "4,2,0", "-", "0,2,18446744073709551616"] {
        assert!(
            Cast::read_tsv(&text.replace(
                "usage:start:fierm001\t0,2,4",
                &format!("usage:start:fierm001\t{bad}")
            ))
            .is_err(),
            "{bad}"
        );
    }
    assert!(Cast::read_tsv(&text.replace("usage:start:fierm001\t0,2,4\n", "")).is_err());
    assert!(Cast::read_tsv(&text.replace("usage:start:fierm001", "usage:start:unknown")).is_err());
    assert!(
        Cast::read_tsv(&text.replace("closed-pitching-cast-v2", "closed-pitching-cast-v1"))
            .is_err()
    );
}

#[test]
fn loader_checks_pinned_provenance_structure_and_only_exposes_cast_names() {
    let text = tsv(
        &[
            ("gonzm005", "Marco Gonzales"),
            ("strih001", "Hunter Strickland"),
        ],
        &[("fierm001", "Michael Fiers"), ("dullr001", "Ryan Dull")],
    );
    let cast = Cast::read_tsv(&text).unwrap();
    assert_eq!(
        cast.names(),
        BTreeMap::from([
            ("gonzm005".to_owned(), "Marco Gonzales".to_owned()),
            ("strih001".to_owned(), "Hunter Strickland".to_owned()),
            ("fierm001".to_owned(), "Michael Fiers".to_owned()),
            ("dullr001".to_owned(), "Ryan Dull".to_owned()),
        ])
    );
    assert!(Cast::read_tsv(&text.replace(
        "availability_basis\trecorded nonstarters assumed available at start; not verified eligibility",
        "availability_basis\tverified eligible"
    ))
    .is_err());
    assert!(Cast::read_tsv(&format!("{text}unknown\tasserted\n")).is_err());
    assert!(Cast::read_tsv(&format!("{text}policy\tpa-mean-closed-cast-v1\n")).is_err());
    assert!(Cast::read_tsv(&text.replace("Marco Gonzales", "Unverified Player")).is_err());
    assert!(Cast::read_tsv(&text.replace("Marco Gonzales", "Away\u{1}Starter")).is_err());
    assert!(Cast::read_tsv(&text.replace(
        "pitcher:gonzm005\t0,0,0,0,0,0,0,0,0",
        "pitcher:gonzm005\t0,0"
    ))
    .is_err());
}

#[test]
fn prepare_adds_only_recorded_nonstarters_and_rejects_personnel_conflicts() {
    let cast = cast();
    let initial = state(1, Half::Top, true);
    let prepared = cast.prepare(&initial).unwrap();
    assert_eq!(prepared.teams[0].bench, ["strih001", "gearc001"]);
    assert_eq!(prepared.teams[1].bench, ["dullr001", "wendj001"]);
    assert_eq!(prepared.teams[0].removed, initial.teams[0].removed);
    assert_eq!(prepared.teams[1].removed, initial.teams[1].removed);
    assert_eq!(prepared.teams[0].pitcher, "gonzm005".into());
    assert_eq!(prepared.teams[1].pitcher, "fierm001".into());

    let mut mismatch = initial.clone();
    mismatch.teams[1].pitcher = "not_fierm001".into();
    assert!(cast.prepare(&mismatch).is_err());
    for membership in 0..4 {
        let mut conflict = initial.clone();
        match membership {
            0 => conflict.teams[0].bench.push("dullr001".into()),
            1 => conflict.teams[1].removed.push("dullr001".into()),
            2 => conflict.teams[0].batting_order[0] = "dullr001".into(),
            _ => {
                conflict.bases[0] = Some(Runner {
                    player_id: "dullr001".into(),
                    responsible_pitcher: "gonzm005".into(),
                })
            }
        }
        assert!(
            cast.prepare(&conflict).is_err(),
            "membership conflict {membership}"
        );
    }
}

#[test]
fn pitch_changes_use_logged_pa_only_keep_runner_charges_and_obey_relief_threshold() {
    let cast = cast();
    let mut initial = state(1, Half::Top, true);
    initial.score[1] = 100; // Isolate workload counting from the struggling-start hook.
    let mut before = cast.prepare(&initial).unwrap();
    let mut log = Vec::new();
    for _ in 0..26 {
        advance(&mut log, &mut before, walk);
    }
    let cursor_before_non_pa = before.teams[0].next_batter;
    let non_pa = apply(&before, balk(&before)).unwrap();
    before = non_pa.after().clone();
    log.push(non_pa);
    assert_eq!(cast.next_change(&before, &log).unwrap(), None);
    assert_eq!(before.teams[1].pitcher, "fierm001".into());
    assert_eq!(before.teams[0].next_batter, cursor_before_non_pa);
    assert_eq!(before.teams[0].next_batter, 8);

    advance(&mut log, &mut before, walk);
    let charged = before.bases.clone();
    let change = cast.next_change(&before, &log).unwrap().unwrap();
    assert_eq!(change.event().kind, EventKind::Substitution);
    assert_eq!(
        change.event().participants.pitcher.as_deref(),
        Some("fierm001")
    );
    assert_eq!(change.event().delta.credits, []);
    assert_eq!(change.event().delta.changes.len(), 1);
    assert_eq!(change.after().teams[1].pitcher, "dullr001".into());
    assert_eq!(
        change
            .after()
            .teams
            .iter()
            .map(|team| team.next_batter)
            .collect::<Vec<_>>(),
        before
            .teams
            .iter()
            .map(|team| team.next_batter)
            .collect::<Vec<_>>()
    );
    assert_eq!(change.after().bases, before.bases);
    assert_eq!(
        change.after().teams[1].next_batter,
        before.teams[1].next_batter
    );
    assert_eq!(change.after().teams[1].bench, ["wendj001"]);
    assert!(change.after().teams[1]
        .removed
        .contains(&"fierm001".to_owned()));
    assert!(charged
        .iter()
        .flatten()
        .all(|runner| runner.responsible_pitcher == "fierm001"));
    assert_eq!(
        charged
            .iter()
            .flatten()
            .map(|runner| (&runner.player_id, &runner.responsible_pitcher))
            .collect::<Vec<_>>(),
        change
            .after()
            .bases
            .iter()
            .flatten()
            .map(|runner| (&runner.player_id, &runner.responsible_pitcher))
            .collect::<Vec<_>>()
    );

    let mut relief_state = change.after().clone();
    log.push(change);
    for _ in 0..5 {
        advance(&mut log, &mut relief_state, walk);
    }
    let non_pa = apply(&relief_state, balk(&relief_state)).unwrap();
    relief_state = non_pa.after().clone();
    log.push(non_pa);
    assert_eq!(cast.next_change(&relief_state, &log).unwrap(), None);
    advance(&mut log, &mut relief_state, walk);
    let second_change = cast.next_change(&relief_state, &log).unwrap().unwrap();
    assert_eq!(second_change.after().teams[1].pitcher, "wendj001".into());
    assert!(second_change.after().teams[1].bench.is_empty());
}

#[test]
fn seventh_inning_boundary_alone_does_not_hook_and_exhaustion_keeps_current() {
    let cast = cast();
    let mut before = cast.prepare(&state(6, Half::Bottom, true)).unwrap();
    let mut log = Vec::new();
    for _ in 0..3 {
        advance(&mut log, &mut before, out_pa);
    }
    assert_eq!((before.inning, before.half), (7, Half::Top));
    assert_eq!(cast.next_change(&before, &log).unwrap(), None);

    let starters_only = Cast::read_tsv(&tsv(
        &[("gonzm005", "Marco Gonzales")],
        &[("fierm001", "Michael Fiers")],
    ))
    .unwrap();
    let mut exhausted = starters_only
        .prepare(&state(6, Half::Bottom, true))
        .unwrap();
    let mut exhausted_log = Vec::new();
    for _ in 0..3 {
        advance(&mut exhausted_log, &mut exhausted, out_pa);
    }
    assert_eq!(
        starters_only
            .next_change(&exhausted, &exhausted_log)
            .unwrap(),
        None
    );
    assert_eq!(exhausted.teams[1].pitcher, "fierm001".into());
}

#[test]
fn no_dh_pitcher_change_replaces_the_pitching_batting_slot() {
    let cast = Cast::read_tsv(&cast_text().replace(
        "workload:start:fierm001\t27,1",
        "workload:start:fierm001\t1,1",
    ))
    .unwrap();
    let mut before = cast.prepare(&state(1, Half::Top, false)).unwrap();
    let mut log = Vec::new();
    for _ in 0..2 {
        advance(&mut log, &mut before, walk);
    }
    let transition = cast.next_change(&before, &log).unwrap().unwrap();
    assert_eq!(transition.after().teams[1].pitcher, "dullr001".into());
    assert_eq!(transition.after().teams[1].batting_order[8], "dullr001");
    assert_eq!(transition.event().delta.changes.len(), 2);
    assert!(transition
        .event()
        .delta
        .changes
        .iter()
        .any(|change| matches!(
            change,
            PersonnelChange::BattingSlot { team: TeamSide::Home, slot: 8, outgoing, incoming }
                if outgoing == "fierm001" && incoming == "dullr001"
        )));
}

fn managed_text() -> String {
    let mut text = generic_cast()
        .replace("closed-pitching-cast-v3", "season-pool-pitching-cast-v1")
        .replace("recorded nonstarters assumed available at start; not verified eligibility", "observed season club players assumed available; no daily injury/minor-league reconstruction")
        .replace("literal terminal-PA participants per pitcher-game appearance; not pitch counts, official BF or fatigue", "source empirical role outings; explicitly marked pooled role priors when unavailable");
    text += "season_policy\tseason-pool-soft-usage-v1\nstarter_scale:Away\t80\nstarter_scale:Home\t100\n";
    for id in ["ap", "hp", "jay-j001"] {
        text += &format!("relief_pace:{id}\t0\n");
    }
    text
}
#[test]
fn season_pool_cast_schema_is_explicit_and_strict() {
    let text = managed_text();
    assert!(Cast::read_complete_tsv(&text).is_ok());
    // Unselected roles may stay genuinely unsupported, without fabricated outings.
    assert!(Cast::read_complete_tsv(
        &text
            .replace("workload:relief:hp\t6,1", "workload:relief:hp\t0,0")
            .replace("usage:relief:hp\t6", "usage:relief:hp\t-")
    )
    .is_ok());
    assert!(
        Cast::read_complete_tsv(&(text.clone() + "budget_prior:relief:jay-j001\ttrue\n")).is_ok()
    );
    for bad in [
        text.replace("season_policy\tseason-pool-soft-usage-v1\n", ""),
        text.replace("starter_scale:Away\t80\n", ""),
        text.replace("starter_scale:Away\t80", "starter_scale:Away\t080"),
        text.replace("starter_scale:Home\t100", "starter_scale:Home\t81"),
        text.replace("relief_pace:hp\t0\n", ""),
        text.replace("relief_pace:hp\t0", "relief_pace:hp\t+1"),
        text.replace("relief_pace:hp\t0", "relief_pace:hp\t2"),
        text.clone() + "relief_pace:unknown\t0\n",
        text.clone() + "budget_prior:relief:jay-j001\tfalse\n",
        text.clone() + "budget_prior:start:unknown\ttrue\n",
        text.clone() + "starter_scale:Away\t80\n",
        text.clone() + "unknown\ttrue\n",
        text.replace("usage:start:hp\t27", "usage:start:hp\t26"),
        text.replace("workload:start:hp\t27,1", "workload:start:hp\t0,0")
            .replace("usage:start:hp\t27", "usage:start:hp\t-"),
        generic_cast() + "starter_scale:Away\t80\n",
        generic_cast() + "budget_prior:relief:jay-j001\ttrue\n",
        generic_cast() + "relief_pace:jay-j001\t0\n",
        text.clone() + "budget_prior:other:jay-j001\ttrue\n",
        text.clone() + "budget_prior:relief:jay-j001\ttrue\nbudget_prior:relief:jay-j001\ttrue\n",
        text.replace(season_pool::AVAILABILITY, AVAILABILITY_FOR_TEST),
        text.replace(season_pool::WORKLOAD_BASIS, "literal terminal-PA participants per pitcher-game appearance; not pitch counts, official BF or fatigue"),
    ] {
        assert!(Cast::read_complete_tsv(&bad).is_err(), "accepted {bad}");
    }
}

fn managed_fixture_text() -> String {
    let mut text = empirical_text()
        .lines()
        .filter(|line| {
            !line.starts_with("bullpen_report_sha256\t")
                && !line.starts_with("workload_report_sha256\t")
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    text = text.replace("closed-pitching-cast-v2", "season-pool-pitching-cast-v1").replace(AVAILABILITY_FOR_TEST, season_pool::AVAILABILITY).replace("literal terminal-PA participants per pitcher-game appearance; not pitch counts, official BF or fatigue",season_pool::WORKLOAD_BASIS);
    text += "game_id\tOAK201903200\ncast_sizes\t3,3\nworkload_recipe_sha256\t512f542cee70c5824c6fff92f1278b51aad2be6a5a066db18b306091a3fae48f\noutcome_events_sha256\tea3fd660ac87deb02c61cddf28977db59f85f9a88f77aaabc0c07ba0f1831d1a\nseason_policy\tseason-pool-soft-usage-v1\nstarter_scale:Away\t100\nstarter_scale:Home\t80\n";
    for id in [
        "gonzm005", "strih001", "gearc001", "fierm001", "dullr001", "wendj001",
    ] {
        text += &format!("relief_pace:{id}\t0\n");
    }
    text
}
const AVAILABILITY_FOR_TEST: &str =
    "recorded nonstarters assumed available at start; not verified eligibility";

fn v2_manager_cast_text() -> String {
    let text = managed_text()
        .replace(
            "season-pool-pitching-cast-v1",
            "season-pool-pitching-cast-v2",
        )
        .replace("season-pool-soft-usage-v1", "season-pool-pitcher-usage-v2");
    text.lines()
        .map(|line| match line.strip_prefix("relief_pace:") {
            Some(rest) => format!("relief_usage:{}\t0,?", rest.split_once('\t').unwrap().0),
            None => line.to_owned(),
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\nclub_clock:Away\t0,161\nclub_clock:Home\t0,162\n"
}

#[test]
fn v2_cast_uses_the_shared_selector_and_exact_pinned_workload_sample() {
    let cast = Cast::read_tsv(&v2_manager_cast_text()).unwrap();
    let initial = GameState {
        teams: [team("a", "ap", true), team("h", "hp", true)],
        ..state(1, Half::Bottom, true)
    };
    let before = cast.prepare(&initial).unwrap();
    let profiles = cast.v2_candidates(0).unwrap();
    assert_eq!(profiles.len(), 2);
    assert_eq!(profiles[0].id, "ap");
    assert!(profiles[0].starter);
    assert_eq!(profiles[0].usage.used(), 0);
    assert_eq!(profiles[0].usage.target(), None);
    assert_eq!((profiles[0].pa_sum, profiles[0].outings), (6, 1));
    assert_eq!(profiles[1].id, "jay-j001");
    assert!(!profiles[1].starter);

    let selection = cast.select_v2(&before, 0).unwrap();
    assert_eq!(selection.chosen(), Some("jay-j001"));
    let budget = cast.v2_budget(&before, 0, &[], u64::MAX).unwrap();
    assert_eq!(budget.word, u64::MAX);
    assert_eq!(budget.sample_index, 0);
    assert_eq!(budget.base, 27);
    assert_eq!(budget.scale, 80);
    assert_eq!(budget.final_budget, 21);
}

#[test]
fn v2_hooks_restore_vacant_pitchers_and_validate_trace_history() {
    use bbsim_engine::PitcherRole;

    let cast = Cast::read_tsv(&v2_manager_cast_text()).unwrap();
    let initial = GameState {
        teams: [team("a", "ap", true), team("h", "hp", true)],
        ..state(1, Half::Top, true)
    };
    let mut before = cast.prepare(&initial).unwrap();
    let mut log = Vec::new();
    for _ in 0..84 {
        advance(&mut log, &mut before, out_pa);
    }
    let hook = cast
        .next_change_v2(&before, &log, u64::MAX)
        .unwrap()
        .unwrap();
    assert_eq!(hook.budget.final_budget, 27);
    assert_eq!(hook.selection.chosen(), None);
    assert!(hook.transition.is_none());

    let trace_log = vec![apply(&cast.prepare(&initial).unwrap(), out_pa(&initial)).unwrap()];
    let trace = pitcher_entry_v2::header("LAN201904010", 17, 1, 0).unwrap();
    assert!(cast
        .validate_v2_trace(&trace, "LAN201904010", 17, &trace_log)
        .unwrap()
        .is_empty());

    let non_dh = GameState {
        teams: [team("a", "ap", false), team("h", "hp", false)],
        ..state(1, Half::Bottom, false)
    };
    let mut vacancy = cast.prepare(&non_dh).unwrap();
    vacancy.teams[0].pitcher = PitcherRole::Vacant { batting_slot: 8 };
    vacancy.teams[0].batting_order[8] = "aph".into();
    vacancy.teams[0].removed.push("ap".into());
    vacancy.teams[0].defense =
        bbsim_engine::DefenseState::Assigned(Box::new(std::array::from_fn(|position| {
            (position > 0).then(|| format!("a{}", position - 1))
        })));
    let (selection, restoration) = cast.restore_pitcher_v2(&vacancy).unwrap().unwrap();
    assert_eq!(selection.chosen(), Some("jay-j001"));
    assert_eq!(
        restoration.after().teams[0].pitcher.active_id(),
        Some("jay-j001")
    );
}

#[test]
fn v2_clock_and_cast_name_helpers_enforce_cross_file_bindings() {
    let cast = Cast::read_tsv(&v2_manager_cast_text()).unwrap();
    cast.validate_v2_clocks(["CHA", "OAK"]).unwrap();
    let ana_cast = Cast::read_tsv(
        &v2_manager_cast_text().replace("club_clock:Away\t0,161", "club_clock:Away\t0,162"),
    )
    .unwrap();
    ana_cast.validate_v2_clocks(["ANA", "OAK"]).unwrap();
    assert_eq!(
        season_pool::scheduled_games("LAA").unwrap_err(),
        "unsupported 2019 club clock"
    );
    assert_eq!(
        cast.validate_v2_clocks(["SEA", "OAK"]).unwrap_err(),
        "v2 club clock does not match lineup schedule"
    );

    let mut names = BTreeMap::from([("ap".to_owned(), "Starter A".to_owned())]);
    cast.merge_names(&mut names).unwrap();
    assert_eq!(names["ap"], "Starter A");
    assert_eq!(names.len(), 3);

    let mut mismatched = BTreeMap::from([("ap".to_owned(), "Different name".to_owned())]);
    assert_eq!(
        cast.merge_names(&mut mismatched).unwrap_err(),
        "lineup/cast display-name mismatch"
    );
}

#[test]
fn managed_relief_selection_prioritizes_role_class_then_pace_then_stable_id() {
    let text = managed_fixture_text()
        .replace(
            "workload:relief:gearc001\t6,1",
            "workload:relief:gearc001\t11,2",
        )
        .replace("usage:relief:gearc001\t6", "usage:relief:gearc001\t5,6")
        .replace("relief_pace:gearc001\t0", "relief_pace:gearc001\t1")
        .replace("relief_pace:strih001\t0", "relief_pace:strih001\t-1");
    let cast = Cast::read_complete_tsv(&text).unwrap();
    assert!(cast.is_managed());
    assert_eq!(
        cast.policy_version(),
        "season-pool-workload-context-2019-v1"
    );
    for (inning, score, expected) in [
        (5, [0, 0], "strih001"),
        (6, [0, 0], "gearc001"),
        (8, [0, 3], "gearc001"),
        (8, [4, 0], "strih001"),
        (8, [0, 4], "strih001"),
    ] {
        let mut initial = state(inning, Half::Bottom, true);
        initial.score = score;
        let mut before = cast.prepare(&initial).unwrap();
        assert_eq!(cast.next_available(&before, 0), Some(expected));
        before.teams[0].bench.retain(|id| id != expected);
        before.teams[0].removed.push(expected.into());
        let other = if expected == "strih001" {
            "gearc001"
        } else {
            "strih001"
        };
        assert_eq!(cast.next_available(&before, 0), Some(other));
    }
    // Same class: pace wins; same pace: lexical ID wins even against supplied rank.
    let tied = managed_fixture_text();
    let cast = Cast::read_tsv(&tied).unwrap();
    let before = cast.prepare(&state(4, Half::Bottom, true)).unwrap();
    assert_eq!(cast.next_available(&before, 0), Some("gearc001"));
    let paced = Cast::read_tsv(&tied.replace("relief_pace:strih001\t0", "relief_pace:strih001\t1"))
        .unwrap();
    assert_eq!(paced.next_available(&before, 0), Some("strih001"));
    assert_eq!(self::cast().next_available(&before, 0), Some("strih001")); // legacy rank
                                                                           // Normal hook uses precisely the same choice.
    let (before, log) = six_innings(
        &Cast::read_tsv(
            &tied
                .replace(
                    "workload:start:fierm001\t27,1",
                    "workload:start:fierm001\t18,1",
                )
                .replace("usage:start:fierm001\t27", "usage:start:fierm001\t18"),
        )
        .unwrap(),
        false,
    );
    let low = Cast::read_tsv(
        &tied
            .replace(
                "workload:start:fierm001\t27,1",
                "workload:start:fierm001\t18,1",
            )
            .replace("usage:start:fierm001\t27", "usage:start:fierm001\t18"),
    )
    .unwrap();
    let change = low.next_change(&before, &log).unwrap().unwrap();
    assert_eq!(
        change.after().teams[1].pitcher.active_id(),
        low.next_available(&before, 1)
    );
}

#[test]
fn managed_starter_scale_floors_at_one_before_existing_bounded_extensions_and_obligations() {
    use bbsim_engine::{PitcherObligation, PitcherRole};
    let text = managed_fixture_text();
    let scaled = Cast::read_tsv(&text).unwrap();
    let full =
        Cast::read_tsv(&text.replace("starter_scale:Home\t80", "starter_scale:Home\t100")).unwrap();
    let (mut before, mut log) = six_innings(&scaled, false);
    for _ in 0..6 {
        advance(&mut log, &mut before, out_pa);
    }
    // floor(27*0.8)=21; unknown hit coverage cannot grant no-hit extension.
    assert!(scaled.next_change(&before, &log).unwrap().is_some());
    assert!(full.next_change(&before, &log).unwrap().is_none());
    for obligation in [
        PitcherObligation::StartingBatter,
        PitcherObligation::ReliefBatter,
    ] {
        let mut held = before.clone();
        held.teams[1].pitcher = PitcherRole::Active {
            player_id: "fierm001".into(),
            obligation,
        };
        assert!(scaled.next_change(&held, &log).unwrap().is_none());
    }
    let tiny = Cast::read_tsv(
        &text
            .replace(
                "workload:start:fierm001\t27,1",
                "workload:start:fierm001\t1,1",
            )
            .replace("usage:start:fierm001\t27", "usage:start:fierm001\t1"),
    )
    .unwrap();
    let mut before = tiny.prepare(&state(1, Half::Top, true)).unwrap();
    let mut log = Vec::new();
    assert!(tiny.next_change(&before, &log).unwrap().is_none());
    advance(&mut log, &mut before, out_pa);
    assert!(tiny.next_change(&before, &log).unwrap().is_some());
    for (known, lead, target) in [(false, 4, 24), (true, 0, 41)] {
        let mut initial = state(1, Half::Top, true);
        initial.score[1] = lead;
        // Only a complete zero-score origin qualifies for the no-hit extension.
        let mut before = scaled.prepare(&initial).unwrap();
        let mut log = Vec::new();
        loop {
            let n = log
                .iter()
                .filter(|t: &&EventTransition| {
                    t.event().participants.pitcher.as_deref() == Some("fierm001")
                })
                .count();
            if before.half == Half::Top && n >= target {
                break;
            }
            advance(
                &mut log,
                &mut before,
                if known { strikeout } else { out_pa },
            );
            if before.half == Half::Top && n >= 21 && n < target - 1 {
                assert!(scaled.next_change(&before, &log).unwrap().is_none());
            }
        }
        assert!(
            scaled.next_change(&before, &log).unwrap().is_some(),
            "known={known} lead={lead}"
        );
    }
}
