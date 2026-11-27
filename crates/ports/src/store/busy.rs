use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Busy {
    #[error("the store lock {0:?} is held by another ingest; wait for it to finish, then run again")]
    Taken(PathBuf),
    #[error("the in-memory store is locked by another holder; end that hold first")]
    Claimed,
    #[error("cannot open the store lock {path:?}: {source}")]
    Unopened { path: PathBuf, source: io::Error },
}
