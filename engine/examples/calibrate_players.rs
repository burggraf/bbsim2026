//! Preregistered offline comparator; inputs prepared by audits.player_calibration.
#[path = "support/player_calibration.rs"]
mod calibration;
use calibration::{fit, Model, Pair};
#[path = "support/player_calibration_tree.rs"]
mod tree;
use tree::Tree;
trait Probability {
    fn probabilities(&self, batter: usize, pitcher: usize) -> [f64; 9];
    fn nb(&self) -> usize;
    fn np(&self) -> usize;
}
impl Probability for Model {
    fn probabilities(&self, b: usize, p: usize) -> [f64; 9] {
        Model::probabilities(self, b, p)
    }
    fn nb(&self) -> usize {
        self.batters.len()
    }
    fn np(&self) -> usize {
        self.pitchers.len()
    }
}
impl Probability for Tree {
    fn probabilities(&self, b: usize, p: usize) -> [f64; 9] {
        Tree::probabilities(self, b, p)
    }
    fn nb(&self) -> usize {
        self.nodes[0].batters.len()
    }
    fn np(&self) -> usize {
        self.nodes[0].pitchers.len()
    }
}
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;
const KEYS: [&str; 9] = [
    "strikeout",
    "walk",
    "hbp",
    "single",
    "double",
    "triple",
    "home_run",
    "roe",
    "contact_out",
];
#[derive(Clone)]
struct Row {
    fold: u8,
    game: String,
    pair: Pair,
}
struct Data {
    rows: Vec<Row>,
    batters: Vec<String>,
    pitchers: Vec<String>,
}
struct Candidate {
    name: String,
    joint: bool,
    strengths: [f64; 2],
    model: Model,
    qualified: bool,
}
struct Score {
    n: f64,
    nll: f64,
    brier: f64,
    observed: [u64; 9],
    expected: [f64; 9],
}
struct Bias {
    role: usize,
    category: usize,
    players: usize,
    mae: f64,
    maximum: f64,
    worst: usize,
    signed: f64,
}

