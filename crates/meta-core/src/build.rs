use crate::head::Standing;
use crate::ids::Sha;
use crate::reply::Refusal;
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

/// Why this binary may not append to the log it reached, and the log's standing, which decides the
/// way out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unverified {
    why: Why,
    standing: Standing,
    aim: Aim,
}

/// What the build was refused: appending to the log, or marking it as the real log. The way out
/// differs: a scratch data dir leads away from appending to a real log, never from marking one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Aim {
    Append,
    Mark,
}

/// A branch build (a commit not on main, edits to tracked files, or an ancestry that cannot be
/// read), or a build with no commit at all (built outside git, or by a git too old to stamp it).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Why {
    Branch(String),
    Unstamped(String),
}

impl Build {
    pub fn commit(commit: Sha) -> Self {
        Self(Stamp::Clean(commit))
    }

    /// From what the build script embedded: the commit of a clean checkout, else the commit under
    /// edits to tracked files, else why there is none. The script sets all three and leaves the
    /// unused ones empty, so an empty one counts as unset.
    pub fn embedded(clean: Option<&str>, dirty: Option<&str>, why: Option<&str>) -> Self {
        let parsed = |commit: Option<&str>| commit.and_then(|commit| commit.parse().ok());
        let why = why.filter(|why| !why.is_empty());
        match (parsed(clean), parsed(dirty)) {
            (Some(commit), _) => Self(Stamp::Clean(commit)),
            (None, Some(commit)) => Self(Stamp::Dirty(commit)),
            (None, None) => Self(Stamp::Unknown(why.unwrap_or("a binary with no commit embedded").to_owned())),
        }
    }

    /// Whether this build may append to a log of this standing. Any build may write a scratch log.
    /// Any other takes a commit that is an ancestor of origin/main; a build with no commit, or an
    /// ancestry that cannot be decided, is unverified: fail closed.
    pub fn verify<E: fmt::Display>(
        &self,
        standing: Standing,
        on_main: impl FnOnce(&Sha) -> Result<bool, E>,
    ) -> Result<(), Unverified> {
        let why = match (standing, &self.0) {
            (Standing::Scratch, _) => return Ok(()),
            (_, Stamp::Unknown(why)) => Why::Unstamped(why.clone()),
            (_, Stamp::Dirty(commit)) => Why::Branch(format!(
                "built from {commit} with edits to tracked files; to clear it, commit or stash them, then rebuild"
            )),
            (_, Stamp::Clean(commit)) => match on_main(commit) {
                Ok(true) => return Ok(()),
                Ok(false) => Why::Branch(format!("built from {commit}, which is not on origin/main")),
                Err(error) => Why::Branch(format!("cannot tell whether {commit} is on origin/main: {error}")),
            },
        };
        Err(Unverified { why, standing, aim: Aim::Append })
    }

    /// Whether this build may mark a log as the real log: only a build from a commit on main may,
    /// whatever the log's standing now.
    pub fn verify_marking<E: fmt::Display>(
        &self,
        on_main: impl FnOnce(&Sha) -> Result<bool, E>,
    ) -> Result<(), Unverified> {
        self.verify(Standing::Real, on_main).map_err(|unverified| Unverified { aim: Aim::Mark, ..unverified })
    }
}

impl Unverified {
    /// The same refusal for a log that turned out to stand so.
    #[must_use]
    pub fn at(self, standing: Standing) -> Self {
        Self { standing, ..self }
    }
}

impl fmt::Display for Unverified {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.why {
            Why::Branch(why) => write!(formatter, "you are running a branch build ({why}); "),
            Why::Unstamped(why) => write!(formatter, "this build carries no commit ({why}); "),
        }?;
        formatter.write_str(match (self.aim, self.standing) {
            (Aim::Mark, _) => {
                "only a build from a commit on main may mark a log as the real log. Next: use a main build."
            }
            (Aim::Append, Standing::MarkedElsewhere) => "this log is marked as the real log; use a main build.",
            (Aim::Append, Standing::Scratch | Standing::AwaitingMark | Standing::Real) => {
                "only a build from a commit on main may write the real log. \
                 Next: use a main build, or set XDG_DATA_HOME to a scratch data dir."
            }
        })
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
    use crate::head::Standing;

    #[test]
    fn a_build_git_could_not_stamp_says_why_and_is_not_called_a_branch_build() {
        let why = "git older than 2.38 cannot stamp the build (git ls-files --format failed)";
        let refusal = Build::embedded(None, None, Some(why))
            .verify(Standing::Real, |_| Ok::<_, String>(true))
            .unwrap_err()
            .to_string();
        assert!(refusal.contains("git older than 2.38 cannot stamp the build"), "{refusal}");
        assert!(!refusal.contains("branch build"), "{refusal}");
    }

    #[test]
    fn an_empty_stamp_variable_counts_as_unset() {
        let commit = "0123456789012345678901234567890123456789";
        let refused = |build: Build| build.verify(Standing::Real, |_| Ok::<_, String>(true)).unwrap_err().to_string();
        let unset = refused(Build::embedded(None, None, None));
        let empty = refused(Build::embedded(Some(""), Some(""), Some("")));
        assert_eq!(empty, unset);
        assert!(empty.contains("a binary with no commit embedded"), "{empty}");
        assert_eq!(Build::embedded(Some(commit), Some(""), Some("")), Build::commit(commit.parse().unwrap()));
        let dirty = refused(Build::embedded(Some(""), Some(commit), Some("")));
        assert!(dirty.contains("with edits to tracked files"), "{dirty}");
        let why = refused(Build::embedded(Some(""), Some(""), Some("no git")));
        assert!(why.contains("no git"), "{why}");
    }
}
