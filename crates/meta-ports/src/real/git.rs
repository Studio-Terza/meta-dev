use super::command::{decides_with, run, run_found};
use super::git_env::scrub;
use super::log::home;
use crate::{Git, PortError};
use meta_core::{Binary, Origin, SelfFile, Sha, TaskId};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy)]
pub struct GitCli;

impl GitCli {
    fn cached(file: SelfFile) -> Result<PathBuf, PortError> {
        home().map(|home| home.join("cache").join(file.path()))
    }

    /// Keeps the copy from main for `local`; a cache that cannot be written only goes stale.
    fn cache(file: SelfFile, text: &str) {
        let written = Self::cached(file).and_then(|path| {
            let parent = path.parent().map_or(Ok(()), fs::create_dir_all);
            parent.and_then(|()| fs::write(&path, text)).map_err(|error| PortError::new(error.to_string()))
        });
        drop(written);
    }

    fn file(path: &Path) -> Result<String, PortError> {
        fs::read_to_string(path).map_err(|error| PortError::new(format!("{}: {error}", path.display())))
    }
}

impl GitCli {
    /// The file from the working tree at `tree`, else from the cache. A working-tree file that
    /// exists but does not parse gives way to a cache that does; when neither parses, the tree's
    /// text is returned so that the caller reports why it is unreadable.
    pub fn local_from(file: SelfFile, tree: &Path, cache: Result<PathBuf, PortError>) -> Result<String, PortError> {
        let cached = || cache.as_ref().map_err(Clone::clone).and_then(|root| Self::file(&root.join(file.path())));
        match Self::file(&tree.join(file.path())) {
            Ok(text) if file.is_readable(&text) => Ok(text),
            Ok(text) => Ok(cached().ok().filter(|kept| file.is_readable(kept)).unwrap_or(text)),
            Err(tree) => cached().map_err(|cache| PortError::new(format!("working tree: {tree}; cache: {cache}"))),
        }
    }
}

impl GitCli {
    /// The clone this binary was built from: the one whose origin/main decides whether it is a main build.
    const BUILT_IN: &str = env!("CARGO_MANIFEST_DIR");

    /// Whether `commit` is an ancestor of origin/main in the clone at `repo`, by its local refs. A
    /// binary must stay in the clone it was built in; when that clone is gone it cannot answer.
    pub fn ancestry(repo: &Path, commit: &Sha) -> Result<bool, PortError> {
        Self::ancestry_inheriting(repo, commit, &[])
    }

    /// `ancestry` as a caller whose environment holds `inherited` gets it.
    fn ancestry_inheriting(repo: &Path, commit: &Sha, inherited: &[(&str, &str)]) -> Result<bool, PortError> {
        if !repo.is_dir() {
            return Err(PortError::new(format!(
                "the clone this binary was built in, {}, has moved or been removed; rebuild meta in a clone that stays put",
                repo.display()
            )));
        }
        let repo = repo.display().to_string();
        let commit = commit.to_string();
        let args = ["-C", &repo, "merge-base", "--is-ancestor", &commit, "refs/remotes/origin/main"];
        decides_with("git", &args, |command| {
            command.envs(inherited.iter().copied());
            scrub(command);
        })
    }

    /// The tip of exactly the branch `refs/heads/<task>` in the repository at `repo`; a branch that
    /// only shares the name as a prefix, such as `<task>/extra`, is not it.
    pub fn head_in(repo: &Path, task: TaskId) -> Result<Option<Sha>, PortError> {
        let at = repo.to_string_lossy();
        let branch = format!("refs/heads/{task}");
        run_found("git", &["-C", &at, "rev-parse", "--verify", "--quiet", &branch])?
            .map(|found| found.parse().map_err(|error| PortError::new(format!("branch {task}: {error}"))))
            .transpose()
    }
}

impl Git for GitCli {
    fn binary(&self) -> Result<Binary, PortError> {
        std::env::current_exe()
            .map_err(|error| PortError::new(format!("binary: {error}")))?
            .to_string_lossy()
            .parse()
            .map_err(|error| PortError::new(format!("binary: {error}")))
    }

    fn origin(&self) -> Result<Origin, PortError> {
        run("git", &["remote", "get-url", "origin"])?
            .parse()
            .map_err(|error| PortError::new(format!("origin: {error}")))
    }

    fn read(&self, file: SelfFile) -> Result<String, PortError> {
        run("git", &["fetch", "--quiet", "origin", "main"])?;
        let text = run("git", &["show", &format!("FETCH_HEAD:{}", file.path())])?;
        Self::cache(file, &text);
        Ok(text)
    }

    fn local(&self, file: SelfFile) -> Result<String, PortError> {
        Self::local_from(file, Path::new(""), home().map(|home| home.join("cache")))
    }

