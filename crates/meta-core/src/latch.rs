use crate::ids::{Observed, Timestamp};

/// One falsification, latched: when a measure first broke its bound, and what was observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Latch {
    at: Timestamp,
    observed: Observed,
}

impl Latch {
    pub(crate) fn new(at: Timestamp, observed: Observed) -> Self {
        Self { at, observed }
    }

    pub(crate) fn shown(&self) -> String {
        format!("falsified at {} ({})", self.at, self.observed)
    }
}
