use std::fmt;

use lint_layout::SourceText;
use newtype::{Counter, Newtype};

use crate::Most;

#[derive(Newtype, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Depth(usize);

impl Depth {
    fn receded(self) -> Depth {
        Depth(self.0.saturating_sub(1))
    }

    fn angled(self, window: &[char]) -> Depth {
        match window {
            ['=', '>'] | [_, '(' | '[' | '{' | ')' | ']' | '}' | ';'] => Depth::zero(),
            ['-', '>'] => self,
            [_, '<'] => self.incremented(),
            [_, '>'] => self.receded(),
            _ => self,
        }
    }
}

#[derive(Debug)]
pub struct Nesting;

impl Nesting {
    const LIMIT: Most = Most::new(256);

    pub fn within_limit(text: &SourceText) -> bool {
        let chars: Vec<char> = text.as_ref().chars().collect();
        let deepest = Self::peak(&chars, Self::delimited).max(Self::peak(&chars, Depth::angled));
        Self::LIMIT.allows(deepest.count())
    }

    fn peak(chars: &[char], step: impl Fn(Depth, &[char]) -> Depth) -> Depth {
        chars
            .iter()
            .scan((Depth::zero(), ' '), |(depth, before), current| {
                *depth = step(*depth, &[*before, *current]);
                *before = *current;
                Some(*depth)
            })
            .max()
            .unwrap_or_default()
    }

    fn delimited(depth: Depth, window: &[char]) -> Depth {
        match window {
            [_, '(' | '[' | '{'] => depth.incremented(),
            [_, ')' | ']' | '}'] => depth.receded(),
            _ => depth,
        }
    }
}

impl fmt::Display for Nesting {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "nested more than {} levels deep, which no handwritten code is", Self::LIMIT)
    }
}
