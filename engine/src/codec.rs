//! Version 20 canonical binary event record. Little-endian integers, length-prefixed UTF-8;
//! no database or external serialization dependency. Decoding always revalidates the contract.
use super::*;

type Decode<T> = Result<T, TransitionError>;

struct Reader<'a>(&'a [u8]);
impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Decode<&'a [u8]> {
        let part = self
            .0
            .get(..count)
            .ok_or(TransitionError::InvalidEncoding)?;
        self.0 = &self.0[count..];
        Ok(part)
    }
}

trait Binary: Sized {
    fn encode(&self, out: &mut Vec<u8>);
    fn decode(reader: &mut Reader<'_>) -> Decode<Self>;
}
impl Binary for u8 {
    fn encode(&self, out: &mut Vec<u8>) {
        out.push(*self);
    }
    fn decode(reader: &mut Reader<'_>) -> Decode<Self> {
        Ok(reader.take(1)?[0])
    }
}
impl Binary for u16 {
    fn encode(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.to_le_bytes());
    }
    fn decode(reader: &mut Reader<'_>) -> Decode<Self> {
        Ok(u16::from_le_bytes(
            reader
                .take(2)?
                .try_into()
                .map_err(|_| TransitionError::InvalidEncoding)?,
        ))
    }
}
impl Binary for u64 {
    fn encode(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.to_le_bytes());
    }
    fn decode(reader: &mut Reader<'_>) -> Decode<Self> {
        Ok(u64::from_le_bytes(
            reader
                .take(8)?
                .try_into()
                .map_err(|_| TransitionError::InvalidEncoding)?,
        ))
    }
}
impl Binary for bool {
    fn encode(&self, out: &mut Vec<u8>) {
        u8::from(*self).encode(out);
    }
    fn decode(reader: &mut Reader<'_>) -> Decode<Self> {
        match u8::decode(reader)? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(TransitionError::InvalidEncoding),
        }
    }
}
impl Binary for String {
    fn encode(&self, out: &mut Vec<u8>) {
        (self.len() as u64).encode(out);
        out.extend_from_slice(self.as_bytes());
    }
    fn decode(reader: &mut Reader<'_>) -> Decode<Self> {
        let count =
            usize::try_from(u64::decode(reader)?).map_err(|_| TransitionError::InvalidEncoding)?;
        String::from_utf8(reader.take(count)?.to_vec())
            .map_err(|_| TransitionError::InvalidEncoding)
    }
}
impl<T: Binary> Binary for Option<T> {
    fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Some(value) => {
                1u8.encode(out);
                value.encode(out);
            }
            None => 0u8.encode(out),
        }
    }
    fn decode(reader: &mut Reader<'_>) -> Decode<Self> {
        match u8::decode(reader)? {
            0 => Ok(None),
            1 => Ok(Some(T::decode(reader)?)),
            _ => Err(TransitionError::InvalidEncoding),
        }
    }
}
impl<T: Binary> Binary for Vec<T> {
    fn encode(&self, out: &mut Vec<u8>) {
        (self.len() as u64).encode(out);
        for value in self {
            value.encode(out);
        }
    }
    fn decode(reader: &mut Reader<'_>) -> Decode<Self> {
        let count =
            usize::try_from(u64::decode(reader)?).map_err(|_| TransitionError::InvalidEncoding)?;
        if count > reader.0.len() {
            return Err(TransitionError::InvalidEncoding);
        }
        (0..count).map(|_| T::decode(reader)).collect()
    }
}
impl<T: Binary, const N: usize> Binary for [T; N] {
    fn encode(&self, out: &mut Vec<u8>) {
        for value in self {
            value.encode(out);
        }
    }
    fn decode(reader: &mut Reader<'_>) -> Decode<Self> {
        let items: Vec<T> = (0..N).map(|_| T::decode(reader)).collect::<Decode<_>>()?;
        items
            .try_into()
            .map_err(|_| TransitionError::InvalidEncoding)
    }
}
macro_rules! binary_enum {
    ($name:ident { $($variant:ident = $tag:literal),+ $(,)? }) => {
        impl Binary for $name {
            fn encode(&self, out: &mut Vec<u8>) {
                (match self { $(Self::$variant => $tag),+ } as u8).encode(out);
            }
            fn decode(reader: &mut Reader<'_>) -> Decode<Self> {
                match u8::decode(reader)? { $($tag => Ok(Self::$variant),)+ _ => Err(TransitionError::InvalidEncoding) }
            }
        }
    };
}
binary_enum!(TeamSide { Away = 0, Home = 1 });
binary_enum!(Half { Top = 0, Bottom = 1 });
binary_enum!(Base { Plate = 0, First = 1, Second = 2, Third = 3 });
binary_enum!(Destination { First = 0, Second = 1, Third = 2, Home = 3, Out = 4 });
binary_enum!(RunnerEvent { WildPitch = 0, PassedBall = 1, StealAttempt = 2, Pickoff = 3, Balk = 4 });
binary_enum!(CreditKind { PlateAppearance = 0, Strikeout = 1, Walk = 2, Hit = 3, Run = 4, Rbi = 5, StolenBase = 6, CaughtStealing = 7, Putout = 8, Assist = 9, Error = 10, PassedBall = 11, AtBat = 12, HitByPitch = 13, SacrificeBunt = 14, SacrificeFly = 15, InterferenceAward = 16, Single = 17, Double = 18, Triple = 19, OrdinaryWalk = 20, IntentionalWalk = 21, GroundedIntoDoublePlay = 22, WildPitch = 23, Balk = 24, NoHit = 25 });
binary_enum!(ThirdOutKind { BatterBeforeFirst = 0, Force = 1, Timing = 2, PrecedingRunnerAppeal = 3, BatterAfterFirstTiming = 4 });
binary_enum!(FieldingReview { Unknown = 0, NoMisplay = 1 });
binary_enum!(ContactClass { Grounded = 0, NotGrounded = 1 });
binary_enum!(ContactContinuity { Continuous = 0, Discontinuous = 1, Unknown = 2 });
binary_enum!(ForceLoss { RemovedByFirstOut = 0, PreviouslyRemoved = 1, NeverForced = 2, Unknown = 3 });
binary_enum!(DeliveryContext { RunnerOnly = 0, UncaughtThirdStrike = 1, DeliveredFourthBall = 2 });
binary_enum!(HandlingAssessment { PitcherWild = 0, CatcherPassed = 1 });
binary_enum!(BalkCause { Unknown = 0, PitcherAction602a = 1, SqueezeOrStealHomeInterference601g = 2, PitchingProhibition602d = 3 });
binary_enum!(BalkDisposition { Unknown = 0, PenaltyApplied = 1, PlayProceedsWithoutReference = 2 });
impl Binary for BalkResolution {
    fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::Unknown => 0u8.encode(out),
            Self::ExactProtectedAwards => 1u8.encode(out),
            Self::ActualMinimumProgress(facts) => {
                2u8.encode(out);
                facts.encode(out);
            }
        }
    }
    fn decode(reader: &mut Reader<'_>) -> Decode<Self> {
        match u8::decode(reader)? {
            0 => Ok(Self::Unknown),
            1 => Ok(Self::ExactProtectedAwards),
            2 => Ok(Self::ActualMinimumProgress(Binary::decode(reader)?)),
            _ => Err(TransitionError::InvalidEncoding),
        }
    }
}
impl Binary for RecoveryReview {
    fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::Unknown => 0u8.encode(out),
            Self::NoRecoveryPutout => 1u8.encode(out),
            Self::BeforeAnyAdvance { runner_id } => {
                2u8.encode(out);
                runner_id.encode(out);
            }
            Self::AfterAdvance {
                runner_id,
                advancer_id,
            } => {
                3u8.encode(out);
                runner_id.encode(out);
                advancer_id.encode(out);
            }
            Self::RecoveredThirdStrikeBatterBeforeFirst => 4u8.encode(out),
            Self::ThirdStrikeBatterAfterFirstThrow {
                putout_fielder_id,
                at,
            } => {
                5u8.encode(out);
                putout_fielder_id.encode(out);
                at.encode(out);
            }
        }
    }
    fn decode(reader: &mut Reader<'_>) -> Decode<Self> {
        match u8::decode(reader)? {
            0 => Ok(Self::Unknown),
            1 => Ok(Self::NoRecoveryPutout),
            2 => Ok(Self::BeforeAnyAdvance {
                runner_id: Binary::decode(reader)?,
            }),
            3 => Ok(Self::AfterAdvance {
                runner_id: Binary::decode(reader)?,
                advancer_id: Binary::decode(reader)?,
            }),
            4 => Ok(Self::RecoveredThirdStrikeBatterBeforeFirst),
            5 => Ok(Self::ThirdStrikeBatterAfterFirstThrow {
                putout_fielder_id: Binary::decode(reader)?,
                at: Binary::decode(reader)?,
            }),
            _ => Err(TransitionError::InvalidEncoding),
        }
    }
}
impl Binary for RetirementCause {
    fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::BatterPutoutBeforeFirstTouch => 0u8.encode(out),
            Self::Force { at } => {
                1u8.encode(out);
                at.encode(out);
            }
            Self::Tag { at, force_loss } => {
                2u8.encode(out);
                at.encode(out);
                force_loss.encode(out);
            }
            Self::BatterAfterFirstTag { at } => {
                3u8.encode(out);
                at.encode(out);
            }
            Self::RunnerInterference => 4u8.encode(out),
            Self::BatterByPrecedingRunnerInterference { interferer_id } => {
                5u8.encode(out);
                interferer_id.encode(out);
            }
        }
    }
    fn decode(reader: &mut Reader<'_>) -> Decode<Self> {
        match u8::decode(reader)? {
            0 => Ok(Self::BatterPutoutBeforeFirstTouch),
            1 => Ok(Self::Force {
                at: Binary::decode(reader)?,
            }),
            2 => Ok(Self::Tag {
                at: Binary::decode(reader)?,
                force_loss: Binary::decode(reader)?,
            }),
            3 => Ok(Self::BatterAfterFirstTag {
                at: Binary::decode(reader)?,
            }),
            4 => Ok(Self::RunnerInterference),
            5 => Ok(Self::BatterByPrecedingRunnerInterference {
                interferer_id: Binary::decode(reader)?,
            }),
            _ => Err(TransitionError::InvalidEncoding),
        }
    }
}
impl Binary for EventKind {
    fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::PlateAppearance => 0u8.encode(out),
            Self::RunnerOnly(kind) => {
                1u8.encode(out);
                kind.encode(out);
            }
            Self::Substitution => 2u8.encode(out),
            Self::OutOfParkHomeRun => 3u8.encode(out),
            Self::FieldersChoice => 4u8.encode(out),
        }
    }
    fn decode(reader: &mut Reader<'_>) -> Decode<Self> {
        match u8::decode(reader)? {
            0 => Ok(Self::PlateAppearance),
            1 => Ok(Self::RunnerOnly(RunnerEvent::decode(reader)?)),
            2 => Ok(Self::Substitution),
            3 => Ok(Self::OutOfParkHomeRun),
            4 => Ok(Self::FieldersChoice),
            _ => Err(TransitionError::InvalidEncoding),
        }
    }
}
macro_rules! binary_struct {
    ($name:ident { $($field:ident),+ $(,)? }) => {
        impl Binary for $name {
            fn encode(&self, out: &mut Vec<u8>) { $(self.$field.encode(out);)+ }
            fn decode(reader: &mut Reader<'_>) -> Decode<Self> {
                Ok(Self { $($field: Binary::decode(reader)?,)+ })
            }
        }
    };
}
binary_struct!(Runner {
    player_id,
    responsible_pitcher
});
binary_struct!(TeamState {
    batting_order,
    next_batter,
    pitcher,
    dh_slot,
    dh_must_bat,
    bench,
    removed
});
binary_struct!(GameState {
    inning,
    half,
    outs,
    score,
    teams,
    bases,
    over
});
binary_struct!(RunnerMove { runner, from, to });
binary_struct!(Participants { batter, pitcher });
binary_struct!(Credit { player_id, kind });
binary_struct!(ThirdOut {
    runner_id,
    kind,
    home_before_out
});
impl Binary for PersonnelChange {
    fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::Pitcher {
                team,
                outgoing,
                incoming,
            } => {
                0u8.encode(out);
                team.encode(out);
                outgoing.encode(out);
                incoming.encode(out);
            }
            Self::BattingSlot {
                team,
                slot,
                outgoing,
                incoming,
            } => {
                1u8.encode(out);
                team.encode(out);
                slot.encode(out);
                outgoing.encode(out);
                incoming.encode(out);
            }
            Self::PinchRunner {
                base,
                outgoing,
                incoming,
            } => {
                2u8.encode(out);
                base.encode(out);
                outgoing.encode(out);
                incoming.encode(out);
            }
        }
    }
    fn decode(reader: &mut Reader<'_>) -> Decode<Self> {
        match u8::decode(reader)? {
            0 => Ok(Self::Pitcher {
                team: Binary::decode(reader)?,
                outgoing: Binary::decode(reader)?,
                incoming: Binary::decode(reader)?,
            }),
            1 => Ok(Self::BattingSlot {
                team: Binary::decode(reader)?,
                slot: Binary::decode(reader)?,
                outgoing: Binary::decode(reader)?,
                incoming: Binary::decode(reader)?,
            }),
            2 => Ok(Self::PinchRunner {
                base: Binary::decode(reader)?,
                outgoing: Binary::decode(reader)?,
                incoming: Binary::decode(reader)?,
            }),
            _ => Err(TransitionError::InvalidEncoding),
        }
    }
}
binary_struct!(Delta {
    moves,
    outs,
    runs,
    credits,
    changes,
    third_out
});
binary_struct!(RetirementFact { runner_id, cause });
binary_struct!(ReviewedErrorPlacement {
    between_outs,
    outside_interval
});
binary_struct!(TwoOutContactFacts {
    contact,
    first_out,
    second_out,
    continuity,
    error_placement
});
binary_struct!(HandlingAdvance { runner_id, reached });
binary_struct!(PreDeliverySteal { runner_id, target });
binary_struct!(SingleDeliveryFacts {
    context,
    handling,
    catcher_id,
    advances,
    pre_delivery_steals,
    recovery
});
binary_struct!(NamedBaseTouch { runner_id, base });
binary_struct!(BalkProgressFacts {
    batter_first_touch,
    runner_next_touches
});
binary_struct!(BalkFacts {
    cause,
    disposition,
    resolution
});
binary_struct!(OrdinaryGroundThrowFacts {
    batter_id,
    thrower_id,
    receiver_id
});
binary_struct!(Event {
    kind,
    participants,
    fielding_review,
    delta,
    two_out_contact,
    single_delivery,
    balk,
    ordinary_ground_throw
});

impl EventTransition {
    /// Stable versioned record; cannot be decoded without passing transition validation.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = b"BBE20".to_vec();
        self.before.encode(&mut out);
        self.event.encode(&mut out);
        self.after.encode(&mut out);
        out
    }
    pub fn from_bytes(bytes: &[u8]) -> Decode<Self> {
        let payload = bytes
            .strip_prefix(b"BBE20")
            .ok_or(TransitionError::InvalidEncoding)?;
        let mut reader = Reader(payload);
        let before = GameState::decode(&mut reader)?;
        let event = Event::decode(&mut reader)?;
        let after = GameState::decode(&mut reader)?;
        if !reader.0.is_empty() {
            return Err(TransitionError::InvalidEncoding);
        }
        Self::try_new(before, event, after)
    }
}
