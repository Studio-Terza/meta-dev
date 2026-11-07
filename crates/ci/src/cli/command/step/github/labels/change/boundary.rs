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
    fn from(letter: char) -> Self {
        Self::LETTERS.into_iter().find(|(known, _)| *known == letter).map_or(Self::Other, |(_, status)| status)
    }
}

impl Status {
    const LETTERS: [(char, Self); 3] = [('A', Self::Added), ('C', Self::Copied), ('R', Self::Renamed)];
    const PAIR: usize = 2;

    pub(super) fn change(self, paths: &[&str]) -> Option<Change> {
        match self {
            Self::Added => Self::single(paths).map(Change::Added),
            Self::Copied => <&[&str; Self::PAIR]>::try_from(paths)
                .ok()
                .map(|[_, path]| Change::Copied(PathBuf::from(path)))
                .or_else(|| Self::single(paths).map(Change::Touched)),
            Self::Renamed => <&[&str; Self::PAIR]>::try_from(paths)
                .ok()
                .map(|[from, to]| Change::Renamed { from: PathBuf::from(from), to: PathBuf::from(to) })
                .or_else(|| Self::single(paths).map(Change::Touched)),
            Self::Other => Self::single(paths).map(Change::Touched),
        }
    }

    fn single(paths: &[&str]) -> Option<PathBuf> {
        <&[&str; 1]>::try_from(paths).ok().map(|[path]| PathBuf::from(path))
    }
}
