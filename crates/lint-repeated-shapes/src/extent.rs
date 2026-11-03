use proc_macro2::{LineColumn, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Extent {
    start: LineColumn,
    end: LineColumn,
}

impl Extent {
    pub(crate) fn holds<'inner>(&self, inner: &'inner Extent) -> Option<&'inner Extent> {
        Some(inner).filter(|inner| self.start <= inner.start).filter(|inner| inner.end <= self.end)
    }
}

impl From<Span> for Extent {
    fn from(span: Span) -> Extent {
        Extent { start: span.start(), end: span.end() }
    }
}
