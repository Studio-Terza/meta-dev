use std::num::NonZeroU32;

use invariants::Ignoring;
use logos::Logos;

use crate::report::BranchName;
use crate::{Grammar, Unlexed};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(skip r"[ \t\r\n]+")]
enum Word {
    #[token("gh")]
    Gh,
    #[token("pr")]
    Pr,
    #[token("merge")]
    Merge,
    #[regex("[0-9]+", |lexer| lexer.slice().parse::<NonZeroU32>().ignoring(Unlexed::OutOfRange))]
    Number(NonZeroU32),
    #[regex("[^ \t\r\n0-9][^ \t\r\n]*")]
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Heard {
    Nothing,
    Gh,
    Pr,
    Merge,
    Merged(NonZeroU32),
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Porcelain<'text> {
    #[regex("worktree [^\n]+", |lexer| lexer.slice().get(9..), priority = 8, allow_greedy = true)]
    Path(&'text str),
    #[regex("HEAD [0-9a-f]{40}", |lexer| lexer.slice().get(5..), priority = 8)]
    Head(&'text str),
    #[regex("branch refs/heads/[^\n]+", |lexer| lexer.slice().get(18..), priority = 8, allow_greedy = true)]
    Branch(&'text str),
    #[regex("[^\n]+", allow_greedy = true)]
    Other,
    #[token("\n")]
    Break,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MergeCommand(NonZeroU32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Worktree<'text> {
    path: &'text str,
    head: Option<&'text str>,
    branch: Option<BranchName<'text>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Worktrees<'text>(Vec<Worktree<'text>>);

impl Heard {
    fn after(self, word: Word) -> Heard {
        match (self, word) {
            (Heard::Merged(pull), _) | (Heard::Merge, Word::Number(pull)) => Heard::Merged(pull),
            (Heard::Pr, Word::Merge) => Heard::Merge,
            (Heard::Gh, Word::Pr) => Heard::Pr,
            (Heard::Nothing | Heard::Gh | Heard::Pr | Heard::Merge, Word::Gh) => Heard::Gh,
            (
                Heard::Nothing | Heard::Gh | Heard::Pr | Heard::Merge,
                Word::Pr | Word::Merge | Word::Number(_) | Word::Other,
            ) => Heard::Nothing,
        }
    }

    fn merged(self) -> Option<NonZeroU32> {
        match self {
            Heard::Merged(pull) => Some(pull),
            Heard::Nothing | Heard::Gh | Heard::Pr | Heard::Merge => None,
        }
    }
}

impl Grammar<'_> for MergeCommand {
    fn read(command: &str) -> Option<MergeCommand> {
        Word::lexer(command)
            .filter_map(|word| word.ignoring(Unlexed::NotThisGrammar))
            .fold(Heard::Nothing, Heard::after)
            .merged()
            .map(MergeCommand)
    }
}

impl MergeCommand {
    #[must_use]
    pub fn pull(self) -> NonZeroU32 {
        self.0
    }
}

impl<'text> From<&'text str> for Worktrees<'text> {
    fn from(listed: &'text str) -> Worktrees<'text> {
        Porcelain::lexer(listed)
            .filter_map(|line| line.ignoring(Unlexed::NotThisGrammar))
            .fold(Worktrees::default(), Worktrees::heard)
    }
}

impl<'text> Worktrees<'text> {
    fn heard(self, line: Porcelain<'text>) -> Worktrees<'text> {
        let Worktrees(mut listed) = self;
        let kept: Vec<Worktree<'text>> = match (line, listed.pop()) {
            (Porcelain::Path(path), open) => {
                open.into_iter().chain([Worktree { path, head: None, branch: None }]).collect()
            }
            (Porcelain::Head(head), Some(open)) => vec![Worktree { head: Some(head), ..open }],
            (Porcelain::Branch(branch), Some(open)) => vec![Worktree { branch: BranchName::read(branch), ..open }],
            (Porcelain::Head(_) | Porcelain::Branch(_) | Porcelain::Other | Porcelain::Break, open) => {
                open.into_iter().collect()
            }
        };
        Worktrees(listed.into_iter().chain(kept).collect())
    }

    #[must_use]
    pub fn listed(&self) -> &[Worktree<'text>] {
        &self.0
    }
}

impl<'text> Worktree<'text> {
    #[must_use]
    pub fn path(&self) -> &'text str {
        self.path
    }

    #[must_use]
    pub fn head(&self) -> Option<&'text str> {
        self.head
    }

    #[must_use]
    pub fn branch(&self) -> Option<BranchName<'text>> {
        self.branch
    }
}
