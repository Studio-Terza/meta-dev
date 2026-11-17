use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Branching(usize);

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Propagated(usize);

impl From<Propagated> for Branching {
    fn from(propagated: Propagated) -> Branching {
        Branching(propagated.0)
    }
}
