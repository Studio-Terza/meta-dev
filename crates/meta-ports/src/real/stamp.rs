//! The build stamp: what the build script (`build.rs`) embeds about the commit a binary is built
//! from, and the files that make cargo run the script again. The build script includes this file,
//! and the crate's tests include it too, so the logic is tested where it runs.

use std::path::Path;
use std::process::Command;

/// What a checkout stamps on a binary built from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stamp {
    /// A commit with no edits to tracked files.
    Clean(String),
    /// A commit with edits to tracked files: commit or stash them, then rebuild.
    Dirty(String),
    /// No commit could be read: a build outside git, or git failed.
    Unknown(String),
}

/// The git files that change on a commit, a switch or a stage.
const STATE: [&str; 3] = ["HEAD", "index", "logs/HEAD"];

fn git(repo: &Path, args: &[&str]) -> Result<String, String> {
    let shown = format!("git -C {} {}", repo.display(), args.join(" "));
    let output =
        Command::new("git").arg("-C").arg(repo).args(args).output().map_err(|error| format!("{shown}: {error}"))?;
    match output.status.success() {
        true => String::from_utf8(output.stdout).map_err(|error| format!("{shown}: {error}")),
        false => Err(format!("{shown} failed")),
    }
}

impl Stamp {
    /// The stamp of the checkout at `repo`.
    pub fn of(repo: &Path) -> Self {
        let commit = match git(repo, &["log", "-1", "--format=format:%H"]) {
            Ok(commit) => commit,
            Err(why) => return Self::Unknown(why),
        };
        // Only edits to tracked files count: an untracked file changes nothing that is built, and
        // the build script is not rerun when one appears or goes.
        match git(repo, &["status", "--porcelain", "--untracked-files=no"]).map(|status| status.is_empty()) {
            Ok(true) => Self::Clean(commit),
            Ok(false) => Self::Dirty(commit),
            Err(why) => Self::Unknown(why),
        }
    }

    /// The cargo directive that embeds this stamp; `host.rs` reads it with `option_env!`.
    pub fn directive(&self) -> String {
        match self {
            Self::Clean(commit) => format!("cargo::rustc-env=META_BUILD_COMMIT={commit}\n"),
            Self::Dirty(commit) => format!("cargo::rustc-env=META_BUILD_DIRTY={commit}\n"),
            Self::Unknown(why) => format!("cargo::rustc-env=META_BUILD_UNVERIFIED={why}\n"),
        }
    }
}

/// The directives that rerun the build script whenever the stamp could change: one for every
/// tracked file, as git lists it (so no list here can go stale), and one for each git state file.
/// A file that cannot be listed leaves the stamp unknown: fail closed.
fn watched(repo: &Path) -> Result<String, String> {
    // Paths are relative to `repo`, where cargo runs the build script; `:/` lists the whole tree.
    let tracked = git(repo, &["ls-files", "--format=cargo::rerun-if-changed=%(path)", "--", ":/"])?;
    // `git rev-parse --git-path` ends its path with the newline that ends the directive.
    let state = STATE.map(|file| git(repo, &["rev-parse", "--git-path", file]));
    state.into_iter().try_fold(tracked, |directives, path| Ok(directives + "cargo::rerun-if-changed=" + &path?))
}

/// Everything the build script prints for the checkout at `repo`.
pub fn directives(repo: &Path) -> String {
    match watched(repo) {
        Ok(watched) => watched + &Stamp::of(repo).directive(),
        Err(why) => Stamp::Unknown(why).directive(),
    }
}

#[cfg(test)]
mod tests {
    use super::{Stamp, directives, git};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// A fresh repository with `a.txt` and `dir/b.txt` committed, and its head commit.
    fn repo(name: &str) -> (PathBuf, String) {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let repo = std::env::temp_dir().join(format!("meta-dev-stamp-{name}-{nanos}"));
        fs::create_dir_all(repo.join("dir")).unwrap();
        fs::write(repo.join("a.txt"), "a").unwrap();
        fs::write(repo.join("dir").join("b.txt"), "b").unwrap();
        let identity = ["-c", "user.name=meta", "-c", "user.email=meta@example.invalid", "-c", "commit.gpgsign=false"];
        git(&repo, &["init", "--quiet"]).unwrap();
        git(&repo, &["add", "."]).unwrap();
        git(&repo, &[&identity[..], &["commit", "--quiet", "-m", "start"]].concat()).unwrap();
        let head = git(&repo, &["log", "-1", "--format=format:%H"]).unwrap();
        (repo, head)
    }

    #[test]
    fn an_untracked_file_leaves_the_stamp_clean_and_only_a_tracked_edit_makes_it_dirty() {
        let (repo, head) = repo("dirty");
        fs::write(repo.join("stray.txt"), "untracked").unwrap();
        let with_stray = Stamp::of(&repo);
        fs::write(repo.join("a.txt"), "edited").unwrap();
        let with_edit = Stamp::of(&repo);
        fs::remove_dir_all(&repo).unwrap();
        assert_eq!((with_stray, with_edit), (Stamp::Clean(head.clone()), Stamp::Dirty(head)));
    }

    #[test]
    fn the_build_script_reruns_on_every_tracked_file_and_on_git_state_and_not_on_untracked_files() {
        let (repo, head) = repo("watched");
        fs::write(repo.join("stray.txt"), "untracked").unwrap();
        let printed = directives(&repo);
        fs::remove_dir_all(&repo).unwrap();
        let rerun = |path: &str| format!("cargo::rerun-if-changed={path}\n");
        let expected = [rerun("a.txt"), rerun("dir/b.txt"), rerun(".git/HEAD"), rerun(".git/index")];
        assert!(expected.iter().all(|line| printed.contains(line.as_str())), "{printed}");
        assert!(!printed.contains("stray.txt") && !printed.contains("../../"), "{printed}");
        assert!(printed.contains(&format!("META_BUILD_COMMIT={head}\n")), "{printed}");
    }

    #[test]
    fn a_build_outside_git_is_stamped_unknown() {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let gone = std::env::temp_dir().join(format!("meta-dev-stamp-none-{nanos}"));
        assert!(directives(&gone).contains("META_BUILD_UNVERIFIED="));
    }
}
