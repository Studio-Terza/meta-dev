use newtype::{Arbitrary, Counter, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Count(usize);

impl Count {
    pub(crate) fn nonzero(self) -> Option<Count> {
        Some(self).filter(|count| *count != Count::zero())
    }
}
