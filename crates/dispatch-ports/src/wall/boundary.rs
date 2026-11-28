use std::time::{SystemTime, UNIX_EPOCH};

use invariants::Ignoring;
use newtype::{Arbitrary, Deserialize, Newtype};

use crate::ignored::Ignored;

#[derive(Newtype, Arbitrary, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Wall(i64);

#[derive(Newtype, Arbitrary, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timeout(u32);

#[derive(Newtype, Arbitrary, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pause(u32);

impl Wall {
    pub(super) fn after(self, timeout: Timeout) -> Wall {
        Wall(self.0.saturating_add(i64::from(timeout.0)))
    }
}

impl From<SystemTime> for Wall {
    fn from(time: SystemTime) -> Wall {
        let seconds = time.duration_since(UNIX_EPOCH).ignoring(Ignored::BeforeEpoch).unwrap_or_default().as_secs();
        Wall(i64::try_from(seconds).ignoring(Ignored::BeyondRange).unwrap_or(i64::MAX))
    }
}
