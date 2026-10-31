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

/// Why this binary may not append to the real log: it is a branch build (a commit not on main,
/// edits to tracked files, or an ancestry that cannot be read), or it carries no commit at all (a
/// build outside git, or a git too old to stamp it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unverified {
    Branch(String),
    Unstamped(String),
}

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
            Stamp::Unknown(why) => Err(Unverified::Unstamped(why.clone())),
            Stamp::Dirty(commit) => Err(Unverified::Branch(format!(
                "built from {commit} with edits to tracked files; to clear it, commit or stash them, then rebuild"
            ))),
            Stamp::Clean(commit) => match on_main(commit) {
                Ok(true) => Ok(()),
                Ok(false) => Err(Unverified::Branch(format!("built from {commit}, which is not on origin/main"))),
                Err(error) => {
                    Err(Unverified::Branch(format!("cannot tell whether {commit} is on origin/main: {error}")))
                }
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
        match self {
            Self::Branch(why) => write!(formatter, "you are running a branch build ({why}); "),
            Self::Unstamped(why) => write!(formatter, "this build carries no commit ({why}); "),
        }?;
        formatter.write_str(
            "only a build from a commit on main may write the real log. \
             Next: use a main build, or set XDG_DATA_HOME to a scratch data dir.",
        )
    }
}

impl From<Unverified> for Refusal {
    fn from(unverified: Unverified) -> Self {
        Self::because(unverified.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::Build;

    #[test]
    fn a_build_git_could_not_stamp_says_why_and_is_not_called_a_branch_build() {
        let why = "git older than 2.38 cannot stamp the build (git ls-files --format failed)";
        let refusal = Build::embedded(None, None, Some(why)).verify(|_| Ok::<_, String>(true)).unwrap_err().to_string();
        assert!(refusal.contains("git older than 2.38 cannot stamp the build"), "{refusal}");
        assert!(!refusal.contains("branch build"), "{refusal}");
    }

    #[test]
    fn an_empty_stamp_variable_counts_as_unset() {
        let commit = "0123456789012345678901234567890123456789";
        let on_main = |_: &crate::ids::Sha| Ok::<_, String>(true);
        let unset = Build::embedded(None, None, None).verify(on_main).unwrap_err().to_string();
        let empty = Build::embedded(Some(""), Some(""), Some("")).verify(on_main).unwrap_err().to_string();
        assert_eq!(empty, unset);
        assert!(empty.contains("a binary with no commit embedded"), "{empty}");
        assert_eq!(Build::embedded(Some(commit), Some(""), Some("")), Build::commit(commit.parse().unwrap()));
        let dirty = Build::embedded(Some(""), Some(commit), Some("")).verify(on_main).unwrap_err().to_string();
        assert!(dirty.contains("with edits to tracked files"), "{dirty}");
        let why = Build::embedded(Some(""), Some(""), Some("no git")).verify(on_main).unwrap_err().to_string();
        assert!(why.contains("no git"), "{why}");
    }
}
