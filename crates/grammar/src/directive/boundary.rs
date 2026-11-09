use std::ops::Range;

use logos::{Lexer, Logos};
use newtype::{Arbitrary, Display, Newtype};

use crate::cited::Cited;

#[derive(Logos, Clone, Debug, PartialEq, Eq)]
#[logos(skip r"[ \t]+")]
enum Token {
    #[token("(")]
    Open,
    #[token(")")]
    Close,
    #[token(",")]
    Comma,
    #[regex(r"\r?\n")]
    Break,
    #[regex(r"[A-Za-z_][A-Za-z0-9_-]*", |lexer| lexer.slice().to_owned())]
    Word(String),
    #[regex(r"new[ \t]+[A-Za-z_][A-Za-z0-9_-]*", Fresh::lexed)]
    Fresh(Fresh),
    #[regex(r"[A-Za-z0-9_][A-Za-z0-9_-]*\.(0|[1-9][0-9]*)", |lexer| Cited::read(lexer.slice()))]
    Segment(Cited),
    #[regex(r#""([^"\\\r\n]|\\["\\])*""#, Quote::lexed)]
    Quoted(Quote),
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Escape {
    #[regex(r#"\\["\\]"#)]
    Escaped,
    #[regex(r#"[^\\]+"#)]
    Plain,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Labelled {
    #[regex(r"new[ \t]+")]
    Keyword,
    #[regex(r"[A-Za-z_][A-Za-z0-9_-]*")]
    Label,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Unescaped {
    #[regex(r#"["\\]"#)]
    Special,
    #[regex(r#"[^"\\]+"#)]
    Plain,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quote(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fresh(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Piece {
    Open,
    Close,
    Comma,
    Break,
    End,
    Word(String),
    Fresh(Fresh),
    Segment(Cited),
    Quoted(Quote),
    Stray(String),
}

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineNumber(usize);

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Column(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Location {
    line: LineNumber,
    column: Column,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lexed {
    piece: Piece,
    location: Location,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pieces: Vec<Lexed>,
    end: Lexed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lines {
    complete: Vec<Line>,
    last: Line,
}

#[derive(Clone, Copy, Debug, Default)]
struct Start {
    line: usize,
    offset: usize,
}

#[derive(Debug, Default)]
struct Scanned {
    start: Start,
    lexed: Vec<Lexed>,
}

#[derive(Debug, Default)]
struct Gathered {
    complete: Vec<Line>,
    open: Vec<Lexed>,
}

#[derive(Debug)]
pub struct Pieces;

impl Pieces {
    #[must_use]
    pub fn line(text: &str) -> Line {
        let scanned = Scanned::lexing(text);
        let end = scanned.ended(text);
        Line { pieces: scanned.lexed, end }
    }

    #[must_use]
    pub fn lines(text: &str) -> Lines {
        let scanned = Scanned::lexing(text);
        let end = scanned.ended(text);
        let gathered = scanned.lexed.into_iter().fold(Gathered::default(), Gathered::taking);
        Lines { complete: gathered.complete, last: Line { pieces: gathered.open, end } }
    }
}

impl Scanned {
    fn lexing(text: &str) -> Scanned {
        Token::lexer(text)
            .spanned()
            .map(|(token, range)| (Token::piece(token, text.get(range.clone()).unwrap_or_default()), range))
            .fold(Scanned::default(), |scanned, (piece, range)| scanned.taking(text, piece, range))
    }

    fn taking(self, text: &str, piece: Piece, range: Range<usize>) -> Scanned {
        let location = self.start.located(text, range.start);
        let start = self.start.after(&piece, range.end);
        Scanned { start, lexed: self.lexed.into_iter().chain([Lexed { piece, location }]).collect() }
    }

    fn ended(&self, text: &str) -> Lexed {
        Lexed { piece: Piece::End, location: self.start.located(text, text.len()) }
    }
}

impl Gathered {
    fn taking(self, lexed: Lexed) -> Gathered {
        match lexed.piece {
            Piece::Break => Gathered {
                complete: self.complete.into_iter().chain([Line { pieces: self.open, end: lexed }]).collect(),
                open: Vec::new(),
            },
            Piece::Open
            | Piece::Close
            | Piece::Comma
            | Piece::End
            | Piece::Word(_)
            | Piece::Fresh(_)
            | Piece::Segment(_)
            | Piece::Quoted(_)
            | Piece::Stray(_) => Gathered { open: self.open.into_iter().chain([lexed]).collect(), ..self },
        }
    }
}

impl Start {
    fn located(self, text: &str, offset: usize) -> Location {
        let column = text.get(self.offset..offset).unwrap_or_default().chars().count();
        Location { line: LineNumber(self.line.saturating_add(1)), column: Column(column.saturating_add(1)) }
    }

    fn after(self, piece: &Piece, end: usize) -> Start {
        match piece {
            Piece::Break => Start { line: self.line.saturating_add(1), offset: end },
            Piece::Open
            | Piece::Close
            | Piece::Comma
            | Piece::End
            | Piece::Word(_)
            | Piece::Fresh(_)
            | Piece::Segment(_)
            | Piece::Quoted(_)
            | Piece::Stray(_) => self,
        }
    }
}

impl Piece {
    #[must_use]
    pub fn word(&self) -> Option<&str> {
        match self {
            Piece::Word(word) => Some(word),
            Piece::Open
            | Piece::Close
            | Piece::Comma
            | Piece::Break
            | Piece::End
            | Piece::Fresh(_)
            | Piece::Segment(_)
            | Piece::Quoted(_)
            | Piece::Stray(_) => None,
        }
    }

    #[must_use]
    pub fn fresh(&self) -> Option<&Fresh> {
        match self {
            Piece::Fresh(label) => Some(label),
            Piece::Open
            | Piece::Close
            | Piece::Comma
            | Piece::Break
            | Piece::End
            | Piece::Word(_)
            | Piece::Segment(_)
            | Piece::Quoted(_)
            | Piece::Stray(_) => None,
        }
    }

    #[must_use]
    pub fn cited(&self) -> Option<&Cited> {
        match self {
            Piece::Segment(cited) => Some(cited),
            Piece::Open
            | Piece::Close
            | Piece::Comma
            | Piece::Break
            | Piece::End
            | Piece::Word(_)
            | Piece::Fresh(_)
            | Piece::Quoted(_)
            | Piece::Stray(_) => None,
        }
    }

    #[must_use]
    pub fn quoted(&self) -> Option<&Quote> {
        match self {
            Piece::Quoted(quote) => Some(quote),
            Piece::Open
            | Piece::Close
            | Piece::Comma
            | Piece::Break
            | Piece::End
            | Piece::Word(_)
            | Piece::Fresh(_)
            | Piece::Segment(_)
            | Piece::Stray(_) => None,
        }
    }
}

impl AsRef<LineNumber> for Location {
    fn as_ref(&self) -> &LineNumber {
        &self.line
    }
}

impl Location {
    #[must_use]
    pub fn new(line: usize, column: usize) -> Location {
        Location { line: LineNumber(line), column: Column(column) }
    }

    #[must_use]
    pub fn column(self) -> Column {
        self.column
    }
}

impl Lexed {
    #[must_use]
    pub fn piece(&self) -> &Piece {
        &self.piece
    }
}

impl From<&Lexed> for Location {
    fn from(lexed: &Lexed) -> Location {
        lexed.location
    }
}

impl Line {
    #[must_use]
    pub fn pieces(&self) -> &[Lexed] {
        &self.pieces
    }

    #[must_use]
    pub fn end(&self) -> &Lexed {
        &self.end
    }
}

impl Lines {
    pub fn iter(&self) -> impl Iterator<Item = &Line> {
        self.complete.iter().chain([&self.last])
    }
}

impl Quote {
    fn lexed(lexer: &Lexer<'_, Token>) -> Quote {
        let inner = lexer.slice().get(1..lexer.slice().len().saturating_sub(1)).unwrap_or_default();
        Quote(
            Escape::lexer(inner)
                .spanned()
                .filter_map(|(escape, range)| inner.get(Escape::kept(escape, range)))
                .collect(),
        )
    }

    #[must_use]
    pub fn quoting(text: &str) -> String {
        let escaped: String = Unescaped::lexer(text)
            .spanned()
            .map(|(unescaped, range)| Unescaped::escaped(unescaped, text.get(range).unwrap_or_default()))
            .collect();
        format!("\"{escaped}\"")
    }
}

impl AsRef<str> for Quote {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Unescaped {
    fn escaped(unescaped: Result<Unescaped, ()>, written: &str) -> String {
        match unescaped {
            Ok(Unescaped::Special) => format!("\\{written}"),
            Ok(Unescaped::Plain) | Err(()) => written.to_owned(),
        }
    }
}

impl Fresh {
    fn lexed(lexer: &Lexer<'_, Token>) -> Option<Fresh> {
        let fresh = lexer.slice();
        Labelled::lexer(fresh)
            .spanned()
            .filter_map(|(labelled, range)| Labelled::label(labelled, range))
            .last()
            .and_then(|range| fresh.get(range))
            .map(|label| Fresh(label.to_owned()))
    }
}

impl AsRef<str> for Fresh {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Token {
    fn piece(token: Result<Token, ()>, written: &str) -> Piece {
        match token {
            Ok(Token::Open) => Piece::Open,
            Ok(Token::Close) => Piece::Close,
            Ok(Token::Comma) => Piece::Comma,
            Ok(Token::Break) => Piece::Break,
            Ok(Token::Word(word)) => Piece::Word(word),
            Ok(Token::Fresh(label)) => Piece::Fresh(label),
            Ok(Token::Segment(cited)) => Piece::Segment(cited),
            Ok(Token::Quoted(quote)) => Piece::Quoted(quote),
            Err(()) => Piece::Stray(written.to_owned()),
        }
    }
}

impl Labelled {
    fn label(labelled: Result<Labelled, ()>, range: Range<usize>) -> Option<Range<usize>> {
        match labelled {
            Ok(Labelled::Label) => Some(range),
            Ok(Labelled::Keyword) | Err(()) => None,
        }
    }
}

impl Escape {
    fn kept(escape: Result<Escape, ()>, range: Range<usize>) -> Range<usize> {
        match escape {
            Ok(Escape::Escaped) => range.start.saturating_add(1)..range.end,
            Ok(Escape::Plain) | Err(()) => range,
        }
    }
}
