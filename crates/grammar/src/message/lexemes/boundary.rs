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
    #[regex(r"\n[0-9]+[.)][ \t]+", Token::numbered)]
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
                    Span::new(
                        Position::new(range.start).without_sentinel(),
                        Position::new(range.end).without_sentinel(),
                    ),
                )
            })
            .collect()
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
