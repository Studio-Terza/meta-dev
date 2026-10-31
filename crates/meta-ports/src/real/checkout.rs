//! The read-only tree of a PR's head that a review worker reads. The fetch runs in a staged place
//! outside the log's lock; a start then moves the tree into the session's scratch directory.

use super::command::run_prepared;
use super::git_env::scrub;
use crate::PortError;
use meta_core::{AgentId, Origin, Sha};
use std::fs::{self, DirBuilder, Permissions};
use std::io::ErrorKind;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Command;

/// In a staged place: the tree a session reads, and beside it the git data, which stays behind, so
/// that nothing of a remote or a credential can reach the session.
const TREE: &str = "tree";
const GIT: &str = "git";
/// The one ref in the staged git data.
const REVIEWED: &str = "refs/heads/reviewed";

const PRIVATE_DIR: u32 = 0o700;
/// Nobody writes, and a file runs nothing.
const FROZEN_DIR: u32 = 0o500;
const FROZEN_FILE: u32 = 0o400;

fn at(path: &Path, error: std::io::Error) -> PortError {
    PortError::new(format!("{}: {error}", path.display()))
}

/// A new place under `parent`, private, that no other start shares.
fn new_place(parent: &Path, worker: &AgentId) -> Result<PathBuf, PortError> {
    DirBuilder::new().recursive(true).mode(PRIVATE_DIR).create(parent).map_err(|error| at(parent, error))?;
    for attempt in 1u32.. {
        let place = parent.join(format!("{worker}.{attempt}"));
        match DirBuilder::new().mode(PRIVATE_DIR).create(&place) {
            Ok(()) => return Ok(place),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(error) => return Err(at(&place, error)),
        }
    }
    Err(PortError::new(format!("{}: no free name", parent.display())))
}

/// git that never asks: a fetch with no credential to hand fails, and does not wait on a terminal.
fn fetching(command: &mut Command) {
    scrub(command).env("GIT_TERMINAL_PROMPT", "0");
}

/// git that ignores this machine's configuration: nothing here may start a filter, a hook or a
/// monitor while PR content is written. This also leaves `.gitattributes` unable to name a filter.
fn alone(command: &mut Command) {
    scrub(command).env("GIT_CONFIG_GLOBAL", "/dev/null").env("GIT_CONFIG_NOSYSTEM", "1");
}

/// The User-Agent of the fetch: web requests name `basho-trip-planner`.
const USER_AGENT: &str = "http.userAgent=basho-trip-planner";

/// The arguments of the one fetch of `wanted` from `url` into the git data at `git`.
fn fetch_args<'a>(git: &'a str, url: &'a str, wanted: &'a str) -> Vec<&'a str> {
    let depth = ["fetch", "--quiet", "--depth=1", "--no-tags", "--no-recurse-submodules"];
    [&["-c", USER_AGENT, "--git-dir", git][..], &depth[..], &["--", url, wanted]].concat()
}

/// Writes the tree of `head` from `origin` in `place`. The fetch names the origin only on its
/// command line, so no remote is configured, and it keeps the machine's credential helper, which a
/// private origin needs. The commit is asked for by its id, so what comes is that commit or nothing.
/// `core.symlinks=false` writes each symlink as a file that holds its target, so that no link in a
/// PR can reach out of the tree.
fn fetch(place: &Path, origin: &Origin, head: &Sha) -> Result<(), PortError> {
    let (git, tree) = (place.join(GIT), place.join(TREE));
    DirBuilder::new().mode(PRIVATE_DIR).create(&tree).map_err(|error| at(&tree, error))?;
    let (git, tree) = (git.display().to_string(), tree.display().to_string());
    let (url, wanted) = (origin.to_string(), format!("{head}:{REVIEWED}"));
    run_prepared("git", &["init", "--quiet", "--bare", "--template=", &git], alone)?;
    let fetched = fetch_args(&git, &url, &wanted);
    run_prepared("git", &fetched, fetching)?;
    let safe = ["-c", "core.symlinks=false", "-c", "core.hooksPath=/dev/null"];
    let places = ["--git-dir", git.as_str(), "--work-tree", tree.as_str()];
    let written = [&safe[..], &places[..], &["checkout", "--quiet", "--force", "--detach", REVIEWED]].concat();
    run_prepared("git", &written, alone).map(drop)
}

