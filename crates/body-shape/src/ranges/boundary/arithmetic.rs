#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct Code(u64);

impl Code {
    const SHIFT: u32 = 58;
    const PAYLOAD: u64 = (1 << Self::SHIFT) - 1;
    const MARK: u64 = 1;
    const WORD: u64 = 2;
    const LITERAL: u64 = 4;
    const DISTANCE: u64 = 6;
    const BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0100_0000_01b3;
    pub(super) const ERASED: Code = Code::tagged(3, 0);
    pub(super) const NEW: Code = Code::tagged(5, 0);

    const fn tagged(tag: u64, payload: u64) -> Code {
        Code((tag << Self::SHIFT) | (payload & Self::PAYLOAD))
    }

    pub(super) fn mark(mark: char) -> Code {
        Code::tagged(Self::MARK, u64::from(mark))
    }

    pub(super) fn word(text: &str) -> Code {
        Code::tagged(
            Self::WORD,
            text.bytes().fold(Self::BASIS, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(Self::PRIME)),
        )
    }

    pub(super) fn literal(self) -> Code {
        Code::tagged(Self::LITERAL, self.0)
    }

    pub(super) fn distance(shaped: usize) -> Code {
        Code::tagged(Self::DISTANCE, u64::try_from(shaped).unwrap_or(Self::PAYLOAD))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Modular {
    Times,
    Plus,
    Minus,
}

impl Modular {
    const MODULUS: u64 = (1 << 61) - 1;
    const BITS: u32 = 61;
    const BASES: [u64; 2] = [0x1f2e_3d4c_5b6a_7988 % Self::MODULUS, 0x0123_4567_89ab_cdef % Self::MODULUS];

    fn applied(self, left: u64, right: u64) -> u64 {
        match self {
            Modular::Times => Self::folded(u128::from(left) * u128::from(right)),
            Modular::Plus => Self::folded(u128::from(left) + u128::from(right)),
            Modular::Minus => Self::folded(u128::from(left) + u128::from(Self::MODULUS) - u128::from(right)),
        }
    }

    fn folded(value: u128) -> u64 {
        let once = (value & u128::from(Self::MODULUS)) + (value >> Self::BITS);
        let twice = (once & u128::from(Self::MODULUS)) + (once >> Self::BITS);
        let reduced = u64::try_from(twice).unwrap_or(0);
        Some(reduced).filter(|found| *found >= Self::MODULUS).map_or(reduced, |over| over - Self::MODULUS)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Hashed {
    first: u64,
    second: u64,
}

impl Hashed {
    const ONE: Hashed = Hashed { first: 1, second: 1 };
    const HALF: u32 = 32;

    fn code(code: Code) -> u64 {
        code.0 % Modular::MODULUS
    }

    pub(super) fn appended(self, code: Code) -> Hashed {
        let [first, second] = Modular::BASES;
        Hashed {
            first: Modular::Plus.applied(Modular::Times.applied(self.first, first), Self::code(code)),
            second: Modular::Plus.applied(Modular::Times.applied(self.second, second), Self::code(code)),
        }
    }

    pub(super) fn folded(self) -> u64 {
        self.first ^ self.second.rotate_left(Self::HALF)
    }

    fn raised(self) -> Hashed {
        let [first, second] = Modular::BASES;
        Hashed { first: Modular::Times.applied(self.first, first), second: Modular::Times.applied(self.second, second) }
    }
}

#[derive(Debug, Default)]
pub(super) struct Polynomial {
    prefix: Vec<Hashed>,
    powers: Vec<Hashed>,
}

impl Polynomial {
    pub(super) fn prefix(codes: &[Code]) -> Polynomial {
        let start = Polynomial { prefix: vec![Hashed::default()], powers: vec![Hashed::ONE] };
        codes.iter().fold(start, |mut built, code| {
            built.prefix.push(built.prefix.last().copied().unwrap_or_default().appended(*code));
            built.powers.push(built.powers.last().copied().unwrap_or_default().raised());
            built
        })
    }

    pub(super) fn range(&self, from: usize, to: usize) -> Hashed {
        let whole = self.prefix.get(to).copied().unwrap_or_default();
        let before = self.prefix.get(from).copied().unwrap_or_default();
        let power = self.power(to.saturating_sub(from));
        Hashed {
            first: Modular::Minus.applied(whole.first, Modular::Times.applied(before.first, power.first)),
            second: Modular::Minus.applied(whole.second, Modular::Times.applied(before.second, power.second)),
        }
    }

    pub(super) fn corrected(&self, hash: Hashed, was: Code, after: usize) -> Hashed {
        let power = self.power(after);
        let [old, new] = [Hashed::code(was), Hashed::code(Code::NEW)];
        let shifted = |value: u64, weight: u64, held: u64| {
            Modular::Plus.applied(
                Modular::Minus.applied(held, Modular::Times.applied(value, weight)),
                Modular::Times.applied(new, weight),
            )
        };
        Hashed { first: shifted(old, power.first, hash.first), second: shifted(old, power.second, hash.second) }
    }

    fn power(&self, exponent: usize) -> Hashed {
        self.powers.get(exponent).copied().unwrap_or_default()
    }
}
