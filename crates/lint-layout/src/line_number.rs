use newtype::Display;
use syntax::Span;

#[derive(Display, newtype::Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[newtype(from = LineNumber)]
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
