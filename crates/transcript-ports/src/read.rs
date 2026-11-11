mod boundary;

use crate::{Cursor, Line};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Read {
    lines: Vec<Line>,
    cursor: Cursor,
}

impl Read {
    #[must_use]
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }
}

impl IntoIterator for Read {
    type Item = Line;
    type IntoIter = std::vec::IntoIter<Line>;

    fn into_iter(self) -> Self::IntoIter {
        self.lines.into_iter()
    }
}
