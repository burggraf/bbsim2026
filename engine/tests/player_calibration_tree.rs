#[path = "../examples/support/player_calibration.rs"]
mod calibration;
#[path = "../examples/support/player_calibration_tree.rs"]
mod tree;
use calibration::{fit, Pair};

#[test]
fn binary_opponent_fit_and_cold_start_obey_the_same_augmented_likelihood() {
    let pairs = vec![
        Pair::<2> {
            batter: 0,
            pitcher: 0,
            counts: [100, 800],
        },
        Pair::<2> {
            batter: 0,
            pitcher: 1,
            counts: [20, 80],
        },
        Pair::<2> {
            batter: 1,
            pitcher: 0,
            counts: [10, 80],
        },
        Pair::<2> {
            batter: 1,
            pitcher: 1,
            counts: [180, 720],
        },
    ];
    let marginal = calibration::Model::from_counts(&pairs, 3, 3, [1.0, 1.0]).unwrap();
    assert_eq!(marginal.batters[2], marginal.league);
    let fitted = fit(&pairs, 3, 3, [0.15, 0.85], [1.0, 1.0], 5000, 1e-8).unwrap();
    assert!(fitted.iterations > 0 && fitted.max_correction <= 1e-8);
    assert!((fitted.model.batters[0][0] - fitted.model.batters[1][0]).abs() < 0.003);
    assert_eq!(fitted.model.batters[2], [0.15, 0.85]);
    assert_eq!(fitted.model.pitchers[2], [0.15, 0.85]);
    for p in &pairs {
        let q = fitted.model.probabilities(p.batter, p.pitcher);
        assert!((q[0] + q[1] - 1.0).abs() < 1e-12);
        assert!(q.iter().all(|x| *x > 0.0 && *x < 1.0));
    }
}

#[test]
fn tree_is_one_exact_multinomial_vector_not_a_second_hit_out_roll() {
    let rates = [0.2, 0.1, 0.05, 0.3, 0.2, 0.01, 0.2, 0.02];
    let model = tree::Tree::neutral(rates, 1, 1).unwrap();
    let q = model.probabilities(0, 0);
    let expected = [
        0.2, 0.08, 0.036, 0.13001472, 0.03250368, 0.0016416, 0.04104, 0.009576, 0.469224,
    ];
    for (x, y) in q.into_iter().zip(expected) {
        assert!((x - y).abs() < 1e-12);
    }
    assert!((q.iter().sum::<f64>() - 1.0).abs() < 1e-12);
    assert!(q.iter().all(|x| *x > 0.0 && *x < 1.0));
    assert!(tree::league_nodes(q)
        .unwrap()
        .iter()
        .zip(rates)
        .all(|(x, y)| (*x - y).abs() < 1e-12));
    let mut changed = model.clone();
    changed.nodes[4].batters[0] = [0.4, 0.6];
    let other = changed.probabilities(0, 0);
    assert!(other[6] > q[6]);
    assert!(
        ((3..=6).map(|k| q[k]).sum::<f64>() - (3..=6).map(|k| other[k]).sum::<f64>()).abs() < 1e-12
    );
    assert_eq!(other[8], q[8]);
    for bad in [0.0, 1.0, f64::NAN] {
        let mut r = rates;
        r[2] = bad;
        assert!(tree::Tree::neutral(r, 1, 1).is_err());
    }
}

#[test]
fn conditional_binary_losses_decompose_the_complete_multinomial_likelihood() {
    let model = tree::Tree::neutral([0.2, 0.1, 0.05, 0.3, 0.2, 0.01, 0.2, 0.02], 1, 1).unwrap();
    for counts in [[1, 2, 3, 4, 5, 6, 7, 8, 9], [10, 0, 0, 0, 0, 0, 0, 0, 0]] {
        let pair = Pair {
            batter: 0,
            pitcher: 0,
            counts,
        };
        let q = model.probabilities(0, 0);
        let multinomial = -counts
            .iter()
            .zip(q)
            .map(|(n, p)| *n as f64 * p.ln())
            .sum::<f64>();
        let mut binary = 0.0;
        for node in 0..8 {
            if let Some(r) = tree::risk(&pair, node).unwrap() {
                let p = model.nodes[node].probabilities(0, 0);
                binary -= r
                    .counts
                    .iter()
                    .zip(p)
                    .map(|(n, x)| *n as f64 * x.ln())
                    .sum::<f64>();
            }
        }
        assert!((multinomial - binary).abs() < 1e-12);
    }
}
#[test]
fn all_nine_source_categories_have_exact_conditional_risk_sets() {
    let p = Pair {
        batter: 2,
        pitcher: 3,
        counts: [1, 2, 3, 4, 5, 6, 7, 8, 9],
    };
    let expected = [
        [1, 44],
        [2, 42],
        [3, 39],
        [22, 17],
        [7, 15],
        [6, 9],
        [5, 4],
        [8, 9],
    ];
    for (node, counts) in expected.into_iter().enumerate() {
        let r = tree::risk(&p, node).unwrap().unwrap();
        assert_eq!(r.counts, counts);
        assert_eq!((r.batter, r.pitcher), (2, 3));
    }
    assert!(tree::risk(&p, 8).is_err());
    let only_k = Pair {
        batter: 0,
        pitcher: 0,
        counts: [10, 0, 0, 0, 0, 0, 0, 0, 0],
    };
    assert_eq!(tree::risk(&only_k, 0).unwrap().unwrap().counts, [10, 0]);
    assert!(tree::risk(&only_k, 1).unwrap().is_none());
}
