use std::cmp::Ordering;
use std::fmt;

use super::super::label::Label;
use crate::cli::command::step::seconds::Seconds;

#[derive(Debug, PartialEq)]
pub(super) struct Entry {
    label: Label,
    seconds: Seconds,
}

impl Entry {
    pub(super) fn new(label: Label, seconds: Seconds) -> Entry {
        Entry { label, seconds }
    }

    pub(super) fn took(&self) -> Seconds {
        self.seconds
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Entry) -> Option<Ordering> {
        self.seconds.partial_cmp(&other.seconds)
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.seconds, self.label)
    }
}
