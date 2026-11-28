use std::collections::BTreeMap;
use std::fmt;

use pipeline::Measure;

use crate::cli::Count;

#[derive(Debug, Measure)]
pub struct Counts<Of: Ord>(BTreeMap<Of, Count>);

impl<Of: Ord> Default for Counts<Of> {
    fn default() -> Counts<Of> {
        Counts(BTreeMap::new())
    }
}

impl<Of: Ord> FromIterator<Of> for Counts<Of> {
    fn from_iter<Keys: IntoIterator<Item = Of>>(keys: Keys) -> Counts<Of> {
        Counts::total(keys.into_iter().map(|key| Counts(BTreeMap::from([(key, Count::new(1))]))))
    }
}

impl<Of: Ord + fmt::Display> Counts<Of> {
    pub fn listed(&self, formatter: &mut fmt::Formatter<'_>, heading: &str) -> fmt::Result {
        self.0.iter().try_for_each(|(key, count)| writeln!(formatter, "{heading} {key} {count}"))
    }
}
