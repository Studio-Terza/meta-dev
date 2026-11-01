use std::fmt;

use newtype::{Counter, Display, Newtype};

#[derive(Newtype, Display, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Tally(usize);

#[derive(Clone, Copy, Debug)]
pub(crate) struct Agreement {
    shared: Tally,
    all: Tally,
}

impl Agreement {
    const AT_LEAST_PARTS: usize = 4;
    const OUT_OF_PARTS: usize = 5;

    pub(crate) fn counted(shared: usize, all: usize) -> Agreement {
        Agreement { shared: Tally::new(shared), all: Tally::new(all) }
    }

    pub(crate) fn is_near_identical(self) -> bool {
        self.shared.count().saturating_mul(Self::OUT_OF_PARTS) >= self.all.count().saturating_mul(Self::AT_LEAST_PARTS)
    }
}

impl fmt::Display for Agreement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} of {}", self.shared, self.all)
    }
}
