mod boundary;
mod connected;

use std::path::Path;

use rusqlite::{Connection, OpenFlags};
use store_ports::{Batch, Query, StoreFailure, Written};

use super::failed::{Unexecuted, Unopened};
use super::migrations::Migrations;
pub(super) use boundary::Snapshot;
use connected::Connected;

#[derive(Debug)]
pub(super) struct Opened {
    connection: Connection,
}

impl Opened {
    const SETTINGS: &'static str = "PRAGMA foreign_keys = ON;";
    const BUSY: std::time::Duration = std::time::Duration::from_secs(5);
    const STATEMENTS: usize = 100;

    pub(super) fn filed(path: &Path, migrations: Migrations) -> Result<Opened, StoreFailure> {
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let connection = Connection::open_with_flags(path, flags).map_err(Unopened::from)?;
        Opened::configured(connection)?.migrated(migrations)
    }

    pub(super) fn fresh(migrations: Migrations) -> Result<Opened, StoreFailure> {
        let connection = Connection::open_in_memory().map_err(Unopened::from)?;
        Opened::configured(connection)?.migrated(migrations)
    }

    pub(super) fn copied(image: &Snapshot) -> Result<Opened, StoreFailure> {
        let mut connection = Connection::open_in_memory().map_err(Unopened::from)?;
        image.restored(&mut connection).map_err(Unopened::from)?;
        Opened::configured(connection)
    }

    pub(super) fn image(&self) -> Result<Snapshot, StoreFailure> {
        Ok(Snapshot::taken(&self.connection).map_err(Unopened::from)?)
    }

    pub(super) fn read<Q: Query>(&self, query: Q) -> Result<Q::Rows, StoreFailure> {
        query.rows(&mut Connected::over(&self.connection))
    }

    pub(super) fn transacted<B: Batch>(&self, batch: B) -> Result<Written, B::Failure> {
        self.settled(|connected| batch.written(connected))
    }

    fn configured(connection: Connection) -> Result<Opened, StoreFailure> {
        connection.execute_batch(Self::SETTINGS).map_err(Unopened::from)?;
        connection.busy_timeout(Self::BUSY).map_err(Unopened::from)?;
        connection.set_prepared_statement_cache_capacity(Self::STATEMENTS);
        Ok(Opened { connection })
    }

    fn migrated(self, migrations: Migrations) -> Result<Opened, StoreFailure> {
        self.settled(|connected| {
            let pending = migrations.applied(connected.connection())?;
            migrations.rewritten(&pending, connected)
        })?;
        Ok(self)
    }

    fn settled<Failure: From<StoreFailure>>(
        &self,
        writing: impl FnOnce(&mut Connected<'_>) -> Result<Written, Failure>,
    ) -> Result<Written, Failure> {
        let transaction =
            self.connection.unchecked_transaction().map_err(Unexecuted::from).map_err(StoreFailure::from)?;
        let written = writing(&mut Connected::over(&transaction))?;
        transaction.commit().map_err(Unexecuted::from).map_err(StoreFailure::from)?;
        Ok(written)
    }
}
