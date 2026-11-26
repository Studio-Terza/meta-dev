use std::io;

use crate::{Cause, ColumnFailure};

#[derive(Debug, thiserror::Error)]
pub enum StoreFailure {
    #[error("cannot start the store's runtime: {0}")]
    Runtime(#[source] io::Error),
    #[error("cannot create the store's directory: {0}")]
    Directory(#[source] io::Error),
    #[error("cannot open the store: {0}")]
    Open(#[source] Cause),
    #[error("cannot bring the store's schema up to date: {0}")]
    Migrate(#[source] Cause),
    #[error("a store statement failed: {0}")]
    Statement(#[source] Cause),
    #[error("a store statement broke a foreign key: {0}")]
    Referenced(#[source] Cause),
    #[error("a stored column does not read back as its type: {0}")]
    Column(#[from] ColumnFailure),
    #[error("a statement that reads one row found none")]
    Missing,
    #[error("the store's rewrite refuses to open it: {0}")]
    Refused(#[source] Cause),
}
