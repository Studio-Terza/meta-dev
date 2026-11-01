use std::fmt;

use newtype::Newtype;

#[derive(Newtype, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EdgeCount(usize);

impl fmt::Display for EdgeCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
