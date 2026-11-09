use std::path::{Path, PathBuf};

use file_ports::Files;
use lint_layout::Failure;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tree<'given> {
    canonical: PathBuf,
    given: &'given Path,
}

impl<'given> Tree<'given> {
    const CURRENT: &'static str = ".";

    pub fn open_many(files: &impl Files, given: &'given [PathBuf]) -> Vec<Result<Tree<'given>, Failure>> {
        given
            .iter()
            .zip(files.canonical_many(given))
            .map(|(path, resolved)| {
                resolved
                    .map(|canonical| Tree { canonical, given: path })
                    .map_err(|failure| Failure::Io { path: path.clone(), source: failure.into() })
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
        let shown = Some(self.given).filter(|given| *given != Path::new(Self::CURRENT));
        file.strip_prefix(&self.canonical).map_or_else(
            |_| file.to_path_buf(),
            |inside| shown.map_or_else(|| inside.to_path_buf(), |given| given.join(inside)),
        )
    }
}
