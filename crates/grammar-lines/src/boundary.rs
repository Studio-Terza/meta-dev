use std::str::FromStr;
use std::vec::IntoIter;

use grammar::logos::{Lexer, Logos};
use grammar::{Ignoring, Unlexed};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Piece {
    #[regex("[^\n]+", allow_greedy = true)]
    Text,
    #[token("\n")]
    Break,
}

#[derive(Debug)]
struct Broken<'text>(Lexer<'text, Piece>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lines(Vec<String>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Unsplit {}

impl Piece {
    fn line(self, lexer: &mut Lexer<'_, Piece>) -> String {
        match self {
            Piece::Break => String::new(),
            Piece::Text => {
                let text = lexer.slice().to_owned();
                lexer.next();
                text
            }
        }
    }
}

impl Iterator for Broken<'_> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        let piece = self.0.next()?.ignoring(Unlexed::NotThisGrammar).unwrap_or(Piece::Text);
        Some(piece.line(&mut self.0))
    }
}

impl FromStr for Lines {
    type Err = Unsplit;

    fn from_str(text: &str) -> Result<Lines, Unsplit> {
        Ok(Lines(Broken(Piece::lexer(text)).collect()))
    }
}

impl IntoIterator for Lines {
    type Item = String;
    type IntoIter = IntoIter<String>;

    fn into_iter(self) -> IntoIter<String> {
        self.0.into_iter()
    }
}
