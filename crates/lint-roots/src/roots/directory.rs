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
    pub fn real(path: &Path) -> Option<&Path> {
        std::fs::symlink_metadata(path).ok().filter(std::fs::Metadata::is_dir).map(|_| path)
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
            .filter_map(|entry| Self::real(&entry).map(Path::to_path_buf))
            .map(Directory)
            .filter(|directory| directory.skipped(&Ignored::FOR_PACKAGES).is_none())
            .try_fold(vec![self], |found, directory| Ok(found.into_iter().chain(directory.tree()?).collect()))
    }

    pub fn files(self) -> Result<Vec<PathBuf>, Failure> {
        self.entries()?.into_iter().try_fold(Vec::new(), |found, entry| {
            let below = Directory(entry.clone());
            Ok(match (Self::real(&entry), below.skipped(&Ignored::FOR_FILES)) {
                (None, _) => found.into_iter().chain([entry]).collect(),
                (Some(_), Some(_)) => found,
                (Some(_), None) => found.into_iter().chain(below.files()?).collect(),
            })
        })
    }

    fn skipped<'rules>(&self, ignored: &'rules [Ignored]) -> Option<&'rules Ignored> {
        ignored.iter().find(|rule| rule.applying_to(self).is_some())
    }
}

impl AsRef<Path> for Directory {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
