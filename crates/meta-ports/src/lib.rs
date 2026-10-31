pub mod memory;
pub mod real;

use meta_core::{Build, Cleared, Malformed, Marking, Origin, PrNumber, Pull, Record, SelfFile, Sha, TaskId, Timestamp};
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
    /// Whether this is the lead's real log, by its first record (`Head::is_real`): marked, or
    /// unreadable, or the default data dir's log not yet marked. Only a build from a commit on main
    /// may append to it; any other log is scratch and takes any build.
    fn is_real(&self) -> bool;
    /// Puts `marker` first in the log, ahead of the records already there, unless the log already
    /// starts with a marker. Holds the log exclusively while it does.
    fn mark(&mut self, marker: Record) -> Result<Marking, PortError>;
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
    /// Whether `commit` is an ancestor of origin/main, by the local refs of the clone this binary
    /// was built from, with no network.
    fn on_main(&self, commit: &Sha) -> Result<bool, PortError>;
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
}

pub trait Clock {
    fn now(&self) -> Timestamp;
    /// Waits, so that `now` is later by at least the given seconds.
    fn pause(&self, seconds: u64);
}

pub struct Ports<L, G, H, C> {
    pub log: L,
    pub git: G,
    pub github: H,
    pub clock: C,
    /// The commit this binary was built from.
    pub build: Build,
}
