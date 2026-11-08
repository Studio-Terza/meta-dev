use newtype::{Arbitrary, Display, Newtype};

use super::DispatchFailure;

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
pub struct Fault(String);

impl From<DispatchFailure> for Fault {
    fn from(failure: DispatchFailure) -> Fault {
        Fault(failure.to_string())
    }
}
