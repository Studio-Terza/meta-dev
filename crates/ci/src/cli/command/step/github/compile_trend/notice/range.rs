use std::fmt;

use newtype::Newtype;

use crate::cli::command::step::seconds::Seconds;

#[derive(Clone, Copy, Debug)]
pub struct Range {
    least: Seconds,
    most: Seconds,
}

impl Range {
    pub(super) fn spanning(self, other: Self) -> Self {
        Self {
            least: Seconds::new(self.least.into_inner().min(other.least.into_inner())),
            most: Seconds::new(self.most.into_inner().max(other.most.into_inner())),
        }
    }

    pub(super) fn below(self, total: Seconds) -> Option<Self> {
        (total > self.most).then_some(self)
    }
}

impl From<Seconds> for Range {
    fn from(total: Seconds) -> Self {
        Self { least: total, most: total }
    }
}

impl fmt::Display for Range {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}s to {}s", self.least, self.most)
    }
}
