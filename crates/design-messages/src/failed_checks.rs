mod boundary;

use port_core::Op;

pub use boundary::{CheckName, PullNumber};

#[derive(Clone, Copy, Debug, Op)]
pub struct FailedBefore(PullNumber);

#[derive(Debug, Op)]
pub struct Failing {
    pull: PullNumber,
    checks: Vec<CheckName>,
}

impl From<PullNumber> for FailedBefore {
    fn from(pull: PullNumber) -> FailedBefore {
        FailedBefore(pull)
    }
}

impl Failing {
    #[must_use]
    pub fn new(pull: PullNumber, checks: Vec<CheckName>) -> Failing {
        Failing { pull, checks }
    }
}
