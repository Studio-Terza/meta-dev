use std::fmt;
use std::str::FromStr;

use logos::Lexer;
use logos::Logos;
use newtype::{Arbitrary, Checked, Display, Newtype};

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
enum Rest<'text> {
    Empty,
    Blank,
    Written(&'text str),
    Trailed(&'text str),
    Garbled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    Repair,
    Spec(Spec),
}

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
#[newtype(checked, strategy = "[!-~]([ -~]{0,20}[!-~])?")]
pub struct Spec(String);

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
            Line::Spec(spec) => Attachment::try_new(spec.clone().into_inner()).ok(),
            Line::Repair => None,
        }
    }
}

impl Head {
    fn line(self, rest: Rest<'_>) -> Option<Line> {
        match rest {
            Rest::Empty | Rest::Blank => self.unwritten(),
            Rest::Written(written) | Rest::Trailed(written) => self.written(written),
            Rest::Garbled => None,
        }
    }

    fn unwritten(self) -> Option<Line> {
        match self {
            Head::Repair => Some(Line::Repair),
            Head::Spec => None,
        }
    }

    fn written(self, written: &str) -> Option<Line> {
        match self {
            Head::Repair => None,
            Head::Spec => Spec::lexed(written).map(Line::Spec),
        }
    }
}

impl Spec {
    fn lexed(written: &str) -> Option<Spec> {
        Spec::try_new(written.to_owned()).ok()
    }
}

impl<'text> Rest<'text> {
    fn lexed(tail: Lexer<'text, Tail<'text>>) -> Rest<'text> {
        tail.map(Result::ok).fold(Rest::Empty, Rest::then)
    }

    fn then(self, token: Option<Tail<'text>>) -> Rest<'text> {
        match self {
            Rest::Empty => Rest::opened(token),
            Rest::Written(written) => Rest::trailed(written, token),
            Rest::Blank | Rest::Trailed(_) | Rest::Garbled => Rest::Garbled,
        }
    }

    fn opened(token: Option<Tail<'text>>) -> Rest<'text> {
        match token {
            Some(Tail::Value(written)) => Rest::Written(written),
            Some(Tail::Blank) => Rest::Blank,
            None => Rest::Garbled,
        }
    }

    fn trailed(written: &'text str, token: Option<Tail<'text>>) -> Rest<'text> {
        match token {
            Some(Tail::Blank) => Rest::Trailed(written),
            Some(Tail::Value(_)) | None => Rest::Garbled,
        }
    }
}

impl FromStr for Line {
    type Err = Unspelled;

    fn from_str(text: &str) -> Result<Line, Unspelled> {
        let mut head = Head::lexer(text);
        let opened = head.next().and_then(Result::ok);
        let rest = Rest::lexed(head.morph::<Tail<'_>>());
        opened.and_then(|lexed| lexed.line(rest)).ok_or(Unspelled)
    }
}

impl fmt::Display for Line {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Line::Repair => formatter.write_str(Self::REPAIR),
            Line::Spec(spec) => write!(formatter, "{}{spec}", Self::SPEC),
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

impl Checked for Spec {
    type Error = Unspelled;

    fn checked(value: &String) -> Result<&String, Unspelled> {
        match Rest::lexed(Tail::lexer(value)) {
            Rest::Written(_) => Ok(value),
            Rest::Empty | Rest::Blank | Rest::Trailed(_) | Rest::Garbled => Err(Unspelled),
        }
    }
}

impl fmt::Display for Attachment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
