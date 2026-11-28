use newtype::{Arbitrary, Newtype};
use pipeline::Measure;

#[derive(Newtype, Arbitrary, Measure, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[must_use = "a write receipt says how many rows the store changed; use it or do not ask for it"]
pub struct Written(usize);

pub trait Done: Measure {
    fn changed(rows: usize) -> Self;
}

impl Done for () {
    fn changed(_rows: usize) {}
}

impl Done for Written {
    fn changed(rows: usize) -> Written {
        Written(rows)
    }
}