    fn local_head(&self, task: TaskId) -> Result<Option<Sha>, PortError> {
        Self::head_in(Path::new("."), task)
    }

    fn on_main(&self, commit: &Sha) -> Result<bool, PortError> {
        Self::ancestry(Path::new(Self::BUILT_IN), commit)
    }
}

#[cfg(test)]
mod tests {
    use super::{GitCli, run};
    use meta_core::{Sha, TaskId};
    use meta_testkit::Scratch;
    use std::path::Path;

    fn repo(name: &str, branch: &str) -> Scratch {
        let root = Scratch::new(name);
        let at = root.to_string_lossy().into_owned();
        let git = |args: &[&str]| run("git", &[&["-C", &at], args].concat()).unwrap();
        run("git", &["init", "--quiet", &at]).unwrap();
        let who = ["-c", "user.name=Cinglein", "-c", "user.email=64114772+Cinglein@users.noreply.github.com"];
        git(&[&who[..], &["commit", "--quiet", "--allow-empty", "-m", "m"]].concat());
        git(&["branch", branch]);
        root
    }

    fn head(root: &Path) -> Option<Sha> {
        let task: TaskId = "task-1".parse().unwrap();
        GitCli::head_in(root, task).unwrap()
    }

    #[test]
    fn local_head_finds_exactly_the_branch_named_after_the_task() {
        let found = repo("exact", "task-1");
        let tip = run("git", &["-C", &found.to_string_lossy(), "rev-parse", "task-1"]).unwrap();
        assert_eq!(head(&found), Some(tip.parse().unwrap()));
    }

    #[test]
    fn local_head_is_none_for_a_branch_that_only_shares_the_prefix() {
        assert_eq!(head(&repo("nested", "task-1/extra")), None);
        assert_eq!(head(&repo("longer", "task-10")), None);
    }

    #[test]
    fn a_commit_is_on_main_only_when_origin_main_contains_it_and_an_unknown_one_is_an_error() {
        let repo = Scratch::new("ancestry");
        let at = repo.display().to_string();
        let git = |args: &[&str]| run("git", &[&["-C", at.as_str()], args].concat()).unwrap();
        git(&["init", "--quiet"]);
        let commit = |message: &str| {
            let identity =
                ["-c", "user.name=meta", "-c", "user.email=meta@example.invalid", "-c", "commit.gpgsign=false"];
            git(&[&identity[..], &["commit", "--quiet", "--allow-empty", "-m", message]].concat());
            git(&["log", "-1", "--format=format:%H"]).parse::<Sha>().unwrap()
        };
        let main = commit("main");
        git(&["update-ref", "refs/remotes/origin/main", &main.to_string()]);
        let branch = commit("branch");
        let unknown: Sha = "0123456789012345678901234567890123456789".parse().unwrap();
        let answers = [&main, &branch, &unknown].map(|sha| GitCli::ancestry(&repo, sha));
        assert_eq!(answers[..2], [Ok(true), Ok(false)]);
        assert!(answers[2].is_err(), "{:?}", answers[2]);
    }

    #[test]
    fn the_ancestry_check_ignores_variables_inherited_from_the_caller() {
        let repo = Scratch::new("ancestry-env");
        let other = Scratch::new("ancestry-env-other");
        let at = repo.display().to_string();
        let git = |args: &[&str]| run("git", &[&["-C", at.as_str()], args].concat()).unwrap();
        git(&["init", "--quiet"]);
        let identity = ["-c", "user.name=meta", "-c", "user.email=meta@example.invalid", "-c", "commit.gpgsign=false"];
        git(&[&identity[..], &["commit", "--quiet", "--allow-empty", "-m", "main"]].concat());
        let main = git(&["log", "-1", "--format=format:%H"]).parse::<Sha>().unwrap();
        git(&["update-ref", "refs/remotes/origin/main", &main.to_string()]);
        run("git", &["init", "--quiet", &other.display().to_string()]).unwrap();
        let elsewhere = other.join(".git").display().to_string();
        let inherited = [("GIT_DIR", elsewhere.as_str()), ("GIT_WORK_TREE", elsewhere.as_str())];
        assert_eq!(GitCli::ancestry_inheriting(&repo, &main, &inherited), Ok(true));
    }

    #[test]
    fn a_build_whose_clone_moved_or_was_removed_cannot_answer_and_says_so() {
        let gone = Scratch::new("moved").join("gone");
        let commit: Sha = "0123456789012345678901234567890123456789".parse().unwrap();
        let said = GitCli::ancestry(&gone, &commit).unwrap_err().to_string();
        let expected = format!("the clone this binary was built in, {}, has moved or been removed", gone.display());
        assert!(said.contains(&expected), "{said}");
    }
}
