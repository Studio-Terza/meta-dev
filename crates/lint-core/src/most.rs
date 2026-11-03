use newtype::{Arbitrary, Counter, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Most(usize);

impl Most {
    #[must_use]
    pub fn allows<Counted: Counter>(self, count: Counted) -> Option<Counted> {
        (count.count() <= self.0).then_some(count)
    }
}
