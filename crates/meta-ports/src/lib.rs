pub mod memory;
pub mod real;

use meta_core::{Cleared, Malformed, Origin, PrNumber, Pull, Record, SelfFile, Sha, Span, TaskId, Timestamp, Updating};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortError(String);

impl PortError {
    pub fn new(why: impl Into<String>) -> Self {
        Self(why.into())
    }
}

impl fmt::Display for PortError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl From<Malformed> for PortError {
    fn from(malformed: Malformed) -> Self {
        Self(malformed.to_string())
    }
}

pub trait Log {
    fn place(&self) -> String;
    fn read(&self) -> Result<Vec<Record>, PortError>;
    /// Holds the log exclusively from read to append. `decide` gets the records, or why they do not
    /// parse; the records it returns are appended. A log that cannot be opened or locked is an error.
    fn transact<T>(
        &mut self,
        decide: impl FnOnce(Result<Vec<Record>, PortError>) -> (Vec<Record>, T),
    ) -> Result<T, PortError>;
}

pub trait Git {
    fn origin(&self) -> Result<Origin, PortError>;
    /// A data file under `self/` as it stands on main.
    fn read(&self, file: SelfFile) -> Result<String, PortError>;
    /// A data file under `self/` without git or network: the working tree's copy, else the copy
    /// cached by the last `read` from main.
    fn local(&self, file: SelfFile) -> Result<String, PortError>;
    /// The commit at the tip of the local branch named after the task; `None` when there is none.
    fn local_head(&self, task: TaskId) -> Result<Option<Sha>, PortError>;
}

#[derive(Debug, Clone)]
pub struct GhReplies {
    pub view: String,
    pub required: Result<String, PortError>,
    pub runs: String,
}

impl GhReplies {
    pub fn pull(self) -> Result<Pull, PortError> {
        Ok(Pull::from_gh(&self.view, &self.required?, &self.runs)?)
    }
}

pub trait GitHub {
    fn pull(&self, pr: PrNumber) -> Result<Pull, PortError>;
    fn merged_list(&self) -> Result<String, PortError>;
    fn squash_merge(&mut self, cleared: &Cleared) -> Result<(), PortError>;
    /// Merges main into the PR's branch on GitHub, which moves the head; never merges the PR.
    fn update_branch(&mut self, updating: &Updating) -> Result<(), PortError>;
}

pub trait Clock {
    fn now(&self) -> Timestamp;
    /// Waits, so that `now` is later by at least `span`.
    fn pause(&self, span: Span);
}

pub struct Ports<L, G, H, C> {
    pub log: L,
    pub git: G,
    pub github: H,
    pub clock: C,
}
