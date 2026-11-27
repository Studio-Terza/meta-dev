use std::ffi::OsString;
use std::io;
use std::path::PathBuf;

use dispatch_ports::DispatchFailure;
use file_ports::FileFailure;
use port_core::{ReleaseFailures, Snapshot};
use store_ports::{Busy, StoreFailure, Unlock};
use transcript_ports::ReadFailure;

use crate::Outcome;

#[derive(Debug, thiserror::Error)]
pub enum Failure {
    #[error(
        "usage: design [--workspace <dir>] (ingest [--dry-run] | stops | hook (stop | subagent-stop | --settings))"
    )]
    Usage,
    #[error("the environment does not name {0:?}")]
    Unset(OsString),
    #[error("the workspace path {0:?} is not text")]
    Unreadable(PathBuf),
    #[error("cannot reach the transcripts: {0}")]
    Files(#[from] FileFailure),
    #[error("cannot read the transcripts: {0}")]
    Read(#[from] ReadFailure),
    #[error("the design store lock was refused: {0}")]
    Busy(#[from] Busy),
    #[error("the design store failed: {0}")]
    Store(#[from] StoreFailure),
    #[error("cannot release the design store lock: {0}")]
    Unlock(#[from] Unlock),
    #[error("the reading agent could not be run: {0}")]
    Dispatch(#[from] DispatchFailure),
    #[error("cannot hear the hook input: {0}")]
    Deaf(#[source] io::Error),
    #[error("cannot read the hook input: {0}")]
    Unheard(#[source] serde_json::Error),
    #[error("cannot add the escalation to the discussion queue: {0}")]
    Unqueued(#[source] FileFailure),
    #[error("cannot render the hook settings: {0}")]
    Render(#[source] serde_json::Error),
    #[error("a resource was not released cleanly: {0}")]
    Released(#[from] ReleaseFailures),
    #[error("{command}; on the way out, a resource was not released cleanly either: {released}")]
    Unreleased { command: Box<Failure>, released: ReleaseFailures },
}

impl Failure {
    pub(crate) fn settled(
        outcome: Result<Outcome, Failure>,
        released: Result<Snapshot, ReleaseFailures>,
    ) -> Result<Outcome, Failure> {
        match released {
            Ok(_snapshot) => outcome,
            Err(failures) => Err(Failure::unreleased(outcome, failures)),
        }
    }

    fn unreleased(outcome: Result<Outcome, Failure>, failures: ReleaseFailures) -> Failure {
        match outcome {
            Ok(_done) => Failure::Released(failures),
            Err(command) => Failure::Unreleased { command: Box::new(command), released: failures },
        }
    }
}
