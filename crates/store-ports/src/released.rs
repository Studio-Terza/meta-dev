use newtype::Counter;
use sqlx::Error;
use sqlx::error::ErrorKind;

use crate::{StoreFailure, Written};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Released {
    Deleted(Written),
    Referenced,
}

impl Released {
    pub(crate) fn refused(failed: Error) -> Result<Released, StoreFailure> {
        let kind = failed.as_database_error().map(|database| database.kind());
        (kind == Some(ErrorKind::ForeignKeyViolation))
            .then_some(Released::Referenced)
            .ok_or_else(|| StoreFailure::from(failed))
    }
}

impl From<Released> for Written {
    fn from(released: Released) -> Written {
        match released {
            Released::Deleted(written) => written,
            Released::Referenced => Written::zero(),
        }
    }
}
