use std::time::{SystemTime, UNIX_EPOCH};

use invariants::Ignoring;

use super::{Deadline, Timeout, Wall};
use crate::Ignored;

impl Deadline {
    #[must_use]
    pub fn after(start: Wall, timeout: Timeout) -> Deadline {
        Deadline(Wall(start.0.saturating_add(i64::from(timeout.0))))
    }
}

impl From<SystemTime> for Wall {
    fn from(time: SystemTime) -> Wall {
        let seconds = time.duration_since(UNIX_EPOCH).ignoring(Ignored::BeforeEpoch).unwrap_or_default().as_secs();
        Wall(i64::try_from(seconds).ignoring(Ignored::BeyondRange).unwrap_or(i64::MAX))
    }
}
