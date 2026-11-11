use std::io;
use std::path::PathBuf;

use workspace_metadata::Unreadable;

use crate::line_number::LineNumber;

#[derive(Debug, thiserror::Error)]
pub enum Failure {
    #[error("cannot read {path}: {source}", path = path.display())]
    Io { path: PathBuf, source: io::Error },
    #[error("cannot parse {path}:{line}: {source}", path = path.display())]
    Parse { path: PathBuf, line: LineNumber, source: syn::Error },
    #[error("cannot parse {path}: {source}", path = path.display())]
    Manifest { path: PathBuf, source: toml::de::Error },
    #[error("no Rust sources found under {path}", path = path.display())]
    NoSources { path: PathBuf },
    #[error("cannot run cargo metadata in {path}: {source}", path = path.display())]
    Cargo { path: PathBuf, source: io::Error },
    #[error("cannot read cargo metadata of {path}: {source}", path = path.display())]
    Metadata { path: PathBuf, source: Unreadable },
}
