mod boundary;

use crate::{Cursor, Record};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Read {
    records: Vec<Record>,
    cursor: Cursor,
}

impl Read {
    #[must_use]
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }
}

impl IntoIterator for Read {
    type Item = Record;
    type IntoIter = std::vec::IntoIter<Record>;

    fn into_iter(self) -> Self::IntoIter {
        self.records.into_iter()
    }
}
