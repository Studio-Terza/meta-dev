use std::io;
use std::path::PathBuf;

use crate::line_number::LineNumber;

#[derive(Debug, thiserror::Error)]
pub enum Failure {
    #[error("cannot read {path}: {source}", path = path.display())]
    Io { path: PathBuf, source: io::Error },
    #[error("cannot parse {path}:{line}: {source}", path = path.display())]
    Parse { path: PathBuf, line: LineNumber, source: syn::Error },
    #[error("no Rust sources found under {path}", path = path.display())]
    NoSources { path: PathBuf },
}
