use std::ffi::OsString;
use std::io;
use std::path::PathBuf;

use dispatch_ports::DispatchFailure;
use file_ports::FileFailure;
use store_ports::{Busy, StoreFailure};
use transcript_ports::ReadFailure;

#[derive(Debug, thiserror::Error)]
pub enum Failure {
    #[error(
        "usage: design [--workspace <dir>] (ingest [--dry-run] | stops | hook (stop | subagent-stop | post-tool-use | --settings))"
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
    #[error("the reading agent could not be run: {0}")]
    Dispatch(#[from] DispatchFailure),
    #[error("cannot hear the hook input: {0}")]
    Deaf(#[source] io::Error),
    #[error("cannot read the hook input: {0}")]
    Unheard(#[source] serde_json::Error),
    #[error("cannot add the escalation to the discussion queue: {0}")]
    Unqueued(#[source] FileFailure),
    #[error("cannot add the escalation to the discussion queue: the workspace local.toml names no absolute queue path")]
    Unplaced,
    #[error("cannot render the hook settings: {0}")]
    Render(#[source] serde_json::Error),
}
