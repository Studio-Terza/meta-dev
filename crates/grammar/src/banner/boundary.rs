use std::fmt;

use logos::Logos;

use crate::unspelled::Unspelled;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Banner<'text> {
    Running(&'text str),
    DocTests(&'text str),
}

impl<'text> Banner<'text> {
    const DOCTEST_PREFIX: &'static str = "doc-";

    fn running(after: &[Spelling<'text>]) -> Option<Banner<'text>> {
        let last = after.iter().rposition(|spelling| *spelling == Spelling::Target)?;
        let deps = after.get(last..)?.get(..5)?;
        <&[Spelling<'text>; 5]>::try_from(deps).ok().and_then(Spelling::binary)
    }

    fn documented(name: Option<&Spelling<'text>>) -> Option<Banner<'text>> {
        name.and_then(|spelling| spelling.word()).map(Banner::DocTests)
    }
}

impl<'text> Spelling<'text> {
    fn binary([_target, profile, deps, name, close]: &[Spelling<'text>; 5]) -> Option<Banner<'text>> {
        profile.word()?;
        Some(deps).filter(|spelling| **spelling == Spelling::Deps)?;
        Some(close).filter(|spelling| **spelling == Spelling::Close)?;
        name.word().map(Banner::Running)
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

    fn headed(self, after: &[Spelling<'text>]) -> Option<Banner<'text>> {
        match self {
            Spelling::Running => Banner::running(after),
            Spelling::DocTests => Banner::documented(after.first()),
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

impl<'text> TryFrom<&'text str> for Banner<'text> {
    type Error = Unspelled;

    fn try_from(line: &'text str) -> Result<Banner<'text>, Unspelled> {
        let spelled: Vec<Spelling<'text>> = Spelling::lexer(line).collect::<Result<_, ()>>().map_err(|()| Unspelled)?;
        let start = spelled.iter().position(|spelling| *spelling != Spelling::Space).ok_or(Unspelled)?;
        let (head, rest) = spelled.get(start..).and_then(<[Spelling<'text>]>::split_first).ok_or(Unspelled)?;
        let (space, after) = rest.split_first().ok_or(Unspelled)?;
        Some(space)
            .filter(|spelling| **spelling == Spelling::Space)
            .and_then(|_space| head.headed(after))
            .ok_or(Unspelled)
    }
}

impl fmt::Display for Banner<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Banner::Running(name) => formatter.write_str(name),
            Banner::DocTests(name) => write!(formatter, "{}{name}", Self::DOCTEST_PREFIX),
        }
    }
}
