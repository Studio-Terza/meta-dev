use std::fmt;
use std::num::NonZeroU32;

use logos::Logos;

use crate::{Ignoring, Unlexed};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = logos)]
enum Cite<'text> {
    #[regex("[a-z0-9-]+", |lexer| lexer.slice())]
    Project(&'text str),
    #[regex("#D[0-9]+", |lexer| lexer.slice().get(2..).and_then(Cited::numbered))]
    Detail(NonZeroU32),
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = logos)]
enum Mark<'text> {
    #[regex("D[0-9]+", |lexer| lexer.slice(), priority = 10)]
    Detail(&'text str),
    #[token(" (")]
    Aside,
    #[regex("[^ ]+", |lexer| lexer.slice())]
    Word(&'text str),
    #[token(" ")]
    Space,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cited<'text> {
    project: &'text str,
    detail: NonZeroU32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Heading {
    detail: NonZeroU32,
    text: String,
}

impl<'text> Cited<'text> {
    #[must_use]
    pub fn read(text: &'text str) -> Option<Cited<'text>> {
        let pieces = Cite::lexer(text).collect::<Result<Vec<Cite<'text>>, ()>>().ignoring(Unlexed::NotThisGrammar)?;
        match <[Cite<'text>; 2]>::try_from(pieces).ignoring(Unlexed::OtherShape)? {
            [Cite::Project(project), Cite::Detail(detail)] => Some(Cited { project, detail }),
            [Cite::Project(_) | Cite::Detail(_), _] => None,
        }
    }

    fn numbered(digits: &str) -> Option<NonZeroU32> {
        digits.parse::<NonZeroU32>().ignoring(Unlexed::OutOfRange)
    }

    #[must_use]
    pub fn project(&self) -> &'text str {
        self.project
    }

    #[must_use]
    pub fn detail(&self) -> NonZeroU32 {
        self.detail
    }
}

impl Heading {
    #[must_use]
    pub fn read(line: &str) -> Option<Heading> {
        let pieces = Mark::lexer(line).collect::<Result<Vec<Mark<'_>>, ()>>().ignoring(Unlexed::NotThisGrammar)?;
        match pieces.as_slice() {
            [Mark::Detail(opening), Mark::Space, rest @ ..] => {
                let detail = opening.get(1..).and_then(Cited::numbered)?;
                let text: String = rest.iter().take_while(|mark| **mark != Mark::Aside).map(Mark::text).collect();
                text.chars().next().map(|_said| Heading { detail, text })
            }
            _other => None,
        }
    }

    #[must_use]
    pub fn detail(&self) -> NonZeroU32 {
        self.detail
    }
}

impl<'text> Mark<'text> {
    fn text(&self) -> &'text str {
        match self {
            Mark::Detail(text) | Mark::Word(text) => text,
            Mark::Aside => " (",
            Mark::Space => " ",
        }
    }
}

impl fmt::Display for Heading {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "D{} {}", self.detail, self.text)
    }
}
