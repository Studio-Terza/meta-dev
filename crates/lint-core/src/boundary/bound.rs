use std::fmt;

use newtype::Counter;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bound {
    Most(usize),
    Least(usize),
}

impl Bound {
    #[must_use]
    pub fn allows<Counted: Counter>(self, count: Counted) -> Option<Counted> {
        match self {
            Self::Most(most) => (count.count() <= most).then_some(count),
            Self::Least(least) => (count.count() >= least).then_some(count),
        }
    }

    #[must_use]
    pub const fn limit(self) -> usize {
        match self {
            Self::Most(limit) | Self::Least(limit) => limit,
        }
    }
}

impl fmt::Display for Bound {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.limit().fmt(formatter)
    }
}
