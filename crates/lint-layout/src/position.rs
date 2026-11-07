use proc_macro2::Span;

use crate::line_number::LineNumber;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Column(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    line: LineNumber,
    column: Column,
}

impl Position {
    #[must_use]
    pub const fn line(self) -> LineNumber {
        self.line
    }
}

impl From<Span> for Position {
    fn from(span: Span) -> Self {
        let start = span.start();
        Self { line: LineNumber::from(start.line), column: Column(start.column) }
    }
}
