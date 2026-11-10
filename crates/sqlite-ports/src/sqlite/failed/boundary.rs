use rusqlite::ffi;
use store_ports::StoreFailure;

#[derive(Debug)]
pub(in super::super) struct Unopened(rusqlite::Error);

#[derive(Debug)]
pub(in super::super) struct Unexecuted(rusqlite::Error);

#[derive(Debug, thiserror::Error)]
pub(in super::super) enum Unmigrated {
    #[error("migration {0} was left partly applied")]
    Dirty(i64),
    #[error("migration {0} is applied but the code does not declare it")]
    Missing(i64),
    #[error("migration {0} was applied with different contents")]
    Mismatched(i64),
    #[error("{0}")]
    Statement(rusqlite::Error),
}

impl Unexecuted {
    fn kinded(extended: Option<i32>, failed: rusqlite::Error) -> StoreFailure {
        match extended {
            Some(ffi::SQLITE_CONSTRAINT_FOREIGNKEY) => StoreFailure::Referenced(Box::new(failed)),
            Some(_) | None => StoreFailure::Statement(Box::new(failed)),
        }
    }
}

impl From<rusqlite::Error> for Unopened {
    fn from(failed: rusqlite::Error) -> Unopened {
        Unopened(failed)
    }
}

impl From<rusqlite::Error> for Unexecuted {
    fn from(failed: rusqlite::Error) -> Unexecuted {
        Unexecuted(failed)
    }
}

impl From<rusqlite::Error> for Unmigrated {
    fn from(failed: rusqlite::Error) -> Unmigrated {
        Unmigrated::Statement(failed)
    }
}

impl From<Unopened> for StoreFailure {
    fn from(failed: Unopened) -> StoreFailure {
        StoreFailure::Open(Box::new(failed.0))
    }
}

impl From<Unexecuted> for StoreFailure {
    fn from(failed: Unexecuted) -> StoreFailure {
        let extended = failed.0.sqlite_error().map(|error| error.extended_code);
        Unexecuted::kinded(extended, failed.0)
    }
}

impl From<Unmigrated> for StoreFailure {
    fn from(failed: Unmigrated) -> StoreFailure {
        StoreFailure::Migrate(Box::new(failed))
    }
}
