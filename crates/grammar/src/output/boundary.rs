use std::iter::Peekable;
use std::str::FromStr;
use std::vec::IntoIter;

use logos::{Lexer, Logos};
use newtype::{Guarded, Inner, Newtype, Stored, Validated};

#[derive(Logos, Clone, Debug, PartialEq, Eq)]
#[logos(skip r"[ \t]+")]
enum Lexeme {
    #[regex(r"[A-Za-z0-9_][A-Za-z0-9_-]*")]
    Bare,
    #[regex(r"[A-Za-z0-9_][A-Za-z0-9_-]*\.(0|[1-9][0-9]*)")]
    Cited,
    #[regex(r#""([^"\\\r\n]|\\["\\nrt])*""#, Lexeme::unquoted)]
    Quoted(String),
    #[regex(r"\r?\n")]
    End,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Decimal {
    #[regex(r"0|[1-9][0-9]*")]
    Digits,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Numbered {
    #[regex(r"n(0|[1-9][0-9]*)")]
    Node,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Bare,
    Cited,
    Quoted,
    End,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    kind: Kind,
    text: String,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("{0:?} is not a bare word, a segment ref or a quoted text")]
pub struct Stray(String);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Unread {
    #[error("{0}")]
    Stray(#[from] Stray),
    #[error("the line ends before it is complete")]
    Ended,
    #[error("{0:?} does not belong there in a line of design's output")]
    Unexpected(Token),
    #[error("a keyword of the line is missing or out of place")]
    Unkeyed,
}

#[derive(Debug)]
pub struct Fields(Peekable<IntoIter<Token>>);

pub trait Field {
    type Value;

    fn taken(token: Token) -> Result<Self::Value, Token>;
}

pub trait Unwrapping<T> {
    fn unwrapped(text: String) -> Option<T>;
}

impl<T: Newtype<Inner = String>> Unwrapping<T> for Inner {
    fn unwrapped(text: String) -> Option<T> {
        Some(T::new(text))
    }
}

impl<T: Guarded<Inner = String>> Unwrapping<T> for Validated {
    fn unwrapped(text: String) -> Option<T> {
        T::try_new(text).ok()
    }
}

impl Fields {
    pub fn lexed(text: &str) -> Result<Fields, Stray> {
        let tokens = Lexeme::lexer(text)
            .spanned()
            .map(|(lexed, range)| {
                let slice = text.get(range).unwrap_or_default().to_owned();
                lexed.map(|lexeme| Token::from((lexeme, slice.clone()))).map_err(|()| Stray(slice))
            })
            .collect::<Result<Vec<Token>, Stray>>()?;
        Ok(Fields(tokens.into_iter().peekable()))
    }

    #[must_use]
    pub fn bare(text: &str) -> Option<&str> {
        let mut lexer = Lexeme::lexer(text);
        let whole = lexer.next().filter(|lexed| *lexed == Ok(Lexeme::Bare)).map(|_bare| lexer.span());
        whole.filter(|span| *span == (0..text.len())).map(|_whole| text)
    }

    #[must_use]
    pub fn quoting(text: &str) -> String {
        let escaped: String = Special::lexer(text)
            .spanned()
            .map(|(special, range)| Special::escaped(special, text.get(range).unwrap_or_default()))
            .collect();
        format!("\"{escaped}\"")
    }

    pub fn field<F: Field>(&mut self) -> Result<F::Value, Unread> {
        let token = self.0.next().ok_or(Unread::Ended)?;
        F::taken(token).map_err(Unread::Unexpected)
    }

    pub fn optional<F: Field>(&mut self) -> Option<F::Value> {
        self.0.next_if(|token| F::taken(token.clone()).is_ok()).and_then(|token| F::taken(token).ok())
    }

    pub fn past<K: FromStr + PartialEq>(&mut self, keys: &[K]) -> Result<&mut Fields, Unread> {
        keys.iter().try_fold(self, |fields, key| {
            let token = fields.0.next().ok_or(Unread::Ended)?;
            let word: K = token.parsed(Kind::Bare).map_err(Unread::Unexpected)?;
            (word == *key).then_some(fields).ok_or(Unread::Unkeyed)
        })
    }

    pub fn ended<T>(mut self, value: T) -> Result<T, Unread> {
        self.0.find(|token| token.kind != Kind::End).map_or(Ok(value), |stray| Err(Unread::Unexpected(stray)))
    }
}

impl Token {
    pub(crate) fn expected(self, kind: Kind) -> Result<String, Token> {
        (self.kind == kind).then(|| self.text.clone()).ok_or(self)
    }

    pub(crate) fn spelled<L: for<'text> Logos<'text, Source = str, Extras = ()>>(self) -> Result<Token, Token> {
        let kept = self.clone();
        let length = self.text.len();
        let mut lexer = L::lexer(&self.text);
        let whole = lexer.next().and_then(Result::ok).map(|_spelling| lexer.span());
        whole.filter(|span| *span == (0..length)).map(|_whole| self).ok_or(kept)
    }

    pub(crate) fn parsed<T: FromStr>(self, kind: Kind) -> Result<T, Token> {
        let kept = self.clone();
        self.expected(kind).and_then(|text| text.parse().map_err(|_unparsed| kept))
    }

    pub(crate) fn wrapped<T: Stored<Kind: Unwrapping<T>>>(self, kind: Kind) -> Result<T, Token> {
        let kept = self.clone();
        self.expected(kind).and_then(|text| T::Kind::unwrapped(text).ok_or(kept))
    }
}

impl From<(Lexeme, String)> for Token {
    fn from((lexeme, slice): (Lexeme, String)) -> Token {
        match lexeme {
            Lexeme::Bare => Token { kind: Kind::Bare, text: slice },
            Lexeme::Cited => Token { kind: Kind::Cited, text: slice },
            Lexeme::Quoted(quoted) => Token { kind: Kind::Quoted, text: quoted },
            Lexeme::End => Token { kind: Kind::End, text: slice },
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