fn write_new(path: &Path, text: &str) -> AnyResult<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(text.as_bytes())?;
    Ok(())
}
fn read(path: &Path) -> AnyResult<Data> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    let expected = format!("fold\tgame_id\tbatter_id\tpitcher_id\t{}", KEYS.join("\t"));
    if lines.next() != Some(expected.as_str()) {
        return Err("invalid calibration input header".into());
    }
    let mut raw = Vec::new();
    let mut bs = BTreeSet::new();
    let mut ps = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let mut games = BTreeMap::new();
    for line in lines {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 13
            || fields[1..4].iter().any(|s| {
                s.is_empty()
                    || !s
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
            })
        {
            return Err("invalid calibration row".into());
        }
        let fold = fields[0].parse::<u8>()?;
        if fold > 9
            || !seen.insert((
                fields[1].to_owned(),
                fields[2].to_owned(),
                fields[3].to_owned(),
            ))
        {
            return Err("invalid calibration grain".into());
        }
        if games
            .insert(fields[1].to_owned(), fold)
            .is_some_and(|old| old != fold)
        {
            return Err("game crosses calibration folds".into());
        }
        let mut counts = [0; 9];
        for (k, n) in counts.iter_mut().enumerate() {
            *n = fields[k + 4].parse::<u64>()?;
        }
        let n = counts
            .iter()
            .try_fold(0_u64, |n, x| n.checked_add(*x))
            .ok_or("calibration row overflow")?;
        if n == 0 || n > 185703 {
            return Err("invalid calibration exposure".into());
        }
        bs.insert(fields[2].to_owned());
        ps.insert(fields[3].to_owned());
        raw.push((
            fold,
            fields[1].to_owned(),
            fields[2].to_owned(),
            fields[3].to_owned(),
            counts,
        ));
    }
    if raw.is_empty() {
        return Err("empty calibration input".into());
    }
    let batters: Vec<_> = bs.into_iter().collect();
    let pitchers: Vec<_> = ps.into_iter().collect();
    let bm: BTreeMap<_, _> = batters
        .iter()
        .enumerate()
        .map(|(i, s)| (s.clone(), i))
        .collect();
    let pm: BTreeMap<_, _> = pitchers
        .iter()
        .enumerate()
        .map(|(i, s)| (s.clone(), i))
        .collect();
    let rows = raw
        .into_iter()
        .map(|(fold, game, b, p, counts)| Row {
            fold,
            game,
            pair: Pair {
                batter: bm[&b],
                pitcher: pm[&p],
                counts,
            },
        })
        .collect();
    Ok(Data {
        rows,
        batters,
        pitchers,
    })
}
fn aggregate(rows: &[Row]) -> Vec<Pair> {
    let mut pairs = BTreeMap::new();
    for row in rows {
        let counts = pairs
            .entry((row.pair.batter, row.pair.pitcher))
            .or_insert([0_u64; 9]);
        for (a, b) in counts.iter_mut().zip(row.pair.counts) {
            *a = a.checked_add(b).expect("validated source total");
        }
    }
    pairs
        .into_iter()
        .map(|((batter, pitcher), counts)| Pair {
            batter,
            pitcher,
            counts,
        })
        .collect()
}
fn score(model: &impl Probability, rows: &[Row]) -> Score {
    let mut s = Score {
        n: 0.0,
        nll: 0.0,
        brier: 0.0,
        observed: [0; 9],
        expected: [0.0; 9],
    };
    for row in rows {
        let n = row.pair.counts.iter().sum::<u64>() as f64;
        let q = model.probabilities(row.pair.batter, row.pair.pitcher);
        s.n += n;
        s.brier += n * (1.0 + q.iter().map(|x| x * x).sum::<f64>());
        for (k, probability) in q.iter().enumerate() {
            s.nll -= row.pair.counts[k] as f64 * probability.ln();
            s.brier -= 2.0 * row.pair.counts[k] as f64 * probability;
            s.observed[k] += row.pair.counts[k];
            s.expected[k] += n * probability;
        }
    }
    s.nll /= s.n;
    s.brier /= s.n;
    s
}
type PlayerMoments = (
    Vec<[f64; 9]>,
    Vec<[f64; 9]>,
    Vec<[f64; 9]>,
    Vec<[f64; 9]>,
    Vec<[f64; 9]>,
    Vec<[f64; 9]>,
);
fn player_counts(model: &impl Probability, pairs: &[Pair]) -> PlayerMoments {
    let mut bo = vec![[0.0; 9]; model.nb()];
    let mut po = vec![[0.0; 9]; model.np()];
    let mut be = bo.clone();
    let mut pe = po.clone();
    let mut bv = bo.clone();
    let mut pv = po.clone();
    for pair in pairs {
        let n = pair.counts.iter().sum::<u64>() as f64;
        let q = model.probabilities(pair.batter, pair.pitcher);
        for k in 0..9 {
            bo[pair.batter][k] += pair.counts[k] as f64;
            po[pair.pitcher][k] += pair.counts[k] as f64;
            be[pair.batter][k] += n * q[k];
            pe[pair.pitcher][k] += n * q[k];
            bv[pair.batter][k] += n * q[k] * (1.0 - q[k]);
            pv[pair.pitcher][k] += n * q[k] * (1.0 - q[k]);
        }
    }
    (bo, po, be, pe, bv, pv)
}
fn biases(model: &impl Probability, pairs: &[Pair]) -> Vec<Bias> {
    let (bo, po, be, pe, _, _) = player_counts(model, pairs);
    let mut out = Vec::new();
    for (role, (observed, expected)) in [(bo, be), (po, pe)].into_iter().enumerate() {
        for category in 0..10 {
            let mut bias = Bias {
                role,
                category,
                players: 0,
                mae: 0.0,
                maximum: 0.0,
                worst: 0,
                signed: 0.0,
            };
            for (id, (o, e)) in observed.iter().zip(&expected).enumerate() {
                let n = o.iter().sum::<f64>();
                if n < 200.0 {
                    continue;
                }
                let delta = if category == 9 {
                    (3..=6).map(|k| e[k] - o[k]).sum::<f64>() / n
                } else {
                    (e[category] - o[category]) / n
                };
                bias.players += 1;
                bias.mae += delta.abs();
                if delta.abs() > bias.maximum {
                    bias.maximum = delta.abs();
                    bias.worst = id;
                    bias.signed = delta;
                }
            }
            if bias.players > 0 {
                bias.mae /= bias.players as f64;
            }
            out.push(bias);
        }
    }
    out
}
fn qualifies(biases: &[Bias]) -> bool {
    biases.iter().all(|b| match b.category {
        0 | 1 | 6 | 9 => b.mae <= 0.01 && b.maximum <= 0.03,
        2 | 5 | 7 => b.mae <= 0.005 && b.maximum <= 0.02,
        _ => true,
    })
}
fn hit_mix_qualifies(values: &[Bias], baseline: &[Bias]) -> bool {
    values
        .iter()
        .filter(|b| [3, 4].contains(&b.category))
        .all(|b| {
            baseline
                .iter()
                .find(|old| old.role == b.role && old.category == b.category)
                .is_some_and(|old| {
                    b.mae <= 0.01
                        && b.maximum <= 0.03
                        && b.mae <= old.mae
                        && b.maximum <= old.maximum
                })
        })
}
fn hit_mix_guardrails(vs_v2: [f64; 3]) -> bool {
    vs_v2.iter().all(|x| x.is_finite())
        && vs_v2[1] <= vs_v2[0]
        && vs_v2[0] <= vs_v2[2]
        && vs_v2[0] <= 0.001
}
fn bias_csv(data: &Data, values: &[Bias]) -> String {
    let mut out =
        "role,category,players,mae_rate,max_abs_rate,worst_id,worst_signed_rate\n".to_owned();
    for b in values {
        let (role, ids) = if b.role == 0 {
            ("batter", &data.batters)
        } else {
            ("pitcher", &data.pitchers)
        };
        let category = if b.category == 9 {
            "hits"
        } else {
            KEYS[b.category]
        };
        out += &format!(
            "{role},{category},{},{},{},{},{}\n",
            b.players, b.mae, b.maximum, ids[b.worst], b.signed
        );
    }
    out
}
fn difference(a: &impl Probability, b: &impl Probability, rows: &[Row]) -> [f64; 3] {
    let mut games: BTreeMap<&str, (f64, f64)> = BTreeMap::new();
    for row in rows {
        let qa = a.probabilities(row.pair.batter, row.pair.pitcher);
        let qb = b.probabilities(row.pair.batter, row.pair.pitcher);
        let d = -(0..9)
            .map(|k| row.pair.counts[k] as f64 * (qa[k].ln() - qb[k].ln()))
            .sum::<f64>();
        let entry = games.entry(&row.game).or_default();
        entry.0 += d;
        entry.1 += row.pair.counts.iter().sum::<u64>() as f64;
    }
    let n = games.values().map(|x| x.1).sum::<f64>();
    let mean = games.values().map(|x| x.0).sum::<f64>() / n;
    let g = games.len() as f64;
    let se = (g / (g - 1.0)
        * games
            .values()
            .map(|(d, n)| (d - mean * n).powi(2))
            .sum::<f64>())
    .sqrt()
        / n;
    [mean, mean - 1.96 * se, mean + 1.96 * se]
}
fn band(n: u64) -> usize {
    match n {
        0 => 0,
        1..=9 => 1,
        10..=49 => 2,
        50..=199 => 3,
        200..=499 => 4,
        _ => 5,
    }
}
fn band_scores(model: &impl Probability, train: &[Pair], rows: &[Row]) -> String {
    let mut bn = vec![0; model.nb()];
    let mut pn = vec![0; model.np()];
    for p in train {
        let n = p.counts.iter().sum::<u64>();
        bn[p.batter] += n;
        pn[p.pitcher] += n;
    }
    let mut out = "role,training_exposure_band,n,nll,brier\n".to_owned();
    for (role, ns) in [("batter", bn), ("pitcher", pn)] {
        for bucket in 0..6 {
            let selected: Vec<_> = rows
                .iter()
                .filter(|r| {
                    band(
                        ns[if role == "batter" {
                            r.pair.batter
                        } else {
                            r.pair.pitcher
                        }],
                    ) == bucket
                })
                .cloned()
                .collect();
            let s = score(model, &selected);
            if s.n > 0.0 {
                out += &format!(
                    "{role},{},{},{},{}\n",
                    ["0", "1-9", "10-49", "50-199", "200-499", "500+"][bucket],
                    s.n,
                    s.nll,
                    s.brier
                );
            }
        }
    }
    out
}
fn ensembles(
    data: &Data,
    model: &impl Probability,
    pairs: &[Pair],
    output: &Path,
) -> AnyResult<bool> {
    let (bo, po, be, pe, bv, pv) = player_counts(model, pairs);
    let nb = bo.len();
    let np = po.len();
    let mut histories = vec![Vec::new(); (nb + np) * 9];
    let mut raw =
        "seed,role,player_id,strikeout,walk,hbp,single,double,triple,home_run,roe,contact_out\n"
            .to_owned();
    let probabilities: Vec<_> = pairs
        .iter()
        .map(|p| model.probabilities(p.batter, p.pitcher))
        .collect();
    for seed in 1..=200_u64 {
        let mut state = seed;
        let mut bc = vec![[0_u64; 9]; nb];
        let mut pc = vec![[0_u64; 9]; np];
        for (p, q) in pairs.iter().zip(&probabilities) {
            for _ in 0..p.counts.iter().sum::<u64>() {
                state = state.wrapping_add(0x9e3779b97f4a7c15);
                let mut word = state;
                word = (word ^ (word >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
                word = (word ^ (word >> 27)).wrapping_mul(0x94d049bb133111eb);
                word ^= word >> 31;
                let u = (word >> 11) as f64 / (1_u64 << 53) as f64;
                let mut cumulative = 0.0;
                let mut category = 8;
                for (k, p) in q.iter().enumerate() {
                    cumulative += p;
                    if u < cumulative {
                        category = k;
                        break;
                    }
                }
                bc[p.batter][category] += 1;
                pc[p.pitcher][category] += 1;
            }
        }
        for (role, ids, counts, offset) in [
            ("batter", &data.batters, &bc, 0),
            ("pitcher", &data.pitchers, &pc, nb),
        ] {
            for (id, c) in counts.iter().enumerate() {
                raw += &format!(
                    "{seed},{role},{},{}\n",
                    ids[id],
                    c.iter().map(u64::to_string).collect::<Vec<_>>().join(",")
                );
                for k in 0..9 {
                    histories[(offset + id) * 9 + k].push(c[k]);
                }
            }
        }
    }
    write_new(&output.join("ensembles.csv"), &raw)?;
    let mut out="role,player_id,exposure,category,observed,expected,variance,mean,sd,q05,median,q95,mean_pass,sd_pass\n".to_owned();
    let mut passed = true;
    for (role, ids, o, e, v, offset) in [
        ("batter", &data.batters, bo, be, bv, 0),
        ("pitcher", &data.pitchers, po, pe, pv, nb),
    ] {
        for id in 0..ids.len() {
            let n = o[id].iter().sum::<f64>();
            for k in 0..9 {
                let samples = &mut histories[(offset + id) * 9 + k];
                let mean = samples.iter().sum::<u64>() as f64 / 200.0;
                let sd = (samples
                    .iter()
                    .map(|x| (*x as f64 - mean).powi(2))
                    .sum::<f64>()
                    / 199.0)
                    .sqrt();
                samples.sort();
                let mean_ok = (mean - e[id][k]).abs() <= 5.0 * (v[id][k] / 200.0).sqrt() + 0.05;
                let sd_ok = (v[id][k] < 5.0 || sd > 0.0)
                    && (v[id][k] < 25.0 || (sd / v[id][k].sqrt() - 1.0).abs() <= 0.2);
                passed &= mean_ok && sd_ok;
                out += &format!(
                    "{role},{},{n},{},{},{},{},{mean},{sd},{},{},{},{mean_ok},{sd_ok}\n",
                    ids[id],
                    KEYS[k],
                    o[id][k],
                    e[id][k],
                    v[id][k],
                    samples[9],
                    samples[99],
                    samples[189]
                );
            }
        }
    }
    write_new(&output.join("variation.csv"), &out)?;
    Ok(passed)
}
fn effects(data: &Data, model: &Model, candidate: &str) -> String {
    let mut text = format!(
        "key\tvalue\nformat\tplayer-calibration-candidate-v1\ncandidate\t{candidate}\nleague\t{}\n",
        model
            .league
            .iter()
            .map(f64::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );
    for (role, ids, profiles) in [
        ("batter", &data.batters, &model.batters),
        ("pitcher", &data.pitchers, &model.pitchers),
    ] {
        for (id, q) in ids.iter().zip(profiles) {
            text += &format!(
                "{role}:{id}\t{}\n",
                q.iter().map(f64::to_string).collect::<Vec<_>>().join(",")
            );
        }
    }
    text
}
fn run(input: &Path, output: &Path) -> AnyResult<()> {
    let data = read(input)?;
    let training: Vec<_> = data.rows.iter().filter(|r| r.fold >= 2).cloned().collect();
    let validation: Vec<_> = data.rows.iter().filter(|r| r.fold == 1).cloned().collect();
    let test: Vec<_> = data.rows.iter().filter(|r| r.fold == 0).cloned().collect();
    if training.is_empty() || validation.is_empty() || test.is_empty() {
        return Err("empty calibration partition".into());
    }
    fs::create_dir(output)?;
    let train = aggregate(&training);
    let nb = data.batters.len();
    let np = data.pitchers.len();
    let baseline = Model::from_counts(&train, nb, np, [200.0, 200.0])?;
    let mut league = baseline.clone();
    for q in league.batters.iter_mut().chain(&mut league.pitchers) {
        *q = league.league;
    }
    let mut candidates = Vec::new();
    let mut table="candidate,joint,strength_b,strength_p,converged,iterations,max_correction,train_qualified,validation_nll,validation_brier\n".to_owned();
    let mut add = |name: String,
                   joint: bool,
                   strengths: [f64; 2],
                   model: Model,
                   iterations: usize,
                   correction: f64| {
        let qualified = qualifies(&biases(&model, &train));
        let s = score(&model, &validation);
        table += &format!(
            "{name},{joint},{},{},true,{iterations},{correction},{qualified},{},{}\n",
            strengths[0], strengths[1], s.nll, s.brier
        );
        eprintln!(
            "candidate {name} qualified={qualified} validation_nll={}",
            s.nll
        );
        candidates.push(Candidate {
            name,
            joint,
            strengths,
            model,
            qualified,
        });
    };
    add("league".into(), false, [0.0, 0.0], league.clone(), 0, 0.0);
    add(
        "fixed200".into(),
        false,
        [200.0, 200.0],
        baseline.clone(),
        0,
        0.0,
    );
    for b in [1.0, 5.0, 20.0, 50.0, 100.0, 200.0, 400.0, 800.0, 1600.0] {
        for p in [1.0, 5.0, 20.0, 50.0, 100.0, 200.0, 400.0, 800.0, 1600.0] {
            add(
                format!("marginal-{b}-{p}"),
                false,
                [b, p],
                Model::from_counts(&train, nb, np, [b, p])?,
                0,
                0.0,
            );
        }
    }
    let mut failures = String::new();
    for b in [1.0, 5.0, 20.0, 200.0] {
        for p in [1.0, 5.0, 20.0, 200.0] {
            let name = format!("joint-{b}-{p}");
            eprintln!("fitting {name}");
            match fit(&train, nb, np, baseline.league, [b, p], 5000, 1e-8) {
                Ok(fitted) => add(
                    name,
                    true,
                    [b, p],
                    fitted.model,
                    fitted.iterations,
                    fitted.max_correction,
                ),
                Err(error) => {
                    eprintln!("failed {name}: {error}");
                    failures += &format!("{name}: {error}\n");
                }
            }
        }
    }
    write_new(&output.join("validation.csv"), &table)?;
    write_new(&output.join("fit-failures.txt"), &failures)?;
    let mut chosen = None;
    let mut best = f64::INFINITY;
    for (i, c) in candidates.iter().enumerate() {
        if c.qualified {
            let s = score(&c.model, &validation);
            if s.nll < best - 1e-10 {
                chosen = Some(i);
                best = s.nll;
            }
        }
    }
    let Some(chosen) = chosen else {
        write_new(
            &output.join("selection.json"),
            "{\"qualified\":false,\"reason\":\"no training-reconstruction-qualified candidate\"}\n",
        )?;
        return Ok(());
    };
    let selected = &candidates[chosen];
    // Selection is fixed here, before any test-score evaluation or all-season refit.
    write_new(
        &output.join("selection-before-test.txt"),
        &format!("{}\n", selected.name),
    )?;
    write_new(
        &output.join("training-effects.tsv"),
        &effects(&data, &selected.model, &selected.name),
    )?;
    let mut tests = "candidate,n,nll,brier,observed,expected\n".to_owned();
    for c in &candidates {
        let s = score(&c.model, &test);
        tests += &format!(
            "{},{},{},{},\"{:?}\",\"{:?}\"\n",
            c.name, s.n, s.nll, s.brier, s.observed, s.expected
        );
        write_new(
            &output.join(format!("test-bands-{}.csv", c.name)),
            &band_scores(&c.model, &train, &test),
        )?;
    }
    write_new(&output.join("test.csv"), &tests)?;
    let vs_league = difference(&selected.model, &league, &test);
    let vs_baseline = difference(&selected.model, &baseline, &test);
    let guardrails = vs_league[2] < 0.0 && vs_baseline[0] <= 0.01;
    let all = aggregate(&data.rows);
    let full_baseline = Model::from_counts(&all, nb, np, [200.0, 200.0])?;
    let full = if selected.joint {
        fit(
            &all,
            nb,
            np,
            full_baseline.league,
            selected.strengths,
            5000,
            1e-8,
        )?
        .model
    } else if selected.name == "league" {
        let mut m = full_baseline.clone();
        for q in m.batters.iter_mut().chain(&mut m.pitchers) {
            *q = m.league;
        }
        m
    } else {
        Model::from_counts(&all, nb, np, selected.strengths)?
    };
    let full_bias = biases(&full, &all);
    let old_bias = biases(&full_baseline, &all);
    let common = [0, 1, 6, 9];
    let no_worse = full_bias
        .iter()
        .zip(&old_bias)
        .filter(|(b, _)| common.contains(&b.category))
        .all(|(b, old)| b.mae <= old.mae && b.maximum <= old.maximum);
    let reconstruction = qualifies(&full_bias) && no_worse;
    write_new(&output.join("full-bias.csv"), &bias_csv(&data, &full_bias))?;
    write_new(
        &output.join("baseline-bias.csv"),
        &bias_csv(&data, &old_bias),
    )?;
    write_new(
        &output.join("full-effects.tsv"),
        &effects(&data, &full, &selected.name),
    )?;
    let variation = ensembles(&data, &full, &all, output)?;
    let accepted = guardrails && reconstruction && variation;
    let result=format!("{{\"candidate\":\"{}\",\"batter_strength\":{},\"pitcher_strength\":{},\"test_nll_difference_vs_league\":{:?},\"test_nll_difference_vs_fixed200\":{:?},\"heldout_guardrails\":{guardrails},\"full_reconstruction\":{reconstruction},\"variation\":{variation},\"accepted\":{accepted}}}\n",selected.name,selected.strengths[0],selected.strengths[1],vs_league,vs_baseline);
    write_new(&output.join("selection.json"), &result)?;
    println!("{result}");
    Ok(())
}
fn tree_effects(data: &Data, model: &Tree, hit_mix: bool) -> String {
    let rates: [f64; 8] = std::array::from_fn(|n| model.nodes[n].league[0]);
    let version = if hit_mix { 3 } else { 2 };
    let mut text = format!(
        "key\tvalue\nformat\tplayer-calibration-tree-candidate-v{version}\nleague_nodes\t{}\n",
        rates
            .iter()
            .map(f64::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );
    for (role, ids) in [("batter", &data.batters), ("pitcher", &data.pitchers)] {
        for (id, name) in ids.iter().enumerate() {
            let q: [f64; 8] = std::array::from_fn(|n| {
                if role == "batter" {
                    model.nodes[n].batters[id][0]
                } else {
                    model.nodes[n].pitchers[id][0]
                }
            });
            text += &format!(
                "{role}:{name}\t{}\n",
                q.iter().map(f64::to_string).collect::<Vec<_>>().join(",")
            );
        }
    }
    text
}
fn fit_tree(
    data: &Data,
    train: &[Pair],
    validation: &[Row],
    league: [f64; 9],
    output: &Path,
    hit_mix: bool,
) -> AnyResult<(Tree, [[f64; 2]; 8])> {
    let mut model = Tree::neutral(
        tree::league_nodes(league)?,
        data.batters.len(),
        data.pitchers.len(),
    )?;
    let mut strengths = [[0.0; 2]; 8];
    let reference = Model::from_counts(
        train,
        data.batters.len(),
        data.pitchers.len(),
        [200.0, 200.0],
    )?;
    let reference_bias = biases(&reference, train);
    for (node, chosen_strength) in strengths.iter_mut().enumerate() {
        let risks = train
            .iter()
            .map(|p| tree::risk(p, node))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let mut best = None;
        let mut best_score = f64::INFINITY;
        let mut table="node,batter_strength,pitcher_strength,converged,iterations,max_correction,train_qualified,validation_conditional_nll\n".to_owned();
        for b in [1.0, 5.0, 20.0, 200.0] {
            for p in [1.0, 5.0, 20.0, 200.0] {
                let frozen = if node < 5 { 20.0 } else { 200.0 };
                if hit_mix && node != 6 && [b, p] != [frozen, frozen] {
                    continue;
                }
                match fit(
                    &risks,
                    data.batters.len(),
                    data.pitchers.len(),
                    model.nodes[node].league,
                    [b, p],
                    5000,
                    1e-8,
                ) {
                    Err(error) => {
                        eprintln!("tree fit failed node{node} {b}/{p}: {error}");
                        table += &format!("{node},{b},{p},false,5000,,,\n");
                    }
                    Ok(fitted) => {
                        let mut trial = model.clone();
                        trial.nodes[node] = fitted.model;
                        let category = [
                            Some(0),
                            Some(1),
                            Some(2),
                            Some(9),
                            Some(6),
                            Some(5),
                            None,
                            Some(7),
                        ][node];
                        let trial_bias = biases(&trial, train);
                        let relative = trial_bias
                            .iter()
                            .zip(&reference_bias)
                            .filter(|(b, _)| {
                                Some(b.category) == category && [0, 1, 6, 9].contains(&b.category)
                            })
                            .all(|(b, old)| b.mae <= old.mae && b.maximum <= old.maximum);
                        let eligible = if hit_mix && node == 6 {
                            write_new(
                                &output.join(format!("hit-mix-{b}-{p}-training-effects.tsv")),
                                &tree_effects(data, &trial, true),
                            )?;
                            hit_mix_qualifies(&trial_bias, &reference_bias)
                        } else {
                            relative
                                && qualifies(
                                    &trial_bias
                                        .into_iter()
                                        .filter(|x| Some(x.category) == category)
                                        .collect::<Vec<_>>(),
                                )
                        };
                        let mut loss = 0.0;
                        let mut n = 0.0;
                        for row in validation {
                            if let Some(r) = tree::risk(&row.pair, node)? {
                                let q = trial.nodes[node].probabilities(r.batter, r.pitcher);
                                n += r.counts.iter().sum::<u64>() as f64;
                                loss -=
                                    r.counts[0] as f64 * q[0].ln() + r.counts[1] as f64 * q[1].ln();
                            }
                        }
                        if n == 0.0 {
                            return Err("empty tree validation risk set".into());
                        }
                        loss /= n;
                        table += &format!(
                            "{node},{b},{p},true,{},{},{eligible},{loss}\n",
                            fitted.iterations, fitted.max_correction
                        );
                        if eligible && loss < best_score - 1e-10 {
                            best_score = loss;
                            best = Some(trial.nodes[node].clone());
                            *chosen_strength = [b, p];
                        }
                    }
                }
            }
        }
        write_new(&output.join(format!("tree-node-{node}.csv")), &table)?;
        model.nodes[node] =
            best.ok_or_else(|| format!("no training-qualified fit at tree node{node}"))?;
        eprintln!(
            "tree selected node{node} {:?}; validation conditional NLL {best_score}",
            chosen_strength
        );
    }
    Ok((model, strengths))
}
fn frozen_v2(model: &Tree, pairs: &[Pair]) -> AnyResult<Tree> {
    let risks = pairs
        .iter()
        .map(|p| tree::risk(p, 6))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let mut result = model.clone();
    result.nodes[6] = fit(
        &risks,
        model.nb(),
        model.np(),
        model.nodes[6].league,
        [200.0, 200.0],
        5000,
        1e-8,
    )?
    .model;
    Ok(result)
}
fn run_tree(input: &Path, output: &Path, hit_mix: bool) -> AnyResult<()> {
    let data = read(input)?;
    let training: Vec<_> = data.rows.iter().filter(|r| r.fold >= 2).cloned().collect();
    let validation: Vec<_> = data.rows.iter().filter(|r| r.fold == 1).cloned().collect();
    let test: Vec<_> = data.rows.iter().filter(|r| r.fold == 0).cloned().collect();
    if training.is_empty() || validation.is_empty() || test.is_empty() {
        return Err("empty calibration partition".into());
    }
    fs::create_dir(output)?;
    let train = aggregate(&training);
    let nb = data.batters.len();
    let np = data.pitchers.len();
    let baseline = Model::from_counts(&train, nb, np, [200.0, 200.0])?;
    let mut league = baseline.clone();
    for q in league.batters.iter_mut().chain(&mut league.pitchers) {
        *q = league.league;
    }
    let (selected, strengths) =
        fit_tree(&data, &train, &validation, baseline.league, output, hit_mix)?;
    let label = if hit_mix {
        "outcome-tree-hit-mix-v3"
    } else {
        "outcome-tree-v2"
    };
    write_new(
        &output.join("selection-before-test.json"),
        &format!("{{\"candidate\":\"{label}\",\"strengths\":{strengths:?}}}\n"),
    )?;
    write_new(
        &output.join("training-effects.tsv"),
        &tree_effects(&data, &selected, hit_mix),
    )?;
    let v2 = if hit_mix {
        Some(frozen_v2(&selected, &train)?)
    } else {
        None
    };
    if let Some(model) = &v2 {
        write_new(
            &output.join("v2-training-effects.tsv"),
            &tree_effects(&data, model, false),
        )?;
    }
    let selected_test = score(&selected, &test);
    let league_test = score(&league, &test);
    let baseline_test = score(&baseline, &test);
    let old_joint = fit(&train, nb, np, baseline.league, [20.0, 20.0], 5000, 1e-8)?.model;
    let joint_test = score(&old_joint, &test);
    let mut table = "candidate,n,nll,brier,observed,expected\n".to_owned();
    for (name, s) in [
        (label, selected_test),
        ("league", league_test),
        ("fixed200", baseline_test),
        ("joint-20-20", joint_test),
    ] {
        table += &format!(
            "{name},{},{},{},\"{:?}\",\"{:?}\"\n",
            s.n, s.nll, s.brier, s.observed, s.expected
        );
    }
    if let Some(model) = &v2 {
        let s = score(model, &test);
        table += &format!(
            "outcome-tree-v2,{},{},{},\"{:?}\",\"{:?}\"\n",
            s.n, s.nll, s.brier, s.observed, s.expected
        );
    }
    write_new(&output.join("test.csv"), &table)?;
    write_new(
        &output.join("test-bands.csv"),
        &band_scores(&selected, &train, &test),
    )?;
    let vs_league = difference(&selected, &league, &test);
    let vs_baseline = difference(&selected, &baseline, &test);
    let vs_v2 = v2.as_ref().map(|model| difference(&selected, model, &test));
    let guardrails =
        vs_league[2] < 0.0 && vs_baseline[0] <= 0.01 && vs_v2.is_none_or(hit_mix_guardrails);
    let all = aggregate(&data.rows);
    let full_baseline = Model::from_counts(&all, nb, np, [200.0, 200.0])?;
    let mut full = Tree::neutral(tree::league_nodes(full_baseline.league)?, nb, np)?;
    let mut refits = "node,batter_strength,pitcher_strength,iterations,max_correction\n".to_owned();
    for (node, strength) in strengths.iter().enumerate() {
        let risks = all
            .iter()
            .map(|p| tree::risk(p, node))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let fitted = fit(
            &risks,
            nb,
            np,
            full.nodes[node].league,
            *strength,
            5000,
            1e-8,
        )?;
        refits += &format!(
            "{node},{},{},{},{}\n",
            strength[0], strength[1], fitted.iterations, fitted.max_correction
        );
        full.nodes[node] = fitted.model;
    }
    write_new(&output.join("full-refits.csv"), &refits)?;
    let full_bias = biases(&full, &all);
    let old_bias = biases(&full_baseline, &all);
    let no_worse = full_bias
        .iter()
        .zip(&old_bias)
        .filter(|(b, _)| [0, 1, 6, 9].contains(&b.category))
        .all(|(b, old)| b.mae <= old.mae && b.maximum <= old.maximum);
    let mut reconstruction = qualifies(&full_bias) && no_worse;
    if hit_mix {
        let v2_full = frozen_v2(&full, &all)?;
        reconstruction &= hit_mix_qualifies(&full_bias, &old_bias)
            && hit_mix_qualifies(&full_bias, &biases(&v2_full, &all));
        write_new(
            &output.join("v2-full-effects.tsv"),
            &tree_effects(&data, &v2_full, false),
        )?;
        write_new(
            &output.join("v2-full-bias.csv"),
            &bias_csv(&data, &biases(&v2_full, &all)),
        )?;
    }
    write_new(&output.join("full-bias.csv"), &bias_csv(&data, &full_bias))?;
    write_new(
        &output.join("baseline-bias.csv"),
        &bias_csv(&data, &old_bias),
    )?;
    write_new(
        &output.join("full-effects.tsv"),
        &tree_effects(&data, &full, hit_mix),
    )?;
    let variation = ensembles(&data, &full, &all, output)?;
    let accepted = guardrails && reconstruction && variation;
    let extra = vs_v2
        .map(|value| format!(",\"test_nll_difference_vs_v2\":{value:?}"))
        .unwrap_or_default();
    let result=format!("{{\"candidate\":\"{label}\",\"strengths\":{strengths:?},\"reused_test_benchmark\":true,\"test_nll_difference_vs_league\":{vs_league:?},\"test_nll_difference_vs_fixed200\":{vs_baseline:?}{extra},\"heldout_guardrails\":{guardrails},\"full_reconstruction\":{reconstruction},\"variation\":{variation},\"accepted\":{accepted}}}\n");
    write_new(&output.join("selection.json"), &result)?;
    println!("{result}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hit_mix_requires_separate_bounds_and_no_worse_baseline_for_both_roles() {
        let make = |maximum| {
            (0..2)
                .flat_map(|role| {
                    (3..=4).map(move |category| Bias {
                        role,
                        category,
                        players: 1,
                        mae: 0.01,
                        maximum,
                        worst: 0,
                        signed: maximum,
                    })
                })
                .collect::<Vec<_>>()
        };
        let old = make(0.04);
        let exact = make(0.03);
        assert!(hit_mix_qualifies(&exact, &old));
        let over = make(0.0300000001);
        assert!(qualifies(&over)); // The previous acceptance never constrained 1B/2B.
        assert!(!hit_mix_qualifies(&over, &old));
        for i in 0..4 {
            let mut bad = make(0.02);
            bad[i].mae = 0.0100000001;
            assert!(!hit_mix_qualifies(&bad, &old));
            let mut stricter = make(0.04);
            stricter[i].maximum = 0.019;
            assert!(!hit_mix_qualifies(&make(0.02), &stricter));
        }
        assert!(hit_mix_guardrails([-0.002, -0.004, 0.0]));
        assert!(hit_mix_guardrails([0.001, 0.0, 0.002]));
        assert!(!hit_mix_guardrails([0.0010000001, 0.0, 0.002]));
        assert!(!hit_mix_guardrails([f64::NAN, 0.0, 0.002]));
    }
    #[test]
    fn literal_hyphenated_participant_is_not_filtered_and_bad_grain_refuses() {
        let path = std::env::temp_dir().join(format!(
            "bbsim-calibration-parser-{}-{}.tsv",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let header = format!(
            "fold\tgame_id\tbatter_id\tpitcher_id\t{}\n",
            KEYS.join("\t")
        );
        let row = "0\tG1\tjay-j001\tp1\t1\t0\t0\t0\t0\t0\t0\t0\t0\n";
        write_new(&path, &format!("{header}{row}")).unwrap();
        let parsed = read(&path);
        fs::remove_file(&path).unwrap();
        assert_eq!(parsed.unwrap().batters, vec!["jay-j001"]);
        for bad in [
            format!("{header}{row}{row}"),
            format!(
                "{header}{row}{}",
                row.replacen("0\tG1\tjay-j001", "1\tG1\tother", 1)
            ),
            format!("{header}{}", row.replacen("\t1\t0\t0", "\t-1\t0\t0", 1)),
        ] {
            write_new(&path, &bad).unwrap();
            let result = read(&path);
            fs::remove_file(&path).unwrap();
            assert!(result.is_err());
        }
    }
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3
        && !(args.len() == 4 && ["--tree-v2", "--hit-mix-v3"].contains(&args[3].as_str()))
    {
        eprintln!(
            "Usage: calibrate_players INPUT.tsv NEW_OUTPUT_DIRECTORY [--tree-v2|--hit-mix-v3]"
        );
        std::process::exit(2);
    }
    let result = if args.len() == 4 {
        run_tree(
            Path::new(&args[1]),
            Path::new(&args[2]),
            args[3] == "--hit-mix-v3",
        )
    } else {
        run(Path::new(&args[1]), Path::new(&args[2]))
    };
    if let Err(error) = result {
        eprintln!("calibration refused: {error}");
        std::process::exit(1);
    }
}
