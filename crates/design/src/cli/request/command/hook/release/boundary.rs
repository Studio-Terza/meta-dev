use std::path::{Path, PathBuf};

use design_messages::PullNumber;
use dispatch_ports::{Clock, Dispatch, Ran, Shell, Timeout};
use grammar_git::{MergeCommand, Worktree};
use invariants::Ignoring;
use serde::Deserialize;

use super::super::subagent::{Branch, CommitSha};
use super::Releasing;
use crate::cli::request::ignored::Ignored;

impl<D: Dispatch, C: Clock, G: Shell> Releasing<'_, '_, D, C, G> {
    const PROGRAM: &'static str = "git";
    const QUICK: Timeout = Timeout::new(60);

    pub(super) fn git(&self, arguments: &[&str], directory: &Path) -> Option<Ran> {
        self.crew.spoke(Self::PROGRAM, arguments, directory, Self::QUICK)
    }
}

#[derive(Debug, Deserialize)]
pub struct Used {
    cwd: PathBuf,
    #[serde(default)]
    tool_input: Input,
}

#[derive(Debug, Default, Deserialize)]
struct Input {
    #[serde(default)]
    command: Merge,
}

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(from = "Option<String>")]
struct Merge(Option<PullNumber>);

#[derive(Debug, Deserialize)]
pub(crate) struct Landed {
    state: State,
    #[serde(rename = "headRefName")]
    branch: Branch,
    #[serde(rename = "headRefOid")]
    head: CommitSha,
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    #[serde(rename = "MERGED")]
    Merged,
    #[serde(rename = "OPEN")]
    Open,
    #[serde(rename = "CLOSED")]
    Closed,
}

#[derive(Debug)]
pub(crate) struct Checkout {
    path: PathBuf,
    head: Option<CommitSha>,
    branch: Option<Branch>,
}

impl Used {
    pub(crate) fn merged(&self) -> Option<PullNumber> {
        self.tool_input.command.0
    }

    pub fn directory(&self) -> &Path {
        &self.cwd
    }
}

impl From<Option<String>> for Merge {
    fn from(command: Option<String>) -> Merge {
        Merge(command.as_deref().and_then(MergeCommand::read).map(|merge| PullNumber::new(merge.pull())))
    }
}

impl Landed {
    pub(crate) fn state(&self) -> State {
        self.state
    }

    pub(crate) fn found(&self, checkouts: impl IntoIterator<Item = Checkout>) -> Option<Checkout> {
        checkouts.into_iter().find(|checkout| checkout.branch.as_ref() == Some(&self.branch))
    }

    pub(crate) fn pushed<'checkout>(&self, checkout: &'checkout Checkout) -> Option<&'checkout Checkout> {
        (checkout.head == Some(self.head)).then_some(checkout)
    }
}

impl From<&Worktree<'_>> for Checkout {
    fn from(listed: &Worktree<'_>) -> Checkout {
        Checkout {
            path: PathBuf::from(listed.path()),
            head: listed.head().and_then(|head| head.parse().ignoring(Ignored::Unheaded)),
            branch: listed.branch().map(Branch::from),
        }
    }
}

impl AsRef<Path> for Checkout {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}
