use std::error::Error;

use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
pub struct Reason(String);

impl Reason {
    pub(super) fn told(failure: &dyn Error) -> Reason {
        Reason(failure.to_string())
    }
}
