mod boundary;

use newtype::{Arbitrary, Newtype};

pub use boundary::{Moment, Pause, Timeout, Wall};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Deadline(Wall);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Due {
    Pending,
    Passed,
}

impl Deadline {
    #[must_use]
    pub(crate) fn due(self, now: Wall) -> Due {
        match now.cmp(&self.0) {
            std::cmp::Ordering::Less => Due::Pending,
            std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => Due::Passed,
        }
    }
}
