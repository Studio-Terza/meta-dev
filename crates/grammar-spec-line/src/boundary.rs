mod head;
mod piece;

use std::fmt;
use std::iter;
use std::str::FromStr;

use grammar::logos::{Lexer, Logos};

use head::Head;
use piece::Piece;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    Repair,
    Spec(Spec),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spec {
    value: String,
    attachment: Option<Attachment>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attachment(String);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("the line is neither a spec line nor repair: main")]
pub struct Unspecified;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Lexed {
    piece: Piece,
    text: String,
}

impl Line {
    const REPAIR: &'static str = "repair: main";
    const SPEC: &'static str = "spec: ";

    #[must_use]
    pub fn spec(&self) -> Option<&Spec> {
        match self {
            Line::Spec(spec) => Some(spec),
            Line::Repair => None,
        }
    }

    fn repaired(rest: &[Lexed]) -> Option<Line> {
        rest.first().map_or(Some(Line::Repair), |_more| None)
    }
}

impl Head {
    fn line(self, spoken: &[Lexed]) -> Option<Line> {
        match self {
            Head::Repair => Line::repaired(spoken),
            Head::Spec => Spec::after(spoken).map(Line::Spec),
        }
    }
}

impl Spec {
    fn after(rest: &[Lexed]) -> Option<Spec> {
        rest.first().map(|_said| Spec::read(rest))
    }

    fn read(lexed: &[Lexed]) -> Spec {
        let only = <&[Lexed; 1]>::try_from(lexed).ok();
        Spec {
            value: lexed.iter().map(|lexed| lexed.text.as_str()).collect(),
            attachment: only.and_then(|[only]| only.attachment()),
        }
    }

    #[must_use]
    pub fn attachment(&self) -> Option<&Attachment> {
        self.attachment.as_ref()
    }
}

impl Lexed {
    fn attachment(&self) -> Option<Attachment> {
        self.piece.attachment(self.text.clone())
    }

    fn spoken(lexed: &[Lexed]) -> &[Lexed] {
        let last = lexed.iter().rposition(|lexed| lexed.piece.spoken().is_some());
        lexed.get(..last.map_or(0, |last| last + 1)).unwrap_or_default()
    }

    fn next(pieces: &mut Lexer<'_, Piece>) -> Option<Result<Lexed, ()>> {
        pieces.next().map(|piece| piece.map(|piece| Lexed { piece, text: pieces.slice().to_owned() }))
    }
}

impl FromStr for Line {
    type Err = Unspecified;

    fn from_str(text: &str) -> Result<Line, Unspecified> {
        let mut head = Head::lexer(text);
        let opening = head.next().and_then(Result::ok);
        let mut pieces = head.morph::<Piece>();
        let lexed = iter::from_fn(|| Lexed::next(&mut pieces)).collect::<Result<Vec<Lexed>, ()>>();
        opening.zip(lexed.ok()).and_then(|(opening, lexed)| opening.line(Lexed::spoken(&lexed))).ok_or(Unspecified)
    }
}

impl fmt::Display for Line {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Line::Repair => formatter.write_str(Self::REPAIR),
            Line::Spec(spec) => write!(formatter, "{}{}", Self::SPEC, spec.value),
        }
    }
}

impl fmt::Display for Attachment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
