use std::path::PathBuf;

use super::Change;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Status {
    Added,
    Copied,
    Renamed,
    Other,
}

impl From<char> for Status {
    fn from(letter: char) -> Status {
        Self::LETTERS.into_iter().find(|(known, _)| *known == letter).map_or(Status::Other, |(_, status)| status)
    }
}

impl Status {
    const LETTERS: [(char, Status); 3] = [('A', Status::Added), ('C', Status::Copied), ('R', Status::Renamed)];
    const PAIR: usize = 2;

    pub(super) fn change(self, paths: &[&str]) -> Option<Change> {
        match self {
            Status::Added => Self::single(paths).map(Change::Added),
            Status::Copied => <&[&str; Self::PAIR]>::try_from(paths)
                .ok()
                .map(|[_, path]| Change::Copied(PathBuf::from(path)))
                .or_else(|| Self::single(paths).map(Change::Touched)),
            Status::Renamed => <&[&str; Self::PAIR]>::try_from(paths)
                .ok()
                .map(|[from, to]| Change::Renamed { from: PathBuf::from(from), to: PathBuf::from(to) })
                .or_else(|| Self::single(paths).map(Change::Touched)),
            Status::Other => Self::single(paths).map(Change::Touched),
        }
    }

    fn single(paths: &[&str]) -> Option<PathBuf> {
        <&[&str; 1]>::try_from(paths).ok().map(|[path]| PathBuf::from(path))
    }
}
