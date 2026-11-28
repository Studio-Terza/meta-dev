mod boundary;

use std::error::Error;
use std::fmt;
use std::mem;
use std::sync::Mutex;

use invariants::Unpoisoned;
use pipeline::Measure;

use super::op::OpName;
use super::snapshot::Snapshot;
use boundary::Told;

#[derive(Debug, thiserror::Error)]
#[error("releasing {resource} failed: {failure}")]
pub struct ReleaseFailure {
    resource: OpName,
    failure: Told,
}

#[derive(Debug, Default, Measure)]
pub struct ReleaseFailures(Vec<ReleaseFailure>);

#[derive(Debug, Default)]
struct Ledger {
    calls: Mutex<Snapshot>,
    failures: Mutex<ReleaseFailures>,
}

#[derive(Debug)]
pub struct Measured {
    snapshot: Snapshot,
    failures: ReleaseFailures,
}

impl ReleaseFailures {
    fn failed(&mut self, failure: ReleaseFailure) {
        *self = mem::take(self).joined(ReleaseFailures(vec![failure]));
    }
}

impl Error for ReleaseFailures {}

impl fmt::Display for ReleaseFailures {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
            .split_first()
            .map_or(Ok(()), |(first, rest)| write!(formatter, "{first} ({} further release failures)", rest.len()))
    }
}

impl Measured {
    #[must_use]
    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }

    pub fn settled(self) -> Result<Snapshot, ReleaseFailures> {
        let Measured { snapshot, failures } = self;
        match failures.0.first() {
            None => Ok(snapshot),
            Some(_first) => Err(failures),
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
        self.ledger.failures.lock().unpoisoned().failed(ReleaseFailure { resource, failure: Told::from(failure) });
    }
}
