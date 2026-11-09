use std::array::TryFromSliceError;
use std::borrow::Cow;
use std::io;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum Failure<'path> {
    #[error("usage: mutation-report <shards> <sources> <report.md> <report.json>")]
    Usage(#[source] TryFromSliceError),
    #[error("cannot use {path}: {source}", path = path.display())]
    Io { path: Cow<'path, Path>, source: io::Error },
    #[error("cannot parse {path}: {source}", path = path.display())]
    Parse { path: Cow<'path, Path>, source: serde_json::Error },
}
