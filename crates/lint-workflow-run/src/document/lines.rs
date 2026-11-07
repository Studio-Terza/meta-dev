use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Lines(usize);

impl Lines {
    pub(crate) const ONE: Self = Self::new(1);
}
