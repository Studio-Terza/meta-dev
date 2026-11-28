use std::vec::IntoIter;

use invariants::Ignoring;
use logos::{Lexer, Logos};

use crate::Unlexed;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Piece {
    #[regex("[^\n]+", allow_greedy = true)]
    Text,
    #[token("\n")]
    Break,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Cell {
    #[regex("[^\t]+", allow_greedy = true)]
    Text,
    #[token("\t")]
    Tab,
}

#[derive(Debug)]
struct Broken<'text>(Lexer<'text, Piece>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fields(Vec<String>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lines(Vec<String>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeptLines(Vec<String>);

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
        let ending = lexer.next().map_or_default(|_break| lexer.slice());
        Line { text: text.to_owned(), kept: [text, ending].concat() }
    }
}

impl Iterator for Broken<'_> {
    type Item = Line;

    fn next(&mut self) -> Option<Line> {
        let piece = self.0.next()?.ignoring(Unlexed::NotThisGrammar).unwrap_or(Piece::Text);
        Some(piece.line(&mut self.0))
    }
}

impl From<&str> for Lines {
    fn from(text: &str) -> Lines {
        Lines(Broken(Piece::lexer(text)).map(|line| line.text).collect())
    }
}

impl From<&str> for KeptLines {
    fn from(text: &str) -> KeptLines {
        KeptLines(Broken(Piece::lexer(text)).map(|line| line.kept).collect())
    }
}

impl From<&str> for Fields {
    fn from(text: &str) -> Fields {
        let cells = Cell::lexer(text).spanned().filter(|(cell, _)| *cell == Ok(Cell::Text));
        Fields(cells.filter_map(|(_, span)| text.get(span)).map(str::to_owned).collect())
    }
}

impl AsRef<[String]> for Fields {
    fn as_ref(&self) -> &[String] {
        &self.0
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
