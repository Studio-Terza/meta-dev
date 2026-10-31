use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Failure {
    #[error("cannot read {path}: {source}", path = path.display())]
    Io { path: PathBuf, source: io::Error },
    #[error("cannot parse {path}: {source}", path = path.display())]
    Parse { path: PathBuf, source: syn::Error },
}
