mod ignored;

use std::fmt;

use newtype::Counter;

use crate::count::Count;
use crate::coverage::share::ignored::Ignored;
use invariants::Ignoring;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Share(f64);

impl Share {
    pub(crate) fn between(part: Count, whole: Count) -> Share {
        let ratio =
            |count: Count| f64::from(u32::try_from(count.count()).ignoring(Ignored::BeyondRange).unwrap_or(u32::MAX));
        Share(
            Some(whole)
                .filter(|candidate| *candidate != Count::zero())
                .map_or(0.0, |nonzero| ratio(part) / ratio(nonzero)),
        )
    }
}

impl fmt::Display for Share {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:.3}", self.0)
    }
}
