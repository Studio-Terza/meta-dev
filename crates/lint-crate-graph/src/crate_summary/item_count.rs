use newtype::{Arbitrary, Counter, Newtype};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ItemCount(usize);

impl ItemCount {
    pub fn smaller_of(self, total: ItemCount) -> ItemCount {
        ItemCount(self.0.min(total.count().saturating_sub(self.0)))
    }
}
