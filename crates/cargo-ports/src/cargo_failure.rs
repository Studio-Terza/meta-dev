use std::io;
use std::path::PathBuf;
use std::process::ExitStatus;

use file_ports::Text;

#[derive(Debug, thiserror::Error)]
pub enum CargoFailure {
    #[error("cannot start cargo metadata for {manifest}: {source}", manifest = manifest.display())]
    Unstarted { manifest: PathBuf, source: io::Error },
    #[error("cargo metadata for {manifest} exited with {status}: {stderr}", manifest = manifest.display())]
    Exited { manifest: PathBuf, status: ExitStatus, stderr: Text },
    #[error("no cargo metadata is recorded for {manifest}", manifest = manifest.display())]
    Unrecorded { manifest: PathBuf },
}
