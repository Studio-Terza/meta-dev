mod boundary;

use algebra::Semigroup;
use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    start: Position,
    end: Position,
}

impl Position {
    pub(crate) fn without_sentinel(self) -> Position {
        Position(self.0.saturating_sub(1))
    }
}

impl Semigroup for Span {
    fn combine(self, later: Span) -> Span {
        Span { start: self.start, end: later.end }
    }
}

impl Span {
    pub(crate) fn new(start: Position, end: Position) -> Span {
        Span { start, end }
    }

    #[must_use]
    pub fn emptied(self) -> Span {
        Span { start: self.start, end: self.start }
    }
}
