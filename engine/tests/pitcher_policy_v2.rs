#[path = "../examples/support/pitcher_policy_v2.rs"]
mod policy;

use policy::{relief_rank, Clock, ReliefUsage};
use std::cmp::Ordering;

fn rank(
    used: u32,
    target: Option<u32>,
    pa: u32,
    outings: u32,
    inning: u32,
    gap: i64,
) -> policy::Rank {
    let usage = ReliefUsage::read(&format!(
        "{used},{}",
        target.map_or("?".into(), |t| t.to_string())
    ))
    .unwrap();
    relief_rank(
        Clock::read("80,162").unwrap(),
        usage,
        pa,
        outings,
        inning,
        gap,
    )
    .unwrap()
}

fn equal_ratio(actual: (i128, i128), expected: (i128, i128)) {
    assert!(actual.1 > 0);
    assert_eq!(actual.0 * expected.1, expected.0 * actual.1);
}

#[test]
fn canonical_wire_values_preserve_unknown_zero_and_integer_outs() {
    let clock = Clock::read("80,162").unwrap();
    let known = relief_rank(clock, ReliefUsage::read("17,60").unwrap(), 5, 1, 6, 3).unwrap();
    assert!(known.target_known());
    equal_ratio(known.debt(), (13, 6));
    equal_ratio(known.clipped(), (1, 1));
    equal_ratio(known.score(), (5, 4));
    let unknown = rank(17, None, 5, 1, 6, 3);
    assert!(!unknown.target_known());
    equal_ratio(unknown.debt(), (0, 1));
    equal_ratio(unknown.score(), (1, 4));
    let zero = rank(17, Some(0), 5, 1, 6, 3);
    assert!(zero.target_known());
    equal_ratio(zero.debt(), (-17, 6));
    equal_ratio(zero.score(), (-3, 4));
}

#[test]
fn wire_rejects_noncanonical_missing_extra_and_overflowing_cells() {
    for bad in [
        "",
        "0",
        "0,",
        ",162",
        "0,162,1",
        "0,0",
        "162,162",
        "2,1",
        "0,163",
        "0,?",
        "?,162",
        "00,162",
        "+0,162",
        "-1,162",
        "0,0162",
        "0,162\n",
        "0, 162",
        "0,１６２",
        "4294967296,162",
    ] {
        assert!(Clock::read(bad).is_err(), "accepted clock {bad:?}");
    }
    for bad in [
        "",
        "0",
        "0,",
        ",0",
        "0,0,0",
        "?,0",
        "?,?",
        "0,-1",
        "-1,0",
        "0,+1",
        "00,0",
        "0,01",
        "0,? ",
        "0,0\n",
        "0, 0",
        "0,٠",
        "4294967296,0",
        "0,4294967296",
    ] {
        assert!(ReliefUsage::read(bad).is_err(), "accepted usage {bad:?}");
    }
}

#[test]
fn first_and_final_clocks_use_the_upcoming_opportunity() {
    for (text, expected) in [
        ("0,162", (1, 162)),
        ("160,161", (161, 161)),
        ("161,162", (162, 162)),
    ] {
        let r = relief_rank(
            Clock::read(text).unwrap(),
            ReliefUsage::read("0,6").unwrap(),
            6,
            1,
            5,
            0,
        )
        .unwrap();
        equal_ratio(r.debt(), expected);
    }
}

#[test]
fn debt_can_overrule_the_preferred_workload_class() {
    let overused_match = rank(60, Some(60), 6, 1, 5, 0);
    let balanced_mismatch = rank(30, Some(60), 5, 1, 5, 0);
    equal_ratio(overused_match.score(), (-3, 4));
    equal_ratio(balanced_mismatch.score(), (0, 1));
    assert_eq!(
        balanced_mismatch.compare(&overused_match),
        Ordering::Greater
    );
}

#[test]
fn boundaries_use_exact_class_and_situation_comparisons() {
    for (inning, gap, wants_long) in [
        (5, 0, true),
        (6, 3, false),
        (6, -3, false),
        (6, 4, true),
        (6, -4, true),
    ] {
        for (pa, outings, long) in [(11, 2, false), (12, 2, true), (13, 2, true)] {
            let r = rank(30, Some(60), pa, outings, inning, gap);
            assert_eq!(r.long(), long);
            assert_eq!(r.matches(), long == wants_long);
            equal_ratio(r.score(), (i128::from(long == wants_long), 4));
        }
    }
}

#[test]
fn unclipped_debt_breaks_saturated_score_ties_in_both_directions() {
    let a = rank(0, Some(120), 6, 1, 5, 0);
    let b = rank(1, Some(120), 6, 1, 5, 0);
    equal_ratio(a.score(), b.score());
    assert_eq!(a.compare(&b), Ordering::Greater);
    let a = rank(90, Some(60), 6, 1, 5, 0);
    let b = rank(91, Some(60), 6, 1, 5, 0);
    equal_ratio(a.score(), b.score());
    assert_eq!(a.compare(&b), Ordering::Greater);
}

