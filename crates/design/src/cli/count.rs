use newtype::{Arbitrary, Display, Newtype};
use pipeline::Measure;

#[derive(Newtype, Arbitrary, Display, Measure, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Count(usize);
