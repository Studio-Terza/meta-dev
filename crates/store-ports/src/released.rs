use sqlx::Error;
use sqlx::error::{DatabaseError, ErrorKind};

use crate::StoreFailure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Released {
    Deleted,
    Referenced,
}

impl Released {
    pub(crate) fn refused(failed: Error) -> Result<Released, StoreFailure> {
        let kind = failed.as_database_error().map(DatabaseError::kind);
        (kind == Some(ErrorKind::ForeignKeyViolation))
            .then_some(Released::Referenced)
            .ok_or_else(|| StoreFailure::from(failed))
    }
}
