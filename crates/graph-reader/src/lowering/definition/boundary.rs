#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::lowering) struct RowIndex(usize);

impl<Row> From<&[Row]> for RowIndex {
    fn from(rows: &[Row]) -> RowIndex {
        RowIndex(rows.len())
    }
}

impl RowIndex {
    pub(in crate::lowering) fn slot_in<Row>(self, rows: &mut [Row]) -> Option<&mut Row> {
        rows.get_mut(self.0)
    }
}
