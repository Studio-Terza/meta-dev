use std::fmt;

use crate::cli::command::step::seconds::Seconds;
use text_lines::OneLine;

#[derive(Debug, PartialEq)]
pub(super) struct Entry {
    label: OneLine,
    seconds: Seconds,
}

impl Entry {
    pub(super) fn new(label: OneLine, seconds: Seconds) -> Self {
        Self { label, seconds }
    }

    pub(super) fn took(&self) -> Seconds {
        self.seconds
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.seconds, self.label)
    }
}
