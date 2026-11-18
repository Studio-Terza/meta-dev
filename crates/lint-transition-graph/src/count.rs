use newtype::{Arbitrary, Counter, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Count(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Positive(Count);

impl From<Positive> for Count {
    fn from(positive: Positive) -> Count {
        positive.0
    }
}

impl Count {
    pub(crate) fn positive(self) -> Option<Positive> {
        Some(self).filter(|count| *count != Count::zero()).map(Positive)
    }
}
