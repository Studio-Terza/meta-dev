use std::collections::BTreeMap;
use std::error::Error;
use std::sync::{Mutex, PoisonError};

use newtype::Counter;

use super::calls::Calls;
use super::op::OpName;
use super::snapshot::Snapshot;

#[derive(Debug, thiserror::Error)]
#[error("releasing {resource} failed: {failure}")]
pub struct ReleaseFailure {
    resource: OpName,
    #[source]
    failure: Box<dyn Error + Send + Sync>,
}

#[derive(Debug, thiserror::Error)]
#[error("{first} ({} further release failures)", .rest.len())]
pub struct ReleaseFailures {
    first: ReleaseFailure,
    rest: Vec<ReleaseFailure>,
}

impl ReleaseFailures {
    fn joined(
        settled: Result<Snapshot, ReleaseFailures>,
        failure: ReleaseFailure,
    ) -> Result<Snapshot, ReleaseFailures> {
        match settled {
            Ok(_snapshot) => Err(ReleaseFailures { first: failure, rest: Vec::new() }),
            Err(failures) => Err(failures.appended(failure)),
        }
    }

    fn appended(mut self, failure: ReleaseFailure) -> ReleaseFailures {
        self.rest.push(failure);
        self
    }
}

#[derive(Debug, Default)]
struct Ledger {
    calls: Mutex<BTreeMap<OpName, Calls>>,
    failures: Mutex<Vec<ReleaseFailure>>,
}

#[derive(Debug)]
pub struct Measured {
    snapshot: Snapshot,
    failures: Vec<ReleaseFailure>,
}

impl Measured {
    #[must_use]
    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }

    pub fn settled(self) -> Result<Snapshot, ReleaseFailures> {
        let Measured { snapshot, failures } = self;
        failures.into_iter().fold(Ok(snapshot), ReleaseFailures::joined)
    }
}

#[must_use = "a run counts until it is finished"]
#[derive(Debug)]
pub struct Run {
    ledger: Ledger,
}

impl Run {
    pub fn root() -> Run {
        Run { ledger: Ledger::default() }
    }

    pub fn finish(self) -> Measured {
        let calls = self.ledger.calls.into_inner().unwrap_or_else(PoisonError::into_inner);
        let failures = self.ledger.failures.into_inner().unwrap_or_else(PoisonError::into_inner);
        Measured { snapshot: Snapshot::counted(calls), failures }
    }

    pub(crate) fn count(&self, name: OpName) {
        let mut calls = self.ledger.calls.lock().unwrap_or_else(PoisonError::into_inner);
        let entry = calls.entry(name).or_default();
        *entry = entry.incremented();
    }

    pub(crate) fn fail(&self, resource: OpName, failure: Box<dyn Error + Send + Sync>) {
        self.ledger.failures.lock().unwrap_or_else(PoisonError::into_inner).push(ReleaseFailure { resource, failure });
    }
}
