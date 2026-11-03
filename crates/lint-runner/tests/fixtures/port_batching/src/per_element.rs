use crate::ports::{Id, Store};

pub struct Shelf;

impl Shelf {
    pub fn each(store: &impl Store, ids: &[Id]) -> Vec<Id> {
        ids.iter().map(|id| store.fetch(id)).collect()
    }

    pub fn looped(store: &impl Store, ids: &[Id]) {
        for id in ids {
            store.fetch(id);
        }
    }
}
