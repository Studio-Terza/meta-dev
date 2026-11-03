use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Most(usize);

impl Most {
    #[must_use]
    pub const fn count(self) -> usize {
        self.0
    }

    #[must_use]
    pub const fn allows(self, count: usize) -> Option<usize> {
        if count <= self.0 { Some(count) } else { None }
    }
}
