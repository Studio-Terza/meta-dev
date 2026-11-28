use std::time::{SystemTime, UNIX_EPOCH};

use invariants::Ignoring;
use newtype::{Arbitrary, Newtype};

use crate::ignored::Ignored;

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Epoch(i64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stamp {
    Born,
    Touched,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stamps {
    born: Epoch,
    touched: Epoch,
}

impl From<SystemTime> for Epoch {
    fn from(time: SystemTime) -> Epoch {
        let seconds = time.duration_since(UNIX_EPOCH).ignoring(Ignored::BeforeEpoch).map_or(0, |since| since.as_secs());
        Epoch(i64::try_from(seconds).ignoring(Ignored::BeyondRange).unwrap_or(i64::MAX))
    }
}

impl Stamps {
    #[must_use]
    pub fn new(born: Epoch, touched: Epoch) -> Stamps {
        Stamps { born, touched }
    }

    #[must_use]
    pub const fn dated(self, stamp: Stamp) -> Epoch {
        match stamp {
            Stamp::Born => self.born,
            Stamp::Touched => self.touched,
        }
    }
}
