use std::ops::Range;

use grammar_core::logos::Logos;

use crate::{Ignoring, Unlexed};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar_core::logos)]
enum Piece {
    #[regex(r"[^\p{White_Space}]+", allow_greedy = true)]
    Word,
    #[regex(r"[^\p{White_Space}]*[.?!]", priority = 6, allow_greedy = true)]
    Ending,
    #[token("\n")]
    Break,
    #[regex(r"[\p{White_Space}&&[^\n]]+")]
    Space,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sentence {
    Open,
    Closed,
}

#[derive(Clone, Debug)]
struct Reading {
    words: usize,
    last: Option<Range<usize>>,
    sentence: Sentence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Prose<'text> {
    words: usize,
    last: Option<&'text str>,
}

impl Reading {
    const UNREAD: Reading = Reading { words: 0, last: None, sentence: Sentence::Closed };

    fn read(self, piece: Piece, span: Range<usize>) -> Reading {
        match piece {
            Piece::Word => self.worded(span, Sentence::Open),
            Piece::Ending => self.worded(span, Sentence::Closed),
            Piece::Break => Reading { sentence: Sentence::Closed, ..self },
            Piece::Space => self,
        }
    }

    fn worded(self, span: Range<usize>, sentence: Sentence) -> Reading {
        let start = match self.sentence {
            Sentence::Open => self.last.as_ref().map_or(span.start, |open| open.start),
            Sentence::Closed => span.start,
        };
        Reading { words: self.words.saturating_add(1), last: Some(start..span.end), sentence }
    }
}

impl<'text> Prose<'text> {
    #[must_use]
    pub fn read(text: &'text str) -> Prose<'text> {
        let reading = Piece::lexer(text)
            .spanned()
            .filter_map(|(lexed, span)| lexed.ignoring(Unlexed::NotThisGrammar).map(|piece| (piece, span)))
            .fold(Reading::UNREAD, |reading, (piece, span)| reading.read(piece, span));
        Prose { words: reading.words, last: reading.last.and_then(|span| text.get(span)) }
    }

    #[must_use]
    pub fn words(&self) -> usize {
        self.words
    }

    #[must_use]
    pub fn last_sentence(&self) -> Option<&'text str> {
        self.last
    }
}
