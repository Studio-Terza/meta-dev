use std::fmt;

use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
pub struct Repository(String);

impl fmt::Display for Repository {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
