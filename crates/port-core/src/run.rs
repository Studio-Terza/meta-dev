use std::collections::BTreeMap;
use std::sync::{Mutex, PoisonError};

use newtype::Counter;

use super::calls::Calls;
use super::op::OpName;
use super::snapshot::Snapshot;

#[derive(Debug, Default)]
struct Ledger {
    calls: Mutex<BTreeMap<OpName, Calls>>,
}

#[derive(Debug)]
pub struct Measured {
    snapshot: Snapshot,
}

impl Measured {
    #[must_use]
    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
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
        Measured { snapshot: Snapshot::counted(calls) }
    }

    pub(crate) fn count(&self, name: OpName) {
        let mut calls = self.ledger.calls.lock().unwrap_or_else(PoisonError::into_inner);
        let entry = calls.entry(name).or_default();
        *entry = entry.incremented();
    }
}
