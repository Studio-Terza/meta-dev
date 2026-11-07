use newtype::{Arbitrary, Counter, Newtype};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Rows(usize);

impl Rows {
    pub(super) const LISTED: Self = Self(30);

    pub(super) fn kept<Row>(self, rows: impl Iterator<Item = Row>) -> impl Iterator<Item = Row> {
        rows.take(self.count())
    }
}
