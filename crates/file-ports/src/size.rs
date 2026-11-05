use std::iter::Sum;

use newtype::{Arbitrary, Newtype};

use crate::Bytes;

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Size(u64);

impl From<&Bytes> for Size {
    fn from(bytes: &Bytes) -> Size {
        Size(u64::try_from(bytes.as_ref().len()).unwrap_or(u64::MAX))
    }
}

impl Sum for Size {
    fn sum<Items: Iterator<Item = Size>>(items: Items) -> Size {
        Size(items.fold(0_u64, |total, size| total.saturating_add(size.0)))
    }
}
