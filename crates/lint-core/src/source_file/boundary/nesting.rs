use std::fmt;

use lint_layout::SourceText;
use newtype::{Arbitrary, Counter, Newtype};

use crate::Most;

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Depth(usize);

impl Depth {
    fn receded(self) -> Depth {
        Depth(self.0.saturating_sub(1))
    }

    fn angled(self, step: Step) -> Depth {
        match step {
            Step::FatArrow | Step::Open | Step::Close | Step::End => Depth::zero(),
            Step::ThinArrow | Step::Plain => self,
            Step::Less => self.incremented(),
            Step::Greater => self.receded(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    Open,
    Close,
    End,
    Less,
    Greater,
    FatArrow,
    ThinArrow,
    Plain,
}

impl Step {
    const ARROWS: [([char; 2], Step); 2] = [(['=', '>'], Step::FatArrow), (['-', '>'], Step::ThinArrow)];
    const MARKS: [(char, Step); 9] = [
        ('(', Step::Open),
        ('[', Step::Open),
        ('{', Step::Open),
        (')', Step::Close),
        (']', Step::Close),
        ('}', Step::Close),
        (';', Step::End),
        ('<', Step::Less),
        ('>', Step::Greater),
    ];

    fn between(before: char, current: char) -> Step {
        Self::ARROWS
            .into_iter()
            .find(|(arrow, _)| *arrow == [before, current])
            .map(|(_, step)| step)
            .or_else(|| Self::MARKS.into_iter().find(|(mark, _)| *mark == current).map(|(_, step)| step))
            .unwrap_or(Step::Plain)
    }
}

#[derive(Debug)]
pub struct Nesting;

impl Nesting {
    const LIMIT: Most = Most::new(256);

    pub fn within_limit(text: &SourceText) -> Option<&SourceText> {
        let chars: Vec<char> = text.as_ref().chars().collect();
        let deepest = Self::peak(&chars, Self::delimited).max(Self::peak(&chars, Depth::angled));
        Self::LIMIT.allows(deepest).map(|_| text)
    }

    fn peak(chars: &[char], step: impl Fn(Depth, Step) -> Depth) -> Depth {
        chars
            .iter()
            .scan((Depth::zero(), ' '), |(depth, before), current| {
                *depth = step(*depth, Step::between(*before, *current));
                *before = *current;
                Some(*depth)
            })
            .max()
            .unwrap_or_default()
    }

    fn delimited(depth: Depth, step: Step) -> Depth {
        match step {
            Step::Open => depth.incremented(),
            Step::Close => depth.receded(),
            Step::End | Step::Less | Step::Greater | Step::FatArrow | Step::ThinArrow | Step::Plain => depth,
        }
    }
}

impl fmt::Display for Nesting {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "nested more than {} levels deep, which no handwritten code is", Self::LIMIT)
    }
}
