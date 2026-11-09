use sqlx::error::{BoxDynError, DatabaseError, ErrorKind};
use sqlx::migrate::MigrateError;
use store_ports::StoreFailure;

#[derive(Debug)]
pub(super) struct Unopened(sqlx::Error);

#[derive(Debug)]
pub(super) struct Unexecuted(sqlx::Error);

#[derive(Debug)]
pub(super) struct Unmigrated(MigrateError);

#[derive(Debug)]
pub(super) struct Unbound(BoxDynError);

impl Unexecuted {
    fn kinded(kind: Option<&ErrorKind>, failed: sqlx::Error) -> StoreFailure {
        match kind {
            Some(ErrorKind::ForeignKeyViolation) => StoreFailure::Referenced(Box::new(failed)),
            Some(_) | None => StoreFailure::Statement(Box::new(failed)),
        }
    }
}

impl From<sqlx::Error> for Unopened {
    fn from(failed: sqlx::Error) -> Unopened {
        Unopened(failed)
    }
}

impl From<sqlx::Error> for Unexecuted {
    fn from(failed: sqlx::Error) -> Unexecuted {
        Unexecuted(failed)
    }
}

impl From<MigrateError> for Unmigrated {
    fn from(failed: MigrateError) -> Unmigrated {
        Unmigrated(failed)
    }
}

impl From<BoxDynError> for Unbound {
    fn from(failed: BoxDynError) -> Unbound {
        Unbound(failed)
    }
}

impl From<Unopened> for StoreFailure {
    fn from(failed: Unopened) -> StoreFailure {
        StoreFailure::Open(Box::new(failed.0))
    }
}

impl From<Unexecuted> for StoreFailure {
    fn from(failed: Unexecuted) -> StoreFailure {
        let kind = failed.0.as_database_error().map(DatabaseError::kind);
        Unexecuted::kinded(kind.as_ref(), failed.0)
    }
}

impl From<Unmigrated> for StoreFailure {
    fn from(failed: Unmigrated) -> StoreFailure {
        StoreFailure::Migrate(Box::new(failed.0))
    }
}

impl From<Unbound> for StoreFailure {
    fn from(failed: Unbound) -> StoreFailure {
        StoreFailure::Statement(failed.0)
    }
}
