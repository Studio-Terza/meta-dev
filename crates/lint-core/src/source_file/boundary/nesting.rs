use std::fmt;

use lint_layout::SourceText;
use newtype::{Arbitrary, Counter, Newtype};

use crate::Bound;

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Depth(usize);

impl Depth {
    fn receded(self) -> Self {
        Self(self.0.saturating_sub(1))
    }

    fn after(self, step: Step) -> Self {
        match step {
            Step::Reset => Self::zero(),
            Step::Deeper => self.incremented(),
            Step::Shallower => self.receded(),
            Step::Level => self,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    Reset,
    Deeper,
    Shallower,
    Level,
}

impl Step {
    const ARROWS: [([u8; 2], Self); 2] = [(*b"=>", Self::Reset), (*b"->", Self::Level)];
    const ANGLED: [(u8, Self); 9] = [
        (b'(', Self::Reset),
        (b'[', Self::Reset),
        (b'{', Self::Reset),
        (b')', Self::Reset),
        (b']', Self::Reset),
        (b'}', Self::Reset),
        (b';', Self::Reset),
        (b'<', Self::Deeper),
        (b'>', Self::Shallower),
    ];
    const DELIMITED: [(u8, Self); 6] = [
        (b'(', Self::Deeper),
        (b'[', Self::Deeper),
        (b'{', Self::Deeper),
        (b')', Self::Shallower),
        (b']', Self::Shallower),
        (b'}', Self::Shallower),
    ];

    fn angled(before: u8, current: u8) -> Self {
        Self::ARROWS
            .into_iter()
            .find(|(arrow, _)| *arrow == [before, current])
            .map(|(_, step)| step)
            .or_else(|| Self::ANGLED.into_iter().find(|(mark, _)| *mark == current).map(|(_, step)| step))
            .unwrap_or(Self::Level)
    }

    fn delimited(current: u8) -> Self {
        Self::DELIMITED.into_iter().find(|(mark, _)| *mark == current).map_or(Self::Level, |(_, step)| step)
    }
}

#[derive(Debug)]
pub struct Nesting;

impl Nesting {
    const LIMIT: Bound = Bound::Most(256);

    pub fn within_limit(text: &SourceText) -> Option<&SourceText> {
        let bytes = text.as_ref().as_bytes();
        let deepest = Self::peak(bytes, |_, current| Step::delimited(current)).max(Self::peak(bytes, Step::angled));
        Self::LIMIT.allows(deepest).map(|_| text)
    }

    fn peak(bytes: &[u8], step: impl Fn(u8, u8) -> Step) -> Depth {
        bytes
            .iter()
            .scan((Depth::zero(), b' '), |(depth, before), current| {
                *depth = depth.after(step(*before, *current));
                *before = *current;
                Some(*depth)
            })
            .max()
            .unwrap_or_default()
    }
}

impl fmt::Display for Nesting {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "nested more than {} levels deep, which no handwritten code is", Self::LIMIT)
    }
}
