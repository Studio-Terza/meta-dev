pub mod memory;
pub mod real;

use meta_core::{
    Build, Cleared, Malformed, Marking, Origin, PrNumber, Pull, Record, SelfFile, Sha, SideLine, Span, Standing,
    TaskId, Timestamp, UnverifiedEscape, UnverifiedEscapes, Updating,
};
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
    /// What this log is, by its first record (`Head::standing`): real when marked, unreadable, or
    /// the default data dir's log not yet marked. Only a build from a commit on main may append to
    /// a real log; any other log is scratch and takes any build.
    fn standing(&self) -> Standing;
    /// Puts `marker` first in the log, ahead of the records already there, unless the log already
    /// starts with a marker. Holds the log exclusively while it does.
    fn mark(&mut self, marker: Record) -> Result<Marking, PortError>;
    fn read(&self) -> Result<Vec<Record>, PortError>;
    /// Appends an escape from a build not verified to be from main to the side file beside the
    /// log, leaving the log untouched, and says where it went.
    fn set_aside(&mut self, escape: &UnverifiedEscape) -> Result<String, PortError>;
    /// What the side file holds, read leniently; no side file holds nothing. An error means the
    /// file as a whole could not be read.
    fn set_aside_escapes(&self) -> Result<UnverifiedEscapes, PortError>;
    /// Moves the side file's escapes into the log, holding the log exclusively throughout. `decide`
    /// gets the standing as read inside the lock, the records, and the side file's lines as written
    /// (no side file has none). The records it returns are appended and made durable first; only
    /// then is the side file rewritten, by rename, to the lines it returns, so that a crash between
    /// the two leaves every line in the side file and the log to recognise it by (`Importing`). It
    /// returns `None` for the lines to leave the side file untouched.
    fn import_set_aside<T>(
        &mut self,
        decide: impl FnOnce(
            Standing,
            Result<Vec<Record>, PortError>,
            Vec<SideLine>,
        ) -> (Vec<Record>, Option<Vec<SideLine>>, T),
    ) -> Result<T, PortError>;
    /// Holds the log exclusively from read to append. `decide` gets the log's standing as read
    /// inside the lock (from its first record, so a marking that landed after `standing` was read
    /// shows), and the records, or why they do not parse; the records it returns are appended. A
    /// log that cannot be opened or locked is an error.
    fn transact<T>(
        &mut self,
        decide: impl FnOnce(Standing, Result<Vec<Record>, PortError>) -> (Vec<Record>, T),
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
    /// The one `gh pr list --state open` reply, with the number, base and commits of each PR.
    fn open_list(&self) -> Result<String, PortError>;
    /// The `gh run list --json databaseId` reply for the PR's branch, merged or not.
    fn runs(&self, pr: PrNumber) -> Result<String, PortError>;
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
    /// The commit this binary was built from.
    pub build: Build,
}
