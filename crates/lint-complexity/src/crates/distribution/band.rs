use std::fmt;

use body_lifts::Branching;
use lint_core::Bound;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Band {
    Straight,
    Few,
    Several,
    Many,
    Hairy,
}

impl Band {
    const STRAIGHT: Bound = Bound::Most(1);
    const FEW: Bound = Bound::Most(3);
    const SEVERAL: Bound = Bound::Most(6);
    const MANY: Bound = Bound::Most(10);

    pub(crate) fn containing(score: Branching) -> Self {
        [
            (Self::STRAIGHT, Self::Straight),
            (Self::FEW, Self::Few),
            (Self::SEVERAL, Self::Several),
            (Self::MANY, Self::Many),
        ]
        .into_iter()
        .find(|(limit, _)| limit.allows(score).is_some())
        .map_or(Self::Hairy, |(_, band)| band)
    }
}

impl fmt::Display for Band {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Straight => write!(formatter, "{}", Self::STRAIGHT),
            Self::Few => write!(formatter, "{}-{}", Self::STRAIGHT.limit() + 1, Self::FEW),
            Self::Several => write!(formatter, "{}-{}", Self::FEW.limit() + 1, Self::SEVERAL),
            Self::Many => write!(formatter, "{}-{}", Self::SEVERAL.limit() + 1, Self::MANY),
            Self::Hairy => write!(formatter, "{}+", Self::MANY.limit() + 1),
        }
    }
}
