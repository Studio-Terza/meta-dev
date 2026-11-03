use std::fmt;

use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
pub struct RunId(String);

impl fmt::Display for RunId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
