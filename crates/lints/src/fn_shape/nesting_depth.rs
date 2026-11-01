use std::ops::{AddAssign, SubAssign};

use lint_core::Most;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct NestingDepth(usize);

impl NestingDepth {
    pub const LEVEL: NestingDepth = NestingDepth(1);

    #[must_use]
    pub fn beyond(most: Most) -> NestingDepth {
        NestingDepth(most.count() + Self::LEVEL.0)
    }
}

impl AddAssign for NestingDepth {
    fn add_assign(&mut self, other: NestingDepth) {
        self.0 += other.0;
    }
}

impl SubAssign for NestingDepth {
    fn sub_assign(&mut self, other: NestingDepth) {
        self.0 -= other.0;
    }
}
