use super::command::{decides, run};
use super::log::home;
use crate::{Git, PortError};
use meta_core::{Origin, SelfFile, Sha, TaskId};
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
        if !repo.is_dir() {
            return Err(PortError::new(format!(
                "the clone this binary was built in, {}, has moved or been removed; rebuild meta in a clone that stays put",
                repo.display()
            )));
        }
        let repo = repo.display().to_string();
        let commit = commit.to_string();
        decides("git", &["-C", &repo, "merge-base", "--is-ancestor", &commit, "refs/remotes/origin/main"])
    }
}

impl Git for GitCli {
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
        let found = run("git", &["for-each-ref", "--format=%(objectname)", &format!("refs/heads/{task}")])?;
        match found.is_empty() {
            true => Ok(None),
            false => found.parse().map(Some).map_err(|error| PortError::new(format!("branch {task}: {error}"))),
        }
    }

    fn on_main(&self, commit: &Sha) -> Result<bool, PortError> {
        Self::ancestry(Path::new(Self::BUILT_IN), commit)
    }
}

#[cfg(test)]
mod tests {
    use super::GitCli;
    use super::run;
    use meta_core::Sha;
    use std::path::Path;

    #[test]
    fn a_commit_is_on_main_only_when_origin_main_contains_it_and_an_unknown_one_is_an_error() {
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let repo = std::env::temp_dir().join(format!("meta-dev-ancestry-{nanos}"));
        let at = repo.display().to_string();
        let git = |args: &[&str]| run("git", &[&["-C", at.as_str()], args].concat()).unwrap();
        std::fs::create_dir_all(&repo).unwrap();
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
        let answers = [&main, &branch, &unknown].map(|sha| GitCli::ancestry(Path::new(&repo), sha));
        std::fs::remove_dir_all(&repo).unwrap();
        assert_eq!(answers[..2], [Ok(true), Ok(false)]);
        assert!(answers[2].is_err(), "{:?}", answers[2]);
    }

    #[test]
    fn a_build_whose_clone_moved_or_was_removed_cannot_answer_and_says_so() {
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let gone = std::env::temp_dir().join(format!("meta-dev-moved-{nanos}"));
        let commit: Sha = "0123456789012345678901234567890123456789".parse().unwrap();
        let said = GitCli::ancestry(&gone, &commit).unwrap_err().to_string();
        let expected = format!("the clone this binary was built in, {}, has moved or been removed", gone.display());
        assert!(said.contains(&expected), "{said}");
    }
}
