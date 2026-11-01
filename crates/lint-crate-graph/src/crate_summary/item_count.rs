use std::iter::Sum;
use std::ops::Add;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ItemCount(usize);

impl ItemCount {
    pub fn of_len(len: usize) -> ItemCount {
        ItemCount(len)
    }

    pub fn smaller_of(self, total: ItemCount) -> ItemCount {
        ItemCount(self.0.min(total.0.saturating_sub(self.0)))
    }

    pub fn count(self) -> usize {
        self.0
    }
}

impl Add for ItemCount {
    type Output = ItemCount;

    fn add(self, other: ItemCount) -> ItemCount {
        ItemCount(self.0 + other.0)
    }
}

impl Sum for ItemCount {
    fn sum<Counts: Iterator<Item = ItemCount>>(counts: Counts) -> ItemCount {
        counts.fold(ItemCount::default(), Add::add)
    }
}
