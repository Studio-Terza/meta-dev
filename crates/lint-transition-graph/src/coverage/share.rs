use std::fmt;

use newtype::Counter;

use crate::count::Count;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Share(f64);

impl Share {
    pub(crate) fn between(part: Count, whole: Count) -> Share {
        let ratio = |count: Count| f64::from(u32::try_from(count.count()).unwrap_or(u32::MAX));
        Share(Some(whole).filter(|whole| *whole != Count::zero()).map_or(0.0, |whole| ratio(part) / ratio(whole)))
    }
}

impl fmt::Display for Share {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:.3}", self.0)
    }
}
