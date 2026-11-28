use newtype::{Arbitrary, Newtype};
use pipeline::Measure;

#[derive(Newtype, Arbitrary, Measure, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Calls(usize);
