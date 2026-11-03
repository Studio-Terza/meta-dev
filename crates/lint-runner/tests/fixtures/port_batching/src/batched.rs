use crate::ports::{Id, Store};

pub struct Shelf;

impl Shelf {
    pub fn all(store: &impl Store, ids: &[Id]) -> Vec<Id> {
        store.fetch_many(ids)
    }
}
