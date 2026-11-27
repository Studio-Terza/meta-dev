use std::num::NonZeroU32;

use logos::Logos;

use crate::{BranchName, Ignoring, Unlexed};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = logos)]
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
#[logos(crate = logos)]
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

impl MergeCommand {
    #[must_use]
    pub fn read(command: &str) -> Option<MergeCommand> {
        Word::lexer(command)
            .filter_map(|word| word.ignoring(Unlexed::NotThisGrammar))
            .fold(Heard::Nothing, Heard::after)
            .merged()
            .map(MergeCommand)
    }

    #[must_use]
    pub fn pull(self) -> NonZeroU32 {
        self.0
    }
}

impl<'text> Worktrees<'text> {
    #[must_use]
    pub fn read(listed: &'text str) -> Worktrees<'text> {
        Porcelain::lexer(listed)
            .filter_map(|line| line.ignoring(Unlexed::NotThisGrammar))
            .fold(Worktrees::default(), Worktrees::heard)
    }

    fn heard(self, line: Porcelain<'text>) -> Worktrees<'text> {
        let Worktrees(mut listed) = self;
        match (line, listed.pop()) {
            (Porcelain::Path(path), last) => {
                listed.extend(last);
                listed.push(Worktree { path, head: None, branch: None });
            }
            (Porcelain::Head(head), Some(last)) => listed.push(Worktree { head: Some(head), ..last }),
            (Porcelain::Branch(branch), Some(last)) => {
                listed.push(Worktree { branch: BranchName::read(branch), ..last });
            }
            (Porcelain::Head(_) | Porcelain::Branch(_) | Porcelain::Other | Porcelain::Break, last) => {
                listed.extend(last);
            }
        }
        Worktrees(listed)
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
