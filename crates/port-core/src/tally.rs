use std::any::{TypeId, type_name};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, PoisonError, RwLock};

use newtype::Newtype;

use crate::{Calls, Op, OpName};

#[derive(Newtype, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct TypeName(&'static str);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    name: OpName,
    type_name: TypeName,
    id: TypeId,
}

impl Key {
    fn identifying<O: Op + 'static>() -> Key {
        Key { name: O::NAME, type_name: TypeName::new(type_name::<O>()), id: TypeId::of::<O>() }
    }
}

#[derive(Debug)]
struct Slot {
    name: OpName,
    calls: AtomicUsize,
}

#[derive(Debug, Default)]
pub struct Tally(RwLock<BTreeMap<Key, Arc<Slot>>>);

impl Tally {
    pub(crate) fn record<O: Op + 'static>(&self) {
        let found = self.0.read().unwrap_or_else(PoisonError::into_inner).get(&Key::identifying::<O>()).cloned();
        let slot = found.unwrap_or_else(|| {
            let mut slots = self.0.write().unwrap_or_else(PoisonError::into_inner);
            Arc::clone(
                slots
                    .entry(Key::identifying::<O>())
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
pub struct Snapshot(BTreeMap<Key, Counted>);

impl Snapshot {
    #[must_use]
    pub fn calls<O: Op + 'static>(&self) -> Calls {
        self.0.get(&Key::identifying::<O>()).map_or_else(Calls::default, Counted::calls)
    }

    pub fn iter(&self) -> impl Iterator<Item = Counted> + '_ {
        self.0.values().copied()
    }
}
