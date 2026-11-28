use newtype::{Arbitrary, Newtype};
use pipeline::Measure;

#[derive(Newtype, Arbitrary, Measure, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Calls(usize);
