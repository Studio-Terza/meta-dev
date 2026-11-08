use std::collections::BTreeMap;
use std::fmt;
use std::iter;

use logos::Logos;

use super::{Cite, Date, Layout, Quote, Unclosed};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Row {
    #[regex(r"[ \t]+", priority = 3)]
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
    #[token("'")]
    Single,
    #[regex(r"[0-9a-f]{8}")]
    #[regex(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}")]
    Cite,
    #[regex(r"[0-9]{4}-[0-9]{2}-[0-9]{2}")]
    Date,
    #[regex(r"[\p{L}\p{N}_]+('[\p{L}\p{N}]+)*")]
    Word,
    #[regex(r"[ \t]+")]
    Space,
}

pub trait Reading {
    type Output;

    fn layout(self, line: &Line) -> Self::Output;

    fn heading(self, line: &Line) -> Self::Output;

    fn ruling(self, line: &Line, entry: &Entry) -> Self::Output;

    fn lead(self, line: &Line, entry: &Entry) -> Self::Output;

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
    Lead,
    Remark,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Opening {
    Heading,
    Prose,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Follow {
    End,
    Gap,
    Word,
    Mark,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
    Opening,
    Closing,
    Plain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Inside {
    Opening,
    Closing,
    Nested,
    Unnested,
    Plain,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Piece {
    Text(String),
    Quoted(Mark, Quote),
    Unclosed(Mark, Unclosed),
    Cited(Cite),
    Dated(Date),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Pieces(Vec<Piece>);

#[derive(Clone, Debug, PartialEq, Eq)]
struct Heading(String);

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
    Heading(Heading),
    Entry(Entry),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    number: usize,
    form: Form,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Ending {
    #[default]
    Break,
    Open,
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
    prior: Option<Lexed<'text>>,
    rest: &'lexed [Lexed<'text>],
}

#[derive(Debug)]
struct Step<'lexed, 'text> {
    piece: Piece,
    cursor: Cursor<'lexed, 'text>,
}

#[derive(Debug)]
struct Deepened {
    depth: usize,
    inside: Inside,
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
    pub fn gathered(lines: impl IntoIterator<Item = Line>) -> Notes {
        let numbered: BTreeMap<usize, Line> = lines.into_iter().map(|line| (line.number, line)).collect();
        let last = numbered.keys().next_back().copied().unwrap_or_default();
        Notes {
            lines: (1..=last)
                .map(|number| {
                    numbered.get(&number).cloned().unwrap_or(Line { number, form: Form::Layout(Layout::default()) })
                })
                .collect(),
            ending: Ending::Break,
        }
    }

    #[must_use]
    pub fn lines(&self) -> &[Line] {
        &self.lines
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
            Form::Content(Content::Heading(_)) => reader.heading(self),
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
        lexed.first().map_or(Opening::Prose, |first| first.opening()).content(text, &lexed)
    }
}

impl Opening {
    fn content(self, text: &str, lexed: &[Lexed<'_>]) -> Content {
        match self {
            Opening::Heading => Content::Heading(Heading(text.to_owned())),
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
            | Lexeme::Single
            | Lexeme::Cite
            | Lexeme::Word => Some(Kind::Remark),
        }
    }

    fn candidate(self) -> Role {
        match self {
            Lexeme::Space | Lexeme::Open => Role::Opening,
            Lexeme::Hash
            | Lexeme::Dash
            | Lexeme::Ruling
            | Lexeme::Lead
            | Lexeme::Close
            | Lexeme::Double
            | Lexeme::Single
            | Lexeme::Cite
            | Lexeme::Date
            | Lexeme::Word => Role::Closing,
        }
    }

    fn follow(self) -> Follow {
        match self {
            Lexeme::Space => Follow::Gap,
            Lexeme::Ruling | Lexeme::Lead | Lexeme::Cite | Lexeme::Date | Lexeme::Word => Follow::Word,
            Lexeme::Hash | Lexeme::Dash | Lexeme::Open | Lexeme::Close | Lexeme::Double | Lexeme::Single => {
                Follow::Mark
            }
        }
    }

    fn inside(self, role: Role) -> Inside {
        match self {
            Lexeme::Single => Inside::from(role),
            Lexeme::Open => Inside::Nested,
            Lexeme::Close => Inside::Unnested,
            Lexeme::Hash
            | Lexeme::Dash
            | Lexeme::Ruling
            | Lexeme::Lead
            | Lexeme::Double
            | Lexeme::Cite
            | Lexeme::Date
            | Lexeme::Word
            | Lexeme::Space => Inside::Plain,
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
                | Lexeme::Single
                | Lexeme::Cite
                | Lexeme::Date
                | Lexeme::Word
                | Lexeme::Space,
            )
            | None => Opening::Prose,
        }
    }

    fn taken<'lexed>(self, prior: Option<Lexed<'text>>, rest: &'lexed [Lexed<'text>]) -> Step<'lexed, 'text> {
        match self.lexeme {
            Some(Lexeme::Double) => Mark::Double.quoted(self, rest),
            Some(Lexeme::Single) => Role::around(prior, rest.first().copied()).single(self, rest),
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
                | Lexeme::Space,
            )
            | None => Step::after(Piece::read(self), self, rest),
        }
    }

    fn inside(self, prior: Lexed<'text>, later: Option<Lexed<'text>>) -> Inside {
        self.lexeme.map_or(Inside::Plain, |lexeme| lexeme.inside(Role::around(Some(prior), later)))
    }

    fn joined(lexed: &[Lexed<'text>]) -> String {
        lexed.iter().map(|lexed| lexed.text).collect()
    }
}

impl<'lexed, 'text> Cursor<'lexed, 'text> {
    fn step(self) -> Option<Step<'lexed, 'text>> {
        self.rest.split_first().map(|(first, rest)| first.taken(self.prior, rest))
    }
}

impl<'lexed, 'text> Step<'lexed, 'text> {
    fn after(piece: Piece, lexed: Lexed<'text>, rest: &'lexed [Lexed<'text>]) -> Step<'lexed, 'text> {
        Step { piece, cursor: Cursor { prior: Some(lexed), rest } }
    }
}

impl Role {
    fn around(prior: Option<Lexed<'_>>, later: Option<Lexed<'_>>) -> Role {
        let candidate = prior.map_or(Role::Opening, |prior| prior.lexeme.map_or(Role::Closing, Lexeme::candidate));
        candidate.standing(later.map_or(Follow::End, |later| later.lexeme.map_or(Follow::Mark, Lexeme::follow)))
    }

    fn standing(self, follow: Follow) -> Role {
        match follow {
            Follow::End | Follow::Gap => Some(self).filter(|role| *role != Role::Opening).unwrap_or(Role::Plain),
            Follow::Word => Some(self).filter(|role| *role != Role::Closing).unwrap_or(Role::Plain),
            Follow::Mark => self,
        }
    }

    fn single<'lexed, 'text>(self, mark: Lexed<'text>, rest: &'lexed [Lexed<'text>]) -> Step<'lexed, 'text> {
        match self {
            Role::Opening => Mark::Single.quoted(mark, rest),
            Role::Closing | Role::Plain => Step::after(Piece::read(mark), mark, rest),
        }
    }
}

impl From<Role> for Inside {
    fn from(role: Role) -> Inside {
        match role {
            Role::Opening => Inside::Opening,
            Role::Closing => Inside::Closing,
            Role::Plain => Inside::Plain,
        }
    }
}

impl Inside {
    fn deepened(self, depth: usize) -> Option<Deepened> {
        match self {
            Inside::Opening => None,
            Inside::Nested => Some(Deepened { depth: depth.saturating_add(1), inside: self }),
            Inside::Unnested => depth.checked_sub(1).map(|depth| Deepened { depth, inside: self }),
            Inside::Closing | Inside::Plain => Some(Deepened { depth, inside: self }),
        }
    }

    fn last_closing(opener: Lexed<'_>, rest: &[Lexed<'_>]) -> Option<usize> {
        let priors = iter::once(opener).chain(rest.iter().copied());
        let laters = rest.iter().copied().skip(1).map(Some).chain(iter::once(None));
        rest.iter()
            .zip(priors)
            .zip(laters)
            .map(|((lexed, prior), later)| lexed.inside(prior, later))
            .scan(0, |depth, inside| {
                inside.deepened(*depth).map(|deepened| {
                    *depth = deepened.depth;
                    deepened.inside
                })
            })
            .enumerate()
            .filter(|(_at, inside)| *inside == Inside::Closing)
            .map(|(at, _inside)| at)
            .last()
    }
}

impl Mark {
    fn quoted<'lexed, 'text>(self, opener: Lexed<'text>, rest: &'lexed [Lexed<'text>]) -> Step<'lexed, 'text> {
        let closed = self
            .closing(opener, rest)
            .and_then(|at| rest.get(..at).zip(rest.get(at..)))
            .and_then(|(inner, from)| from.split_first().map(|(closer, after)| (inner, *closer, after)));
        closed.map_or_else(
            || Step::after(Piece::Unclosed(self, Unclosed::new(Quote::new(Lexed::joined(rest)))), opener, rest),
            |(inner, closer, after)| Step::after(Piece::Quoted(self, Quote::new(Lexed::joined(inner))), closer, after),
        )
    }

    fn closing(self, opener: Lexed<'_>, rest: &[Lexed<'_>]) -> Option<usize> {
        match self {
            Mark::Double => rest.iter().position(|lexed| lexed.lexeme == Some(Lexeme::Double)),
            Mark::Single => Inside::last_closing(opener, rest),
        }
    }

    const fn sign(self) -> char {
        match self {
            Mark::Double => '"',
            Mark::Single => '\'',
        }
    }
}

impl Pieces {
    fn parsed(lexed: &[Lexed<'_>]) -> Pieces {
        let start = Cursor { prior: None, rest: lexed };
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
                | Lexeme::Single
                | Lexeme::Word
                | Lexeme::Space,
            )
            | None => Piece::Text(lexed.text.to_owned()),
        }
    }

    fn quote(&self) -> Option<Result<&Quote, &Unclosed>> {
        match self {
            Piece::Quoted(_, quote) => Some(Ok(quote)),
            Piece::Unclosed(_, unclosed) => Some(Err(unclosed)),
            Piece::Text(_) | Piece::Cited(_) | Piece::Dated(_) => None,
        }
    }

    fn cite(&self) -> Option<&Cite> {
        match self {
            Piece::Cited(cite) => Some(cite),
            Piece::Text(_) | Piece::Quoted(..) | Piece::Unclosed(..) | Piece::Dated(_) => None,
        }
    }

    fn date(&self) -> Option<&Date> {
        match self {
            Piece::Dated(date) => Some(date),
            Piece::Text(_) | Piece::Quoted(..) | Piece::Unclosed(..) | Piece::Cited(_) => None,
        }
    }
}

impl fmt::Display for Notes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.lines.split_last().map_or(Ok(()), |(last, earlier)| {
            earlier.iter().try_for_each(|line| writeln!(formatter, "{line}"))?;
            write!(formatter, "{last}{}", self.ending)
        })
    }
}

impl fmt::Display for Ending {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ending::Break => writeln!(formatter),
            Ending::Open => Ok(()),
        }
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
            Content::Heading(heading) => write!(formatter, "{heading}"),
            Content::Entry(entry) => write!(formatter, "{entry}"),
        }
    }
}

impl fmt::Display for Heading {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
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
            Piece::Quoted(mark, quote) => write!(formatter, "{0}{quote}{0}", mark.sign()),
            Piece::Cited(cite) => write!(formatter, "{cite}"),
            Piece::Unclosed(mark, _) => write!(formatter, "{}", mark.sign()),
            Piece::Dated(date) => write!(formatter, "{date}"),
        }
    }
}
