use newtype::Display;
use proc_macro2::Span;

#[derive(Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LineNumber(usize);

impl LineNumber {
    pub const FIRST: LineNumber = LineNumber(1);
}

impl From<usize> for LineNumber {
    fn from(line: usize) -> LineNumber {
        LineNumber(line)
    }
}

impl From<Span> for LineNumber {
    fn from(span: Span) -> LineNumber {
        LineNumber(span.start().line)
    }
}
