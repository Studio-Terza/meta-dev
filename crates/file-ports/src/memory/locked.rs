use std::sync::{Mutex, MutexGuard, PoisonError};

#[derive(Debug, Default)]
pub struct Locked<Held>(Mutex<Held>);

impl<Held> Locked<Held> {
    pub const fn new(held: Held) -> Self {
        Self(Mutex::new(held))
    }

    pub fn held(&self) -> MutexGuard<'_, Held> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
