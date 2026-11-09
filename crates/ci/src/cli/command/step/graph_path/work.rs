use std::fmt;

use crate::cli::command::step::seconds::Seconds;

#[derive(Clone, Copy, Debug)]
pub(super) struct Work(Seconds);

impl From<Seconds> for Work {
    fn from(seconds: Seconds) -> Work {
        Work(seconds)
    }
}

impl fmt::Display for Work {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "graph-work {} seconds, the sum of the same units", self.0)
    }
}
