use newtype::{Arbitrary, Counter, Newtype};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[must_use = "a write receipt says how many rows the store changed; use it or do not ask for it"]
pub struct Written(usize);

pub trait Done: Sized {
    fn changed(rows: usize) -> Self;

    fn total(done: impl Iterator<Item = Self>) -> Self;

    #[must_use]
    fn unchanged() -> Self {
        Self::changed(usize::MIN)
    }

    #[must_use]
    fn joined(self, other: Self) -> Self {
        Self::total([self, other].into_iter())
    }
}

impl Done for () {
    fn changed(_rows: usize) {}

    fn total(_done: impl Iterator<Item = ()>) {}
}

impl Done for Written {
    fn changed(rows: usize) -> Written {
        Written(rows)
    }

    fn total(done: impl Iterator<Item = Written>) -> Written {
        Counter::total(done)
    }
}
