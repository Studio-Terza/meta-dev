use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Failure {
    #[error("usage: mutation-report <shards> <sources> <report.md> <report.json>")]
    Usage,
    #[error("cannot use {path}: {source}", path = path.display())]
    Io { path: PathBuf, source: io::Error },
    #[error("cannot parse {path}: {source}", path = path.display())]
    Parse { path: PathBuf, source: serde_json::Error },
}
