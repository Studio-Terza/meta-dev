use newtype::Newtype;

#[derive(Newtype, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Flagged(&'static [&'static str]);
