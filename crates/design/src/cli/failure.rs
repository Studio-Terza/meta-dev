use std::ffi::OsString;
use std::io;
use std::path::PathBuf;

use file_ports::FileFailure;
use port_core::ReleaseFailures;
use store_ports::{Busy, StoreFailure, Unlock};
use transcript_ports::{ReadFailure, Uuid};

#[derive(Debug, thiserror::Error)]
pub enum Failure {
    #[error("usage: design [--workspace <dir>] (ingest | show <uuid>)")]
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
}
