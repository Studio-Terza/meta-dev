use sqlx::sqlite::{SqliteConnectOptions, SqliteOwnedBuf};
use sqlx::{ConnectOptions, Connection, Sqlite, SqliteConnection, Transaction};
use tokio::runtime::{Builder, Runtime};

use super::migrations::Migrations;
use crate::{Batch, Query, Statements, StoreFailure, Written};

#[derive(Debug)]
pub(super) struct Opened {
    runtime: Runtime,
    connection: SqliteConnection,
}

impl Opened {
    pub(super) fn connected(options: &SqliteConnectOptions, migrations: Migrations) -> Result<Opened, StoreFailure> {
        let runtime = Builder::new_current_thread().build().map_err(StoreFailure::Runtime)?;
        let connection = runtime.block_on(options.connect()).map_err(StoreFailure::Open)?;
        let mut opened = Opened { runtime, connection };
        opened.migrated(migrations)?;
        Ok(opened)
    }

    pub(super) fn copied(options: &SqliteConnectOptions, image: SqliteOwnedBuf) -> Result<Opened, StoreFailure> {
        let runtime = Builder::new_current_thread().build().map_err(StoreFailure::Runtime)?;
        let mut connection = runtime.block_on(options.connect()).map_err(StoreFailure::Open)?;
        runtime.block_on(connection.deserialize(None, image, false)).map_err(StoreFailure::Open)?;
        Ok(Opened { runtime, connection })
    }

    pub(super) fn image(&mut self) -> Result<SqliteOwnedBuf, StoreFailure> {
        self.runtime.block_on(self.connection.serialize(None)).map_err(StoreFailure::Open)
    }

    pub(super) fn read<Q: Query>(&mut self, query: Q) -> Result<Q::Rows, StoreFailure> {
        query.rows(&mut Statements::over(&mut self.connection, &self.runtime))
    }

    pub(super) fn transacted<B: Batch>(&mut self, batch: B) -> Result<Written, B::Failure> {
        self.settled(|statements| batch.written(statements))
    }

    fn migrated(&mut self, migrations: Migrations) -> Result<Written, StoreFailure> {
        self.settled(|statements| {
            let pending = statements.migrated(migrations.migrator())?;
            migrations.rewritten(&pending, statements)
        })
    }

    fn settled<Failure: From<StoreFailure>>(
        &mut self,
        writing: impl FnOnce(&mut Statements<'_>) -> Result<Written, Failure>,
    ) -> Result<Written, Failure> {
        let runtime = &self.runtime;
        let mut transaction = runtime.block_on(self.connection.begin()).map_err(StoreFailure::from)?;
        let written = writing(&mut Statements::over(&mut transaction, runtime));
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
                .map_err(|failed| Failure::from(StoreFailure::from(failed))),
            Err(failure) => runtime
                .block_on(transaction.rollback())
                .map_err(|failed| Failure::from(StoreFailure::from(failed)))
                .and(Err(failure)),
        }
    }
}
