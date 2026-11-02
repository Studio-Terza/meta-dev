use newtype::{Display, Newtype};

#[derive(Newtype, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Score(usize);
