use std::collections::BTreeMap;
use std::iter;
use std::ops::Deref;
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

#[derive(Clone, Copy, Debug)]
pub struct Settled<'survey>(&'survey Survey);

impl Deref for Settled<'_> {
    type Target = Survey;

    fn deref(&self) -> &Survey {
        self.0
    }
}

impl Survey {
    #[must_use]
    pub fn learn(self, files: &impl Files, fact: Fact, paths: &[PathBuf]) -> Self {
        match fact {
            Fact::Kind => Self {
                kinds: Self::extended(self.kinds, paths, files.kinds(paths).into_iter().map(Some).collect()),
                ..self
            },
            Fact::Canonical => Self {
                canonical: Self::extended(
                    self.canonical,
                    paths,
                    files.canonical_many(paths).into_iter().map(Result::ok).collect(),
                ),
                ..self
            },
            Fact::Followed => self.learn(files, Fact::Canonical, paths).followed(files, paths),
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
    pub fn file(&self, path: &Path) -> Option<Kind> {
        self.canonical(path).map(|real| self.kind(real)).filter(|kind| *kind == Kind::File)
    }

    #[must_use]
    pub fn entries(&self, directory: &Path) -> &[PathBuf] {
        self.listed.get(directory).map_or(&[], Vec::as_slice)
    }

    pub fn settled(&mut self, order: &[impl AsRef<Path>]) -> Result<Settled<'_>, Failure> {
        let latest = order.iter().map(AsRef::as_ref).find_map(|directory| {
            self.failed.iter().rposition(|failure| matches!(failure, Failure::Io { path, .. } if path == directory))
        });
        let removed = latest.map(|index| self.failed.remove(index));
        removed.map(Err).unwrap_or(Ok(Settled(&*self)))
    }

    fn extended<Value>(
        mut found: BTreeMap<PathBuf, Value>,
        paths: &[PathBuf],
        values: Vec<Option<Value>>,
    ) -> BTreeMap<PathBuf, Value> {
        found.extend(paths.iter().cloned().zip(values).filter_map(|(path, value)| Some((path, value?))));
        found
    }

    fn followed(self, files: &impl Files, paths: &[PathBuf]) -> Self {
        let reals: Vec<PathBuf> = paths.iter().filter_map(|path| self.canonical(path)).map(Path::to_path_buf).collect();
        self.learn(files, Fact::Kind, &reals)
    }

    fn listing(self, directory: PathBuf, listing: Result<Vec<PathBuf>, FileFailure>) -> Self {
        match listing {
            Ok(entries) => Self { listed: Self::sorted_into(self.listed, directory, entries), ..self },
            Err(failure) => Self {
                failed: self
                    .failed
                    .into_iter()
                    .chain(iter::once(Failure::Io { path: directory, source: failure.into() }))
                    .collect(),
                ..self
            },
        }
    }

    fn sorted_into(
        mut listed: BTreeMap<PathBuf, Vec<PathBuf>>,
        directory: PathBuf,
        mut entries: Vec<PathBuf>,
    ) -> BTreeMap<PathBuf, Vec<PathBuf>> {
        entries.sort();
        listed.insert(directory, entries);
        listed
    }
}
