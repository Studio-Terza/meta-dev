use std::collections::BTreeMap;
use std::mem;

use pipeline::Measure;

use super::calls::Calls;
use super::op::{Op, OpName};

#[derive(Clone, Debug, PartialEq, Eq, Measure)]
pub struct Snapshot(BTreeMap<OpName, Calls>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Counted {
    name: OpName,
    calls: Calls,
}

impl Counted {
    #[must_use]
    pub fn name(&self) -> OpName {
        self.name
    }

    #[must_use]
    pub fn calls(&self) -> Calls {
        self.calls
    }
}

impl Snapshot {
    #[must_use]
    pub fn calls<O: Op>(&self) -> Calls {
        self.named(O::NAME)
    }

    #[must_use]
    pub fn named(&self, name: OpName) -> Calls {
        self.0.get(&name).copied().unwrap_or_default()
    }

    pub fn iter(&self) -> impl Iterator<Item = Counted> + '_ {
        self.0.iter().map(|(name, calls)| Counted { name: *name, calls: *calls })
    }

    pub(crate) fn count(&mut self, name: OpName) {
        let called = Snapshot(BTreeMap::from([(name, Calls::new(1))]));
        *self = mem::take(self).joined(called);
    }
}
