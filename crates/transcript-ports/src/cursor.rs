use file_ports::Offset;
use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cursor(Offset);
