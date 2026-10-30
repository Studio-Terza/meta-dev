use crate::ids::Sha;
use crate::reply::Refusal;
use serde::{Deserialize, Serialize};
use std::fmt;

/// The commit a binary was built from, embedded at build time, or why it has none: a dirty
/// working tree, or a build outside git.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Build(Result<Sha, String>);

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
        Self(Ok(commit))
    }

    /// From what the build script embedded: the commit of a clean checkout, else why there is none.
    pub fn embedded(commit: Option<&str>, why: Option<&str>) -> Self {
        match commit.map(str::parse) {
            Some(Ok(commit)) => Self(Ok(commit)),
            _ => Self(Err(why.unwrap_or("a binary with no commit embedded").to_owned())),
        }
    }

    /// Verified only when the commit is an ancestor of origin/main. A build with no commit, or an
    /// ancestry that cannot be decided, is unverified: fail closed.
    pub fn verify<E: fmt::Display>(&self, on_main: impl FnOnce(&Sha) -> Result<bool, E>) -> Result<(), Unverified> {
        match &self.0 {
            Err(why) => Err(Unverified(format!("built from {why}"))),
            Ok(commit) => match on_main(commit) {
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
