use newtype::{Arbitrary, Counter, Newtype};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Rows(usize);

impl Rows {
    pub(crate) const LISTED: Rows = Rows(30);

    pub(crate) fn kept<Row>(self, rows: impl Iterator<Item = Row>) -> impl Iterator<Item = Row> {
        rows.take(self.count())
    }
}
