//! Offline point estimates on the observed opponent graph; no rules or outcome draws.
#[derive(Clone, Debug)]
pub struct Pair<const K: usize = 9> {
    pub batter: usize,
    pub pitcher: usize,
    pub counts: [u64; K],
}

#[derive(Clone, Debug)]
pub struct Model<const K: usize = 9> {
    pub league: [f64; K],
    pub batters: Vec<[f64; K]>,
    pub pitchers: Vec<[f64; K]>,
}

pub struct Fit<const K: usize = 9> {
    pub model: Model<K>,
    pub iterations: usize,
    pub max_correction: f64,
}

type Margins<const K: usize> = (Vec<[f64; K]>, Vec<[f64; K]>, [f64; K]);
fn margins<const K: usize>(pairs: &[Pair<K>], nb: usize, np: usize) -> Result<Margins<K>, String> {
    if K < 2 || pairs.is_empty() || nb == 0 || np == 0 {
        return Err("empty calibration population".into());
    }
    let mut batters = vec![[0.0; K]; nb];
    let mut pitchers = vec![[0.0; K]; np];
    let mut counts = [0_u64; K];
    for pair in pairs {
        if pair.batter >= nb || pair.pitcher >= np || pair.counts.iter().all(|n| *n == 0) {
            return Err("invalid calibration pair".into());
        }
        for (k, n) in pair.counts.iter().enumerate() {
            counts[k] = counts[k]
                .checked_add(*n)
                .ok_or("calibration count overflow")?;
            batters[pair.batter][k] += *n as f64;
            pitchers[pair.pitcher][k] += *n as f64;
        }
    }
    let total = counts
        .iter()
        .try_fold(0_u64, |n, x| n.checked_add(*x))
        .ok_or("calibration exposure overflow")?;
    if total > (1_u64 << 53) {
        return Err("unsupported calibration exposure".into());
    }
    let league = counts.map(|n| (n as f64 + 0.5) / (total as f64 + K as f64 * 0.5));
    Ok((batters, pitchers, league))
}

fn normalize<const K: usize>(q: &mut [f64; K]) -> Result<(), String> {
    let sum = q.iter().sum::<f64>();
    if !sum.is_finite() || sum <= 0.0 || q.iter().any(|x| !x.is_finite() || *x <= 0.0) {
        return Err("invalid calibration distribution".into());
    }
    for x in q {
        *x /= sum;
    }
    Ok(())
}

impl<const K: usize> Model<K> {
    pub fn from_counts(
        pairs: &[Pair<K>],
        nb: usize,
        np: usize,
        kappas: [f64; 2],
    ) -> Result<Self, String> {
        let (mut batters, mut pitchers, league) = margins(pairs, nb, np)?;
        for (profiles, kappa) in [(&mut batters, kappas[0]), (&mut pitchers, kappas[1])] {
            if !kappa.is_finite() || kappa <= 0.0 {
                return Err("invalid calibration strength".into());
            }
            for q in profiles {
                let n = q.iter().sum::<f64>();
                for k in 0..K {
                    q[k] = if n == 0.0 {
                        league[k]
                    } else {
                        (q[k] + kappa * league[k]) / (n + kappa)
                    };
                }
            }
        }
        Ok(Self {
            league,
            batters,
            pitchers,
        })
    }

    pub fn probabilities(&self, batter: usize, pitcher: usize) -> [f64; K] {
        let mut q = std::array::from_fn(|k| {
            self.batters[batter][k] * self.pitchers[pitcher][k] / self.league[k]
        });
        let sum = q.iter().sum::<f64>();
        for x in &mut q {
            *x /= sum;
        }
        q
    }
}

fn balance_neutral<const K: usize>(
    a: &mut [[f64; K]],
    c: &mut [[f64; K]],
    league: [f64; K],
    lambdas: [f64; 2],
) -> Result<(), String> {
    // Opposite category rescaling preserves every real matchup; only neutral priors move.
    for _ in 0..20 {
        let mut aa = [0.0; K];
        let mut bb = [0.0; K];
        for q in a.iter() {
            for k in 0..K {
                aa[k] += lambdas[0] * league[k] * q[k];
            }
        }
        for q in c.iter() {
            for k in 0..K {
                bb[k] += lambdas[1] * league[k] * q[k];
            }
        }
        let d = lambdas[0] * a.len() as f64 - lambdas[1] * c.len() as f64;
        let g: [f64; K] = std::array::from_fn(|k| {
            let delta = d * league[k];
            let root = (delta * delta + 4.0 * aa[k] * bb[k]).sqrt();
            if delta >= 0.0 {
                (delta + root) / (2.0 * aa[k])
            } else {
                2.0 * bb[k] / (root - delta)
            }
        });
        for (profiles, divide) in [(&mut *a, false), (&mut *c, true)] {
            for q in profiles {
                let mut neutral: [f64; K] = std::array::from_fn(|k| {
                    league[k] * q[k] * if divide { 1.0 / g[k] } else { g[k] }
                });
                normalize(&mut neutral)?;
                for k in 0..K {
                    q[k] = neutral[k] / league[k];
                }
            }
        }
    }
    Ok(())
}

