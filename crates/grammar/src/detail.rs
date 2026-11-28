use std::fmt;
use std::num::NonZeroU32;

use invariants::Ignoring;
use logos::Logos;

use crate::{Grammar, Unlexed, Whole};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Cite<'text> {
    #[regex("[a-z0-9-]+", |lexer| lexer.slice())]
    Project(&'text str),
    #[regex("#D[0-9]+", |lexer| lexer.slice().get(2..).and_then(Cited::numbered))]
    Detail(NonZeroU32),
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(skip r"[, ]+")]
enum Listed<'text> {
    #[regex("[a-z0-9-]+#D[0-9]+", |lexer| lexer.slice())]
    Cite(&'text str),
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

impl<'text> Grammar<'text> for Cited<'text> {
    fn read(text: &'text str) -> Option<Cited<'text>> {
        match Whole::exactly(text)? {
            [Cite::Project(project), Cite::Detail(detail)] => Some(Cited { project, detail }),
            [Cite::Project(_) | Cite::Detail(_), _] => None,
        }
    }
}

impl<'text> Cited<'text> {
    #[must_use]
    pub fn each(text: &'text str) -> Vec<Cited<'text>> {
        Listed::lexer(text)
            .filter_map(|listed| listed.ignoring(Unlexed::NotThisGrammar))
            .filter_map(|Listed::Cite(cite)| Cited::read(cite))
            .collect()
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

impl Grammar<'_> for Heading {
    fn read(line: &str) -> Option<Heading> {
        match Whole::tokens(line)?.as_slice() {
            [Mark::Detail(opening), Mark::Space, rest @ ..] => {
                let detail = opening.get(1..).and_then(Cited::numbered)?;
                let text: String = rest.iter().take_while(|mark| **mark != Mark::Aside).map(Mark::text).collect();
                text.chars().next().map(|_said| Heading { detail, text })
            }
            _other => None,
        }
    }
}

impl Heading {
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
