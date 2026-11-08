use std::collections::BTreeMap;
use std::fmt;
use std::iter;

use logos::Logos;

use super::{Cite, Date, Ending, Layout, Quote, Unclosed};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Row {
    #[regex(r"[ \t\r]+", priority = 3)]
    Layout,
    #[regex(r"[^\n]+", allow_greedy = true)]
    Text,
    #[token("\n")]
    Break,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Lexeme {
    #[regex(r"#+[ \t]+")]
    Hash,
    #[token("-")]
    Dash,
    #[token("RULING")]
    Ruling,
    #[token("LEAD")]
    Lead,
    #[token("(")]
    Open,
    #[token(")")]
    Close,
    #[token("\"")]
    Double,
    #[regex(r"[0-9a-f]{8}")]
    #[regex(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}")]
    Cite,
    #[regex(r"[0-9]{4}-[0-9]{2}-[0-9]{2}")]
    Date,
    #[regex(r"[\p{L}\p{N}_]+")]
    #[regex(r"[\p{L}\p{N}_]*[\p{L}\p{N}_&&[^0-9a-f]][\p{L}\p{N}_]*('[\p{L}\p{N}]+)+")]
    Word,
    #[token("'s")]
    Possessive,
    #[regex(r"[ \t]+")]
    Space,
}

pub trait Reading {
    type Output;

    fn layout(self, line: &Line) -> Self::Output;

    fn heading(self, line: &Line, entry: &Entry) -> Self::Output;

    fn ruling(self, line: &Line, entry: &Entry) -> Self::Output;

    fn lead(self, line: &Line, entry: &Entry) -> Self::Output;

    fn remark(self, line: &Line, entry: &Entry) -> Self::Output;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Ruling,
    Lead,
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
    Quoted(Quote),
    Unclosed(Unclosed),
    Cited(Cite),
    Dated(Date),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Pieces(Vec<Piece>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    kind: Kind,
    pieces: Pieces,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Form {
    Layout(Layout),
    Content(Content),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Content {
    Heading(Entry),
    Entry(Entry),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    number: usize,
    form: Form,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Notes {
    lines: Vec<Line>,
    ending: Ending,
}

#[derive(Clone, Copy, Debug)]
struct Lexed<'text> {
    lexeme: Option<Lexeme>,
    text: &'text str,
}

#[derive(Clone, Copy, Debug)]
struct Cursor<'lexed, 'text> {
    rest: &'lexed [Lexed<'text>],
}

#[derive(Debug)]
struct Step<'lexed, 'text> {
    piece: Piece,
    cursor: Cursor<'lexed, 'text>,
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
        rows.finished()
    }

    #[must_use]
    pub fn gathered(lines: impl IntoIterator<Item = Line>, ending: Ending) -> Notes {
        let numbered: BTreeMap<usize, Line> = lines.into_iter().map(|line| (line.number, line)).collect();
        let last = numbered.keys().next_back().copied().unwrap_or_default();
        Notes {
            lines: (1..=last)
                .map(|number| {
                    numbered.get(&number).cloned().unwrap_or(Line { number, form: Form::Layout(Layout::default()) })
                })
                .collect(),
            ending,
        }
    }

    #[must_use]
    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    #[must_use]
    pub fn ending(&self) -> Ending {
        match self.lines.last().map(|line| &line.form) {
            Some(Form::Content(_)) => self.ending,
            Some(Form::Layout(_)) | None => Ending::Break,
        }
    }
}

impl Ending {
    fn closed(self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ending::Break => writeln!(formatter),
            Ending::Open => Ok(()),
        }
    }
}

impl<'text> Rows<'text> {
    fn taking(self, row: Result<Row, ()>, slice: &'text str) -> Rows<'text> {
        match row {
            Ok(Row::Break) => self.closed(),
            Ok(Row::Layout | Row::Text) | Err(()) => Rows { open: Some(slice), ..self },
        }
    }

    fn finished(self) -> Notes {
        match self.open {
            Some(_) => Notes { lines: self.closed().lines, ending: Ending::Open },
            None => Notes { lines: self.lines, ending: Ending::Break },
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
        Line {
            number,
            form: Row::lexer(text).next().map_or(Form::Layout(Layout::default()), |row| Form::read(row, text)),
        }
    }

    #[must_use]
    pub fn number(&self) -> usize {
        self.number
    }

    pub fn read_by<Reader: Reading>(&self, reader: Reader) -> Reader::Output {
        match &self.form {
            Form::Layout(_) => reader.layout(self),
            Form::Content(Content::Heading(entry)) => reader.heading(self, entry),
            Form::Content(Content::Entry(entry)) => entry.kind.reading(reader, self, entry),
        }
    }
}

impl Form {
    fn read(row: Result<Row, ()>, text: &str) -> Form {
        match row {
            Ok(Row::Layout) => Form::Layout(Layout::new(text.to_owned())),
            Ok(Row::Text | Row::Break) | Err(()) => Form::Content(Content::read(text)),
        }
    }
}

impl Content {
    fn read(text: &str) -> Content {
        let lexed: Vec<Lexed<'_>> = Lexeme::lexer(text)
            .spanned()
            .map(|(lexeme, range)| Lexed { lexeme: lexeme.ok(), text: text.get(range).unwrap_or_default() })
            .collect();
        lexed.first().map_or(Opening::Prose, |first| first.opening()).content(&lexed)
    }
}

impl Opening {
    fn content(self, lexed: &[Lexed<'_>]) -> Content {
        match self {
            Opening::Heading => Content::Heading(Entry { kind: Kind::Remark, pieces: Pieces::parsed(lexed) }),
            Opening::Prose => Content::Entry(Entry { kind: Kind::opened(lexed), pieces: Pieces::parsed(lexed) }),
        }
    }
}

impl Kind {
    fn opened(lexed: &[Lexed<'_>]) -> Kind {
        lexed.iter().find_map(|lexed| lexed.lexeme.map_or(Some(Kind::Remark), Lexeme::kind)).unwrap_or(Kind::Remark)
    }

    fn reading<Reader: Reading>(self, reader: Reader, line: &Line, entry: &Entry) -> Reader::Output {
        match self {
            Kind::Ruling => reader.ruling(line, entry),
            Kind::Lead => reader.lead(line, entry),
            Kind::Remark => reader.remark(line, entry),
        }
    }
}

impl Entry {
    #[must_use]
    pub fn quotes(&self) -> Vec<Result<&Quote, &Unclosed>> {
        self.pieces.0.iter().filter_map(Piece::quote).collect()
    }

    #[must_use]
    pub fn cites(&self) -> Vec<&Cite> {
        self.pieces.0.iter().filter_map(Piece::cite).collect()
    }

    #[must_use]
    pub fn dates(&self) -> Vec<&Date> {
        self.pieces.0.iter().filter_map(Piece::date).collect()
    }
}

impl Lexeme {
    fn kind(self) -> Option<Kind> {
        match self {
            Lexeme::Dash | Lexeme::Date | Lexeme::Space => None,
            Lexeme::Ruling => Some(Kind::Ruling),
            Lexeme::Lead => Some(Kind::Lead),
            Lexeme::Hash
            | Lexeme::Open
            | Lexeme::Close
            | Lexeme::Double
            | Lexeme::Cite
            | Lexeme::Word
            | Lexeme::Possessive => Some(Kind::Remark),
        }
    }
}

impl<'text> Lexed<'text> {
    fn opening(self) -> Opening {
        match self.lexeme {
            Some(Lexeme::Hash) => Opening::Heading,
            Some(
                Lexeme::Dash
                | Lexeme::Ruling
                | Lexeme::Lead
                | Lexeme::Open
                | Lexeme::Close
                | Lexeme::Double
                | Lexeme::Cite
                | Lexeme::Date
                | Lexeme::Word
                | Lexeme::Possessive
                | Lexeme::Space,
            )
            | None => Opening::Prose,
        }
    }

    fn taken<'lexed>(self, rest: &'lexed [Lexed<'text>]) -> Step<'lexed, 'text> {
        match self.lexeme {
            Some(Lexeme::Double) => Step::quoted(rest),
            Some(
                Lexeme::Hash
                | Lexeme::Dash
                | Lexeme::Ruling
                | Lexeme::Lead
                | Lexeme::Open
                | Lexeme::Close
                | Lexeme::Cite
                | Lexeme::Date
                | Lexeme::Word
                | Lexeme::Possessive
                | Lexeme::Space,
            )
            | None => Step::after(Piece::read(self), rest),
        }
    }

    fn joined(lexed: &[Lexed<'text>]) -> String {
        lexed.iter().map(|lexed| lexed.text).collect()
    }
}

impl<'lexed, 'text> Cursor<'lexed, 'text> {
    fn step(self) -> Option<Step<'lexed, 'text>> {
        self.rest.split_first().map(|(first, rest)| first.taken(rest))
    }
}

impl<'lexed, 'text> Step<'lexed, 'text> {
    fn after(piece: Piece, rest: &'lexed [Lexed<'text>]) -> Step<'lexed, 'text> {
        Step { piece, cursor: Cursor { rest } }
    }

    fn quoted(rest: &'lexed [Lexed<'text>]) -> Step<'lexed, 'text> {
        let closed = rest
            .iter()
            .position(|lexed| lexed.lexeme == Some(Lexeme::Double))
            .and_then(|at| rest.split_at_checked(at))
            .and_then(|(inner, from)| from.split_first().map(|(_closer, after)| (inner, after)));
        closed.map_or_else(
            || Step::after(Piece::Unclosed(Unclosed::new(Quote::new(Lexed::joined(rest)))), rest),
            |(inner, after)| Step::after(Piece::Quoted(Quote::new(Lexed::joined(inner))), after),
        )
    }
}

impl Pieces {
    fn parsed(lexed: &[Lexed<'_>]) -> Pieces {
        let start = Cursor { rest: lexed };
        Pieces(iter::successors(start.step(), |step| step.cursor.step()).map(|step| step.piece).collect())
    }
}

impl Piece {
    fn read(lexed: Lexed<'_>) -> Piece {
        match lexed.lexeme {
            Some(Lexeme::Cite) => Piece::Cited(Cite::new(lexed.text.to_owned())),
            Some(Lexeme::Date) => Piece::Dated(Date::new(lexed.text.to_owned())),
            Some(
                Lexeme::Hash
                | Lexeme::Dash
                | Lexeme::Ruling
                | Lexeme::Lead
                | Lexeme::Open
                | Lexeme::Close
                | Lexeme::Double
                | Lexeme::Word
                | Lexeme::Possessive
                | Lexeme::Space,
            )
            | None => Piece::Text(lexed.text.to_owned()),
        }
    }

    fn quote(&self) -> Option<Result<&Quote, &Unclosed>> {
        match self {
            Piece::Quoted(quote) => Some(Ok(quote)),
            Piece::Unclosed(unclosed) => Some(Err(unclosed)),
            Piece::Text(_) | Piece::Cited(_) | Piece::Dated(_) => None,
        }
    }

    fn cite(&self) -> Option<&Cite> {
        match self {
            Piece::Cited(cite) => Some(cite),
            Piece::Text(_) | Piece::Quoted(_) | Piece::Unclosed(_) | Piece::Dated(_) => None,
        }
    }

    fn date(&self) -> Option<&Date> {
        match self {
            Piece::Dated(date) => Some(date),
            Piece::Text(_) | Piece::Quoted(_) | Piece::Unclosed(_) | Piece::Cited(_) => None,
        }
    }
}

impl fmt::Display for Notes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.lines.split_last().map_or(Ok(()), |(last, earlier)| {
            earlier.iter().try_for_each(|line| writeln!(formatter, "{line}"))?;
            write!(formatter, "{last}")?;
            self.ending.closed(formatter)
        })
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
            Form::Layout(layout) => write!(formatter, "{layout}"),
            Form::Content(content) => write!(formatter, "{content}"),
        }
    }
}

impl fmt::Display for Content {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Content::Heading(entry) | Content::Entry(entry) => write!(formatter, "{entry}"),
        }
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.pieces)
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
            Piece::Quoted(quote) => write!(formatter, "\"{quote}\""),
            Piece::Cited(cite) => write!(formatter, "{cite}"),
            Piece::Unclosed(_) => formatter.write_str("\""),
            Piece::Dated(date) => write!(formatter, "{date}"),
        }
    }
}
