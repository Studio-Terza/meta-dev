use super::command::run;
use super::log::home;
use crate::{Git, PortError};
use meta_core::{Origin, SelfFile};
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
        Self::file(Path::new(file.path())).or_else(|tree| {
            let cached = Self::cached(file).and_then(|path| Self::file(&path));
            cached.map_err(|cache| PortError::new(format!("working tree: {tree}; cache: {cache}")))
        })
    }
}