/// A new place under `parent` that holds the tree of `head`. A fetch that fails leaves no place.
pub fn stage(parent: &Path, worker: &AgentId, origin: &Origin, head: &Sha) -> Result<PathBuf, PortError> {
    let place = new_place(parent, worker)?;
    fetch(&place, origin, head).inspect_err(|_| drop(fs::remove_dir_all(&place)))?;
    Ok(place)
}

/// Makes `path` and all under it read-only. A symlink stops it, for a change of mode would reach
/// through the link, and a tree with one in it could lead a session out of the tree.
fn freeze(path: &Path) -> Result<(), PortError> {
    let kind = fs::symlink_metadata(path).map_err(|error| at(path, error))?.file_type();
    if kind.is_symlink() {
        return Err(PortError::new(format!("{}: a symlink in a checkout", path.display())));
    }
    if kind.is_dir() {
        for entry in fs::read_dir(path).map_err(|error| at(path, error))? {
            freeze(&entry.map_err(|error| at(path, error))?.path())?;
        }
    }
    let mode = if kind.is_dir() { FROZEN_DIR } else { FROZEN_FILE };
    fs::set_permissions(path, Permissions::from_mode(mode)).map_err(|error| at(path, error))
}

/// Moves the tree of the staged `place` to `target` and freezes it there. It moves first: a
/// directory that may not be written cannot change its parent.
pub fn take(place: &Path, target: &Path) -> Result<(), PortError> {
    fs::rename(place.join(TREE), target).map_err(|error| at(target, error))?;
    freeze(target)
}

#[cfg(test)]
mod tests {
    use super::{FROZEN_DIR, FROZEN_FILE, fetch_args, freeze, new_place, stage, take};
    use crate::real::command::run;
    use meta_core::{AgentId, Origin, Sha};
    use meta_testkit::Scratch;
    use std::fs;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::path::Path;

    /// A repository to fetch from, and its two commits. The first holds a nested file, an
    /// executable, a symlink and a `.gitattributes` that names a filter; the second adds `later`.
    fn origin(root: &Scratch) -> (Origin, Sha, Sha) {
        let at = root.join("origin").display().to_string();
        run("git", &["init", "--quiet", &at]).unwrap();
        let git = |args: &[&str]| run("git", &[&["-C", at.as_str()], args].concat()).unwrap();
        let who = ["-c", "user.name=meta", "-c", "user.email=meta@example.invalid", "-c", "commit.gpgsign=false"];
        let commit = |message: &str| {
            git(&["add", "."]);
            git(&[&who[..], &["commit", "--quiet", "-m", message]].concat());
            git(&["rev-parse", "HEAD"]).parse::<Sha>().unwrap()
        };
        let tree = Path::new(&at);
        fs::create_dir(tree.join("deep")).unwrap();
        fs::write(tree.join("deep/kept"), "unchanged code\n").unwrap();
        fs::write(tree.join("run.sh"), "#!/bin/sh\n").unwrap();
        fs::set_permissions(tree.join("run.sh"), fs::Permissions::from_mode(0o755)).unwrap();
        fs::write(tree.join(".gitattributes"), "* filter=reach\n").unwrap();
        symlink("/etc/passwd", tree.join("reach")).unwrap();
        let first = commit("first");
        fs::write(tree.join("later"), "later\n").unwrap();
        let second = commit("second");
        (at.parse().unwrap(), first, second)
    }

    fn worker() -> AgentId {
        "worker-1".parse().unwrap()
    }

    #[test]
    fn the_staged_tree_is_that_commit_alone_with_no_git_data_and_no_link() {
        let root = Scratch::new("checkout-tree");
        let (url, first, second) = origin(&root);
        let parent = root.join("staged");
        let place = stage(&parent, &worker(), &url, &first).unwrap();
        let target = root.join("checkout");
        take(&place, &target).unwrap();
        assert_eq!(fs::read_to_string(target.join("deep/kept")).unwrap(), "unchanged code\n");
        assert!(!target.join("later").exists(), "a later commit reached the tree");
        assert!(!target.join(".git").exists(), "git data reached the tree");
        let link = fs::symlink_metadata(target.join("reach")).unwrap();
        assert!(link.is_file(), "a symlink stays a link: {link:?}");
        assert_eq!(fs::read_to_string(target.join("reach")).unwrap(), "/etc/passwd");
        let newer = stage(&parent, &worker(), &url, &second).unwrap();
        assert_ne!(newer, place, "each start stages in a place of its own");
        assert_eq!(fs::read_to_string(newer.join("tree/later")).unwrap(), "later\n");
    }

