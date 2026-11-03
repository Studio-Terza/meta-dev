use newtype::{Display, Newtype};

#[derive(Newtype, Display, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Word(&'static str);
