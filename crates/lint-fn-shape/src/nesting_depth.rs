use lint_core::Bound;
use lint_core::Joins;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct NestingDepth(usize);

impl NestingDepth {
    pub const LEVEL: NestingDepth = NestingDepth(1);

    #[must_use]
    pub fn beyond(most: Bound) -> NestingDepth {
        NestingDepth(most.limit() + Self::LEVEL.0)
    }
}

impl Joins for NestingDepth {
    fn combine(self, other: NestingDepth) -> NestingDepth {
        NestingDepth(self.0 + other.0)
    }
}
