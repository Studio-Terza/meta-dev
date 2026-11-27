use std::cmp::Ordering;
use std::fmt;

use newtype::Wrapping;

use super::super::Pace;
use crate::cli::command::step::seconds::Seconds;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Range {
    least: Seconds,
    most: Seconds,
}

impl Range {
    pub(super) fn combine(self, other: Range) -> Range {
        Range {
            least: Seconds::new(self.least.into_inner().min(other.least.into_inner())),
            most: Seconds::new(self.most.into_inner().max(other.most.into_inner())),
        }
    }

    pub(super) fn pace(self, total: Seconds) -> Pace {
        match total.partial_cmp(&self.most) {
            Some(Ordering::Greater) => Pace::Slower,
            Some(Ordering::Less | Ordering::Equal) | None => Pace::Steady,
        }
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
