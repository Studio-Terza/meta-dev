mod piece;

use std::fmt;
use std::ops::Range;

use logos::Logos;

use piece::Piece;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Line<'line> {
    Repair,
    Spec(Spec<'line>),
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spec<'line> {
    value: &'line str,
    attachment: Option<Attachment<'line>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attachment<'line>(&'line str);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Opening {
    Repair,
    Spec,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Lexed {
    piece: Result<Piece, ()>,
    span: Range<usize>,
}

impl<'line> Line<'line> {
    const REPAIR: &'static str = "repair: main";
    const SPEC: &'static str = "spec: ";

    #[must_use]
    pub fn read(text: &'line str) -> Line<'line> {
        let lexed: Vec<Lexed> = Piece::lexer(text).spanned().map(|(piece, span)| Lexed { piece, span }).collect();
        let last = lexed.iter().rposition(|lexed| lexed.piece.ok().is_none_or(|piece| piece.spoken().is_some()));
        let spoken = lexed.get(..last.map_or(0, |last| last + 1)).unwrap_or_default();
        spoken.split_first().map_or(Line::Other, |(first, rest)| Line::opened(text, first, rest))
    }

    #[must_use]
    pub fn spec(self) -> Option<Spec<'line>> {
        match self {
            Line::Spec(spec) => Some(spec),
            Line::Repair | Line::Other => None,
        }
    }

    fn opened(text: &'line str, first: &Lexed, rest: &[Lexed]) -> Line<'line> {
        match first.piece.map_or(Opening::Other, Piece::opening) {
            Opening::Repair => Line::repaired(rest),
            Opening::Spec => Spec::after(text, first, rest).map_or(Line::Other, Line::Spec),
            Opening::Other => Line::Other,
        }
    }

    fn repaired(rest: &[Lexed]) -> Line<'line> {
        rest.first().map_or(Line::Repair, |_more| Line::Other)
    }
}

impl<'line> Spec<'line> {
    fn after(text: &'line str, opening: &Lexed, rest: &[Lexed]) -> Option<Spec<'line>> {
        rest.last().and_then(|last| text.get(opening.span.end..last.span.end)).map(|value| Spec::read(value, rest))
    }

    fn read(value: &'line str, lexed: &[Lexed]) -> Spec<'line> {
        let only = <&[Lexed; 1]>::try_from(lexed).ok().and_then(|[only]| only.piece.ok());
        Spec { value, attachment: only.and_then(|piece| piece.attachment(value)) }
    }

    #[must_use]
    pub fn attachment(self) -> Option<Attachment<'line>> {
        self.attachment
    }
}

impl fmt::Display for Line<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Line::Repair => formatter.write_str(Self::REPAIR),
            Line::Spec(spec) => write!(formatter, "{}{}", Self::SPEC, spec.value),
            Line::Other => Ok(()),
        }
    }
}

impl fmt::Display for Attachment<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}
