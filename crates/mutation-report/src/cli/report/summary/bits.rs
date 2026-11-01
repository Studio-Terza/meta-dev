use newtype::{Display, Newtype};
use serde::Serialize;

#[derive(Newtype, Display, Serialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
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

    pub fn first(self, items: &[String]) -> impl Iterator<Item = &String> {
        items.iter().take(self.0)
    }
}
