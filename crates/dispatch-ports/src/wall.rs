mod boundary;

use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Wall(i64);

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timeout(u32);

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pause(u32);

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Deadline(Wall);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Due {
    Pending,
    Passed,
}

impl Deadline {
    #[must_use]
    pub fn due(self, now: Wall) -> Due {
        match now.cmp(&self.0) {
            std::cmp::Ordering::Less => Due::Pending,
            std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => Due::Passed,
        }
    }
}
