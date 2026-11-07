use newtype::Display;
use proc_macro2::Span;

#[derive(Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineNumber(usize);

impl LineNumber {
    pub const FIRST: Self = Self(1);
}

impl From<usize> for LineNumber {
    fn from(line: usize) -> Self {
        Self(line)
    }
}

impl From<Span> for LineNumber {
    fn from(span: Span) -> Self {
        Self(span.start().line)
    }
}
