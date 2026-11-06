use std::fmt;

use newtype::{Arbitrary, Counter, Newtype};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Passed(usize);

impl fmt::Display for Passed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} workloads verified", self.count())
    }
}
