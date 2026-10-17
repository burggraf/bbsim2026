//! Fixed outcome-specific factorization, evaluated before one multinomial parent draw.
use super::calibration::{Model, Pair};
#[derive(Clone)]
pub struct Tree {
    pub nodes: [Model<2>; 8],
}
const POSITIVE: [&[usize]; 8] = [&[0], &[1], &[2], &[3, 4, 5, 6], &[6], &[5], &[4], &[7]];
const RISK: [&[usize]; 8] = [
    &[0, 1, 2, 3, 4, 5, 6, 7, 8],
    &[1, 2, 3, 4, 5, 6, 7, 8],
    &[2, 3, 4, 5, 6, 7, 8],
    &[3, 4, 5, 6, 7, 8],
    &[3, 4, 5, 6],
    &[3, 4, 5],
    &[3, 4],
    &[7, 8],
];
pub fn risk(pair: &Pair, node: usize) -> Result<Option<Pair<2>>, String> {
    if node >= 8 {
        return Err("invalid calibration tree node".into());
    }
    let sum = |indices: &[usize]| {
        indices
            .iter()
            .try_fold(0_u64, |n, k| n.checked_add(pair.counts[*k]))
            .ok_or("tree risk exposure overflow")
    };
    let n = sum(RISK[node])?;
    let yes = sum(POSITIVE[node])?;
    Ok((n > 0).then_some(Pair {
        batter: pair.batter,
        pitcher: pair.pitcher,
        counts: [yes, n - yes],
    }))
}
pub fn league_nodes(league: [f64; 9]) -> Result<[f64; 8], String> {
    if league.iter().any(|x| !x.is_finite() || *x <= 0.0)
        || (league.iter().sum::<f64>() - 1.0).abs() > 1e-12
    {
        return Err("invalid calibration tree league".into());
    }
    Ok(std::array::from_fn(|node| {
        POSITIVE[node].iter().map(|k| league[*k]).sum::<f64>()
            / RISK[node].iter().map(|k| league[*k]).sum::<f64>()
    }))
}
impl Tree {
    pub fn neutral(rates: [f64; 8], nb: usize, np: usize) -> Result<Self, String> {
        if nb == 0
            || np == 0
            || rates
                .iter()
                .any(|x| !x.is_finite() || *x <= 0.0 || *x >= 1.0)
        {
            return Err("invalid calibration tree rates".into());
        }
        Ok(Self {
            nodes: std::array::from_fn(|i| Model {
                league: [rates[i], 1.0 - rates[i]],
                batters: vec![[rates[i], 1.0 - rates[i]]; nb],
                pitchers: vec![[rates[i], 1.0 - rates[i]]; np],
            }),
        })
    }
    pub fn probabilities(&self, batter: usize, pitcher: usize) -> [f64; 9] {
        let r: [f64; 8] = std::array::from_fn(|i| self.nodes[i].probabilities(batter, pitcher)[0]);
        let mut q = [0.0; 9];
        let mut remaining = 1.0;
        for k in 0..3 {
            q[k] = remaining * r[k];
            remaining *= 1.0 - r[k];
        }
        let mut hits = remaining * r[3];
        let nonhits = remaining * (1.0 - r[3]);
        q[6] = hits * r[4];
        hits *= 1.0 - r[4];
        q[5] = hits * r[5];
        hits *= 1.0 - r[5];
        q[4] = hits * r[6];
        q[3] = hits * (1.0 - r[6]);
        q[7] = nonhits * r[7];
        q[8] = nonhits * (1.0 - r[7]);
        q
    }
}
