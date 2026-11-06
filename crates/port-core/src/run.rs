use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::sync::{Mutex, PoisonError};

use newtype::Counter;

use super::calls::Calls;
use super::op::OpName;
use super::snapshot::Snapshot;

#[derive(Debug)]
pub struct ReleaseFailure {
    resource: OpName,
    failure: Box<dyn Error + Send + Sync>,
}

impl ReleaseFailure {
    pub(crate) fn new(resource: OpName, failure: Box<dyn Error + Send + Sync>) -> Self {
        ReleaseFailure { resource, failure }
    }

    #[must_use]
    pub fn resource(&self) -> OpName {
        self.resource
    }

    #[must_use]
    pub fn failure(&self) -> &(dyn Error + Send + Sync + 'static) {
        &*self.failure
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

    #[must_use]
    pub fn failures(&self) -> &[ReleaseFailure] {
        &self.failures
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rooting {
    Shared,
    Sole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootRefused;

impl fmt::Display for RootRefused {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a sole run refuses every other root while it lives")
    }
}

impl Error for RootRefused {}

#[derive(Debug, Clone, Copy)]
enum Roots {
    Shared(Calls),
    Sole,
}

static ROOTS: Mutex<Roots> = Mutex::new(Roots::Shared(Calls::new(0)));

impl Rooting {
    fn admitting(self, live: Calls) -> Result<Roots, RootRefused> {
        match self {
            Rooting::Shared => Ok(Roots::Shared(live.incremented())),
            Rooting::Sole => Roots::alone(live),
        }
    }
}

impl Roots {
    fn admitted(self, rooting: Rooting) -> Result<Roots, RootRefused> {
        self.live().and_then(|live| rooting.admitting(live))
    }

    fn live(self) -> Result<Calls, RootRefused> {
        match self {
            Roots::Shared(live) => Ok(live),
            Roots::Sole => Err(RootRefused),
        }
    }

    fn alone(live: Calls) -> Result<Roots, RootRefused> {
        (live == Calls::zero()).then_some(Roots::Sole).ok_or(RootRefused)
    }

    fn left(self) -> Roots {
        match self {
            Roots::Shared(live) => Roots::Shared(Calls::new(live.count().saturating_sub(1))),
            Roots::Sole => Roots::Shared(Calls::zero()),
        }
    }
}

#[must_use = "a run counts until it is finished"]
#[derive(Debug)]
pub struct Run {
    ledger: Ledger,
}

impl Run {
    pub fn root(rooting: Rooting) -> Result<Run, RootRefused> {
        let mut roots = ROOTS.lock().unwrap_or_else(PoisonError::into_inner);
        roots.admitted(rooting).map(|admitted| {
            *roots = admitted;
            Run { ledger: Ledger::default() }
        })
    }

    pub fn finish(self) -> Measured {
        let calls = std::mem::take(&mut *self.ledger.calls.lock().unwrap_or_else(PoisonError::into_inner));
        let failures = std::mem::take(&mut *self.ledger.failures.lock().unwrap_or_else(PoisonError::into_inner));
        Measured { snapshot: Snapshot::counted(calls), failures }
    }

    pub(crate) fn count(&self, name: OpName) {
        let mut calls = self.ledger.calls.lock().unwrap_or_else(PoisonError::into_inner);
        let entry = calls.entry(name).or_default();
        *entry = entry.incremented();
    }

    pub(crate) fn fail(&self, failure: ReleaseFailure) {
        self.ledger.failures.lock().unwrap_or_else(PoisonError::into_inner).push(failure);
    }
}

impl Drop for Run {
    fn drop(&mut self) {
        let mut roots = ROOTS.lock().unwrap_or_else(PoisonError::into_inner);
        *roots = roots.left();
    }
}
