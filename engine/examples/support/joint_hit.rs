//! Isolated synthetic joint-hit prerequisite, not a production game policy.

pub fn legal_vectors(value: u8, mask: u8) -> Result<Vec<Vec<u8>>, String> {
    if !matches!(value, 1 | 2) || mask > 7 {
        return Err("invalid hit value/base mask".into());
    }
    // Ordered construction is independent of the test's Cartesian/pairwise oracle.
    fn extend(origins: &[u8], vector: &mut Vec<u8>, result: &mut Vec<Vec<u8>>) {
        let Some((&origin, rest)) = origins.split_first() else {
            result.push(vector.clone());
            return;
        };
        let previous = *vector.last().expect("batter endpoint is present");
        let lower = origin.max(if previous == 4 { 4 } else { previous + 1 });
        for target in lower..=4 {
            vector.push(target);
            extend(rest, vector, result);
            vector.pop();
        }
    }
    let origins: Vec<u8> = (0..3)
        .filter(|i| mask & (1 << i) != 0)
        .map(|i| i + 1)
        .collect();
    let mut result = Vec::new();
    extend(&origins, &mut vec![value], &mut result);
    Ok(result)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Context {
    pub value: u8,
    pub mask: u8,
    pub outs: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Model {
    UniformLegal,
    PooledMask,
    BaseOut,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ratio {
    pub numerator: u128,
    pub denominator: u128,
}

/// Explicit synthetic cause assumption, never inferred from an observed endpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModeledCause {
    UnaidedNoMisplay,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrainingCell {
    pub context: Context,
    pub counts: Vec<u64>,
}

pub struct Kernel {
    context: Context,
    vectors: Vec<Vec<u8>>,
    probabilities: Vec<Ratio>,
    edges: Vec<u64>,
}

impl Kernel {
    pub fn fit(cells: &[TrainingCell], context: Context, model: Model) -> Result<Self, String> {
        fn index(c: Context) -> Result<usize, String> {
            if !matches!(c.value, 1 | 2) || c.mask > 7 || c.outs > 2 {
                return Err("invalid training context".into());
            }
            Ok(usize::from(c.value - 1) * 24 + usize::from(c.mask) * 3 + usize::from(c.outs))
        }
        let selected = index(context)?;
        if cells.len() != 48 {
            return Err("training table requires all48 contexts".into());
        }
        let mut table = vec![None; 48];
        let mut total = 0u64;
        for cell in cells {
            let slot = index(cell.context)?;
            if table[slot].is_some()
                || cell.counts.len() != legal_vectors(cell.context.value, cell.context.mask)?.len()
            {
                return Err("duplicate context or invalid count support".into());
            }
            for count in &cell.counts {
                total = total
                    .checked_add(*count)
                    .ok_or("training aggregation overflow")?;
            }
            table[slot] = Some(cell);
        }
        let vectors = legal_vectors(context.value, context.mask)?;
        let mut counts = vec![0u64; vectors.len()];
        match model {
            Model::UniformLegal => {}
            Model::BaseOut => {
                counts.clone_from(&table[selected].expect("complete unique table").counts)
            }
            Model::PooledMask => {
                for outs in 0..3 {
                    let cell =
                        table[index(Context { outs, ..context })?].expect("complete unique table");
                    for (sum, count) in counts.iter_mut().zip(&cell.counts) {
                        *sum = sum
                            .checked_add(*count)
                            .ok_or("pooled aggregation overflow")?;
                    }
                }
            }
        }
        let n = counts
            .iter()
            .try_fold(0u64, |sum, count| sum.checked_add(*count))
            .ok_or("context aggregation overflow")?;
        let denominator = 2 * u128::from(n) + vectors.len() as u128;
        const GRID: u64 = 1 << 53;
        let mut edges = Vec::with_capacity(counts.len());
        let mut cumulative = 0u128;
        let mut previous = 0u64;
        for count in &counts {
            let numerator = u128::from(*count)
                .checked_mul(2)
                .and_then(|n| n.checked_add(1))
                .ok_or("grid arithmetic overflow")?;
            cumulative = cumulative
                .checked_add(numerator)
                .ok_or("grid arithmetic overflow")?;
            let scaled = cumulative
                .checked_mul(u128::from(GRID))
                .ok_or("grid arithmetic overflow")?;
            let ceiling = (scaled / denominator)
                .checked_add(u128::from(scaled % denominator != 0))
                .ok_or("grid arithmetic overflow")?;
            let edge = u64::try_from(ceiling).map_err(|_| "grid edge overflow")?;
            if edge <= previous || edge > GRID {
                return Err("unrepresentable positive grid interval".into());
            }
            edges.push(edge);
            previous = edge;
        }
        if previous != GRID {
            return Err("grid normalization failure".into());
        }
        let probabilities = counts
            .into_iter()
            .map(|count| {
                let numerator = 2 * u128::from(count) + 1;
                let (mut a, mut b) = (numerator, denominator);
                while b != 0 {
                    (a, b) = (b, a % b);
                }
                Ratio {
                    numerator: numerator / a,
                    denominator: denominator / a,
                }
            })
            .collect();
        Ok(Self {
            context,
            vectors,
            probabilities,
            edges,
        })
    }

    pub fn probabilities(&self) -> &[Ratio] {
        &self.probabilities
    }

    pub fn vectors(&self) -> &[Vec<u8>] {
        &self.vectors
    }

    pub fn edges(&self) -> &[u64] {
        &self.edges
    }

    pub fn masses(&self) -> Vec<u64> {
        self.edges
            .iter()
            .scan(0, |previous, edge| {
                let mass = edge - *previous;
                *previous = *edge;
                Some(mass)
            })
            .collect()
    }

    /// One supplied word, explicit modeled cause, immutable canonical apply.
    /// All potentially terminal bottom halves are refused BEFORE choosing a vector.
    pub fn project(
        &self,
        before: &bbsim_engine::GameState,
        word: u64,
        cause: ModeledCause,
    ) -> Result<bbsim_engine::EventTransition, String> {
        use bbsim_engine::{
            apply, Base, Credit, CreditKind, Delta, Destination, Event, EventKind, FieldingReview,
            Half, Participants, Runner, RunnerMove, SoleSafeHitFacts,
        };
        if before.half == Half::Bottom && before.inning >= 9 {
            return Err("terminal bottom-half projection unsupported".into());
        }
        if before.over || before.inning == 0 || cause != ModeledCause::UnaidedNoMisplay {
            return Err("invalid state or missing modeled unaided cause".into());
        }
        let mask = before
            .bases
            .iter()
            .enumerate()
            .fold(0u8, |mask, (i, runner)| {
                mask | (u8::from(runner.is_some()) << i)
            });
        if mask != self.context.mask || before.outs != self.context.outs {
            return Err("projection context mismatch".into());
        }
        let offense = before.half.offense();
        let batter = before.teams[offense]
            .batting_order
            .get(usize::from(before.teams[offense].next_batter))
            .ok_or("invalid batting cursor")?
            .clone();
        let pitcher = before.teams[1 - offense].pitcher.clone();
        let vector = self.sample(word);
        let destinations = [
            Destination::First,
            Destination::Second,
            Destination::Third,
            Destination::Home,
        ];
        let credit = |id: &str, kind| Credit {
            player_id: id.into(),
            kind,
        };
        let mut moves = vec![RunnerMove {
            runner: Runner {
                player_id: batter.clone(),
                responsible_pitcher: pitcher.clone(),
            },
            from: Base::Plate,
            to: destinations[usize::from(vector[0] - 1)],
        }];
        let mut credits = vec![
            credit(&batter, CreditKind::PlateAppearance),
            credit(&batter, CreditKind::Hit),
            credit(
                &batter,
                if self.context.value == 1 {
                    CreditKind::Single
                } else {
                    CreditKind::Double
                },
            ),
        ];
        let mut runs = 0;
        let mut position = 1;
        for (i, origin) in [Base::First, Base::Second, Base::Third]
            .into_iter()
            .enumerate()
        {
            if let Some(runner) = &before.bases[i] {
                let target = destinations[usize::from(vector[position] - 1)];
                moves.push(RunnerMove {
                    runner: runner.clone(),
                    from: origin,
                    to: target,
                });
                if target == Destination::Home {
                    runs += 1;
                    credits.extend([
                        credit(&runner.player_id, CreditKind::Run),
                        credit(&batter, CreditKind::Rbi),
                    ]);
                }
                position += 1;
            }
        }
        let event = Event {
            kind: EventKind::PlateAppearance,
            participants: Participants {
                batter: Some(batter.clone()),
                pitcher,
            },
            delta: Delta {
                moves,
                outs: 0,
                runs,
                credits,
                changes: vec![],
                third_out: None,
            },
            // Literal MODEL ASSUMPTIONS. No source endpoints are authenticated here.
            fielding_review: FieldingReview::NoMisplay,
            sole_safe_hit: Some(SoleSafeHitFacts { batter_id: batter }),
            two_out_contact: None,
            single_delivery: None,
            balk: None,
            ordinary_ground_throw: None,
            caught_strikeout: None,
            protected_award: None,
            sole_home_run: None,
            muffed_ground_ball: None,
            ordinary_air_catch: None,
        };
        apply(before, event).map_err(|error| format!("canonical projection refused: {error:?}"))
    }

    pub fn sample(&self, word: u64) -> &[u8] {
        let draw = word >> 11;
        let index = self
            .edges
            .iter()
            .position(|edge| draw < *edge)
            .expect("positive normalized grid covers every high53-bit word");
        &self.vectors[index]
    }
}
