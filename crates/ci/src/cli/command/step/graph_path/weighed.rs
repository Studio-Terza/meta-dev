use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;

use super::unit_key::UnitKey;
use crate::cli::command::step::seconds::Seconds;

#[derive(Debug)]
pub(super) struct Weighed {
    seconds: Seconds,
    key: UnitKey,
}

impl Weighed {
    pub(super) fn new(seconds: Seconds, key: UnitKey) -> Weighed {
        Weighed { seconds, key }
    }

    pub(super) fn into_key(self) -> UnitKey {
        self.key
    }

    pub(super) fn compared(&self, other: &Weighed) -> Ordering {
        self.seconds.partial_cmp(&other.seconds).unwrap_or(Ordering::Equal)
    }

    pub(super) fn heaviest(self, mut weights: BTreeMap<UnitKey, Seconds>) -> BTreeMap<UnitKey, Seconds> {
        let held = weights.entry(self.key).or_default();
        *held = held.longer(self.seconds);
        weights
    }
}

impl AsRef<Seconds> for Weighed {
    fn as_ref(&self) -> &Seconds {
        &self.seconds
    }
}

impl AsRef<UnitKey> for Weighed {
    fn as_ref(&self) -> &UnitKey {
        &self.key
    }
}

impl fmt::Display for Weighed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.seconds, self.key)
    }
}
