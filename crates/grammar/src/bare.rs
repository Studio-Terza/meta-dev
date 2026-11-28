use logos::Logos;

use crate::{Grammar, Whole};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Lexeme {
    #[regex(r"[A-Za-z0-9_][A-Za-z0-9_-]*")]
    Bare,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bare;

impl Grammar<'_> for Bare {
    fn read(text: &str) -> Option<Bare> {
        Whole::exactly(text).map(|[Lexeme::Bare]| Bare)
    }
}
