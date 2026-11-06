use std::ffi::OsString;
use std::io;
use std::path::PathBuf;

use file_ports::FileFailure;
use port_core::{ReleaseFailures, Snapshot};
use store_ports::{Busy, StoreFailure, Unlock};
use transcript_ports::{ReadFailure, Uuid};

use crate::Outcome;

#[derive(Debug, thiserror::Error)]
pub enum Failure {
    #[error("usage: design [--workspace <dir>] [--dry-run] (ingest | show <uuid> | search [--all] <words...>)")]
    Usage,
    #[error("the environment does not name {0:?}")]
    Unset(OsString),
    #[error("the workspace path {0:?} is not text")]
    Unreadable(PathBuf),
    #[error("cannot reach the transcripts: {0}")]
    Files(#[from] FileFailure),
    #[error("cannot read the transcripts: {0}")]
    Read(#[from] ReadFailure),
    #[error("cannot write the report: {0}")]
    Write(#[from] io::Error),
    #[error("no message {0} in the design store; run design ingest first")]
    Missing(Uuid),
    #[error("ingest was refused the design store lock: {0}")]
    Busy(#[from] Busy),
    #[error("the design store failed: {0}")]
    Store(#[from] StoreFailure),
    #[error("cannot release the design store lock: {0}")]
    Unlock(#[from] Unlock),
    #[error("a resource was not released cleanly: {0}")]
    Released(#[from] ReleaseFailures),
    #[error("{command}; on the way out, a resource was not released cleanly either: {released}")]
    Unreleased { command: Box<Failure>, released: ReleaseFailures },
}

impl Failure {
    pub fn settled(
        outcome: Result<Outcome, Failure>,
        released: Result<Snapshot, ReleaseFailures>,
    ) -> Result<Outcome, Failure> {
        match outcome {
            Ok(outcome) => released.map(|_snapshot| outcome).map_err(Failure::Released),
            Err(command) => Err(Failure::unreleasing(command, released)),
        }
    }

    fn unreleasing(command: Failure, released: Result<Snapshot, ReleaseFailures>) -> Failure {
        match released {
            Ok(_snapshot) => command,
            Err(released) => Failure::Unreleased { command: Box::new(command), released },
        }
    }
}
