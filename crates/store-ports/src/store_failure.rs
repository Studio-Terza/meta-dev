use std::io;

use sqlx::migrate::MigrateError;

use crate::ColumnFailure;

#[derive(Debug, thiserror::Error)]
pub enum StoreFailure {
    #[error("cannot start the store's runtime: {0}")]
    Runtime(#[source] io::Error),
    #[error("cannot create the store's directory: {0}")]
    Directory(#[source] io::Error),
    #[error("cannot open the store: {0}")]
    Open(#[source] sqlx::Error),
    #[error("cannot bring the store's schema up to date: {0}")]
    Migrate(#[from] MigrateError),
    #[error("a store statement failed: {0}")]
    Statement(#[from] sqlx::Error),
    #[error("a stored column does not read back as its type: {0}")]
    Column(#[from] ColumnFailure),
}
