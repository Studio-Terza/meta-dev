use std::ops::{Add, Sub};

use lint_core::Bound;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct NestingDepth(usize);

impl NestingDepth {
    pub const LEVEL: NestingDepth = NestingDepth(1);

    #[must_use]
    pub fn beyond(most: Bound) -> NestingDepth {
        NestingDepth(most.limit() + Self::LEVEL.0)
    }
}

impl Add for NestingDepth {
    type Output = NestingDepth;

    fn add(self, other: NestingDepth) -> NestingDepth {
        NestingDepth(self.0 + other.0)
    }
}

impl Sub for NestingDepth {
    type Output = NestingDepth;

    fn sub(self, other: NestingDepth) -> NestingDepth {
        NestingDepth(self.0 - other.0)
    }
}
