use std::error::Error;

use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
pub struct Refusal(String);

impl Refusal {
    pub(super) fn told(refusal: &dyn Error) -> Refusal {
        Refusal(refusal.to_string())
    }
}
