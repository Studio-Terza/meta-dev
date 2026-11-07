use std::ops::{AddAssign, SubAssign};

use lint_core::Bound;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct NestingDepth(usize);

impl NestingDepth {
    pub const LEVEL: Self = Self(1);

    #[must_use]
    pub fn beyond(most: Bound) -> Self {
        Self(most.limit() + Self::LEVEL.0)
    }
}

impl AddAssign for NestingDepth {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl SubAssign for NestingDepth {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}
