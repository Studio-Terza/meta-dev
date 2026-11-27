use std::collections::BTreeMap;
use std::fmt;

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

impl<Of: fmt::Display> Counts<Of> {
    pub fn listed(&self, formatter: &mut fmt::Formatter<'_>, heading: impl fmt::Display) -> fmt::Result {
        self.0.iter().try_for_each(|(key, count)| writeln!(formatter, "{heading} {key} {count}"))
    }
}
