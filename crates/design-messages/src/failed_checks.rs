mod boundary;

use port_core::Op;

pub use boundary::{CheckName, CommitSha, PullNumber};

#[derive(Clone, Copy, Debug, Op)]
pub struct FailedBefore {
    pull: PullNumber,
    head: CommitSha,
}

#[derive(Debug, Op)]
pub struct Failing {
    pull: PullNumber,
    head: CommitSha,
    checks: Vec<CheckName>,
}

impl FailedBefore {
    #[must_use]
    pub fn new(pull: PullNumber, head: CommitSha) -> FailedBefore {
        FailedBefore { pull, head }
    }
}

impl Failing {
    #[must_use]
    pub fn new(pull: PullNumber, head: CommitSha, checks: Vec<CheckName>) -> Failing {
        Failing { pull, head, checks }
    }
}
