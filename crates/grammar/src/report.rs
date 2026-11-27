use std::fmt;
use std::num::{NonZeroU32, NonZeroU64};

use logos::Logos;

use crate::{Ignoring, Unlexed};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = logos)]
#[logos(skip r"[ \t\r\n]+")]
enum Piece<'text> {
    #[token("DONE", |_lexer| Ended::Done)]
    #[token("BLOCKED", |_lexer| Ended::Blocked)]
    Ended(Ended),
    #[regex("#[0-9]+", |lexer| lexer.slice().get(1..).and_then(Piece::numbered))]
    Pull(NonZeroU32),
    #[regex("[a-z0-9][a-z0-9-]*(/[a-z0-9][a-z0-9-]*)+", |lexer| lexer.slice())]
    Branch(&'text str),
    #[regex("[0-9a-fA-F]{40}", |lexer| lexer.slice())]
    Head(&'text str),
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = logos)]
enum Linked {
    #[regex("/job/[0-9]+", |lexer| lexer.slice().get(5..).and_then(Linked::numbered))]
    Job(NonZeroU64),
    #[regex("[^/]+")]
    Text,
    #[token("/")]
    Slash,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ended {
    Done,
    Blocked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target<'text> {
    Pull(NonZeroU32),
    Branch(BranchName<'text>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BranchName<'text>(&'text str);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AgentReport<'text> {
    ended: Ended,
    target: Target<'text>,
    head: &'text str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JobLink(NonZeroU64);

impl<'text> Piece<'text> {
    fn numbered(digits: &str) -> Option<NonZeroU32> {
        digits.parse::<NonZeroU32>().ignoring(Unlexed::OutOfRange)
    }

    fn target(self) -> Option<Target<'text>> {
        match self {
            Piece::Pull(pull) => Some(Target::Pull(pull)),
            Piece::Branch(branch) => Some(Target::Branch(BranchName(branch))),
            Piece::Ended(_) | Piece::Head(_) => None,
        }
    }
}

impl<'text> BranchName<'text> {
    #[must_use]
    pub fn read(text: &'text str) -> Option<BranchName<'text>> {
        let pieces = Piece::lexer(text).collect::<Result<Vec<Piece<'text>>, ()>>().ignoring(Unlexed::NotThisGrammar)?;
        match <[Piece<'text>; 1]>::try_from(pieces).ignoring(Unlexed::OtherShape)? {
            [Piece::Branch(branch)] => Some(BranchName(branch)),
            [Piece::Ended(_) | Piece::Pull(_) | Piece::Head(_)] => None,
        }
    }
}

impl fmt::Display for BranchName<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl Linked {
    fn numbered(digits: &str) -> Option<NonZeroU64> {
        digits.parse::<NonZeroU64>().ignoring(Unlexed::OutOfRange)
    }
}

impl<'text> AgentReport<'text> {
    #[must_use]
    pub fn read(text: &'text str) -> Option<AgentReport<'text>> {
        let pieces = Piece::lexer(text).collect::<Result<Vec<Piece<'text>>, ()>>().ignoring(Unlexed::NotThisGrammar)?;
        match <[Piece<'text>; 3]>::try_from(pieces).ignoring(Unlexed::OtherShape)? {
            [Piece::Ended(ended), aimed, Piece::Head(head)] => {
                aimed.target().map(|target| AgentReport { ended, target, head })
            }
            [Piece::Ended(_) | Piece::Pull(_) | Piece::Branch(_) | Piece::Head(_), _, _] => None,
        }
    }

    #[must_use]
    pub fn ended(&self) -> Ended {
        self.ended
    }

    #[must_use]
    pub fn target(&self) -> Target<'text> {
        self.target
    }

    #[must_use]
    pub fn head(&self) -> &'text str {
        self.head
    }
}

impl JobLink {
    #[must_use]
    pub fn read(text: &str) -> Option<JobLink> {
        Linked::lexer(text)
            .filter_map(|linked| linked.ignoring(Unlexed::NotThisGrammar))
            .filter_map(|linked| match linked {
                Linked::Job(job) => Some(JobLink(job)),
                Linked::Text | Linked::Slash => None,
            })
            .last()
    }

    #[must_use]
    pub fn job(self) -> NonZeroU64 {
        self.0
    }
}
