use std::fmt;

use lint_layout::SourceText;
use newtype::{Arbitrary, Counter, Newtype};

use crate::Bound;

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Depth(usize);

impl Depth {
    fn receded(self) -> Depth {
        Depth(self.0.saturating_sub(1))
    }

    fn after(self, step: Step) -> Depth {
        match step {
            Step::Reset => Depth::zero(),
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
    const ARROWS: [([u8; 2], Step); 2] = [(*b"=>", Step::Reset), (*b"->", Step::Level)];
    const ANGLED: [(u8, Step); 9] = [
        (b'(', Step::Reset),
        (b'[', Step::Reset),
        (b'{', Step::Reset),
        (b')', Step::Reset),
        (b']', Step::Reset),
        (b'}', Step::Reset),
        (b';', Step::Reset),
        (b'<', Step::Deeper),
        (b'>', Step::Shallower),
    ];
    const DELIMITED: [(u8, Step); 6] = [
        (b'(', Step::Deeper),
        (b'[', Step::Deeper),
        (b'{', Step::Deeper),
        (b')', Step::Shallower),
        (b']', Step::Shallower),
        (b'}', Step::Shallower),
    ];

    fn angled(before: u8, current: u8) -> Step {
        Self::ARROWS
            .into_iter()
            .find(|(arrow, _)| *arrow == [before, current])
            .map(|(_, step)| step)
            .or_else(|| Self::ANGLED.into_iter().find(|(mark, _)| *mark == current).map(|(_, step)| step))
            .unwrap_or(Step::Level)
    }

    fn delimited(current: u8) -> Step {
        Self::DELIMITED.into_iter().find(|(mark, _)| *mark == current).map_or(Step::Level, |(_, step)| step)
    }
}

#[derive(Debug)]
pub struct Nesting;

impl Nesting {
    const LIMIT: Bound = Bound::Most(256);

    pub fn within_limit(text: &SourceText) -> Option<&SourceText> {
        let peaks = text.as_ref().as_bytes().iter().fold(Peaks::default(), |peaks, current| peaks.after(*current));
        Self::LIMIT.allows(peaks.deepest).map(|_| text)
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Peaks {
    delimited: Depth,
    angled: Depth,
    before: Option<u8>,
    deepest: Depth,
}

impl Peaks {
    const START: u8 = b' ';

    fn after(self, current: u8) -> Peaks {
        let delimited = self.delimited.after(Step::delimited(current));
        let angled = self.angled.after(Step::angled(self.before.unwrap_or(Self::START), current));
        Peaks { delimited, angled, before: Some(current), deepest: self.deepest.max(delimited).max(angled) }
    }
}

impl fmt::Display for Nesting {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "nested more than {} levels deep, which no handwritten code is", Self::LIMIT)
    }
}
