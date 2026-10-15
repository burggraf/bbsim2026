//! Standalone nonterminal dispatch; no producer integration or internal RNG.
use super::{
    joint,
    joint_hit::{Context, Kernel, Model, ModeledCause, TrainingCell},
    preview::Outcome,
};
use bbsim_engine::{apply, EventTransition, GameState, Half};

pub struct HitDraw {
    pub word: u64,
    pub cause: ModeledCause,
}

pub struct ConditionalHits {
    kernels: Vec<Kernel>,
}

impl ConditionalHits {
    /// An explicit model and every representable context are required up front.
    pub fn new(cells: &[TrainingCell], model: Model) -> Result<Self, String> {
        let mut kernels = Vec::with_capacity(48);
        for value in [1, 2] {
            for mask in 0..8 {
                for outs in 0..3 {
                    kernels.push(Kernel::fit(cells, Context { value, mask, outs }, model)?);
                }
            }
        }
        Ok(Self { kernels })
    }

    /// One supplied PA shape; hits alone require one supplied advancement word.
    pub fn transition(
        &self,
        before: &GameState,
        sample: joint::Sample,
        hit_draw: Option<HitDraw>,
    ) -> Result<EventTransition, String> {
        if before.half == Half::Bottom && before.inning >= 9 {
            return Err("terminal bottom-half dispatch unsupported".into());
        }
        if before.over || before.inning == 0 || before.outs > 2 {
            return Err("invalid or ended dispatch state".into());
        }
        if (sample.outcome == Outcome::ContactOut) != sample.contact.is_some() {
            return Err("inconsistent outcome/contact pair".into());
        }
        if matches!(sample.outcome, Outcome::Single | Outcome::Double) {
            let draw = hit_draw.ok_or("missing explicit hit draw/cause")?;
            let value_index = usize::from(sample.outcome == Outcome::Double);
            let mask = before
                .bases
                .iter()
                .enumerate()
                .fold(0, |mask, (i, runner)| {
                    mask | (usize::from(runner.is_some()) << i)
                });
            self.kernels[value_index * 24 + mask * 3 + usize::from(before.outs)]
                .project(before, draw.word, draw.cause)
        } else {
            if hit_draw.is_some() {
                return Err("unexpected hit draw for non-hit shape".into());
            }
            apply(before, joint::event_for(before, sample)?)
                .map_err(|error| format!("canonical non-hit dispatch refused: {error:?}"))
        }
    }
}
