use super::command::run;
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
}

#[cfg(test)]
mod tests {
    use super::{GitCli, run};
    use meta_core::{Sha, TaskId};
    use std::path::{Path, PathBuf};

    fn repo(name: &str, branch: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("meta-dev-{name}-{nanos}"));
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
}
