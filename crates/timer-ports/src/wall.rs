use std::time::{Duration, SystemTime};

use chrono::{DateTime, TimeDelta, Utc};
use invariants::Ignoring;
use newtype::{Arbitrary, Newtype};

use crate::Elapsed;
use crate::ignored::Ignored;

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

impl Wall {
    #[must_use]
    pub fn now() -> Wall {
        Wall::from(Utc::now())
    }

    #[must_use]
    pub fn from_seconds(seconds: f64) -> Option<Wall> {
        let span = Duration::try_from_secs_f64(seconds).ignoring(Ignored::Unnamed)?;
        let since = TimeDelta::from_std(span).ignoring(Ignored::Unnamed)?;
        DateTime::UNIX_EPOCH.checked_add_signed(since).map(Wall::from)
    }

    #[must_use]
    pub fn since(self, earlier: Wall) -> Elapsed {
        Elapsed::from((self.at() - earlier.at()).to_std().ignoring(Ignored::Reversed).unwrap_or_default())
    }

    #[must_use]
    pub fn after(self, span: Elapsed) -> Wall {
        let delta = TimeDelta::from_std(span.into()).ignoring(Ignored::Unbounded);
        let later = delta.and_then(|ahead| self.at().checked_add_signed(ahead));
        later.map_or(Wall(i64::MAX), Wall::from)
    }

    fn at(self) -> DateTime<Utc> {
        DateTime::from_timestamp_nanos(self.0)
    }
}

impl From<DateTime<Utc>> for Wall {
    fn from(at: DateTime<Utc>) -> Wall {
        Wall(at.timestamp_nanos_opt().unwrap_or(i64::MAX))
    }
}

impl From<SystemTime> for Wall {
    fn from(time: SystemTime) -> Wall {
        Wall::from(DateTime::<Utc>::from(time))
    }
}

impl From<Wall> for SystemTime {
    fn from(wall: Wall) -> SystemTime {
        SystemTime::from(wall.at())
    }
}

impl From<Timeout> for Elapsed {
    fn from(timeout: Timeout) -> Elapsed {
        Elapsed::from(Duration::from_secs(u64::from(timeout.0)))
    }
}

impl From<Pause> for Duration {
    fn from(pause: Pause) -> Duration {
        Duration::from_secs(u64::from(pause.0))
    }
}

impl Deadline {
    #[must_use]
    pub fn after(start: Wall, timeout: Timeout) -> Deadline {
        Deadline(start.after(Elapsed::from(timeout)))
    }

    #[must_use]
    pub fn due(self, now: Wall) -> Due {
        if now < self.0 { Due::Pending } else { Due::Passed }
    }
}
