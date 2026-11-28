use std::fmt;
use std::num::{NonZeroU32, NonZeroU64};

use invariants::Ignoring;
use logos::Logos;

use crate::{Grammar, Unlexed, Whole};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
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
enum Linked {
    #[regex("/job/[0-9]+", |lexer| lexer.slice().get(5..).and_then(Linked::numbered))]
    Job(NonZeroU64),
    #[regex("[^/]+")]
    Text,
    #[token("/")]
    Slash,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(skip r"[ \t]+")]
enum Logged {
    #[token("MISSED")]
    Missed,
    #[regex(r"[^ \t]+")]
    Word,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MissedMutant;

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

impl<'text> Grammar<'text> for BranchName<'text> {
    fn read(text: &'text str) -> Option<BranchName<'text>> {
        match Whole::exactly(text)? {
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

impl<'text> Grammar<'text> for AgentReport<'text> {
    fn read(text: &'text str) -> Option<AgentReport<'text>> {
        match Whole::exactly(text)? {
            [Piece::Ended(ended), aimed, Piece::Head(head)] => {
                aimed.target().map(|target| AgentReport { ended, target, head })
            }
            [Piece::Ended(_) | Piece::Pull(_) | Piece::Branch(_) | Piece::Head(_), _, _] => None,
        }
    }
}

impl<'text> AgentReport<'text> {
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

impl Grammar<'_> for JobLink {
    fn read(text: &str) -> Option<JobLink> {
        Linked::lexer(text)
            .filter_map(|linked| linked.ignoring(Unlexed::NotThisGrammar))
            .filter_map(|linked| match linked {
                Linked::Job(job) => Some(JobLink(job)),
                Linked::Text | Linked::Slash => None,
            })
            .last()
    }
}

impl JobLink {
    #[must_use]
    pub fn job(self) -> NonZeroU64 {
        self.0
    }
}

impl Grammar<'_> for MissedMutant {
    fn read(line: &str) -> Option<MissedMutant> {
        Logged::lexer(line).filter_map(|logged| logged.ignoring(Unlexed::NotThisGrammar)).find_map(
            |logged| match logged {
                Logged::Missed => Some(MissedMutant),
                Logged::Word => None,
            },
        )
    }
}
