use std::fs::TryLockError;
use std::io;
use std::path::PathBuf;

use crate::Busy;

#[derive(Debug)]
pub(super) enum Refusal {
    Unopened(io::Error),
    Taken,
}

impl From<TryLockError> for Refusal {
    fn from(refused: TryLockError) -> Refusal {
        match refused {
            TryLockError::WouldBlock => Refusal::Taken,
            TryLockError::Error(source) => Refusal::Unopened(source),
        }
    }
}

impl Refusal {
    pub(super) fn busy(self, path: PathBuf) -> Busy {
        match self {
            Refusal::Unopened(source) => Busy::Unopened { path, source },
            Refusal::Taken => Busy::Taken(path),
        }
    }
}
