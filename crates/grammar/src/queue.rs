use grammar_core::logos::Logos;

use crate::{Ignoring, Unlexed};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar_core::logos)]
enum Piece {
    #[regex("- [^\n]*", priority = 4, allow_greedy = true)]
    Item,
    #[token("## Deferred", priority = 8)]
    Deferred,
    #[regex("## [^\n]*", priority = 6, allow_greedy = true)]
    Section,
    #[regex("[^\n]+", allow_greedy = true)]
    Other,
    #[token("\n")]
    Break,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Waiting {
    Open,
    Deferred,
}

#[derive(Clone, Debug)]
struct Reading {
    waiting: Waiting,
    items: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Queue(Vec<String>);

impl Reading {
    fn read(self, piece: Piece, line: &str) -> Reading {
        match (piece, self.waiting) {
            (Piece::Item, Waiting::Open) => {
                let mut items = self.items;
                items.push(line.to_owned());
                Reading { items, ..self }
            }
            (Piece::Deferred, _) => Reading { waiting: Waiting::Deferred, ..self },
            (Piece::Section, _) => Reading { waiting: Waiting::Open, ..self },
            (Piece::Item, Waiting::Deferred) | (Piece::Other | Piece::Break, _) => self,
        }
    }
}

impl Queue {
    #[must_use]
    pub fn read(text: &str) -> Queue {
        let unread = Reading { waiting: Waiting::Open, items: Vec::new() };
        let reading = Piece::lexer(text)
            .spanned()
            .filter_map(|(lexed, span)| lexed.ignoring(Unlexed::NotThisGrammar).zip(text.get(span)))
            .fold(unread, |reading, (piece, line)| reading.read(piece, line));
        Queue(reading.items)
    }

    #[must_use]
    pub fn items(&self) -> &[String] {
        &self.0
    }
}
