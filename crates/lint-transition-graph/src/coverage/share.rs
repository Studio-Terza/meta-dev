use std::fmt;

use newtype::Counter;

use crate::count::Count;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Share(f64);

impl Share {
    pub(super) fn between(part: Count, whole: Count) -> Share {
        let ratio = |count: Count| f64::from(u32::try_from(count.count()).unwrap_or(u32::MAX));
        Share(if whole == Count::zero() { 0.0 } else { ratio(part) / ratio(whole) })
    }
}

impl fmt::Display for Share {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:.3}", self.0)
    }
}
