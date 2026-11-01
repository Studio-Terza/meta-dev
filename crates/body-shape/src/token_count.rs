use newtype::{Display, Newtype};

#[derive(Newtype, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TokenCount(usize);
