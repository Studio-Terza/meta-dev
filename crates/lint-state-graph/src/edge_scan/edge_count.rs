use newtype::{Display, Newtype};

#[derive(Newtype, Display, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EdgeCount(usize);
