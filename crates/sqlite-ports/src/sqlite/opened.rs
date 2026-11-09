mod connected;

use sqlx::sqlite::{SqliteConnectOptions, SqliteOwnedBuf};
use sqlx::{ConnectOptions, Connection, Sqlite, SqliteConnection, Transaction};
use store_ports::{Batch, Query, StoreFailure, Written};
use tokio::runtime::{Builder, Runtime};

use super::failed::{Unexecuted, Unopened};
use super::migrations::Migrations;
use connected::Connected;

#[derive(Debug)]
pub(super) struct Opened {
    runtime: Runtime,
    connection: SqliteConnection,
}

impl Opened {
    pub(super) fn connected(options: &SqliteConnectOptions, migrations: Migrations) -> Result<Opened, StoreFailure> {
        let runtime = Builder::new_current_thread().build().map_err(StoreFailure::Runtime)?;
        let connection = runtime.block_on(options.connect()).map_err(Unopened::from)?;
        let mut opened = Opened { runtime, connection };
        opened.migrated(migrations)?;
        Ok(opened)
    }

    pub(super) fn copied(options: &SqliteConnectOptions, image: SqliteOwnedBuf) -> Result<Opened, StoreFailure> {
        let runtime = Builder::new_current_thread().build().map_err(StoreFailure::Runtime)?;
        let mut connection = runtime.block_on(options.connect()).map_err(Unopened::from)?;
        runtime.block_on(connection.deserialize(None, image, false)).map_err(Unopened::from)?;
        Ok(Opened { runtime, connection })
    }

    pub(super) fn image(&mut self) -> Result<SqliteOwnedBuf, StoreFailure> {
        self.runtime.block_on(self.connection.serialize(None)).map_err(Unopened::from).map_err(StoreFailure::from)
    }

    pub(super) fn read<Q: Query>(&mut self, query: Q) -> Result<Q::Rows, StoreFailure> {
        query.rows(&mut Connected::over(&mut self.connection, &self.runtime))
    }

    pub(super) fn transacted<B: Batch>(&mut self, batch: B) -> Result<Written, B::Failure> {
        self.settled(|connected| batch.written(connected))
    }

    fn migrated(&mut self, migrations: Migrations) -> Result<Written, StoreFailure> {
        self.settled(|connected| {
            let pending = connected.migrated(&migrations.migrator())?;
            migrations.rewritten(&pending, connected)
        })
    }

    fn settled<Failure: From<StoreFailure>>(
        &mut self,
        writing: impl FnOnce(&mut Connected<'_>) -> Result<Written, Failure>,
    ) -> Result<Written, Failure> {
        let runtime = &self.runtime;
        let mut transaction =
            runtime.block_on(self.connection.begin()).map_err(Unexecuted::from).map_err(StoreFailure::from)?;
        let written = writing(&mut Connected::over(&mut transaction, runtime));
        Self::ended(runtime, transaction, written)
    }

    fn ended<Failure: From<StoreFailure>>(
        runtime: &Runtime,
        transaction: Transaction<'_, Sqlite>,
        written: Result<Written, Failure>,
    ) -> Result<Written, Failure> {
        match written {
            Ok(rows) => runtime
                .block_on(transaction.commit())
                .map(|()| rows)
                .map_err(|failed| Failure::from(StoreFailure::from(Unexecuted::from(failed)))),
            Err(failure) => runtime
                .block_on(transaction.rollback())
                .map_err(|failed| Failure::from(StoreFailure::from(Unexecuted::from(failed))))
                .and(Err(failure)),
        }
    }
}
