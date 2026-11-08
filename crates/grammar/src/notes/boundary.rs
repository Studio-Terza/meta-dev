use std::collections::BTreeMap;
use std::fmt;

use logos::Logos;

use super::{Cite, Date, Quote};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Row {
    #[regex(r"[^\n]+", allow_greedy = true)]
    Text,
    #[token("\n")]
    Break,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Lexeme {
    #[regex(r"#+[ \t]+")]
    Hash,
    #[token("RULING")]
    Ruling,
    #[regex(r":[ \t]+")]
    Separator,
    #[token("(")]
    Open,
    #[token(")")]
    Close,
    #[regex(r#""[^"\n]*""#)]
    Double,
    #[regex(r"'([^'\n]|'\p{L})*'")]
    Single,
    #[regex(r"[0-9a-f]{8}")]
    #[regex(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}")]
    Cite,
    #[regex(r"[0-9]{4}-[0-9]{2}-[0-9]{2}")]
    Date,
    #[regex(r"[\p{L}\p{N}_]+('[\p{L}\p{N}]+)*'?")]
    Word,
    #[regex(r"[ \t]+")]
    Space,
}

pub trait Reading {
    type Output;

    fn blank(self, line: &Line) -> Self::Output;

    fn heading(self, line: &Line) -> Self::Output;

    fn ruling(self, line: &Line, entry: &Entry) -> Self::Output;

    fn remark(self, line: &Line, entry: &Entry) -> Self::Output;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mark {
    Double,
    Single,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Ruling,
    Remark,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Opening {
    Heading,
    Prose,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Piece {
    Text(String),
    Quoted(Mark, Quote),
    Cited(Cite),
    Dated(Date),
    Ruling,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Pieces(Vec<Piece>);

#[derive(Clone, Debug, PartialEq, Eq)]
struct Head {
    pieces: Pieces,
    separator: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Heading {
    marker: String,
    title: Pieces,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    head: Option<Head>,
    body: Pieces,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Form {
    Blank,
    Heading(Heading),
    Entry(Entry),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    number: usize,
    form: Form,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Notes(Vec<Line>);

#[derive(Clone, Copy, Debug)]
struct Lexed<'text> {
    lexeme: Option<Lexeme>,
    text: &'text str,
}

#[derive(Debug, Default)]
struct Rows<'text> {
    lines: Vec<Line>,
    open: Option<&'text str>,
}

impl Notes {
    #[must_use]
    pub fn read(text: &str) -> Notes {
        let rows = Row::lexer(text)
            .spanned()
            .fold(Rows::default(), |rows, (row, range)| rows.taking(row, text.get(range).unwrap_or_default()));
        Notes(rows.finished())
    }

    #[must_use]
    pub fn gathered(lines: impl IntoIterator<Item = Line>) -> Notes {
        let numbered: BTreeMap<usize, Line> = lines.into_iter().map(|line| (line.number, line)).collect();
        let last = numbered.keys().next_back().copied().unwrap_or_default();
        Notes(
            (1..=last)
                .map(|number| numbered.get(&number).cloned().unwrap_or(Line { number, form: Form::Blank }))
                .collect(),
        )
    }

    #[must_use]
    pub fn lines(&self) -> &[Line] {
        &self.0
    }
}

impl<'text> Rows<'text> {
    fn taking(self, row: Result<Row, ()>, slice: &'text str) -> Rows<'text> {
        match row {
            Ok(Row::Break) => self.closed(),
            Ok(Row::Text) | Err(()) => Rows { open: Some(slice), ..self },
        }
    }

    fn finished(self) -> Vec<Line> {
        match self.open {
            Some(_) => self.closed().lines,
            None => self.lines,
        }
    }

    fn closed(self) -> Rows<'text> {
        let line = Line::read(self.lines.len().saturating_add(1), self.open.unwrap_or_default());
        Rows { lines: self.lines.into_iter().chain([line]).collect(), open: None }
    }
}

impl Line {
    #[must_use]
    pub fn read(number: usize, text: &str) -> Line {
        let lexed: Vec<Lexed<'_>> = Lexeme::lexer(text)
            .spanned()
            .map(|(lexeme, range)| Lexed { lexeme: lexeme.ok(), text: text.get(range).unwrap_or_default() })
            .collect();
        Line { number, form: Form::opened(&lexed) }
    }

    #[must_use]
    pub fn number(&self) -> usize {
        self.number
    }

    pub fn read_by<Reader: Reading>(&self, reader: Reader) -> Reader::Output {
        match &self.form {
            Form::Blank => reader.blank(self),
            Form::Heading(_) => reader.heading(self),
            Form::Entry(entry) => entry.kind().reading(reader, self, entry),
        }
    }
}

impl Form {
    fn opened(lexed: &[Lexed<'_>]) -> Form {
        lexed.split_first().map_or(Form::Blank, |(first, rest)| first.opening().form(*first, rest, lexed))
    }
}

impl Opening {
    fn form(self, first: Lexed<'_>, rest: &[Lexed<'_>], lexed: &[Lexed<'_>]) -> Form {
        match self {
            Opening::Heading => Form::Heading(Heading { marker: first.text.to_owned(), title: Pieces::gathered(rest) }),
            Opening::Prose => Form::Entry(Entry::read(lexed)),
        }
    }
}

impl Kind {
    fn reading<Reader: Reading>(self, reader: Reader, line: &Line, entry: &Entry) -> Reader::Output {
        match self {
            Kind::Ruling => reader.ruling(line, entry),
            Kind::Remark => reader.remark(line, entry),
        }
    }
}

impl Entry {
    fn read(lexed: &[Lexed<'_>]) -> Entry {
        let split = lexed
            .iter()
            .scan(0, |depth, lexed| {
                let level = *depth;
                *depth = lexed.deepened(level);
                Some((level, lexed.lexeme))
            })
            .position(|step| step == (0, Some(Lexeme::Separator)));
        let parted = split.and_then(|at| lexed.get(..at).zip(lexed.get(at)).zip(lexed.get(at.saturating_add(1)..)));
        let head = parted.map(|((head, separator), _body)| Head {
            pieces: Pieces::gathered(head),
            separator: separator.text.to_owned(),
        });
        let body = parted.map_or(lexed, |(_head, body)| body);
        Entry { head, body: Pieces::gathered(body) }
    }

    #[must_use]
    pub fn quotes(&self) -> Vec<&Quote> {
        self.pieces().filter_map(Piece::quote).collect()
    }

    #[must_use]
    pub fn cites(&self) -> Vec<&Cite> {
        self.pieces().filter_map(Piece::cite).collect()
    }

    #[must_use]
    pub fn dates(&self) -> Vec<&Date> {
        self.pieces().filter_map(Piece::date).collect()
    }

    fn kind(&self) -> Kind {
        let headed = self.head.iter().flat_map(|head| &head.pieces.0);
        headed.filter(|piece| **piece == Piece::Ruling).map(|_ruling| Kind::Ruling).next().unwrap_or(Kind::Remark)
    }

    fn pieces(&self) -> impl Iterator<Item = &Piece> {
        self.head.iter().flat_map(|head| &head.pieces.0).chain(&self.body.0)
    }
}

impl Lexed<'_> {
    fn deepened(self, level: usize) -> usize {
        match self.lexeme {
            Some(Lexeme::Open) => level.saturating_add(1),
            Some(Lexeme::Close) => level.saturating_sub(1),
            Some(
                Lexeme::Hash
                | Lexeme::Ruling
                | Lexeme::Separator
                | Lexeme::Double
                | Lexeme::Single
                | Lexeme::Cite
                | Lexeme::Date
                | Lexeme::Word
                | Lexeme::Space,
            )
            | None => level,
        }
    }

    fn opening(self) -> Opening {
        match self.lexeme {
            Some(Lexeme::Hash) => Opening::Heading,
            Some(
                Lexeme::Ruling
                | Lexeme::Separator
                | Lexeme::Open
                | Lexeme::Close
                | Lexeme::Double
                | Lexeme::Single
                | Lexeme::Cite
                | Lexeme::Date
                | Lexeme::Word
                | Lexeme::Space,
            )
            | None => Opening::Prose,
        }
    }

    fn piece(self) -> Piece {
        match self.lexeme {
            Some(Lexeme::Double) => Piece::Quoted(Mark::Double, Mark::unquoted(self.text)),
            Some(Lexeme::Single) => Piece::Quoted(Mark::Single, Mark::unquoted(self.text)),
            Some(Lexeme::Cite) => Piece::Cited(Cite::new(self.text.to_owned())),
            Some(Lexeme::Date) => Piece::Dated(Date::new(self.text.to_owned())),
            Some(Lexeme::Ruling) => Piece::Ruling,
            Some(Lexeme::Hash | Lexeme::Separator | Lexeme::Open | Lexeme::Close | Lexeme::Word | Lexeme::Space)
            | None => Piece::Text(self.text.to_owned()),
        }
    }
}

impl Pieces {
    fn gathered(lexed: &[Lexed<'_>]) -> Pieces {
        Pieces(lexed.iter().map(|lexed| lexed.piece()).collect())
    }
}

impl Piece {
    fn quote(&self) -> Option<&Quote> {
        match self {
            Piece::Quoted(_, quote) => Some(quote),
            Piece::Text(_) | Piece::Cited(_) | Piece::Dated(_) | Piece::Ruling => None,
        }
    }

    fn cite(&self) -> Option<&Cite> {
        match self {
            Piece::Cited(cite) => Some(cite),
            Piece::Text(_) | Piece::Quoted(..) | Piece::Dated(_) | Piece::Ruling => None,
        }
    }

    fn date(&self) -> Option<&Date> {
        match self {
            Piece::Dated(date) => Some(date),
            Piece::Text(_) | Piece::Quoted(..) | Piece::Cited(_) | Piece::Ruling => None,
        }
    }
}

impl Mark {
    fn unquoted(quoted: &str) -> Quote {
        Quote::new(quoted.get(1..quoted.len().saturating_sub(1)).unwrap_or_default().to_owned())
    }

    const fn sign(self) -> char {
        match self {
            Mark::Double => '"',
            Mark::Single => '\'',
        }
    }
}

impl fmt::Display for Notes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|line| writeln!(formatter, "{line}"))
    }
}

impl fmt::Display for Line {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.form)
    }
}

impl fmt::Display for Form {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Form::Blank => Ok(()),
            Form::Heading(heading) => write!(formatter, "{heading}"),
            Form::Entry(entry) => write!(formatter, "{entry}"),
        }
    }
}

impl fmt::Display for Heading {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{}", self.marker, self.title)
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.head.iter().try_for_each(|head| write!(formatter, "{head}"))?;
        write!(formatter, "{}", self.body)
    }
}

impl fmt::Display for Head {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{}", self.pieces, self.separator)
    }
}

impl fmt::Display for Pieces {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|piece| write!(formatter, "{piece}"))
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Piece::Text(text) => formatter.write_str(text),
            Piece::Quoted(mark, quote) => write!(formatter, "{0}{quote}{0}", mark.sign()),
            Piece::Cited(cite) => write!(formatter, "{cite}"),
            Piece::Dated(date) => write!(formatter, "{date}"),
            Piece::Ruling => formatter.write_str("RULING"),
        }
    }
}
