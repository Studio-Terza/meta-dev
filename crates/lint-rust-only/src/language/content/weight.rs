use newtype::Newtype;

#[derive(Newtype, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Weight(u8);
