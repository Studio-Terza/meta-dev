use std::ffi::OsString;
use std::io;
use std::path::PathBuf;

use file_ports::FileFailure;
use transcript_ports::{ReadFailure, Uuid};

#[derive(Debug, thiserror::Error)]
pub enum Failure {
    #[error("usage: design ingest [--workspace <dir>] | design show <uuid> [--workspace <dir>]")]
    Usage,
    #[error("the environment does not name {0:?}")]
    Unset(OsString),
    #[error("the workspace path {0:?} is not text")]
    Unreadable(PathBuf),
    #[error("{0}")]
    Files(#[from] FileFailure),
    #[error("{0}")]
    Read(#[from] ReadFailure),
    #[error("cannot write the report: {0}")]
    Write(#[from] io::Error),
    #[error("no message {0} in the transcripts")]
    Missing(Uuid),
}
