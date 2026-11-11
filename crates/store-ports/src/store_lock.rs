mod refusal;

use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use port_core::{Pass, Release, ReleaseReceipt, Released, Resource};

use crate::Busy;
use refusal::Refusal;

#[derive(Debug, Resource)]
pub struct StoreLock {
    hold: Hold,
}

#[derive(Debug)]
enum Hold {
    File(File),
    Memory(Arc<Mutex<Claim>>),
}

#[derive(Debug, Default)]
pub enum Claim {
    #[default]
    Free,
    Taken,
}

#[derive(Debug, thiserror::Error)]
#[error("cannot unlock the store lock file: {0}")]
pub struct Unlock(#[from] io::Error);

impl StoreLock {
    pub fn filed(lock: PathBuf, open: impl FnOnce(&Path) -> io::Result<File>) -> Result<StoreLock, Busy> {
        open(&lock)
            .map_err(Refusal::Unopened)
            .and_then(|file| file.try_lock().map(|()| file).map_err(Refusal::from))
            .map(|file| StoreLock { hold: Hold::File(file) })
            .map_err(|refusal| refusal.busy(lock))
    }

    pub fn claimed(claim: &Arc<Mutex<Claim>>) -> Result<StoreLock, Busy> {
        let previous = std::mem::replace(&mut *claim.lock().unwrap_or_else(PoisonError::into_inner), Claim::Taken);
        previous.taken(claim)
    }
}

impl Claim {
    fn taken(self, claim: &Arc<Mutex<Claim>>) -> Result<StoreLock, Busy> {
        match self {
            Claim::Free => Ok(StoreLock { hold: Hold::Memory(Arc::clone(claim)) }),
            Claim::Taken => Err(Busy::Claimed),
        }
    }

    fn freed(claim: &Mutex<Claim>, pass: Pass<'_, Released<StoreLock>>) -> ReleaseReceipt {
        *claim.lock().unwrap_or_else(PoisonError::into_inner) = Claim::Free;
        pass.released()
    }
}

impl Release for StoreLock {
    type Failure = Unlock;

    fn release(&mut self, pass: Pass<'_, Released<StoreLock>>) -> Result<ReleaseReceipt, Unlock> {
        match &self.hold {
            Hold::File(file) => file.unlock().map(|()| pass.released()).map_err(Unlock::from),
            Hold::Memory(claim) => Ok(Claim::freed(claim, pass)),
        }
    }
}
