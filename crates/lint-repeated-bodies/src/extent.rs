use proc_macro2::{LineColumn, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Extent {
    start: LineColumn,
    end: LineColumn,
}

impl Extent {
    pub(crate) fn holds(&self, inner: &Extent) -> bool {
        self.start <= inner.start && inner.end <= self.end
    }
}

impl From<Span> for Extent {
    fn from(span: Span) -> Extent {
        Extent { start: span.start(), end: span.end() }
    }
}
