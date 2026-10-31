use crate::ids::{LeaseId, OwnerQuestion, Timestamp};
use std::collections::BTreeSet;

/// A task waiting on the owner: since when, the question, and the leases whose deadlines pause.
/// A lease issued during the wait is not among them.
#[derive(Debug)]
pub(crate) struct Waiting {
    since: Timestamp,
    question: OwnerQuestion,
    leases: BTreeSet<LeaseId>,
}

impl Waiting {
    pub(crate) fn since(since: Timestamp, question: OwnerQuestion, leases: BTreeSet<LeaseId>) -> Self {
        Self { since, question, leases }
    }

    pub(crate) fn started(&self) -> Timestamp {
        self.since
    }

    pub(crate) fn question(&self) -> &OwnerQuestion {
        &self.question
    }

    pub(crate) fn covers(&self, lease: LeaseId) -> bool {
        self.leases.contains(&lease)
    }
}
