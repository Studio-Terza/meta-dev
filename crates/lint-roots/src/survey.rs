mod boundary;

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Deref;
use std::path::{Path, PathBuf};

use algebra::{Monoid, Semigroup};
use file_ports::{FileFailure, Files, Kind};
use lint_layout::Failure;
use newtype::{Arbitrary, Newtype};

use crate::ignored::Ignored;
use boundary::Spot;
use invariants::Ignoring;

#[derive(Clone, Copy, Debug)]
pub enum Fact {
    Kind,
    Canonical,
    Followed,
    Listed,
}

#[derive(Debug, Default)]
pub struct Survey {
    kinds: BTreeMap<Spot, Kind>,
    canonical: BTreeMap<Spot, PathBuf>,
    listed: BTreeMap<Spot, Vec<PathBuf>>,
    failed: Vec<Failure>,
}

#[derive(Debug)]
pub(crate) struct Failed(Vec<Failure>);

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Position(usize);

impl Semigroup for Survey {
    fn combine(self, other: Survey) -> Survey {
        let mut kinds = self.kinds;
        kinds.extend(other.kinds);
        let mut canonical = self.canonical;
        canonical.extend(other.canonical);
        let mut listed = self.listed;
        listed.extend(other.listed);
        Survey { kinds, canonical, listed, failed: self.failed.combine(other.failed) }
    }
}

impl Monoid for Survey {}

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
    pub fn learned(files: &impl Files, fact: Fact, paths: &[PathBuf]) -> Survey {
        match fact {
            Fact::Kind => Survey {
                kinds: Self::found(paths, files.kinds(paths).into_iter().map(Some).collect()),
                ..Survey::default()
            },
            Fact::Canonical => Survey {
                canonical: Self::found(
                    paths,
                    files
                        .canonical_many(paths)
                        .into_iter()
                        .map(|canonical| canonical.ignoring(Ignored::Uncanonical))
                        .collect(),
                ),
                ..Survey::default()
            },
            Fact::Followed => Self::followed(files, paths),
            Fact::Listed => Monoid::concat(
                paths
                    .iter()
                    .cloned()
                    .zip(files.entries_many(paths))
                    .map(|(directory, listing)| Self::listing(directory, listing)),
            ),
        }
    }

    #[must_use]
    pub fn kind(&self, path: &Path) -> Kind {
        self.kinds.get(Spot::bytes(path)).copied().unwrap_or(Kind::Missing)
    }

    #[must_use]
    pub fn canonical(&self, path: &Path) -> Option<&Path> {
        self.canonical.get(Spot::bytes(path)).map(PathBuf::as_path)
    }

    #[must_use]
    pub fn file(&self, path: &Path) -> Option<Kind> {
        self.canonical(path).map(|real| self.kind(real)).filter(|kind| *kind == Kind::File)
    }

    #[must_use]
    pub fn entries(&self, directory: &Path) -> &[PathBuf] {
        self.listed.get(Spot::bytes(directory)).map_or(&[], Vec::as_slice)
    }

    pub(crate) fn settling<Made>(self, settle: impl FnOnce(&Survey, Failed) -> Made) -> Made {
        let Survey { kinds, canonical, listed, failed } = self;
        settle(&Survey { kinds, canonical, listed, failed: Vec::new() }, Failed(failed))
    }

    fn found<Value>(paths: &[PathBuf], values: Vec<Option<Value>>) -> BTreeMap<Spot, Value> {
        paths.iter().cloned().map(Spot::from).zip(values).filter_map(|(path, value)| Some((path, value?))).collect()
    }

    fn followed(files: &impl Files, paths: &[PathBuf]) -> Survey {
        let canonical = Self::learned(files, Fact::Canonical, paths);
        let reals: Vec<PathBuf> =
            paths.iter().filter_map(|path| canonical.canonical(path)).map(Path::to_path_buf).collect();
        canonical.combine(Self::learned(files, Fact::Kind, &reals))
    }

    fn listing(directory: PathBuf, listing: Result<Vec<PathBuf>, FileFailure>) -> Survey {
        match listing {
            Ok(entries) => Survey {
                listed: BTreeMap::from([(Spot::from(directory), BTreeSet::from_iter(entries).into_iter().collect())]),
                ..Survey::default()
            },
            Err(failure) => {
                Survey { failed: vec![Failure::Io { path: directory, source: failure.into() }], ..Survey::default() }
            }
        }
    }
}

impl Failed {
    pub(crate) fn settled<'known>(
        self,
        known: &'known Survey,
        orders: &[Vec<&Path>],
    ) -> Vec<Result<Settled<'known>, Failure>> {
        let taken = orders.iter().fold(Vec::new(), |mut taken, order| {
            let latest = self.latest(order, &taken);
            taken.push(latest);
            taken
        });
        let mut owned: BTreeMap<Position, Failure> =
            self.0.into_iter().enumerate().map(|(at, failure)| (Position::new(at), failure)).collect();
        taken
            .into_iter()
            .map(|latest| latest.and_then(|at| owned.remove(&at)).map_or(Ok(Settled(known)), Err))
            .collect()
    }

    fn latest(&self, order: &[&Path], taken: &[Option<Position>]) -> Option<Position> {
        order.iter().find_map(|directory| self.latest_at(directory, taken))
    }

    fn latest_at(&self, directory: &Path, taken: &[Option<Position>]) -> Option<Position> {
        self.0
            .iter()
            .enumerate()
            .map(|(at, failure)| Positioned { at: Position::new(at), failure })
            .rev()
            .filter(|positioned| taken.iter().all(|took| *took != Some(positioned.at)))
            .find(|positioned| matches!(positioned.failure, Failure::Io { path, .. } if path == directory))
            .map(|positioned| positioned.at)
    }
}

#[derive(Debug)]
struct Positioned<'failed> {
    at: Position,
    failure: &'failed Failure,
}
