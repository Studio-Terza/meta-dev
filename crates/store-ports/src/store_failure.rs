use std::fmt;
use std::io;

use crate::{ColumnFailure, Released};

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct Cause(String);

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
    #[error("a lifecycle move was refused: {0}")]
    Unmoved(#[from] Unmoved),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Unmoved {
    #[error("the move is not an edge of its lifecycle")]
    Unlisted,
    #[error("the row is not in the state the move starts from")]
    Elsewhere,
}

impl Cause {
    #[must_use]
    pub fn told(failure: &impl fmt::Display) -> Cause {
        Cause(failure.to_string())
    }
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
            | StoreFailure::Refused(_)
            | StoreFailure::Unmoved(_)) => Err(failed),
        }
    }
}
