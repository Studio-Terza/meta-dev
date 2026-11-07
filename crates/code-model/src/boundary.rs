#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileId(usize);

impl FileId {
    pub(crate) const fn next<Row>(rows: &[Row]) -> Self {
        Self(rows.len())
    }

    pub(crate) fn within<Row>(self, rows: &[Row]) -> Option<&Row> {
        rows.get(self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemId(usize);

impl ItemId {
    pub(crate) const fn next<Row>(rows: &[Row]) -> Self {
        Self(rows.len())
    }

    pub(crate) const fn index(self) -> usize {
        self.0
    }
}
