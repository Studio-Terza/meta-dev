use std::path::{Path, PathBuf};
use std::str::FromStr;

use super::place::Place;

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
    const ADDED: char = 'A';
    const COPIED: char = 'C';
    const RENAMED: char = 'R';

    pub(crate) fn now_at(&self) -> &Path {
        match self {
            Change::Added(path) | Change::Copied(path) | Change::Touched(path) => path,
            Change::Renamed { to, .. } => to,
        }
    }

    pub(crate) fn touching(&self, place: Place) -> Option<&Change> {
        let touched = match self {
            Change::Renamed { from, to } => [from, to].iter().any(|path| place.containing(path).is_some()),
            Change::Added(path) | Change::Copied(path) | Change::Touched(path) => place.containing(path).is_some(),
        };
        touched.then_some(self)
    }

    pub(crate) fn bringing_in(&self, place: Place) -> Option<&Path> {
        match self {
            Change::Added(path) | Change::Copied(path) => place.containing(path).map(|_| path.as_path()),
            Change::Renamed { from, to } => {
                place.containing(to).filter(|_| place.containing(from).is_none()).map(|_| to.as_path())
            }
            Change::Touched(_) => None,
        }
    }

    fn fields(line: &str) -> Vec<&str> {
        let tabs: Vec<usize> = line.match_indices(Self::SEPARATOR).map(|(at, _)| at).collect();
        let starts = [0].into_iter().chain(tabs.iter().map(|at| at + Self::SEPARATOR.len()));
        let ends = tabs.iter().copied().chain([line.len()]);
        starts.zip(ends).filter_map(|(start, end)| line.get(start..end)).collect()
    }
}

impl FromStr for Change {
    type Err = NotAChange;

    fn from_str(line: &str) -> Result<Change, NotAChange> {
        let fields = Self::fields(line);
        let status = fields.first().and_then(|status| status.chars().next()).ok_or(NotAChange)?;
        match (fields.as_slice(), status) {
            ([_, path], Self::ADDED) => Ok(Change::Added(PathBuf::from(path))),
            ([_, _, path], Self::COPIED) => Ok(Change::Copied(PathBuf::from(path))),
            ([_, from, to], Self::RENAMED) => Ok(Change::Renamed { from: PathBuf::from(from), to: PathBuf::from(to) }),
            ([_, path], _) => Ok(Change::Touched(PathBuf::from(path))),
            _ => Err(NotAChange),
        }
    }
}
