#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FileId(usize);

impl FileId {
    pub(super) fn next<Row>(rows: &[Row]) -> FileId {
        FileId(rows.len())
    }

    pub(super) fn within<Row>(self, rows: &[Row]) -> Option<&Row> {
        rows.get(self.0)
    }
}
