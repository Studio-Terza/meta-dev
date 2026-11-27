use std::collections::BTreeMap;
use std::sync::Mutex;

use invariants::Unpoisoned;
use newtype::Counter;

use super::boundary::{Calls, OpName};
use super::snapshot::Snapshot;

#[must_use = "a run counts until it is finished"]
#[derive(Debug)]
pub struct Run {
    calls: Mutex<BTreeMap<OpName, Calls>>,
}

impl Run {
    pub fn root() -> Run {
        Run { calls: Mutex::default() }
    }

    pub fn finish(self) -> Snapshot {
        Snapshot::counted(self.calls.into_inner().unpoisoned())
    }

    pub(crate) fn count(&self, name: OpName) {
        let mut calls = self.calls.lock().unpoisoned();
        let entry = calls.entry(name).or_default();
        *entry = entry.incremented();
    }
}
