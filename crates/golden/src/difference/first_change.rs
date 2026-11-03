use std::fmt;

use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FirstChange(usize);

impl fmt::Display for FirstChange {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0 + 1)
    }
}
