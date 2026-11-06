use logos::Logos;
use newtype::Newtype;

use super::{Lexeme, Mark, Number, Position, Span};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Token {
    #[token("\n")]
    Break,
    #[regex(r"[.?!]+[ \t]+")]
    Stop,
    #[regex(r"\n[ \t]*[0-9]+[.)][ \t]+", numbered)]
    Numbered(usize),
    #[regex(r"\n[ \t]*[-*•][ \t]+")]
    Bulleted,
    #[regex(r"\n[ \t]*>[ \t]?")]
    Quoted,
    #[regex(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}")]
    Uuid,
    #[regex(r"[^\n]")]
    Plain,
}

fn numbered(lexer: &logos::Lexer<'_, Token>) -> Option<usize> {
    lexer.slice().chars().filter(char::is_ascii_digit).collect::<String>().parse().ok()
}

impl Token {
    pub fn lexed(text: &str) -> Vec<Lexeme> {
        let sentinel = format!("\n{text}");
        Token::lexer(&sentinel)
            .spanned()
            .map(|(token, range)| (token.unwrap_or(Token::Plain), range))
            .map(|(token, range)| Lexeme {
                mark: token.mark(),
                span: Span {
                    start: Position::new(range.start.saturating_sub(1)),
                    end: Position::new(range.end.saturating_sub(1)),
                },
            })
            .filter(|lexeme| lexeme.span.start != lexeme.span.end)
            .collect()
    }

    fn mark(self) -> Mark {
        match self {
            Token::Break => Mark::Break,
            Token::Stop => Mark::Stop,
            Token::Numbered(number) => Mark::Numbered(Number::new(number)),
            Token::Bulleted => Mark::Bulleted,
            Token::Quoted => Mark::Quoted,
            Token::Uuid => Mark::Uuid,
            Token::Plain => Mark::Plain,
        }
    }
}

impl Span {
    #[must_use]
    pub fn slice(self, text: &str) -> &str {
        text.get(self.start.into_inner()..self.end.into_inner()).unwrap_or_default()
    }
}
