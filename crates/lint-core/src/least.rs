use newtype::{Arbitrary, Counter, Newtype};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Least(usize);

impl Least {
    #[must_use]
    pub fn allows<Counted: Counter>(self, count: Counted) -> Option<Counted> {
        (count.count() >= self.0).then_some(count)
    }
}
