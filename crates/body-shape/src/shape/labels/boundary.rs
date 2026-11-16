#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Code(u64);

impl Code {
    const SHIFT: u32 = 58;
    const PAYLOAD: u64 = (1 << Self::SHIFT) - 1;
    const MARK: u64 = 1;
    const WORD: u64 = 2;
    const LITERAL: u64 = 4;
    const DISTANCE: u64 = 6;
    const BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0100_0000_01b3;
    pub(crate) const ERASED: Code = Code::tagged(3, 0);
    pub(crate) const NEW: Code = Code::tagged(5, 0);

    const fn tagged(tag: u64, payload: u64) -> Code {
        Code((tag << Self::SHIFT) | (payload & Self::PAYLOAD))
    }

    pub(crate) fn mark(mark: char) -> Code {
        Code::tagged(Self::MARK, u64::from(mark))
    }

    pub(crate) fn word(text: &str) -> Code {
        Code::tagged(
            Self::WORD,
            text.bytes().fold(Self::BASIS, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(Self::PRIME)),
        )
    }

    pub(crate) fn literal(self) -> Code {
        Code::tagged(Self::LITERAL, self.0)
    }

    pub(crate) fn distance(shaped: usize) -> Code {
        Code::tagged(Self::DISTANCE, u64::try_from(shaped).unwrap_or(Self::PAYLOAD))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Labels(Vec<Code>);

impl Labels {
    pub(crate) fn pushed(mut self, code: Code) -> Labels {
        self.0.push(code);
        self
    }

    pub(crate) fn count(&self) -> usize {
        self.0.len()
    }
}
