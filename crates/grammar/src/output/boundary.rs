use logos::{Lexer, Logos};

use crate::directive::Cited;

#[derive(Logos, Clone, Debug, PartialEq, Eq)]
#[logos(skip r"[ \t]+")]
enum Lexeme {
    #[regex(r"[A-Za-z_][A-Za-z0-9_-]*", |lexer| lexer.slice().to_owned(), priority = 1)]
    Word(String),
    #[regex(r"[0-9a-f][0-9a-f-]{7,}", |lexer| lexer.slice().to_owned(), priority = 2)]
    Message(String),
    #[regex(r"[0-9a-f][0-9a-f-]{7,}\.(0|[1-9][0-9]*)", |lexer| Cited::read(lexer.slice()))]
    Segment(Cited),
    #[regex(r"n(0|[1-9][0-9]*)", |lexer| lexer.slice().to_owned(), priority = 3)]
    Node(String),
    #[regex(r#""([^"\\\r\n]|\\["\\nrt])*""#, Lexeme::unquoted)]
    Quoted(String),
    #[regex(r"0|[1-9][0-9]*", |lexer| lexer.slice().parse().ok(), priority = 3)]
    Integer(u64),
    #[regex(r"\r?\n")]
    End,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Escaped {
    #[token(r#"\""#)]
    Quote,
    #[token(r"\\")]
    Backslash,
    #[token(r"\n")]
    Newline,
    #[token(r"\r")]
    Return,
    #[token(r"\t")]
    Tab,
    #[regex(r"[^\\]+")]
    Plain,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Special {
    #[token("\"")]
    Quote,
    #[token("\\")]
    Backslash,
    #[token("\n")]
    Newline,
    #[token("\r")]
    Return,
    #[token("\t")]
    Tab,
    #[regex(r#"[^"\\\n\r\t]+"#)]
    Plain,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Word(String),
    Message(String),
    Segment(Cited),
    Node(String),
    Quoted(String),
    Integer(u64),
    End,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("{0:?} is not a word, a message or segment ref, a node id, a quoted text or a number")]
pub struct Stray(String);

#[derive(Debug)]
pub struct Tokens;

impl Tokens {
    pub fn lexed(text: &str) -> Result<Vec<Token>, Stray> {
        Lexeme::lexer(text)
            .spanned()
            .map(|(lexeme, range)| {
                lexeme.map(Token::from).map_err(|()| Stray(text.get(range).unwrap_or_default().to_owned()))
            })
            .collect()
    }

    #[must_use]
    pub fn quoting(text: &str) -> String {
        let escaped: String = Special::lexer(text)
            .spanned()
            .map(|(special, range)| Special::escaped(special, text.get(range).unwrap_or_default()))
            .collect();
        format!("\"{escaped}\"")
    }
}

impl From<Lexeme> for Token {
    fn from(lexeme: Lexeme) -> Token {
        match lexeme {
            Lexeme::Word(word) => Token::Word(word),
            Lexeme::Message(message) => Token::Message(message),
            Lexeme::Segment(cited) => Token::Segment(cited),
            Lexeme::Node(node) => Token::Node(node),
            Lexeme::Quoted(quoted) => Token::Quoted(quoted),
            Lexeme::Integer(integer) => Token::Integer(integer),
            Lexeme::End => Token::End,
        }
    }
}

impl Lexeme {
    fn unquoted(lexer: &Lexer<'_, Lexeme>) -> String {
        let inner = lexer.slice().get(1..lexer.slice().len().saturating_sub(1)).unwrap_or_default();
        Escaped::lexer(inner)
            .spanned()
            .map(|(escaped, range)| Escaped::unescaped(escaped, inner.get(range).unwrap_or_default()))
            .collect()
    }
}

impl Escaped {
    fn unescaped(escaped: Result<Escaped, ()>, written: &str) -> &str {
        match escaped {
            Ok(Escaped::Quote) => "\"",
            Ok(Escaped::Backslash) => "\\",
            Ok(Escaped::Newline) => "\n",
            Ok(Escaped::Return) => "\r",
            Ok(Escaped::Tab) => "\t",
            Ok(Escaped::Plain) | Err(()) => written,
        }
    }
}

impl Special {
    fn escaped(special: Result<Special, ()>, written: &str) -> &str {
        match special {
            Ok(Special::Quote) => r#"\""#,
            Ok(Special::Backslash) => r"\\",
            Ok(Special::Newline) => r"\n",
            Ok(Special::Return) => r"\r",
            Ok(Special::Tab) => r"\t",
            Ok(Special::Plain) | Err(()) => written,
        }
    }
}
