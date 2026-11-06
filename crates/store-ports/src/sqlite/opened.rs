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

    pub(super) fn transacted<B: Batch>(&mut self, batch: B) -> Result<Written, B::Failure> {
        let mut transaction = self.runtime.block_on(self.connection.begin()).map_err(StoreFailure::from)?;
        let written = batch.written(&mut Statements::over(&mut transaction, &self.runtime));
        Self::settled(&self.runtime, transaction, written)
    }

    fn settled<Failure: From<StoreFailure>>(
        runtime: &Runtime,
        transaction: Transaction<'_, Sqlite>,
        written: Result<Written, Failure>,
    ) -> Result<Written, Failure> {
        match written {
            Ok(written) => runtime
                .block_on(transaction.commit())
                .map(|()| written)
                .map_err(|failed| Failure::from(StoreFailure::from(failed))),
            Err(failure) => runtime
                .block_on(transaction.rollback())
                .map_err(|failed| Failure::from(StoreFailure::from(failed)))
                .and(Err(failure)),
        }
    }
}
