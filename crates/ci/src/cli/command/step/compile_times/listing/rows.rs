use newtype::{Counter, Newtype};

#[derive(Newtype, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Rows(usize);

impl Rows {
    pub(super) const LISTED: Rows = Rows(30);

    pub(super) fn kept<Row>(self, rows: impl Iterator<Item = Row>) -> impl Iterator<Item = Row> {
        rows.take(self.count())
    }
}
