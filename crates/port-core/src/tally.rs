use std::cell::RefCell;
use std::collections::BTreeMap;

use newtype::Counter;

use crate::{Calls, OpName};

#[derive(Debug, Default)]
pub struct Tally(RefCell<BTreeMap<OpName, Calls>>);

impl Tally {
    pub(crate) fn record(&self, name: OpName) {
        let mut counts = self.0.borrow_mut();
        let calls = counts.entry(name).or_insert_with(Calls::zero);
        *calls = calls.incremented();
    }

    #[must_use]
    pub fn calls(&self, name: OpName) -> Calls {
        self.0.borrow().get(&name).copied().unwrap_or_else(Calls::zero)
    }

    #[must_use]
    pub fn snapshot(&self) -> BTreeMap<OpName, Calls> {
        self.0.borrow().iter().map(|(name, calls)| (*name, *calls)).collect()
    }
}
