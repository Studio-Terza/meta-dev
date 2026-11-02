mod ignored;

use std::io;
use std::path::{Path, PathBuf};

use lint_layout::Failure;
use newtype::Newtype;

use ignored::Ignored;

#[derive(Newtype, Debug)]
pub struct Directory(PathBuf);

impl Directory {
    #[must_use]
    pub fn is_real(path: &Path) -> bool {
        std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_dir())
    }

    pub fn entries(&self) -> Result<Vec<PathBuf>, Failure> {
        let failed = |source: io::Error| Failure::Io { path: self.0.clone(), source };
        std::fs::read_dir(&self.0)
            .map_err(failed)?
            .map(|entry| entry.map(|found| found.path()).map_err(failed))
            .collect::<Result<Vec<PathBuf>, Failure>>()
            .map(|mut entries| {
                entries.sort();
                entries
            })
    }

    pub fn tree(self) -> Result<Vec<Directory>, Failure> {
        let below = self.entries()?;
        below
            .into_iter()
            .filter(|entry| Self::is_real(entry))
            .map(Directory)
            .filter(|directory| !directory.is_skipped(&Ignored::FOR_PACKAGES))
            .try_fold(vec![self], |found, directory| Ok(found.into_iter().chain(directory.tree()?).collect()))
    }

    pub fn files(self) -> Result<Vec<PathBuf>, Failure> {
        self.entries()?.into_iter().try_fold(Vec::new(), |found, entry| {
            let below = Directory(entry.clone());
            Ok(if !Self::is_real(&entry) {
                found.into_iter().chain([entry]).collect()
            } else if below.is_skipped(&Ignored::FOR_FILES) {
                found
            } else {
                found.into_iter().chain(below.files()?).collect()
            })
        })
    }

    fn is_skipped(&self, ignored: &[Ignored]) -> bool {
        ignored.iter().any(|rule| rule.applying_to(self).is_some())
    }
}

impl AsRef<Path> for Directory {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
