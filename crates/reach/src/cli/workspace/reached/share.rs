use std::fmt;

use crate::cli::workspace::crate_count::CrateCount;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Share {
    reached: CrateCount,
    total: CrateCount,
}

impl Share {
    pub fn new(reached: CrateCount, total: CrateCount) -> Share {
        Share { reached, total }
    }
}

impl fmt::Display for Share {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} of {}", self.reached, self.total)
    }
}
