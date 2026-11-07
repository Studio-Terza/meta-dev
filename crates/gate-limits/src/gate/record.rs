use std::fmt;

use super::entry::Entry;

#[derive(Debug, PartialEq)]
pub struct Record(Vec<Entry>);

impl FromIterator<Entry> for Record {
    fn from_iter<Entries: IntoIterator<Item = Entry>>(entries: Entries) -> Self {
        Self(entries.into_iter().collect())
    }
}

impl fmt::Display for Record {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|entry| write!(formatter, "{entry}"))
    }
}