pub fn fit<const K: usize>(
    pairs: &[Pair<K>],
    nb: usize,
    np: usize,
    league: [f64; K],
    lambdas: [f64; 2],
    max_iterations: usize,
    tolerance: f64,
) -> Result<Fit<K>, String> {
    let (mut bt, mut pt, _) = margins(pairs, nb, np)?;
    if league.iter().any(|x| !x.is_finite() || *x <= 0.0)
        || (league.iter().sum::<f64>() - 1.0).abs() > 1e-12
        || lambdas.iter().any(|x| !x.is_finite() || *x <= 0.0)
        || max_iterations == 0
        || !tolerance.is_finite()
        || tolerance <= 0.0
    {
        return Err("invalid calibration parameters".into());
    }
    let cold_b: Vec<bool> = bt.iter().map(|q| q.iter().all(|x| *x == 0.0)).collect();
    let cold_p: Vec<bool> = pt.iter().map(|q| q.iter().all(|x| *x == 0.0)).collect();
    for (targets, lambda) in [(&mut bt, lambdas[0]), (&mut pt, lambdas[1])] {
        for q in targets {
            for k in 0..K {
                q[k] += lambda * league[k];
            }
        }
    }
    let mut a = vec![[1.0; K]; nb];
    let mut c = vec![[1.0; K]; np];
    let mut weights = vec![0.0; pairs.len()];
    for iteration in 1..=max_iterations {
        let mut eb: Vec<[f64; K]> = a
            .iter()
            .map(|q| std::array::from_fn(|k| lambdas[0] * league[k] * q[k]))
            .collect();
        for (pair, weight) in pairs.iter().zip(&mut weights) {
            let mass: [f64; K] =
                std::array::from_fn(|k| league[k] * a[pair.batter][k] * c[pair.pitcher][k]);
            *weight = pair.counts.iter().sum::<u64>() as f64 / mass.iter().sum::<f64>();
            for k in 0..K {
                eb[pair.batter][k] += *weight * mass[k];
            }
        }
        let mut correction = 0.0_f64;
        for b in 0..nb {
            for k in 0..K {
                let ratio = bt[b][k] / eb[b][k];
                correction = correction.max((ratio - 1.0).abs());
                a[b][k] *= ratio;
            }
        }
        let mut ep: Vec<[f64; K]> = c
            .iter()
            .map(|q| std::array::from_fn(|k| lambdas[1] * league[k] * q[k]))
            .collect();
        for (pair, weight) in pairs.iter().zip(&weights) {
            for k in 0..K {
                ep[pair.pitcher][k] += weight * league[k] * a[pair.batter][k] * c[pair.pitcher][k];
            }
        }
        for p in 0..np {
            for k in 0..K {
                let ratio = pt[p][k] / ep[p][k];
                correction = correction.max((ratio - 1.0).abs());
                c[p][k] *= ratio;
            }
        }
        // Row scaling is a gauge: case normalization cancels it, including neutral cases.
        for q in a.iter_mut().chain(&mut c) {
            let mut neutral: [f64; K] = std::array::from_fn(|k| league[k] * q[k]);
            normalize(&mut neutral)?;
            for k in 0..K {
                q[k] = neutral[k] / league[k];
            }
        }
        if !correction.is_finite() {
            return Err("nonfinite calibration correction".into());
        }
        balance_neutral(&mut a, &mut c, league, lambdas)?;
        if iteration == max_iterations && correction > tolerance {
            let mean_b: f64 = a.iter().map(|q| league[0] * q[0]).sum::<f64>() / nb as f64;
            let mean_p: f64 = c.iter().map(|q| league[0] * q[0]).sum::<f64>() / np as f64;
            return Err(format!("calibration did not converge in {max_iterations} cycles; correction={correction}; neutral first-category means={mean_b},{mean_p}"));
        }
        if correction <= tolerance {
            let batters = a
                .iter()
                .zip(&cold_b)
                .map(|(q, cold)| {
                    if *cold {
                        league
                    } else {
                        std::array::from_fn(|k| league[k] * q[k])
                    }
                })
                .collect();
            let pitchers = c
                .iter()
                .zip(&cold_p)
                .map(|(q, cold)| {
                    if *cold {
                        league
                    } else {
                        std::array::from_fn(|k| league[k] * q[k])
                    }
                })
                .collect();
            return Ok(Fit {
                model: Model {
                    league,
                    batters,
                    pitchers,
                },
                iterations: iteration,
                max_correction: correction,
            });
        }
    }
    Err(format!(
        "calibration did not converge in {max_iterations} cycles"
    ))
}
