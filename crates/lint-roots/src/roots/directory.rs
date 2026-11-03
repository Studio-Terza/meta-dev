mod ignored;

use std::path::{Path, PathBuf};

use file_ports::{Files, Kind};
use lint_layout::Failure;
use newtype::{Arbitrary, Newtype};

use ignored::Ignored;

#[derive(Newtype, Arbitrary, Debug)]
pub struct Directory(PathBuf);

impl Directory {
    #[must_use]
    pub fn real<'path>(files: &impl Files, path: &'path Path) -> Option<&'path Path> {
        (files.kind(path) == Kind::Directory).then_some(path)
    }

    pub fn entries(&self, files: &impl Files) -> Result<Vec<PathBuf>, Failure> {
        files.entries(&self.0).map_err(|failure| Failure::Io { path: self.0.clone(), source: failure.into() }).map(
            |mut entries| {
                entries.sort();
                entries
            },
        )
    }

    pub fn tree(self, files: &impl Files) -> Result<Vec<Directory>, Failure> {
        let below = self.entries(files)?;
        below
            .into_iter()
            .filter_map(|entry| Self::real(files, &entry).map(Path::to_path_buf))
            .map(Directory)
            .filter(|directory| directory.skipped(files, &Ignored::FOR_PACKAGES).is_none())
            .try_fold(vec![self], |found, directory| Ok(found.into_iter().chain(directory.tree(files)?).collect()))
    }

    pub fn files(self, files: &impl Files) -> Result<Vec<PathBuf>, Failure> {
        self.entries(files)?.into_iter().try_fold(Vec::new(), |found, entry| {
            let below = Directory(entry.clone());
            Ok(match (Self::real(files, &entry), below.skipped(files, &Ignored::FOR_FILES)) {
                (None, _) => found.into_iter().chain([entry]).collect(),
                (Some(_), Some(_)) => found,
                (Some(_), None) => found.into_iter().chain(below.files(files)?).collect(),
            })
        })
    }

    fn skipped<'rules>(&self, files: &impl Files, ignored: &'rules [Ignored]) -> Option<&'rules Ignored> {
        ignored.iter().find(|rule| rule.applying_to(files, self).is_some())
    }
}

impl AsRef<Path> for Directory {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
