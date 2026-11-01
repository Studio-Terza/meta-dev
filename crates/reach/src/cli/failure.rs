use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Failure {
    #[error("cannot read {path}: {source}", path = path.display())]
    Read { path: PathBuf, source: io::Error },
    #[error("cannot write the report: {0}")]
    Write(#[from] io::Error),
    #[error("cannot parse cargo metadata: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("{manifest} is not inside the workspace", manifest = manifest.display())]
    Outside { manifest: PathBuf },
    #[error(
        "usage: reach affected <metadata> [<changed file>...] | reach history <metadata> [<changed file>...] | reach fan-in <metadata> | reach sources <metadata> <crate>..."
    )]
    Usage,
}
