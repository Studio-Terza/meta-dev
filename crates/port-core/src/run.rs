mod boundary;

use std::error::Error;
use std::sync::Mutex;

use invariants::Unpoisoned;

use super::op::OpName;
use super::snapshot::Snapshot;
use boundary::Told;

#[derive(Debug, thiserror::Error)]
#[error("releasing {resource} failed: {failure}")]
pub struct ReleaseFailure {
    resource: OpName,
    failure: Told,
}

#[derive(Debug, thiserror::Error)]
#[error("{first} ({} further release failures)", .rest.len())]
pub struct ReleaseFailures {
    first: ReleaseFailure,
    rest: Vec<ReleaseFailure>,
}

#[derive(Debug, Default)]
struct Ledger {
    calls: Mutex<Snapshot>,
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
        let Measured { snapshot, mut failures } = self;
        match failures.first().map(|_first| ()) {
            None => Ok(snapshot),
            Some(()) => Err(ReleaseFailures { first: failures.remove(0), rest: failures }),
        }
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
        let calls = self.ledger.calls.into_inner().unpoisoned();
        let failures = self.ledger.failures.into_inner().unpoisoned();
        Measured { snapshot: calls, failures }
    }

    pub(crate) fn count(&self, name: OpName) {
        self.ledger.calls.lock().unpoisoned().count(name);
    }

    pub(crate) fn fail(&self, resource: OpName, failure: &impl Error) {
        self.ledger.failures.lock().unpoisoned().push(ReleaseFailure { resource, failure: Told::from(failure) });
    }
}
