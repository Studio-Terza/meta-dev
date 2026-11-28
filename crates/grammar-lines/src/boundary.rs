use std::str::FromStr;
use std::vec::IntoIter;

use grammar::logos::Logos;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Piece {
    #[regex("[^\n]+", allow_greedy = true)]
    Text,
    #[token("\n")]
    Break,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Cell {
    #[regex("[^\t]+", allow_greedy = true)]
    Text,
    #[token("\t")]
    Tab,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fields(Vec<String>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lines(Vec<String>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Unsplit {}

impl FromStr for Lines {
    type Err = Unsplit;

    fn from_str(text: &str) -> Result<Lines, Unsplit> {
        let (mut lines, open) = Piece::lexer(text).spanned().fold(
            (Vec::new(), None),
            |(mut lines, open): (Vec<String>, Option<String>), (piece, span)| match piece {
                Ok(Piece::Break) => {
                    lines.push(open.unwrap_or_default());
                    (lines, None)
                }
                Ok(Piece::Text) | Err(()) => (lines, text.get(span).map(str::to_owned)),
            },
        );
        lines.extend(open);
        Ok(Lines(lines))
    }
}

impl FromStr for Fields {
    type Err = Unsplit;

    fn from_str(text: &str) -> Result<Fields, Unsplit> {
        let cells = Cell::lexer(text).spanned().filter(|(cell, _)| *cell == Ok(Cell::Text));
        Ok(Fields(cells.filter_map(|(_, span)| text.get(span)).map(str::to_owned).collect()))
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
