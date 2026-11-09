use std::error::Error;
use std::io;

use crate::{ColumnFailure, Released};

#[derive(Debug, thiserror::Error)]
pub enum StoreFailure {
    #[error("cannot start the store's runtime: {0}")]
    Runtime(#[source] io::Error),
    #[error("cannot create the store's directory: {0}")]
    Directory(#[source] io::Error),
    #[error("cannot open the store: {0}")]
    Open(#[source] Box<dyn Error + Send + Sync>),
    #[error("cannot bring the store's schema up to date: {0}")]
    Migrate(#[source] Box<dyn Error + Send + Sync>),
    #[error("a store statement failed: {0}")]
    Statement(#[source] Box<dyn Error + Send + Sync>),
    #[error("a store statement broke a foreign key: {0}")]
    Referenced(#[source] Box<dyn Error + Send + Sync>),
    #[error("a stored column does not read back as its type: {0}")]
    Column(#[from] ColumnFailure),
    #[error("a statement that reads one row found none")]
    Missing,
    #[error("the store's rewrite refuses to open it: {0}")]
    Refused(#[source] Box<dyn Error + Send + Sync>),
}

impl StoreFailure {
    pub(crate) fn released(self) -> Result<Released, StoreFailure> {
        match self {
            StoreFailure::Referenced(_) => Ok(Released::Referenced),
            failed @ (StoreFailure::Runtime(_)
            | StoreFailure::Directory(_)
            | StoreFailure::Open(_)
            | StoreFailure::Migrate(_)
            | StoreFailure::Statement(_)
            | StoreFailure::Column(_)
            | StoreFailure::Missing
            | StoreFailure::Refused(_)) => Err(failed),
        }
    }
}
