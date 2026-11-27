use crate::{Ignoring, Unlexed};
use logos::Logos;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = logos)]
#[logos(skip r"[ \t\r\n]+")]
enum Piece<'text> {
    #[regex("[A-Z][a-z][a-z]", |lexer| lexer.slice())]
    Word(&'text str),
    #[regex("[0-9]+", |lexer| lexer.slice().parse::<u16>().ignoring(Unlexed::OutOfRange))]
    Number(u16),
    #[token(":")]
    Colon,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Unstamped {
    #[error("{0:?} is not nine parts: a weekday, a month, a day, hh:mm:ss and a year")]
    Unshaped(String),
    #[error("{0:?} has a word where a number belongs, or a number where a word belongs")]
    Misplaced(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stamp<'text> {
    words: [&'text str; 2],
    numbers: [u16; 5],
}

impl<'text> Piece<'text> {
    fn word(self) -> Option<&'text str> {
        match self {
            Piece::Word(word) => Some(word),
            Piece::Number(_) | Piece::Colon => None,
        }
    }

    fn number(self) -> Option<u16> {
        match self {
            Piece::Number(number) => Some(number),
            Piece::Word(_) | Piece::Colon => None,
        }
    }

    fn colon(self) -> Option<Piece<'text>> {
        match self {
            Piece::Colon => Some(self),
            Piece::Word(_) | Piece::Number(_) => None,
        }
    }
}

impl<'text> TryFrom<&'text str> for Stamp<'text> {
    type Error = Unstamped;

    fn try_from(text: &'text str) -> Result<Stamp<'text>, Unstamped> {
        let unshaped = || Unstamped::Unshaped(text.to_owned());
        let pieces: Vec<Piece<'text>> =
            Piece::lexer(text).collect::<Result<Vec<Piece<'text>>, ()>>().map_err(|()| unshaped())?;
        let [weekday, month, day, hour, before, minute, after, second, year] =
            <[Piece<'text>; 9]>::try_from(pieces).map_err(|_miscounted| unshaped())?;
        before.colon().zip(after.colon()).ok_or_else(unshaped)?;
        let misplaced = || Unstamped::Misplaced(text.to_owned());
        Ok(Stamp {
            words: [weekday.word().ok_or_else(misplaced)?, month.word().ok_or_else(misplaced)?],
            numbers: [
                day.number().ok_or_else(misplaced)?,
                hour.number().ok_or_else(misplaced)?,
                minute.number().ok_or_else(misplaced)?,
                second.number().ok_or_else(misplaced)?,
                year.number().ok_or_else(misplaced)?,
            ],
        })
    }
}

impl<'text> Stamp<'text> {
    #[must_use]
    pub fn words(&self) -> [&'text str; 2] {
        self.words
    }

    #[must_use]
    pub fn numbers(&self) -> [u16; 5] {
        self.numbers
    }
}
