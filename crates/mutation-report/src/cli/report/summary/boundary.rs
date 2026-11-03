use newtype::{Arbitrary, Display, Newtype};
use serde::Serialize;

#[derive(Newtype, Arbitrary, Display, Serialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(transparent)]
pub struct Count(usize);

impl Count {
    pub const SHOWN: Count = Count(300);

    #[must_use]
    pub fn items_in<Counted>(items: &[Counted]) -> Count {
        Count(items.len())
    }

    #[must_use]
    pub fn beyond(self, limit: Count) -> Count {
        Count(self.0.saturating_sub(limit.0))
    }

    pub(crate) fn first<Item>(self, items: &[Item]) -> impl Iterator<Item = &Item> {
        items.iter().take(self.0)
    }
}
