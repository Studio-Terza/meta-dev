use logos::Logos;

use super::Terms;
use crate::search::Term;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Piece {
    #[regex(r"[\p{L}\p{N}]+")]
    Word,
    #[regex(r"[^\p{L}\p{N}]")]
    Separator,
}

impl Terms {
    #[must_use]
    pub fn lexed<'text>(words: impl IntoIterator<Item = &'text str>) -> Option<Terms> {
        words.into_iter().flat_map(Piece::words).fold(None, |terms, term| Some(Terms::extended(terms, term)))
    }

    fn extended(terms: Option<Terms>, term: Term) -> Terms {
        match terms {
            Some(Terms { first, rest }) => Terms { first, rest: rest.into_iter().chain([term]).collect() },
            None => Terms { first: term, rest: Vec::new() },
        }
    }
}

impl Piece {
    fn words(text: &str) -> Vec<Term> {
        Piece::lexer(text)
            .spanned()
            .filter(|(piece, _)| *piece == Ok(Piece::Word))
            .filter_map(|(_, range)| text.get(range))
            .map(|word| Term::new(word.chars().flat_map(char::to_lowercase).collect()))
            .collect()
    }
}
