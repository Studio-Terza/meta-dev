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
    const ARROWS: [([char; 2], Step); 2] = [(['=', '>'], Step::Reset), (['-', '>'], Step::Level)];
    const ANGLED: [(char, Step); 9] = [
        ('(', Step::Reset),
        ('[', Step::Reset),
        ('{', Step::Reset),
        (')', Step::Reset),
        (']', Step::Reset),
        ('}', Step::Reset),
        (';', Step::Reset),
        ('<', Step::Deeper),
        ('>', Step::Shallower),
    ];
    const DELIMITED: [(char, Step); 6] = [
        ('(', Step::Deeper),
        ('[', Step::Deeper),
        ('{', Step::Deeper),
        (')', Step::Shallower),
        (']', Step::Shallower),
        ('}', Step::Shallower),
    ];

    fn angled(before: char, current: char) -> Step {
        Self::ARROWS
            .into_iter()
            .find(|(arrow, _)| *arrow == [before, current])
            .map(|(_, step)| step)
            .or_else(|| Self::ANGLED.into_iter().find(|(mark, _)| *mark == current).map(|(_, step)| step))
            .unwrap_or(Step::Level)
    }

    fn delimited(current: char) -> Step {
        Self::DELIMITED.into_iter().find(|(mark, _)| *mark == current).map_or(Step::Level, |(_, step)| step)
    }
}

#[derive(Debug)]
pub struct Nesting;

impl Nesting {
    const LIMIT: Most = Most::new(256);

    pub fn within_limit(text: &SourceText) -> Option<&SourceText> {
        let chars: Vec<char> = text.as_ref().chars().collect();
        let deepest = Self::peak(&chars, |_, current| Step::delimited(current)).max(Self::peak(&chars, Step::angled));
        Self::LIMIT.allows(deepest).map(|_| text)
    }

    fn peak(chars: &[char], step: impl Fn(char, char) -> Step) -> Depth {
        chars
            .iter()
            .scan((Depth::zero(), ' '), |(depth, before), current| {
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
