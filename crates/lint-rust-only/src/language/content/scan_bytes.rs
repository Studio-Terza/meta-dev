mod boundary;

use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug)]
pub(crate) struct ScanBytes(u64);
