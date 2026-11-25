use syntax::Span;

use crate::line_number::LineNumber;

#[derive(newtype::Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[newtype(from = Column)]
struct Column(usize);

#[derive(newtype::Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    line: LineNumber,
    column: Column,
}

impl Position {
    #[must_use]
    pub fn line(self) -> LineNumber {
        self.line
    }
}

impl From<Span> for Position {
    fn from(span: Span) -> Position {
        let start = span.start();
        Position { line: LineNumber::from(start.line), column: Column(start.column) }
    }
}
