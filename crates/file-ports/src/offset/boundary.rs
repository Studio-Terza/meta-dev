use invariants::Ignoring;
use newtype::{Arbitrary, Checked, Display, Newtype};

use crate::Ignored;

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[newtype(checked, strategy = 0..=i64::MAX)]
pub struct Offset(i64);

#[derive(Debug, thiserror::Error)]
#[error("an offset into a file is never negative, not {0}")]
pub struct Negative(i64);

impl Checked for Offset {
    type Error = Negative;

    fn checked(inner: &i64) -> Result<&i64, Negative> {
        u64::try_from(*inner).map(|_unsigned| inner).map_err(|_signed| Negative(*inner))
    }
}

impl Offset {
    #[must_use]
    pub fn advanced(self, consumed: usize) -> Option<Offset> {
        i64::try_from(consumed)
            .ignoring(Ignored::BeyondRange)
            .and_then(|counted| self.0.checked_add(counted))
            .map(Offset)
    }

    #[must_use]
    pub fn position(self) -> u64 {
        self.0.unsigned_abs()
    }
}
