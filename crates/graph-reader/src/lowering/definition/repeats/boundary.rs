use std::ops::Range;

use syntax::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Extent {
    start: usize,
    end: usize,
}

impl From<Span> for Extent {
    fn from(span: Span) -> Extent {
        let Range { start, end } = span.byte_range();
        Extent { start, end }
    }
}

impl Extent {
    pub(crate) fn holds(self, inner: Extent) -> Option<()> {
        Some(()).filter(|()| self.start <= inner.start).filter(|()| inner.end <= self.end)
    }
}

impl Extent {
    pub(crate) fn preorder(self) -> Preorder {
        Preorder { start: self.start, end: std::cmp::Reverse(self.end) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Preorder {
    start: usize,
    end: std::cmp::Reverse<usize>,
}
