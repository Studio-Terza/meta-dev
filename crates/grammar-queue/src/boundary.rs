use grammar::logos::Logos;
use grammar::{Ignoring, Unlexed};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Piece {
    #[regex("- [^\n]*", priority = 4, allow_greedy = true)]
    Item,
    #[regex("[^\n]+", allow_greedy = true)]
    Other,
    #[token("\n")]
    Break,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Queue(Vec<String>);

impl Piece {
    fn item(self, text: &str) -> Option<String> {
        match self {
            Piece::Item => Some(text.to_owned()),
            Piece::Other | Piece::Break => None,
        }
    }
}

impl Queue {
    #[must_use]
    pub fn read(text: &str) -> Queue {
        Queue(
            Piece::lexer(text)
                .spanned()
                .filter_map(|(lexed, span)| {
                    lexed
                        .ignoring(Unlexed::NotThisGrammar)
                        .zip(text.get(span))
                        .and_then(|(piece, line)| piece.item(line))
                })
                .collect(),
        )
    }

    #[must_use]
    pub fn items(&self) -> &[String] {
        &self.0
    }
}
