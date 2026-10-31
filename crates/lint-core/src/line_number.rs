use std::fmt;

use proc_macro2::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LineNumber(usize);

impl LineNumber {
    #[must_use]
    pub const fn new(line: usize) -> LineNumber {
        LineNumber(line)
    }
}

impl From<Span> for LineNumber {
    fn from(span: Span) -> LineNumber {
        LineNumber(span.start().line)
    }
}

impl fmt::Display for LineNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
