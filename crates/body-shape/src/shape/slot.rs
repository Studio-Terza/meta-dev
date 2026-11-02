use newtype::Newtype;

#[derive(Newtype, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Slot(usize);
