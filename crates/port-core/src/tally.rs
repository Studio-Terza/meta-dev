use std::any::TypeId;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, PoisonError, RwLock};

use crate::{Calls, Op, OpName};

#[derive(Debug)]
struct Slot {
    name: OpName,
    calls: AtomicUsize,
}

#[derive(Debug, Default)]
pub struct Tally(RwLock<BTreeMap<TypeId, Arc<Slot>>>);

impl Tally {
    pub(crate) fn record<O: Op + 'static>(&self) {
        let found = self.0.read().unwrap_or_else(PoisonError::into_inner).get(&TypeId::of::<O>()).cloned();
        let slot = found.unwrap_or_else(|| {
            let mut slots = self.0.write().unwrap_or_else(PoisonError::into_inner);
            Arc::clone(
                slots
                    .entry(TypeId::of::<O>())
                    .or_insert_with(|| Arc::new(Slot { name: O::NAME, calls: AtomicUsize::new(0) })),
            )
        });
        slot.calls.fetch_add(1, Ordering::Relaxed);
    }

    #[must_use]
    pub fn snapshot(&self) -> Snapshot {
        let slots = self.0.read().unwrap_or_else(PoisonError::into_inner);
        Snapshot(slots.iter().map(|(key, slot)| (*key, Counted::from(slot.as_ref()))).collect())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Counted {
    name: OpName,
    calls: Calls,
}

impl From<&Slot> for Counted {
    fn from(slot: &Slot) -> Counted {
        Counted { name: slot.name, calls: Calls::new(slot.calls.load(Ordering::Relaxed)) }
    }
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot(BTreeMap<TypeId, Counted>);

impl Snapshot {
    #[must_use]
    pub fn calls<O: Op + 'static>(&self) -> Calls {
        self.0.get(&TypeId::of::<O>()).map_or_else(Calls::default, Counted::calls)
    }

    pub fn iter(&self) -> impl Iterator<Item = Counted> + '_ {
        self.0.values().copied()
    }
}
