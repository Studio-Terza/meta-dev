use std::time::Duration;

use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Wall(i64);

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timeout(u32);

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pause(u32);

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Deadline(Wall);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Due {
    Pending,
    Passed,
}

impl From<Duration> for Wall {
    fn from(since_epoch: Duration) -> Wall {
        Wall(i64::try_from(since_epoch.as_secs()).unwrap_or(i64::MAX))
    }
}

impl From<Pause> for Duration {
    fn from(pause: Pause) -> Duration {
        Duration::from_secs(u64::from(pause.0))
    }
}

impl Wall {
    #[must_use]
    pub fn after(self, pause: Pause) -> Wall {
        Wall(self.0.saturating_add(i64::from(pause.0)))
    }
}

impl Deadline {
    #[must_use]
    pub fn counted_from(spawn: Wall, timeout: Timeout) -> Deadline {
        Deadline(Wall(spawn.0.saturating_add(i64::from(timeout.0))))
    }

    #[must_use]
    pub fn due(self, now: Wall) -> Due {
        match now.cmp(&self.0) {
            std::cmp::Ordering::Less => Due::Pending,
            std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => Due::Passed,
        }
    }
}
