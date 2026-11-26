use std::fmt;
use std::str::FromStr;

use grammar::logos::Logos;
use grammar::{Ignoring, Unlexed};
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
    #[token("/build/")]
    Build,
    #[token("/out/")]
    Out,
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
    Built { package: &'text str, stem: &'text str },
    DocTests(&'text str),
}

impl Heading<'_> {
    const DOCTEST_PREFIX: &'static str = "doc-";
    const PACKAGE_JOIN: &'static str = ".";
}

impl<'text> Spelling<'text> {
    fn running(after: &[Spelling<'text>]) -> Option<Heading<'text>> {
        let last = after.iter().rposition(|spelling| *spelling == Spelling::Target)?;
        let window = after.get(last..)?;
        InDeps::read(window).map(Heading::from).or_else(|| InBuild::read(window).map(Heading::from))
    }

    fn word(self) -> Option<&'text str> {
        match self {
            Spelling::Word(word) => Some(word),
            Spelling::Space
            | Spelling::Running
            | Spelling::DocTests
            | Spelling::Target
            | Spelling::Build
            | Spelling::Out
            | Spelling::Deps
            | Spelling::Slash
            | Spelling::Open
            | Spelling::Close => None,
        }
    }

    fn headed(self, after: &[Spelling<'text>]) -> Option<Heading<'text>> {
        match self {
            Spelling::Running => Spelling::running(after),
            Spelling::DocTests => after.first().and_then(|name| name.word()).map(Heading::DocTests),
            Spelling::Space
            | Spelling::Target
            | Spelling::Build
            | Spelling::Out
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
            .ignoring(Unlexed::OtherShape)
            .and_then(|[only]| only.ignoring(Unlexed::NotThisGrammar))
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
            Heading::Built { package, stem } => write!(formatter, "{package}{}{stem}", Self::PACKAGE_JOIN),
            Heading::DocTests(name) => write!(formatter, "{}{name}", Self::DOCTEST_PREFIX),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct InDeps<'text>(&'text str);

#[derive(Clone, Copy, Debug)]
struct InBuild<'text> {
    package: &'text str,
    stem: &'text str,
}

impl<'text> InDeps<'text> {
    fn read(window: &[Spelling<'text>]) -> Option<InDeps<'text>> {
        let [_target, profile, deps, name, close] =
            <&[Spelling<'text>; 5]>::try_from(window.get(..5)?).ignoring(Unlexed::OtherShape)?;
        profile.word()?;
        Some(deps).filter(|spelling| **spelling == Spelling::Deps)?;
        Some(close).filter(|spelling| **spelling == Spelling::Close)?;
        name.word().map(InDeps)
    }
}

impl<'text> InBuild<'text> {
    const HASH_JOIN: &'static str = "-";

    fn read(window: &[Spelling<'text>]) -> Option<InBuild<'text>> {
        let [_target, profile, build, package, slash, hash, out, binary, close] =
            <&[Spelling<'text>; 9]>::try_from(window.get(..9)?).ignoring(Unlexed::OtherShape)?;
        profile.word()?;
        Some(build).filter(|spelling| **spelling == Spelling::Build)?;
        Some(slash).filter(|spelling| **spelling == Spelling::Slash)?;
        Some(out).filter(|spelling| **spelling == Spelling::Out)?;
        Some(close).filter(|spelling| **spelling == Spelling::Close)?;
        let stem = InBuild::stem(binary.word()?, hash.word()?)?;
        Some(InBuild { package: package.word()?, stem })
    }

    fn stem(binary: &'text str, hash: &str) -> Option<&'text str> {
        let cut = binary.len().checked_sub(hash.len())?;
        let joined = cut.checked_sub(Self::HASH_JOIN.len())?;
        Some(()).filter(|()| binary.get(cut..) == Some(hash))?;
        Some(()).filter(|()| binary.get(joined..cut) == Some(Self::HASH_JOIN))?;
        binary.get(..joined)
    }
}

impl<'text> From<InDeps<'text>> for Heading<'text> {
    fn from(InDeps(name): InDeps<'text>) -> Heading<'text> {
        Heading::Running(name)
    }
}

impl<'text> From<InBuild<'text>> for Heading<'text> {
    fn from(InBuild { package, stem }: InBuild<'text>) -> Heading<'text> {
        Heading::Built { package, stem }
    }
}
