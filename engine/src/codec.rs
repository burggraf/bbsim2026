//! Version 1 canonical binary event record. Little-endian integers, length-prefixed UTF-8;
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
binary_enum!(CreditKind { PlateAppearance = 0, Strikeout = 1, Walk = 2, Hit = 3, Run = 4, Rbi = 5, StolenBase = 6, CaughtStealing = 7, Putout = 8, Assist = 9, Error = 10, PassedBall = 11 });
binary_enum!(ThirdOutKind { BatterBeforeFirst = 0, Force = 1, Timing = 2, PrecedingRunnerAppeal = 3 });
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
        }
    }
    fn decode(reader: &mut Reader<'_>) -> Decode<Self> {
        match u8::decode(reader)? {
            0 => Ok(Self::PlateAppearance),
            1 => Ok(Self::RunnerOnly(RunnerEvent::decode(reader)?)),
            2 => Ok(Self::Substitution),
            3 => Ok(Self::OutOfParkHomeRun),
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
binary_struct!(Event {
    kind,
    participants,
    delta
});

impl EventTransition {
    /// Stable versioned record; cannot be decoded without passing transition validation.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = b"BBE7".to_vec();
        self.before.encode(&mut out);
        self.event.encode(&mut out);
        self.after.encode(&mut out);
        out
    }
    pub fn from_bytes(bytes: &[u8]) -> Decode<Self> {
        let payload = bytes
            .strip_prefix(b"BBE7")
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
