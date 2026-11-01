use std::fmt;

use serde::Deserialize;

use crate::gate::seconds::Seconds;
use crate::gate::verdict::Verdict;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub struct Limit {
    warn: Seconds,
    block: Seconds,
}

impl Limit {
    pub fn is_ordered(&self) -> bool {
        self.warn <= self.block
    }

    pub fn judge(&self, measured: Seconds) -> Verdict {
        match (measured > self.block, measured > self.warn) {
            (true, _) => Verdict::Block,
            (false, true) => Verdict::Warn,
            (false, false) => Verdict::Within,
        }
    }
}

impl fmt::Display for Limit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "warn above {}, block above {}", self.warn, self.block)
    }
}
