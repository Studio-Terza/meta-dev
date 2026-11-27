use std::io;

use crate::{Cause, ColumnFailure};

#[derive(Debug, thiserror::Error)]
pub enum StoreFailure {
    #[error("cannot create the store's directory: {0}")]
    Directory(#[source] io::Error),
    #[error("cannot open the store: {0}")]
    Open(#[source] Cause),
    #[error("cannot build the store's schema: {0}")]
    Rebuild(#[source] Cause),
    #[error("a store statement failed: {0}")]
    Statement(#[source] Cause),
    #[error("a store statement broke a foreign key: {0}")]
    Referenced(#[source] Cause),
    #[error("a stored column does not read back as its type: {0}")]
    Column(#[from] ColumnFailure),
    #[error("a statement that reads one row found none")]
    Missing,
}
