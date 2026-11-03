use std::fmt;

use crate::walk::Walk;
use newtype::{Arbitrary, Counter, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Tally(usize);

#[derive(Clone, Copy, Debug)]
pub(crate) struct Agreement {
    shared: Tally,
    all: Tally,
}

impl Agreement {
    const AT_LEAST_PARTS: usize = 4;
    const OUT_OF_PARTS: usize = 5;

    pub(crate) fn between(first: &Walk<'_, '_>, second: &Walk<'_, '_>) -> Agreement {
        Agreement {
            shared: Tally::new(first.steps().intersection(second.steps()).count()),
            all: Tally::new(first.steps().union(second.steps()).count()),
        }
    }

    pub(crate) fn near_identical(self) -> Option<Agreement> {
        Some(self).filter(|agreement| {
            agreement.shared.count().saturating_mul(Self::OUT_OF_PARTS)
                >= agreement.all.count().saturating_mul(Self::AT_LEAST_PARTS)
        })
    }
}

impl fmt::Display for Agreement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} of {}", self.shared, self.all)
    }
}
