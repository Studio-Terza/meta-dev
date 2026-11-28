use invariants::Ignoring;
use newtype::{Arbitrary, Newtype};

use crate::{Bytes, Ignored};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Size(u64);

impl From<&Bytes> for Size {
    fn from(bytes: &Bytes) -> Size {
        Size(u64::try_from(bytes.as_ref().len()).ignoring(Ignored::BeyondRange).unwrap_or(u64::MAX))
    }
}
