use std::ops::Range;

use logos::Logos;

use super::Lexemes;
use crate::message::lexeme::Lexeme;
use crate::message::span::Position;
use crate::message::{Depth, Mark, Number, Span};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Token {
    #[token("\n")]
    Break,
    #[regex(r"[.?!]+[ \t]+")]
    Stop,
    #[regex(r"\n(\*\*)?[0-9]+[.)](\*\*)?[ \t]+", Token::numbered)]
    Numbered(usize),
    #[regex(r"\n[ \t]+[0-9]+[.)][ \t]+", Token::numbered)]
    Nested(usize),
    #[regex(r"\n[ \t]*[-*•][ \t]+")]
    Bulleted,
    #[regex(r"\n[ \t]*>[ \t]?")]
    Quoted,
    #[regex(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}")]
    Uuid,
    #[regex(r"[^\n]")]
    Plain,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BareWord(String);

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Ink {
    #[regex(r"\p{White_Space}+")]
    Blank,
    #[regex(r"\P{White_Space}+")]
    Inked,
}

impl Lexemes {
    #[must_use]
    pub fn scanned(text: &impl AsRef<str>) -> Vec<Lexeme> {
        let sentinel = format!("\n{}", text.as_ref());
        Token::lexer(&sentinel)
            .spanned()
            .filter(|(_, range)| range.end > 1)
            .map(|(token, range)| {
                Lexeme::new(
                    token.unwrap_or(Token::Plain).mark(),
                    Ink::spanned(range.clone()),
                    Ink::inked(sentinel.get(range.clone()).unwrap_or_default(), range.start),
                )
            })
            .collect()
    }

    #[must_use]
    pub fn uuids(text: &impl AsRef<str>) -> Vec<BareWord> {
        Lexemes::scanned(text)
            .into_iter()
            .filter(|lexeme| matches!(lexeme.mark(), Mark::Uuid))
            .map(|lexeme| BareWord(lexeme.span().slice(text.as_ref()).to_owned()))
            .collect()
    }
}

impl BareWord {
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Ink {
    fn spanned(range: Range<usize>) -> Span {
        Span::new(Position::new(range.start).without_sentinel(), Position::new(range.end).without_sentinel())
    }

    fn inked(slice: &str, start: usize) -> Option<Span> {
        let inked: Vec<Range<usize>> =
            Ink::lexer(slice).spanned().filter(|(ink, _)| *ink == Ok(Ink::Inked)).map(|(_, range)| range).collect();
        inked
            .first()
            .zip(inked.last())
            .map(|(first, last)| Ink::spanned(start.saturating_add(first.start)..start.saturating_add(last.end)))
    }
}

impl Token {
    fn numbered(lexer: &logos::Lexer<'_, Token>) -> Option<usize> {
        lexer.slice().chars().filter(char::is_ascii_digit).collect::<String>().parse().ok()
    }

    fn mark(self) -> Mark {
        match self {
            Token::Break => Mark::Break,
            Token::Stop => Mark::Stop,
            Token::Numbered(number) => Mark::Numbered(Number::new(number), Depth::Top),
            Token::Nested(number) => Mark::Numbered(Number::new(number), Depth::Nested),
            Token::Bulleted => Mark::Bulleted,
            Token::Quoted => Mark::Quoted,
            Token::Uuid => Mark::Uuid,
            Token::Plain => Mark::Plain,
        }
    }
}
