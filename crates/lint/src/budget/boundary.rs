use newtype::{Display, Newtype};

#[derive(Newtype, Display, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Count(usize);
