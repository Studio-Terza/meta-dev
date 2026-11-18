use std::fmt;

use algebra::Semigroup;
use newtype::Wrapping;

use crate::cli::command::step::seconds::Seconds;

#[derive(Clone, Copy, Debug)]
pub struct Range {
    least: Seconds,
    most: Seconds,
}

impl Semigroup for Range {
    fn combine(self, other: Range) -> Range {
        Range {
            least: Seconds::new(self.least.into_inner().min(other.least.into_inner())),
            most: Seconds::new(self.most.into_inner().max(other.most.into_inner())),
        }
    }
}

impl Range {
    pub(super) fn below(self, total: Seconds) -> Option<()> {
        (total > self.most).then_some(())
    }
}

impl From<Seconds> for Range {
    fn from(total: Seconds) -> Range {
        Range { least: total, most: total }
    }
}

impl fmt::Display for Range {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}s to {}s", self.least, self.most)
    }
}
