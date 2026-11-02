use newtype::{Display, Newtype};

#[derive(Newtype, Display, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Count(usize);
