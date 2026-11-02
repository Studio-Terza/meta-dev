use std::fmt;

use crate::cli::gate::seconds::Seconds;
use crate::cli::stage::Stage;

#[derive(Debug, PartialEq)]
pub struct Entry {
    stage: Stage,
    seconds: Seconds,
}

impl Entry {
    pub fn new(stage: Stage, seconds: Seconds) -> Entry {
        Entry { stage, seconds }
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{} = {}", self.stage, self.seconds.recorded())
    }
}

#[derive(Debug, PartialEq)]
pub struct Record(Vec<Entry>);

impl FromIterator<Entry> for Record {
    fn from_iter<Entries: IntoIterator<Item = Entry>>(entries: Entries) -> Record {
        Record(entries.into_iter().collect())
    }
}

impl fmt::Display for Record {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|entry| write!(formatter, "{entry}"))
    }
}
