use std::fmt;

use lint_core::Most;

use super::super::score::Score;
use newtype::Counter;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Band {
    Straight,
    Few,
    Several,
    Many,
    Hairy,
}

impl Band {
    const STRAIGHT: Most = Most::new(1);
    const FEW: Most = Most::new(3);
    const SEVERAL: Most = Most::new(6);
    const MANY: Most = Most::new(10);

    pub(crate) fn containing(score: Score) -> Band {
        let at = score.count();
        [
            (Self::STRAIGHT, Band::Straight),
            (Self::FEW, Band::Few),
            (Self::SEVERAL, Band::Several),
            (Self::MANY, Band::Many),
        ]
        .into_iter()
        .find(|(limit, _)| limit.allows(at))
        .map_or(Band::Hairy, |(_, band)| band)
    }
}

impl fmt::Display for Band {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Band::Straight => write!(formatter, "{}", Self::STRAIGHT),
            Band::Few => write!(formatter, "{}-{}", Self::STRAIGHT.count() + 1, Self::FEW),
            Band::Several => write!(formatter, "{}-{}", Self::FEW.count() + 1, Self::SEVERAL),
            Band::Many => write!(formatter, "{}-{}", Self::SEVERAL.count() + 1, Self::MANY),
            Band::Hairy => write!(formatter, "{}+", Self::MANY.count() + 1),
        }
    }
}
