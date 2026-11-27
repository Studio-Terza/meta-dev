use std::str::FromStr;

use invariants::Ignoring;
use logos::Logos;

use crate::{Unlexed, Unspelled};

#[derive(Logos, Clone, Debug, PartialEq, Eq)]
#[logos(crate = logos)]
enum Piece {
    #[regex("[0-9]+", |lexer| lexer.slice().parse().ignoring(Unlexed::OutOfRange), priority = 3)]
    Number(i64),
    #[regex("[a-z0-9]+", |lexer| lexer.slice().to_owned())]
    Word(String),
    #[token("_")]
    Underscore,
    #[token(".sql")]
    Sql,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationFile {
    version: i64,
    words: Vec<String>,
}

impl MigrationFile {
    const SPACE: &'static str = " ";

    #[must_use]
    pub fn version(&self) -> i64 {
        self.version
    }

    #[must_use]
    pub fn description(&self) -> String {
        self.words.join(Self::SPACE)
    }

    fn read(pieces: &[Piece]) -> Option<MigrationFile> {
        let (first, rest) = pieces.split_first()?;
        let (last, middle) = rest.split_last()?;
        Some(last).filter(|ending| **ending == Piece::Sql)?;
        let version = first.number()?;
        let (lead, tail) = middle.split_at_checked(2)?;
        let words =
            std::iter::once(lead).chain(tail.chunks(2)).map(Piece::described).collect::<Option<Vec<String>>>()?;
        Some(MigrationFile { version, words })
    }
}

impl Piece {
    fn number(&self) -> Option<i64> {
        match self {
            Piece::Number(number) => Some(*number),
            Piece::Word(_) | Piece::Underscore | Piece::Sql => None,
        }
    }

    fn worded(&self) -> Option<String> {
        match self {
            Piece::Word(word) => Some(word.clone()),
            Piece::Number(number) => Some(number.to_string()),
            Piece::Underscore | Piece::Sql => None,
        }
    }

    fn described(pair: &[Piece]) -> Option<String> {
        let [underscore, word] = <&[Piece; 2]>::try_from(pair).ignoring(Unlexed::OtherShape)?;
        Some(underscore).filter(|joint| **joint == Piece::Underscore).and_then(|_joint| word.worded())
    }
}

impl FromStr for MigrationFile {
    type Err = Unspelled;

    fn from_str(text: &str) -> Result<MigrationFile, Unspelled> {
        let pieces: Result<Vec<Piece>, ()> = Piece::lexer(text).collect();
        pieces.ignoring(Unlexed::NotThisGrammar).and_then(|lexed| MigrationFile::read(&lexed)).ok_or(Unspelled)
    }
}
