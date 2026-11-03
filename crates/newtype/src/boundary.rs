use crate::plain::Newtype;

pub trait Counter: Newtype<Inner = usize> + Copy {
    #[must_use]
    fn zero() -> Self {
        Self::new(usize::MIN)
    }

    #[must_use]
    fn count(self) -> usize {
        self.into_inner()
    }

    #[must_use]
    fn plus(self, other: Self) -> Self {
        Self::new(self.count().saturating_add(other.count()))
    }

    #[must_use]
    fn incremented(self) -> Self {
        Self::new(self.count().saturating_add(1))
    }

    #[must_use]
    fn total(counts: impl Iterator<Item = Self>) -> Self {
        counts.fold(Self::zero(), Self::plus)
    }
}

impl<Counted: Newtype<Inner = usize> + Copy> Counter for Counted {}
