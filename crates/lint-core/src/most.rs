use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Most(usize);

impl Most {
    #[must_use]
    pub const fn count(self) -> usize {
        self.0
    }

    #[must_use]
    pub const fn allows(self, count: usize) -> bool {
        count <= self.0
    }
}
