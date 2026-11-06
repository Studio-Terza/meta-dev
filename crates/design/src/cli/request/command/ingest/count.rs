use std::collections::BTreeMap;
use std::fmt;

use newtype::{Arbitrary, Counter, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Count(usize);

#[derive(Clone, Copy, Debug)]
pub struct Label(&'static str);

impl Label {
    pub const AUTHORSHIP: Label = Label("authorship");
    pub const FORM: Label = Label("form");
    pub const RELATION: Label = Label("relation");
}

impl fmt::Display for Label {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

#[derive(Debug)]
pub struct Counts<Of>(BTreeMap<Of, Count>);

impl<Of: Ord> FromIterator<Of> for Counts<Of> {
    fn from_iter<Keys: IntoIterator<Item = Of>>(keys: Keys) -> Counts<Of> {
        Counts(keys.into_iter().fold(BTreeMap::new(), |mut counts, key| {
            let entry: &mut Count = counts.entry(key).or_default();
            *entry = entry.incremented();
            counts
        }))
    }
}

impl<Of: fmt::Display> Counts<Of> {
    pub fn lines(&self, label: Label, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|(key, count)| writeln!(formatter, "{label} {key} {count}"))
    }
}
