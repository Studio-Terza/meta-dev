use grammar::logos::{Lexer, Logos};
use grammar::{Ignoring, Unlexed};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Spelling {
    #[regex("n0|n[1-9][0-9]*", Spelling::counted)]
    #[regex("n0[0-9]+", |_padded| Spelled::Misnumbered)]
    Id(Spelled),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spelled {
    Numbered(i64),
    Misnumbered,
    Unspelled,
}

#[derive(Debug, thiserror::Error)]
pub enum Unnumbered {
    #[error("{0:?} is not a node id: its number has a leading zero or is too large for a node")]
    Misnumbered(String),
    #[error("{0:?} is not a node id, which is n followed by its number, as in n42")]
    Unspelled(String),
}

impl Spelled {
    pub fn numbered(self, text: &str) -> Result<i64, Unnumbered> {
        match self {
            Spelled::Numbered(number) => Ok(number),
            Spelled::Misnumbered => Err(Unnumbered::Misnumbered(text.to_owned())),
            Spelled::Unspelled => Err(Unnumbered::Unspelled(text.to_owned())),
        }
    }
}

impl Unnumbered {
    pub fn unspelled(self) -> Result<String, Unnumbered> {
        match self {
            Unnumbered::Unspelled(text) => Ok(text),
            misnumbered @ Unnumbered::Misnumbered(_) => Err(misnumbered),
        }
    }
}

#[derive(Debug)]
pub struct Ids;

impl Ids {
    #[must_use]
    pub fn spelled(text: &str) -> Spelled {
        let spelled: Vec<Result<Spelling, ()>> = Spelling::lexer(text).collect();
        <[Result<Spelling, ()>; 1]>::try_from(spelled)
            .ignoring(Unlexed::OtherShape)
            .and_then(|[only]| only.ignoring(Unlexed::NotThisGrammar))
            .map_or(Spelled::Unspelled, Spelling::spelled)
    }
}

impl Spelling {
    fn counted(lexer: &Lexer<'_, Spelling>) -> Spelled {
        lexer
            .slice()
            .get(1..)
            .and_then(|digits| digits.parse::<i64>().ignoring(Unlexed::OutOfRange))
            .map_or(Spelled::Misnumbered, Spelled::Numbered)
    }

    fn spelled(self) -> Spelled {
        match self {
            Spelling::Id(spelled) => spelled,
        }
    }
}
