use grammar::logos::Logos;
use grammar::{Ignoring, Unlexed};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Ran<'text> {
    #[token("test ")]
    Test,
    #[token(" ... ")]
    Dots,
    #[token(" ")]
    Space,
    #[regex("[^ ]+", |lexer| lexer.slice())]
    Word(&'text str),
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Running<'text> {
    #[regex(" *Running `")]
    Open,
    #[token("/")]
    Slash,
    #[token("`")]
    Close,
    #[regex("[^/` ]+", |lexer| lexer.slice())]
    Segment(&'text str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TestLine<'text> {
    Running { package: &'text str, binary: &'text str },
    Ran { name: &'text str, verdict: Verdict },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verdict {
    Passed,
    Failed,
    Ignored,
}

impl<'text> Running<'text> {
    const HASH: usize = "-0123456789abcdef".len();

    fn segment(&self) -> Option<&'text str> {
        match *self {
            Running::Segment(segment) => Some(segment),
            Running::Open | Running::Slash | Running::Close => None,
        }
    }

    fn binary(file: &'text str, hash: &str) -> Option<&'text str> {
        let binary = file.get(..file.len().checked_sub(Running::HASH)?)?;
        let suffix = file.get(binary.len()..)?;
        (suffix == ["-", hash].concat()).then_some(binary)
    }
}

impl<'text> TestLine<'text> {
    #[must_use]
    pub fn read(line: &'text str) -> Option<TestLine<'text>> {
        TestLine::ran(line).or_else(|| TestLine::running(line))
    }

    fn ran(line: &'text str) -> Option<TestLine<'text>> {
        let pieces = Ran::lexer(line).collect::<Result<Vec<Ran<'text>>, ()>>().ignoring(Unlexed::NotThisGrammar)?;
        match pieces.as_slice() {
            [Ran::Test, Ran::Word(name), Ran::Dots, Ran::Word("ok")] => {
                Some(TestLine::Ran { name, verdict: Verdict::Passed })
            }
            [Ran::Test, Ran::Word(name), Ran::Dots, Ran::Word("FAILED")] => {
                Some(TestLine::Ran { name, verdict: Verdict::Failed })
            }
            [Ran::Test, Ran::Word(name), Ran::Dots, Ran::Word("ignored" | "ignored,"), ..] => {
                Some(TestLine::Ran { name, verdict: Verdict::Ignored })
            }
            _ => None,
        }
    }

    fn running(line: &'text str) -> Option<TestLine<'text>> {
        let pieces =
            Running::lexer(line).collect::<Result<Vec<Running<'text>>, ()>>().ignoring(Unlexed::NotThisGrammar)?;
        let segments: Vec<&'text str> = match pieces.as_slice() {
            [Running::Open, path @ .., Running::Close] => path.iter().filter_map(Running::segment).collect(),
            _ => Vec::new(),
        };
        match segments.as_slice() {
            [.., "build", package, hash, "out", file] => {
                Running::binary(file, hash).map(|binary| TestLine::Running { package, binary })
            }
            _ => None,
        }
    }
}