#[test]
fn equivalent_ratios_tie_without_rounding_or_denominator_bias() {
    let a = relief_rank(
        Clock::read("80,162").unwrap(),
        ReliefUsage::read("0,60").unwrap(),
        6,
        1,
        5,
        0,
    )
    .unwrap();
    let b = relief_rank(
        Clock::read("0,1").unwrap(),
        ReliefUsage::read("0,30").unwrap(),
        12,
        2,
        5,
        0,
    )
    .unwrap();
    assert_eq!(a.compare(&b), Ordering::Equal);
    assert_eq!(a.compare(&a), Ordering::Equal);
}

#[test]
fn invalid_workload_and_situation_are_not_silently_defaulted() {
    let c = Clock::read("0,162").unwrap();
    let u = ReliefUsage::read("0,?").unwrap();
    for (pa, outings, inning, gap) in [
        (0, 0, 1, 0),
        (6, 0, 1, 0),
        (6, 1, 0, 0),
        (6, 1, 1, i64::MIN),
        (6, 1, 1, i64::MAX),
        (6, 1, 1, i64::from(u32::MAX) + 1),
        (6, 1, 1, -i64::from(u32::MAX) - 1),
    ] {
        assert!(relief_rank(c, u, pa, outings, inning, gap).is_err());
    }
    for gap in [i64::from(u32::MAX), -i64::from(u32::MAX)] {
        assert!(relief_rank(c, u, u32::MAX, u32::MAX, u32::MAX, gap).is_ok());
    }
}

#[test]
fn maximum_raw_counts_and_comparisons_fit_checked_wide_arithmetic() {
    let mut ranks = Vec::new();
    for clock in ["0,1", "0,162", "161,162"] {
        for usage in [
            "4294967295,0",
            "0,4294967295",
            "4294967295,4294967295",
            "4294967295,?",
        ] {
            for pa in [0, u32::MAX] {
                let r = relief_rank(
                    Clock::read(clock).unwrap(),
                    ReliefUsage::read(usage).unwrap(),
                    pa,
                    u32::MAX,
                    u32::MAX,
                    -i64::from(u32::MAX),
                )
                .unwrap();
                assert!(r.debt().0.abs() < 1_i128 << 42);
                assert!(r.debt().1 < 1_i128 << 40);
                ranks.push(r);
            }
        }
    }
    for a in &ranks {
        for b in &ranks {
            assert_eq!(a.compare(b), b.compare(a).reverse());
            let (an, ad) = a.score();
            let (bn, bd) = b.score();
            assert!(an.checked_mul(bd).unwrap().abs() < 1_i128 << 85);
            assert!(bn.checked_mul(ad).unwrap().abs() < 1_i128 << 85);
        }
    }
}

#[test]
fn thousands_of_cases_match_an_independent_piecewise_ratio_reference() {
    let mut cases = 0;
    for (c, n) in [(0, 162), (80, 162), (160, 161), (161, 162)] {
        for target in [None, Some(0), Some(1), Some(60), Some(600)] {
            for used in [0, 1, 6, 30, 60, 300, 1000] {
                for (pa, outings) in [(0, 1), (11, 2), (12, 2)] {
                    for (inning, gap) in [(5, 0), (6, 3), (6, -4), (9, 4)] {
                        let r = relief_rank(
                            Clock::read(&format!("{c},{n}")).unwrap(),
                            ReliefUsage::read(&format!(
                                "{used},{}",
                                target.map_or("?".into(), |t| t.to_string())
                            ))
                            .unwrap(),
                            pa,
                            outings,
                            inning,
                            gap,
                        )
                        .unwrap();
                        let (dn, dd) = match target {
                            None => (0, 1),
                            Some(t) => {
                                let expected_n = i128::from(t) * i128::from(c + 1);
                                let expected_d = i128::from(n);
                                let difference = expected_n - i128::from(used) * expected_d;
                                if expected_n <= 30 * expected_d {
                                    (difference, 6 * expected_d)
                                } else {
                                    (5 * difference, expected_n)
                                }
                            }
                        };
                        equal_ratio(r.debt(), (dn, dd));
                        let clipped = if dn < -dd {
                            (-1, 1)
                        } else if dn > dd {
                            (1, 1)
                        } else {
                            (dn, dd)
                        };
                        equal_ratio(r.clipped(), clipped);
                        let matching = (pa >= 6 * outings) == (inning <= 5 || gap.abs() >= 4);
                        equal_ratio(
                            r.score(),
                            (
                                4 * clipped.0 + i128::from(matching) * clipped.1,
                                4 * clipped.1,
                            ),
                        );
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 1680);
}
