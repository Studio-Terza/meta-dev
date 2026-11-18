use std::str::FromStr;

use grammar::logos::Logos;
use grammar::{Ignoring, Unlexed};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Spelling<'text> {
    #[regex("[^#]+", |lexer| lexer.slice())]
    Message(&'text str),
    #[token("#")]
    Hash,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Index {
    #[regex("0|[1-9][0-9]*", |lexer| lexer.slice().parse().ignoring(Unlexed::OutOfRange))]
    Ordinal(u32),
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Unanchored {
    #[error("{0:?} names no message: an anchor is a message id, optionally followed by # and an index")]
    Messageless(String),
    #[error("{0:?} has something after # that is not one plain index, as in #3")]
    Unindexed(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Anchoring {
    message: String,
    ordinal: Option<u32>,
}

impl FromStr for Anchoring {
    type Err = Unanchored;

    fn from_str(text: &str) -> Result<Anchoring, Unanchored> {
        let spelled: Vec<Spelling<'_>> =
            Spelling::lexer(text).collect::<Result<_, ()>>().ignoring(Unlexed::NotThisGrammar).unwrap_or_default();
        let (first, rest) = spelled.split_first().ok_or_else(|| Unanchored::Messageless(text.to_owned()))?;
        let message = first.message().ok_or_else(|| Unanchored::Messageless(text.to_owned()))?;
        let ordinal = rest
            .first()
            .map_or(Some(None), |_hash| {
                <&[Spelling<'_>; 2]>::try_from(rest).ignoring(Unlexed::OtherShape).and_then(Index::hashed).map(Some)
            })
            .ok_or_else(|| Unanchored::Unindexed(text.to_owned()))?;
        Ok(Anchoring { message: message.to_owned(), ordinal })
    }
}

impl Anchoring {
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub fn ordinal(&self) -> Option<u32> {
        self.ordinal
    }
}

impl<'text> Spelling<'text> {
    fn message(self) -> Option<&'text str> {
        match self {
            Spelling::Message(message) => Some(message),
            Spelling::Hash => None,
        }
    }
}

impl Index {
    fn hashed([hash, digits]: &[Spelling<'_>; 2]) -> Option<u32> {
        Some(hash).filter(|spelling| **spelling == Spelling::Hash)?;
        Index::read(digits.message()?)
    }

    fn read(digits: &str) -> Option<u32> {
        let spelled: Vec<Result<Index, ()>> = Index::lexer(digits).collect();
        <[Result<Index, ()>; 1]>::try_from(spelled)
            .ignoring(Unlexed::OtherShape)
            .and_then(|[only]| only.ignoring(Unlexed::NotThisGrammar))
            .map(|Index::Ordinal(ordinal)| ordinal)
    }
}
