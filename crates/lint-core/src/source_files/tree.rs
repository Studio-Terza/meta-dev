use std::fs;
use std::path::{Path, PathBuf};

use crate::failure::Failure;

#[derive(Debug)]
pub struct Tree {
    canonical: PathBuf,
    given: PathBuf,
}

impl Tree {
    pub fn open(given: &Path) -> Result<Tree, Failure> {
        let canonical = fs::canonicalize(given).map_err(|source| Failure::Io { path: given.to_path_buf(), source })?;
        Ok(Tree { canonical, given: given.to_path_buf() })
    }

    #[must_use]
    pub fn canonical(&self) -> &Path {
        &self.canonical
    }

    #[must_use]
    pub fn without_sources(&self) -> Failure {
        Failure::NoSources { path: self.given.clone() }
    }

    #[must_use]
    pub fn display(&self, file: &Path) -> PathBuf {
        match file.strip_prefix(&self.canonical) {
            Ok(inside) if self.given == Path::new(".") => inside.to_path_buf(),
            Ok(inside) => self.given.join(inside),
            Err(_) => file.to_path_buf(),
        }
    }
}
