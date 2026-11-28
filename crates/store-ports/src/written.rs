use newtype::{Arbitrary, Newtype};
use pipeline::Measure;

#[derive(Newtype, Arbitrary, Measure, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[must_use = "a write receipt says how many rows the store changed; use it or do not ask for it"]
pub struct Written(usize);
