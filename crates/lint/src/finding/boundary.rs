use newtype::{Display, Newtype};

#[derive(Newtype, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Line(usize);

#[derive(Clone, Copy, Debug)]
pub(crate) struct Fix(pub &'static str);

#[derive(Debug)]
pub(super) struct Problem(pub String);
