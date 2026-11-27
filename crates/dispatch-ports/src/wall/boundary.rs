use std::time::{Duration, SystemTime, UNIX_EPOCH};

use invariants::Ignoring;
use newtype::{Arbitrary, Newtype};

use super::Deadline;

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Wall(i64);

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Moment(i64);

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timeout(u32);

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pause(u32);
use crate::Ignored;

impl Deadline {
    #[must_use]
    pub fn after(start: Wall, timeout: Timeout) -> Deadline {
        Deadline(Wall(start.0.saturating_add(i64::from(timeout.0))))
    }
}

impl Wall {
    #[must_use]
    pub fn after(self, pause: Pause) -> Wall {
        Wall(self.0.saturating_add(i64::from(pause.0)))
    }
}

impl From<SystemTime> for Wall {
    fn from(time: SystemTime) -> Wall {
        let seconds = time.duration_since(UNIX_EPOCH).ignoring(Ignored::BeforeEpoch).unwrap_or_default().as_secs();
        Wall(i64::try_from(seconds).ignoring(Ignored::BeyondRange).unwrap_or(i64::MAX))
    }
}

impl From<SystemTime> for Moment {
    fn from(time: SystemTime) -> Moment {
        let millis = time.duration_since(UNIX_EPOCH).ignoring(Ignored::BeforeEpoch).unwrap_or_default().as_millis();
        Moment(i64::try_from(millis).ignoring(Ignored::BeyondRange).unwrap_or(i64::MAX))
    }
}

impl From<Wall> for Moment {
    fn from(wall: Wall) -> Moment {
        Moment(wall.0.saturating_mul(Moment::PER_SECOND))
    }
}

impl Moment {
    const PER_SECOND: i64 = 1000;

    #[must_use]
    pub fn since(self, start: Moment) -> i64 {
        self.0.saturating_sub(start.0)
    }
}

impl From<Pause> for Duration {
    fn from(pause: Pause) -> Duration {
        Duration::from_secs(u64::from(pause.0))
    }
}
