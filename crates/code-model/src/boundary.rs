#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileId(usize);

impl FileId {
    pub(crate) fn next<Row>(rows: &[Row]) -> FileId {
        FileId(rows.len())
    }

    pub(crate) fn within<Row>(self, rows: &[Row]) -> Option<&Row> {
        rows.get(self.0)
    }
}
