#[path = "../examples/support/player_calibration.rs"]
mod calibration;
use calibration::{fit, Model, Pair};

fn confounded() -> Vec<Pair> {
    vec![
        Pair {
            batter: 0,
            pitcher: 0,
            counts: [100; 9],
        },
        Pair {
            batter: 0,
            pitcher: 1,
            counts: [20, 10, 10, 10, 10, 10, 10, 10, 10],
        },
        Pair {
            batter: 1,
            pitcher: 0,
            counts: [10; 9],
        },
        Pair {
            batter: 1,
            pitcher: 1,
            counts: [180, 90, 90, 90, 90, 90, 90, 90, 90],
        },
    ]
}

#[test]
fn neutral_fit_is_exact_and_cold_start_stays_neutral() {
    let league = [1.0 / 9.0; 9];
    let fitted = fit(
        &[Pair {
            batter: 0,
            pitcher: 0,
            counts: [10; 9],
        }],
        2,
        2,
        league,
        [1.0, 1.0],
        5000,
        1e-8,
    )
    .unwrap();
    for b in 0..2 {
        for p in 0..2 {
            let q = fitted.model.probabilities(b, p);
            assert!(q.iter().zip(league).all(|(a, b)| (a - b).abs() < 1e-12));
        }
    }
}

#[test]
fn joint_fit_removes_opponent_mix_confounding_and_satisfies_pseudo_margin_identity() {
    let league = [1.0 / 9.0; 9];
    let pairs = confounded();
    let fitted = fit(&pairs, 3, 3, league, [1.0, 1.0], 5000, 1e-8).unwrap();
    assert!(fitted.iterations > 1);
    assert!(fitted.max_correction <= 1e-8);
    assert_eq!(fitted.model.batters[2], league);
    assert_eq!(fitted.model.pitchers[2], league);
    assert!((fitted.model.batters[0][0] - fitted.model.batters[1][0]).abs() < 0.003);
    assert!((fitted.model.probabilities(0, 0)[0] - 1.0 / 9.0).abs() < 0.003);
    assert!((fitted.model.probabilities(1, 1)[0] - 0.2).abs() < 0.003);
    for role in 0..2 {
        for id in 0..3 {
            let mut observed = [0.0; 9];
            let mut expected = [0.0; 9];
            for pair in pairs.iter().filter(|p| {
                if role == 0 {
                    p.batter == id
                } else {
                    p.pitcher == id
                }
            }) {
                let n = pair.counts.iter().sum::<u64>() as f64;
                let q = fitted.model.probabilities(pair.batter, pair.pitcher);
                for k in 0..9 {
                    observed[k] += pair.counts[k] as f64;
                    expected[k] += n * q[k];
                }
            }
            let neutral = if role == 0 {
                fitted.model.batters[id]
            } else {
                fitted.model.pitchers[id]
            };
            for k in 0..9 {
                assert!((expected[k] + neutral[k] - observed[k] - league[k]).abs() < 1e-5);
            }
        }
    }
}

#[test]
fn zeros_stay_positive_repeat_order_invariance_and_failed_fit_refuses() {
    let league = [1.0 / 9.0; 9];
    let pair = Pair {
        batter: 0,
        pitcher: 0,
        counts: [30, 0, 0, 20, 0, 0, 0, 0, 50],
    };
    let fitted = fit(&[pair], 1, 1, league, [1.0, 1.0], 5000, 1e-8).unwrap();
    let q = fitted.model.probabilities(0, 0);
    assert!(q[0] > 0.25);
    assert!(q.iter().all(|x| x.is_finite() && *x > 0.0 && *x < 1.0));
    assert!((q.iter().sum::<f64>() - 1.0).abs() < 1e-12);
    let pairs = confounded();
    let mut reversed = pairs.clone();
    reversed.reverse();
    let a = fit(&pairs, 2, 2, league, [5.0, 20.0], 5000, 1e-8).unwrap();
    let b = fit(&reversed, 2, 2, league, [5.0, 20.0], 5000, 1e-8).unwrap();
    for i in 0..2 {
        for j in 0..2 {
            assert!(a
                .model
                .probabilities(i, j)
                .iter()
                .zip(b.model.probabilities(i, j))
                .all(|(x, y)| (x - y).abs() < 1e-12));
        }
    }
    assert!(fit(&pairs, 2, 2, league, [1.0, 1.0], 1, 1e-8).is_err());
    assert!(fit(&pairs, 1, 2, league, [1.0, 1.0], 5000, 1e-8).is_err());
    assert!(fit(&pairs, 2, 2, league, [0.0, 1.0], 5000, 1e-8).is_err());
    assert!(fit(&pairs, 2, 2, [0.0; 9], [1.0, 1.0], 5000, 1e-8).is_err());
    assert!(Model::from_counts(&pairs, 2, 2, [1.0, 1.0]).is_ok());
}
