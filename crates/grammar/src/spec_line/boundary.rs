use std::fmt;

use logos::Logos;
use newtype::Checked;

use crate::unspelled::Unspelled;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Head {
    #[token("repair: main")]
    Repair,
    #[token("spec: ")]
    Spec,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Tail<'text> {
    #[regex(r"[^\n]*[^\s]", |lexer| lexer.slice(), allow_greedy = true)]
    Value(&'text str),
    #[regex(r"\s+")]
    Blank,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Form {
    #[token("none")]
    Nothing,
    #[regex("owner [0-9A-Fa-f]{8}[0-9A-Fa-f-]*")]
    Owner,
    #[regex("( *[a-z0-9-]+#[DL][0-9]+,)* *[a-z0-9-]+#[DL][0-9]+")]
    Labels,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Line<'body> {
    Repair,
    Spec(&'body str),
    Other,
}

#[derive(newtype::Newtype, Debug, PartialEq, Eq)]
#[newtype(checked)]
pub struct Attachment<'line>(&'line str);

impl<'body> Line<'body> {
    const SPEC: &'static str = "spec: ";
    const REPAIR: &'static str = "repair: main";

    #[must_use]
    pub fn spec(self) -> Option<&'body str> {
        match self {
            Line::Spec(value) => Some(value),
            Line::Repair | Line::Other => None,
        }
    }

    fn repaired(value: Option<&'body str>) -> Option<Line<'body>> {
        value.map_or(Some(Line::Repair), |_trailing| None)
    }
}

impl Head {
    fn line(self, value: Option<&str>) -> Option<Line<'_>> {
        match self {
            Head::Repair => Line::repaired(value),
            Head::Spec => value.map(Line::Spec),
        }
    }
}

impl<'text> Tail<'text> {
    fn written(self) -> Option<&'text str> {
        match self {
            Tail::Value(value) => Some(value),
            Tail::Blank => None,
        }
    }
}

impl<'body> From<&'body str> for Line<'body> {
    fn from(text: &'body str) -> Line<'body> {
        let mut head = Head::lexer(text);
        let opened = head.next().and_then(Result::ok);
        let value = head.morph::<Tail<'body>>().next().and_then(Result::ok).and_then(Tail::written);
        opened.and_then(|lexed| lexed.line(value)).unwrap_or(Line::Other)
    }
}

impl fmt::Display for Line<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Line::Repair => formatter.write_str(Self::REPAIR),
            Line::Spec(value) => write!(formatter, "{}{value}", Self::SPEC),
            Line::Other => Ok(()),
        }
    }
}

impl<'line> Checked for Attachment<'line> {
    type Error = Unspelled;

    fn checked<'held>(value: &'held &'line str) -> Result<&'held &'line str, Unspelled> {
        let spelled: Vec<Result<Form, ()>> = Form::lexer(value).collect();
        <[Result<Form, ()>; 1]>::try_from(spelled).ok().and_then(|[only]| only.ok()).map(|_form| value).ok_or(Unspelled)
    }
}

impl fmt::Display for Attachment<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}
