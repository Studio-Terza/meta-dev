use std::sync::{Mutex, MutexGuard};

use invariants::Unpoisoned;

#[derive(Debug, Default)]
pub struct Locked<Held>(Mutex<Held>);

impl<Held> Locked<Held> {
    pub fn new(held: Held) -> Locked<Held> {
        Locked(Mutex::new(held))
    }

    pub fn held(&self) -> MutexGuard<'_, Held> {
        self.0.lock().unpoisoned()
    }
}
