use std::str::FromStr;
use std::vec::IntoIter;

use logos::{Lexer, Logos};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeptLines(Vec<String>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Unsplit {}

#[derive(Clone, Debug)]
struct Line {
    text: String,
    kept: String,
}

impl Piece {
    fn line(self, lexer: &mut Lexer<'_, Piece>) -> Line {
        match self {
            Piece::Break => Line { text: String::new(), kept: lexer.slice().to_owned() },
            Piece::Text => Piece::broken(lexer),
        }
    }

    fn broken(lexer: &mut Lexer<'_, Piece>) -> Line {
        let text = lexer.slice();
        let ending = lexer.next().map(|_break| lexer.slice()).unwrap_or_default();
        Line { text: text.to_owned(), kept: [text, ending].concat() }
    }
}

impl Iterator for Broken<'_> {
    type Item = Line;

    fn next(&mut self) -> Option<Line> {
        let piece = self.0.next()?.unwrap_or(Piece::Text);
        Some(piece.line(&mut self.0))
    }
}

impl FromStr for Lines {
    type Err = Unsplit;

    fn from_str(text: &str) -> Result<Lines, Unsplit> {
        Ok(Lines(Broken(Piece::lexer(text)).map(|line| line.text).collect()))
    }
}

impl FromStr for KeptLines {
    type Err = Unsplit;

    fn from_str(text: &str) -> Result<KeptLines, Unsplit> {
        Ok(KeptLines(Broken(Piece::lexer(text)).map(|line| line.kept).collect()))
    }
}

impl IntoIterator for Lines {
    type Item = String;
    type IntoIter = IntoIter<String>;

    fn into_iter(self) -> IntoIter<String> {
        self.0.into_iter()
    }
}

impl IntoIterator for KeptLines {
    type Item = String;
    type IntoIter = IntoIter<String>;

    fn into_iter(self) -> IntoIter<String> {
        self.0.into_iter()
    }
}
