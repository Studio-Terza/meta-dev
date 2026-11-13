use lint_core::FileHead;

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

#[derive(Clone, Copy, Debug)]
pub(super) enum Change {
    Lifted,
    Landed,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Flights(usize);

impl Flights {
    pub(super) fn flown(self, change: Change) -> Flights {
        match change {
            Change::Lifted => Flights(self.0 + 1),
            Change::Landed => Flights(self.0.saturating_sub(1)),
        }
    }

    pub(super) fn airborne(self) -> Option<Flights> {
        Some(self).filter(|flights| flights.0 > 0)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Expected(usize);

impl Expected {
    pub(super) fn met(self, heads: &[FileHead]) -> Option<Expected> {
        Some(self).filter(|expected| heads.len() == expected.0)
    }
}

impl Expected {
    pub(super) fn counted(count: usize) -> Expected {
        Expected(count)
    }
}
