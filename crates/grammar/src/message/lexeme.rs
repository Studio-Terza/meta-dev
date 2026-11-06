use super::{Mark, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lexeme {
    pub mark: Mark,
    pub span: Span,
}
