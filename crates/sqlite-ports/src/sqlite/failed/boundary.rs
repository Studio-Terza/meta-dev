use rusqlite::ffi;
use store_ports::{Cause, StoreFailure};

#[derive(Debug)]
pub(in super::super) struct Unopened(rusqlite::Error);

#[derive(Debug)]
pub(in super::super) struct Unexecuted(rusqlite::Error);

#[derive(Debug)]
pub(in super::super) struct Unbuilt(rusqlite::Error);

impl Unexecuted {
    fn kinded(extended: Option<i32>, failed: &rusqlite::Error) -> StoreFailure {
        match extended {
            Some(ffi::SQLITE_CONSTRAINT_FOREIGNKEY) => StoreFailure::Referenced(Cause::told(failed)),
            Some(_) | None => StoreFailure::Statement(Cause::told(failed)),
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

impl From<rusqlite::Error> for Unbuilt {
    fn from(failed: rusqlite::Error) -> Unbuilt {
        Unbuilt(failed)
    }
}

impl From<Unopened> for StoreFailure {
    fn from(failed: Unopened) -> StoreFailure {
        StoreFailure::Open(Cause::told(&failed.0))
    }
}

impl From<Unexecuted> for StoreFailure {
    fn from(failed: Unexecuted) -> StoreFailure {
        let extended = failed.0.sqlite_error().map(|error| error.extended_code);
        Unexecuted::kinded(extended, &failed.0)
    }
}

impl From<Unbuilt> for StoreFailure {
    fn from(failed: Unbuilt) -> StoreFailure {
        StoreFailure::Rebuild(Cause::told(&failed.0))
    }
}
