use std::str::FromStr;

use grammar::logos::Logos;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Spelling<'text> {
    #[regex("[A-Za-z0-9_][A-Za-z0-9_-]*", |lexer| lexer.slice())]
    Word(&'text str),
    #[token(".")]
    Dot,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Uncited {
    #[error("{0:?} is not a prefix and an ordinal joined by a dot, as in a.3")]
    Unshaped(String),
    #[error("{0:?} has an ordinal after its dot that is not a number")]
    Unnumbered(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cited {
    prefix: String,
    ordinal: u32,
}

impl<'text> Spelling<'text> {
    fn word(self) -> Option<&'text str> {
        match self {
            Spelling::Word(word) => Some(word),
            Spelling::Dot => None,
        }
    }
}

impl FromStr for Cited {
    type Err = Uncited;

    fn from_str(cited: &str) -> Result<Cited, Uncited> {
        let spelled: Vec<Spelling<'_>> = Spelling::lexer(cited).collect::<Result<_, ()>>().unwrap_or_default();
        let unshaped = || Uncited::Unshaped(cited.to_owned());
        let [prefix, dot, digits] = <[Spelling<'_>; 3]>::try_from(spelled).map_err(|_| unshaped())?;
        Some(dot).filter(|spelling| *spelling == Spelling::Dot).ok_or_else(unshaped)?;
        let word = prefix.word().ok_or_else(unshaped)?;
        let ordinal = digits.word().ok_or_else(unshaped)?.parse().map_err(|_| Uncited::Unnumbered(cited.to_owned()))?;
        Ok(Cited { prefix: word.to_owned(), ordinal })
    }
}

impl Cited {
    #[must_use]
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    #[must_use]
    pub fn ordinal(&self) -> u32 {
        self.ordinal
    }
}
