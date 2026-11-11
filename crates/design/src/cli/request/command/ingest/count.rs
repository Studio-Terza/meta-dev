use std::collections::BTreeMap;

use newtype::Counter;

use crate::cli::output::{Count, Line};

#[derive(Debug)]
pub struct Counts<Of>(BTreeMap<Of, Count>);

impl<Of: Ord> FromIterator<Of> for Counts<Of> {
    fn from_iter<Keys: IntoIterator<Item = Of>>(keys: Keys) -> Counts<Of> {
        Counts(keys.into_iter().fold(BTreeMap::new(), |mut counts, key| {
            let entry = counts.entry(key).or_insert_with(Count::zero);
            *entry = entry.incremented();
            counts
        }))
    }
}

impl<Of: Clone> Counts<Of> {
    pub fn lines(&self, line: impl Fn(Of, Count) -> Line) -> Vec<Line> {
        self.0.iter().map(|(key, count)| line(key.clone(), *count)).collect()
    }
}
