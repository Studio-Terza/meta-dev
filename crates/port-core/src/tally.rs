use std::any::TypeId;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::marker::PhantomData;

use newtype::Counter;

use crate::{Calls, Metered, OpName};

type Counts = BTreeMap<OpName, Calls>;

thread_local! {
    static REGISTRY: RefCell<BTreeMap<TypeId, Counts>> = const { RefCell::new(BTreeMap::new()) };
}

#[derive(Debug)]
pub struct Tally<P>(PhantomData<fn() -> P>);

impl<P: 'static> Metered<P> {
    #[must_use]
    pub fn tally(&self) -> Tally<P> {
        Tally(PhantomData)
    }
}

impl<P: 'static> Tally<P> {
    pub(crate) fn record(name: OpName) {
        REGISTRY.with_borrow_mut(|registry| {
            let counts = registry.entry(TypeId::of::<P>()).or_default();
            let calls = counts.entry(name).or_insert_with(Calls::zero);
            *calls = calls.incremented();
        });
    }

    #[must_use]
    pub fn calls(&self, name: OpName) -> Calls {
        self.snapshot().get(&name).copied().unwrap_or_else(Calls::zero)
    }

    #[must_use]
    pub fn snapshot(&self) -> BTreeMap<OpName, Calls> {
        REGISTRY.with_borrow(|registry| {
            registry.get(&TypeId::of::<P>()).into_iter().flatten().map(|(name, calls)| (*name, *calls)).collect()
        })
    }

    pub fn reset(&self) {
        REGISTRY.with_borrow_mut(|registry| registry.remove(&TypeId::of::<P>()));
    }
}
