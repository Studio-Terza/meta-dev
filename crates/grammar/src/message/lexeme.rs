use super::{Mark, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lexeme {
    mark: Mark,
    span: Span,
}

impl Lexeme {
    pub(crate) fn new(mark: Mark, span: Span) -> Lexeme {
        Lexeme { mark, span }
    }

    #[must_use]
    pub fn mark(&self) -> Mark {
        self.mark
    }

    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }
}
