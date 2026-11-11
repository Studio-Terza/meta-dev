use grammar::logos::Logos;

use super::Terms;
use crate::Term;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Piece {
    #[regex(r"[^\x00-\x2F\x3A-\x40\x5B-\x60\x7B-\x7F]+")]
    Run,
    #[regex(r"[\x00-\x2F\x3A-\x40\x5B-\x60\x7B-\x7F]")]
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
            .filter(|(piece, _)| *piece == Ok(Piece::Run))
            .filter_map(|(_, range)| text.get(range))
            .flat_map(Piece::alphanumeric)
            .map(|word| Term::new(word.into_iter().flat_map(char::to_lowercase).collect()))
            .collect()
    }

    fn alphanumeric(run: &str) -> Vec<Vec<char>> {
        let letters: Vec<char> = run.chars().collect();
        letters
            .chunk_by(|before, after| before.is_alphanumeric() == after.is_alphanumeric())
            .filter(|chunk| chunk.first().is_some_and(|first| first.is_alphanumeric()))
            .map(<[char]>::to_vec)
            .collect()
    }
}
