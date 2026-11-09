use std::fmt;
use std::str::FromStr;

use grammar::logos::Logos;
use newtype::{Arbitrary, Checked, Newtype};

use grammar_unspelled::Unspelled;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Spelling<'text> {
    #[regex(r"\s+")]
    Space,
    #[token("Running")]
    Running,
    #[token("Doc-tests")]
    DocTests,
    #[token("(target/")]
    Target,
    #[token("/deps/")]
    Deps,
    #[token("/")]
    Slash,
    #[token("(")]
    Open,
    #[token(")")]
    Close,
    #[regex(r"[^\s()/]+", |lexer| lexer.slice())]
    Word(&'text str),
}

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
#[newtype(checked, strategy = "[a-z_-]{1,12}")]
pub struct Banner(String);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Heading<'text> {
    Running(&'text str),
    DocTests(&'text str),
}

impl<'text> Heading<'text> {
    const DOCTEST_PREFIX: &'static str = "doc-";

    fn running(after: &[Spelling<'text>]) -> Option<Heading<'text>> {
        let last = after.iter().rposition(|spelling| *spelling == Spelling::Target)?;
        let deps = after.get(last..)?.get(..5)?;
        <&[Spelling<'text>; 5]>::try_from(deps).ok().and_then(Spelling::binary)
    }

    fn documented(name: Option<&Spelling<'text>>) -> Option<Heading<'text>> {
        name.and_then(|spelling| spelling.word()).map(Heading::DocTests)
    }
}

impl<'text> Spelling<'text> {
    fn binary([_target, profile, deps, name, close]: &[Spelling<'text>; 5]) -> Option<Heading<'text>> {
        profile.word()?;
        Some(deps).filter(|spelling| **spelling == Spelling::Deps)?;
        Some(close).filter(|spelling| **spelling == Spelling::Close)?;
        name.word().map(Heading::Running)
    }

    fn word(self) -> Option<&'text str> {
        match self {
            Spelling::Word(word) => Some(word),
            Spelling::Space
            | Spelling::Running
            | Spelling::DocTests
            | Spelling::Target
            | Spelling::Deps
            | Spelling::Slash
            | Spelling::Open
            | Spelling::Close => None,
        }
    }

    fn headed(self, after: &[Spelling<'text>]) -> Option<Heading<'text>> {
        match self {
            Spelling::Running => Heading::running(after),
            Spelling::DocTests => Heading::documented(after.first()),
            Spelling::Space
            | Spelling::Target
            | Spelling::Deps
            | Spelling::Slash
            | Spelling::Open
            | Spelling::Close
            | Spelling::Word(_) => None,
        }
    }
}

impl Checked for Banner {
    type Error = Unspelled;

    fn checked(label: &String) -> Result<&String, Unspelled> {
        let spelled: Vec<Result<Spelling<'_>, ()>> = Spelling::lexer(label).collect();
        <[Result<Spelling<'_>, ()>; 1]>::try_from(spelled)
            .ok()
            .and_then(|[only]| only.ok())
            .and_then(Spelling::word)
            .map(|_word| label)
            .ok_or(Unspelled)
    }
}

impl FromStr for Banner {
    type Err = Unspelled;

    fn from_str(line: &str) -> Result<Banner, Unspelled> {
        let spelled: Vec<Spelling<'_>> = Spelling::lexer(line).collect::<Result<_, ()>>().map_err(|()| Unspelled)?;
        let start = spelled.iter().position(|spelling| *spelling != Spelling::Space).ok_or(Unspelled)?;
        let (head, rest) = spelled.get(start..).and_then(<[Spelling<'_>]>::split_first).ok_or(Unspelled)?;
        let (space, after) = rest.split_first().ok_or(Unspelled)?;
        Some(space)
            .filter(|spelling| **spelling == Spelling::Space)
            .and_then(|_space| head.headed(after))
            .ok_or(Unspelled)
            .and_then(|heading| Banner::try_new(heading.to_string()))
    }
}

impl fmt::Display for Heading<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Heading::Running(name) => formatter.write_str(name),
            Heading::DocTests(name) => write!(formatter, "{}{name}", Self::DOCTEST_PREFIX),
        }
    }
}
