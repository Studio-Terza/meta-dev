use crate::ids::Sha;
use crate::reply::Refusal;
use serde::{Deserialize, Serialize};
use std::fmt;

/// The commit a binary was built from, embedded at build time, or why it has none: edits to
/// tracked files, or a build outside git.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Build(Stamp);

/// What the build script found: a clean commit, a commit with edits to tracked files, or no commit.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Stamp {
    Clean(Sha),
    Dirty(Sha),
    Unknown(String),
}

/// Why this binary may not append to the real log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unverified(String);

/// Who wrote a record: a build that may write the log, or a build that could not be verified to be
/// from main. Only escapes are ever written by an unverified build.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Writer {
    #[default]
    Verified,
    Unverified,
}

impl Build {
    pub fn commit(commit: Sha) -> Self {
        Self(Stamp::Clean(commit))
    }

    /// From what the build script embedded: the commit of a clean checkout, else the commit under
    /// edits to tracked files, else why there is none.
    pub fn embedded(clean: Option<&str>, dirty: Option<&str>, why: Option<&str>) -> Self {
        let parsed = |commit: Option<&str>| commit.and_then(|commit| commit.parse().ok());
        match (parsed(clean), parsed(dirty)) {
            (Some(commit), _) => Self(Stamp::Clean(commit)),
            (None, Some(commit)) => Self(Stamp::Dirty(commit)),
            (None, None) => Self(Stamp::Unknown(why.unwrap_or("a binary with no commit embedded").to_owned())),
        }
    }

    /// Verified only when the commit is an ancestor of origin/main. A build with no commit, or an
    /// ancestry that cannot be decided, is unverified: fail closed.
    pub fn verify<E: fmt::Display>(&self, on_main: impl FnOnce(&Sha) -> Result<bool, E>) -> Result<(), Unverified> {
        match &self.0 {
            Stamp::Unknown(why) => Err(Unverified(format!("built from {why}"))),
            Stamp::Dirty(commit) => Err(Unverified(format!("built from a dirty working tree at {commit}"))),
            Stamp::Clean(commit) => match on_main(commit) {
                Ok(true) => Ok(()),
                Ok(false) => Err(Unverified(format!("built from {commit}, which is not on origin/main"))),
                Err(error) => Err(Unverified(format!("cannot tell whether {commit} is on origin/main: {error}"))),
            },
        }
    }
}

impl Writer {
    pub fn is_verified(&self) -> bool {
        *self == Self::Verified
    }

    pub(crate) fn of(unverified: Option<&Unverified>) -> Self {
        unverified.map_or(Self::Verified, |_| Self::Unverified)
    }
}

impl fmt::Display for Unverified {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "you are running a branch build ({}); only a build from a commit on main may write the real log. \
             Next: use a main build, or set XDG_DATA_HOME to a scratch data dir.",
            self.0
        )
    }
}

impl From<Unverified> for Refusal {
    fn from(unverified: Unverified) -> Self {
        Self::because(unverified.to_string())
    }
}
