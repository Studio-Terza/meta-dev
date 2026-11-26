use std::collections::BTreeMap;

use newtype::Counter;

use crate::cli::Count;

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

impl<Of> Counts<Of> {
    pub fn counted(&self) -> impl Iterator<Item = (&Of, &Count)> {
        self.0.iter()
    }
}
