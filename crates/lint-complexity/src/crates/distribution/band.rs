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

    pub(crate) fn containing(score: Branching) -> Band {
        [
            (Self::STRAIGHT, Band::Straight),
            (Self::FEW, Band::Few),
            (Self::SEVERAL, Band::Several),
            (Self::MANY, Band::Many),
        ]
        .into_iter()
        .find(|(limit, _)| limit.allows(score).is_some())
        .map_or(Band::Hairy, |(_, band)| band)
    }
}

impl fmt::Display for Band {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Band::Straight => write!(formatter, "{}", Self::STRAIGHT),
            Band::Few => write!(formatter, "{}-{}", Self::STRAIGHT.limit() + 1, Self::FEW),
            Band::Several => write!(formatter, "{}-{}", Self::FEW.limit() + 1, Self::SEVERAL),
            Band::Many => write!(formatter, "{}-{}", Self::SEVERAL.limit() + 1, Self::MANY),
            Band::Hairy => write!(formatter, "{}+", Self::MANY.limit() + 1),
        }
    }
}
