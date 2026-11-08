use std::fmt;
use std::str::FromStr;

use logos::Logos;
use newtype::{Arbitrary, Checked, Newtype};

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    Repair,
    Spec(String),
}

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
#[newtype(
    checked,
    strategy = "none|owner [0-9a-f]{8}[0-9a-f-]{0,8}|[a-z0-9-]{1,8}#[DL][0-9]{1,3}(, ?[a-z0-9-]{1,8}#[DL][0-9]{1,3}){0,2}"
)]
pub struct Attachment(String);

impl Line {
    const SPEC: &'static str = "spec: ";
    const REPAIR: &'static str = "repair: main";

    #[must_use]
    pub fn attachment(&self) -> Option<Attachment> {
        match self {
            Line::Spec(value) => Attachment::try_new(value.clone()).ok(),
            Line::Repair => None,
        }
    }

    fn repaired(value: Option<&str>) -> Option<Line> {
        value.map_or(Some(Line::Repair), |_trailing| None)
    }
}

impl Head {
    fn line(self, value: Option<&str>) -> Option<Line> {
        match self {
            Head::Repair => Line::repaired(value),
            Head::Spec => value.map(|written| Line::Spec(written.to_owned())),
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

impl FromStr for Line {
    type Err = Unspelled;

    fn from_str(text: &str) -> Result<Line, Unspelled> {
        let mut head = Head::lexer(text);
        let opened = head.next().and_then(Result::ok);
        let value = head.morph::<Tail<'_>>().next().and_then(Result::ok).and_then(Tail::written);
        opened.and_then(|lexed| lexed.line(value)).ok_or(Unspelled)
    }
}

impl fmt::Display for Line {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Line::Repair => formatter.write_str(Self::REPAIR),
            Line::Spec(value) => write!(formatter, "{}{value}", Self::SPEC),
        }
    }
}

impl Checked for Attachment {
    type Error = Unspelled;

    fn checked(value: &String) -> Result<&String, Unspelled> {
        let spelled: Vec<Result<Form, ()>> = Form::lexer(value).collect();
        <[Result<Form, ()>; 1]>::try_from(spelled).ok().and_then(|[only]| only.ok()).map(|_form| value).ok_or(Unspelled)
    }
}

impl fmt::Display for Attachment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
