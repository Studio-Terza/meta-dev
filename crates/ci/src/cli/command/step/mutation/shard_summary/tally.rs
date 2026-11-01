use std::fmt;

use super::count::Count;
use super::outcome::Outcome;

#[derive(Debug)]
pub struct Tally {
    outcome: Outcome,
    count: Count,
}

impl Tally {
    pub(crate) fn new(outcome: Outcome, count: Count) -> Tally {
        Tally { outcome, count }
    }
}

impl fmt::Display for Tally {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.outcome, self.count)
    }
}
