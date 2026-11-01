use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Most(usize);

impl Most {
    #[must_use]
    pub const fn new(count: usize) -> Most {
        Most(count)
    }

    #[must_use]
    pub const fn count(self) -> usize {
        self.0
    }

    #[must_use]
    pub const fn allows(self, count: usize) -> bool {
        count <= self.0
    }
}

impl fmt::Display for Most {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
