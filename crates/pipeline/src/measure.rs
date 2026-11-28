use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

pub trait Measure: Default {
    #[must_use]
    fn joined(self, other: Self) -> Self;

    #[must_use]
    fn total(all: impl IntoIterator<Item = Self>) -> Self {
        all.into_iter().fold(Self::default(), Self::joined)
    }
}

impl Measure for usize {
    fn joined(self, other: usize) -> usize {
        self.saturating_add(other)
    }
}

impl Measure for u64 {
    fn joined(self, other: u64) -> u64 {
        self.saturating_add(other)
    }
}

impl Measure for Duration {
    fn joined(self, other: Duration) -> Duration {
        self.saturating_add(other)
    }
}

impl Measure for () {
    fn joined(self, (): ()) {}
}

impl<Item> Measure for Vec<Item> {
    fn joined(self, other: Vec<Item>) -> Vec<Item> {
        self.into_iter().chain(other).collect()
    }
}

impl<Item: Ord> Measure for BTreeSet<Item> {
    fn joined(self, other: BTreeSet<Item>) -> BTreeSet<Item> {
        self.into_iter().chain(other).collect()
    }
}

impl<Key: Ord, Value: Measure> Measure for BTreeMap<Key, Value> {
    fn joined(self, other: BTreeMap<Key, Value>) -> BTreeMap<Key, Value> {
        other.into_iter().fold(self, |mut joined, (key, measure)| {
            let earlier = joined.remove(&key).unwrap_or_default();
            joined.insert(key, earlier.joined(measure));
            joined
        })
    }
}