    #[test]
    fn the_fetch_leaves_no_remote_in_the_git_data_and_only_the_reviewed_commit() {
        let root = Scratch::new("checkout-git");
        let (url, first, _) = origin(&root);
        let place = stage(&root.join("staged"), &worker(), &url, &first).unwrap();
        let git = place.join("git").display().to_string();
        assert_eq!(run("git", &["--git-dir", &git, "remote"]).unwrap(), "");
        let tip = run("git", &["--git-dir", &git, "rev-parse", "refs/heads/reviewed"]).unwrap();
        assert_eq!(tip.parse::<Sha>().unwrap(), first);
        assert_eq!(run("git", &["--git-dir", &git, "rev-list", "--count", "refs/heads/reviewed"]).unwrap(), "1\n");
    }

    #[test]
    fn the_fetch_names_basho_trip_planner_as_its_user_agent() {
        let args = fetch_args("g", "u", "w");
        let at = args.iter().position(|arg| *arg == "http.userAgent=basho-trip-planner").unwrap();
        assert_eq!(args[at - 1], "-c");
        assert!(at < args.iter().position(|arg| *arg == "fetch").unwrap(), "-c must come before the subcommand");
    }

    #[test]
    fn a_taken_tree_cannot_be_written_at_any_depth() {
        let root = Scratch::new("checkout-modes");
        let (url, first, _) = origin(&root);
        let place = stage(&root.join("staged"), &worker(), &url, &first).unwrap();
        let target = root.join("checkout");
        take(&place, &target).unwrap();
        let mode = |path: &str| fs::metadata(target.join(path)).unwrap().permissions().mode() & 0o777;
        assert_eq!([mode(""), mode("deep")], [FROZEN_DIR; 2]);
        assert_eq!([mode("deep/kept"), mode("run.sh"), mode("reach")], [FROZEN_FILE; 3]);
        assert!(fs::write(target.join("deep/kept"), "changed").is_err());
        assert!(fs::write(target.join("deep/new"), "new").is_err());
    }

    #[test]
    fn a_fetch_that_fails_is_an_error_and_leaves_no_place() {
        let root = Scratch::new("checkout-failed");
        let (url, ..) = origin(&root);
        let parent = root.join("staged");
        let absent: Sha = "0123456789012345678901234567890123456789".parse().unwrap();
        let gone: Origin = root.join("nowhere").display().to_string().parse().unwrap();
        assert!(stage(&parent, &worker(), &url, &absent).unwrap_err().to_string().contains("git"));
        assert!(stage(&parent, &worker(), &gone, &absent).is_err());
        assert_eq!(fs::read_dir(&parent).unwrap().count(), 0);
    }

    #[test]
    fn a_symlink_stops_the_freeze_and_its_target_keeps_its_mode() {
        let root = Scratch::new("checkout-link");
        let outside = root.join("outside");
        fs::write(&outside, "mine").unwrap();
        fs::set_permissions(&outside, fs::Permissions::from_mode(0o644)).unwrap();
        let tree = root.join("tree");
        fs::create_dir(&tree).unwrap();
        symlink(&outside, tree.join("link")).unwrap();
        let refused = freeze(&tree).unwrap_err().to_string();
        assert!(refused.contains("a symlink in a checkout"), "{refused}");
        assert_eq!(fs::metadata(&outside).unwrap().permissions().mode() & 0o777, 0o644);
    }

    #[test]
    fn places_for_one_worker_never_collide() {
        let root = Scratch::new("checkout-place");
        let parent = root.join("staged");
        let places = [(); 3].map(|()| new_place(&parent, &worker()).unwrap());
        assert_ne!(places[0], places[1]);
        assert_ne!(places[1], places[2]);
        assert!(places.iter().all(|place| place.is_dir()));
    }
}
