use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use file_ports::{FileFailure, Files, Kind};
use lint_layout::Failure;

#[derive(Clone, Copy, Debug)]
pub enum Fact {
    Kind,
    Canonical,
    Followed,
    Listed,
}

#[derive(Debug, Default)]
pub struct Survey {
    kinds: BTreeMap<PathBuf, Kind>,
    canonical: BTreeMap<PathBuf, PathBuf>,
    listed: BTreeMap<PathBuf, Vec<PathBuf>>,
    failed: Vec<Failure>,
}

impl Survey {
    #[must_use]
    pub fn learn(self, files: &impl Files, fact: Fact, paths: &[PathBuf]) -> Survey {
        match fact {
            Fact::Kind => {
                let found = files.kinds(paths).into_iter().map(Some).collect();
                Survey { kinds: Self::extended(self.kinds, paths, found), ..self }
            }
            Fact::Canonical => {
                let found = files.canonical_many(paths).into_iter().map(Result::ok).collect();
                Survey { canonical: Self::extended(self.canonical, paths, found), ..self }
            }
            Fact::Followed => {
                let survey = self.learn(files, Fact::Canonical, paths);
                let reals: Vec<PathBuf> =
                    paths.iter().filter_map(|path| survey.canonical(path)).map(Path::to_path_buf).collect();
                survey.learn(files, Fact::Kind, &reals)
            }
            Fact::Listed => paths
                .iter()
                .cloned()
                .zip(files.entries_many(paths))
                .fold(self, |survey, (directory, listing)| survey.listing(directory, listing)),
        }
    }

    #[must_use]
    pub fn kind(&self, path: &Path) -> Kind {
        self.kinds.get(path).copied().unwrap_or(Kind::Missing)
    }

    #[must_use]
    pub fn canonical(&self, path: &Path) -> Option<&Path> {
        self.canonical.get(path).map(PathBuf::as_path)
    }

    #[must_use]
    pub fn file<'path>(&self, path: &'path Path) -> Option<&'path Path> {
        self.canonical(path).filter(|real| self.kind(real) == Kind::File).map(|_| path)
    }

    #[must_use]
    pub fn entries(&self, directory: &Path) -> &[PathBuf] {
        self.listed.get(directory).map_or(&[], Vec::as_slice)
    }

    pub fn failure(&mut self, order: &[impl AsRef<Path>]) -> Option<Failure> {
        let latest = order.iter().map(AsRef::as_ref).find_map(|directory| {
            self.failed.iter().rposition(|failure| matches!(failure, Failure::Io { path, .. } if path == directory))
        })?;
        Some(self.failed.remove(latest))
    }

    fn extended<Value>(
        mut found: BTreeMap<PathBuf, Value>,
        paths: &[PathBuf],
        values: Vec<Option<Value>>,
    ) -> BTreeMap<PathBuf, Value> {
        found.extend(paths.iter().cloned().zip(values).filter_map(|(path, value)| Some((path, value?))));
        found
    }

    fn listing(mut self, directory: PathBuf, listing: Result<Vec<PathBuf>, FileFailure>) -> Survey {
        match listing {
            Ok(mut entries) => {
                entries.sort();
                self.listed.insert(directory, entries);
                self
            }
            Err(failure) => {
                self.failed.push(Failure::Io { path: directory, source: failure.into() });
                self
            }
        }
    }
}
