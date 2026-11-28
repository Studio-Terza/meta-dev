use std::path::PathBuf;

use file_ports::FileFailure;

use crate::Undescribed;

#[derive(Debug, thiserror::Error)]
pub enum Uncached {
    #[error("cannot read {}: {source}", path.display())]
    Unread { path: PathBuf, source: FileFailure },
    #[error("cannot parse {}: {source}", path.display())]
    Unparsed { path: PathBuf, source: syn::Error },
    #[error(transparent)]
    Undescribed(#[from] Undescribed),
    #[error("cannot write a cached query: {0}")]
    Unwritten(#[from] serde_json::Error),
    #[error("cannot rewrite {}: {source}", path.display())]
    Unrewritten { path: PathBuf, source: FileFailure },
}
