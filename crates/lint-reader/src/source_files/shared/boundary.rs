use algebra::{Monoid, Semigroup};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Partition(usize);

impl Partition {
    pub const MAIN: Partition = Partition(0);
}

impl From<usize> for Partition {
    fn from(index: usize) -> Partition {
        Partition(index)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Flights {
    sent: usize,
    landed: usize,
}

impl Semigroup for Flights {
    fn combine(self, other: Flights) -> Flights {
        Flights { sent: self.sent + other.sent, landed: self.landed + other.landed }
    }
}

impl Monoid for Flights {}

impl Flights {
    pub(super) const SENT: Flights = Flights { sent: 1, landed: 0 };
    pub(super) const LANDED: Flights = Flights { sent: 0, landed: 1 };

    pub(super) fn airborne(self) -> Option<()> {
        Some(()).filter(|()| self.sent > self.landed)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Rank(usize);

impl From<usize> for Rank {
    fn from(index: usize) -> Rank {
        Rank(index)
    }
}
