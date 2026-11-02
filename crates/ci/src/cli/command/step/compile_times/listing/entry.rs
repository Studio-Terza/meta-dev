use std::cmp::Ordering;
use std::fmt;

use crate::cli::command::step::seconds::Seconds;
use text_lines::OneLine;

#[derive(Debug, PartialEq)]
pub(super) struct Entry {
    label: OneLine,
    seconds: Seconds,
}

impl Entry {
    pub(super) fn new(label: OneLine, seconds: Seconds) -> Entry {
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
