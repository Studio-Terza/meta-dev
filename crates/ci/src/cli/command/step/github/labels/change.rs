use std::path::{Path, PathBuf};
use std::str::FromStr;

mod boundary;

use super::place::Place;
use boundary::Status;

#[derive(Debug, PartialEq, Eq)]
pub struct NotAChange;

#[derive(Debug, PartialEq, Eq)]
pub enum Change {
    Added(PathBuf),
    Copied(PathBuf),
    Renamed { from: PathBuf, to: PathBuf },
    Touched(PathBuf),
}

impl Change {
    const SEPARATOR: &'static str = "\t";

    pub(crate) fn now_at(&self) -> &Path {
        match self {
            Self::Added(path) | Self::Copied(path) | Self::Touched(path) => path,
            Self::Renamed { to, .. } => to,
        }
    }

    pub(crate) fn paths(&self) -> Vec<&Path> {
        match self {
            Self::Added(path) | Self::Copied(path) | Self::Touched(path) => vec![path.as_path()],
            Self::Renamed { from, to } => vec![from.as_path(), to.as_path()],
        }
    }

    pub(crate) fn touching(&self, place: Place) -> Option<&Self> {
        self.paths().into_iter().find_map(|path| place.containing(path)).map(|_| self)
    }

    pub(crate) fn bringing_in(&self, place: Place) -> Option<&Path> {
        match self {
            Self::Added(path) | Self::Copied(path) => place.containing(path).map(|_| path.as_path()),
            Self::Renamed { from, to } => {
                place.containing(to).filter(|_| place.containing(from).is_none()).map(|_| to.as_path())
            }
            Self::Touched(_) => None,
        }
    }
}

impl FromStr for Change {
    type Err = NotAChange;

    fn from_str(line: &str) -> Result<Self, NotAChange> {
        let tabs: Vec<usize> = line.match_indices(Self::SEPARATOR).map(|(at, _)| at).collect();
        let starts = [0].into_iter().chain(tabs.iter().map(|at| at + Self::SEPARATOR.len()));
        let ends = tabs.iter().copied().chain([line.len()]);
        let fields: Vec<&str> = starts.zip(ends).filter_map(|(start, end)| line.get(start..end)).collect();
        let (status, paths) = fields.split_first().ok_or(NotAChange)?;
        let status = status.chars().next().map(Status::from).ok_or(NotAChange)?;
        status.change(paths).ok_or(NotAChange)
    }
}
