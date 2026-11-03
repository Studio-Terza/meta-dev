use std::path::{Path, PathBuf};

use file_ports::Files;
use lint_layout::Failure;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tree<'given> {
    canonical: PathBuf,
    given: &'given Path,
}

impl<'given> Tree<'given> {
    pub fn open_many(files: &impl Files, given: &'given [PathBuf]) -> Vec<Result<Tree<'given>, Failure>> {
        given
            .iter()
            .zip(files.canonical_many(given))
            .map(|(given, canonical)| {
                canonical
                    .map(|canonical| Tree { canonical, given })
                    .map_err(|failure| Failure::Io { path: given.clone(), source: failure.into() })
            })
            .collect()
    }

    #[must_use]
    pub fn canonical(&self) -> &Path {
        &self.canonical
    }

    #[must_use]
    pub fn without_sources(&self) -> Failure {
        Failure::NoSources { path: self.given.to_path_buf() }
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
