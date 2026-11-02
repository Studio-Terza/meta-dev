use std::fmt;

use newtype::Newtype;

use crate::cli::workspace::crate_count::CrateCount;
use crate::cli::workspace::percent::Percent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Share {
    reached: CrateCount,
    total: CrateCount,
}

impl Share {
    pub fn new(reached: CrateCount, total: CrateCount) -> Share {
        Share { reached, total }
    }

    pub fn percent(self) -> Percent {
        Percent::new(
            (self.reached.into_inner() * Percent::WHOLE).checked_div(self.total.into_inner()).unwrap_or_default(),
        )
    }
}

impl fmt::Display for Share {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} of {}", self.reached, self.total)
    }
}
