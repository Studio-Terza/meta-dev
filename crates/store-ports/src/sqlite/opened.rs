use sqlx::migrate::Migrator;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{ConnectOptions, Connection, Sqlite, SqliteConnection, Transaction};
use tokio::runtime::{Builder, Runtime};

use crate::{Batch, Query, Statements, StoreFailure, Written};

#[derive(Debug)]
pub(super) struct Opened {
    runtime: Runtime,
    connection: SqliteConnection,
}

impl Opened {
    pub(super) fn connected(options: &SqliteConnectOptions, migrator: &Migrator) -> Result<Opened, StoreFailure> {
        let runtime = Builder::new_current_thread().build().map_err(StoreFailure::Runtime)?;
        let mut connection = runtime.block_on(options.connect()).map_err(StoreFailure::Open)?;
        runtime.block_on(migrator.run(&mut connection))?;
        Ok(Opened { runtime, connection })
    }

    pub(super) fn read<Q: Query>(&mut self, query: Q) -> Result<Q::Rows, StoreFailure> {
        query.rows(&mut Statements::over(&mut self.connection, &self.runtime))
    }

    pub(super) fn transacted(&mut self, batch: impl Batch) -> Result<Written, StoreFailure> {
        let mut transaction = self.runtime.block_on(self.connection.begin())?;
        let written = batch.written(&mut Statements::over(&mut transaction, &self.runtime));
        Self::settled(&self.runtime, transaction, written)
    }

    fn settled(
        runtime: &Runtime,
        transaction: Transaction<'_, Sqlite>,
        written: Result<Written, StoreFailure>,
    ) -> Result<Written, StoreFailure> {
        match written {
            Ok(written) => runtime.block_on(transaction.commit()).map(|()| written).map_err(StoreFailure::from),
            Err(failure) => runtime.block_on(transaction.rollback()).map_err(StoreFailure::from).and(Err(failure)),
        }
    }
}
