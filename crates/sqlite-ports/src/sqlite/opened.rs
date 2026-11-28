mod boundary;
mod connected;

use std::path::Path;

use rusqlite::Connection;
use rusqlite::config::DbConfig;
use store_ports::{Batch, Created, Query, Record, StoreFailure};

use super::failed::{Unbuilt, Unexecuted, Unopened};
pub(super) use boundary::Stamped;
pub(super) use connected::Connected;

#[derive(Debug)]
pub(super) struct Opened {
    connection: Connection,
}

impl Opened {
    const SETTINGS: &'static str = "PRAGMA foreign_keys = ON; PRAGMA synchronous = OFF; PRAGMA journal_mode = WAL;";
    const STAMP: &'static str = "user_version";
    const EMPTIED: &'static str = "VACUUM;";
    const BUSY: std::time::Duration = std::time::Duration::from_secs(5);
    const STATEMENTS: usize = 100;

    pub(super) fn filed(path: &Path, schema: &Stamped) -> Result<Opened, StoreFailure> {
        let opened = Opened::configured(Connection::open(path).map_err(Unopened::from)?)?;
        let stamp: i32 =
            opened.connection.pragma_query_value(None, Self::STAMP, |row| row.get(0)).map_err(Unopened::from)?;
        if stamp == schema.stamp() { Ok(opened) } else { opened.emptied(schema) }
    }

    fn emptied(self, schema: &Stamped) -> Result<Opened, StoreFailure> {
        let connection = &self.connection;
        connection.set_db_config(DbConfig::SQLITE_DBCONFIG_RESET_DATABASE, true).map_err(Unbuilt::from)?;
        connection.execute_batch(Self::EMPTIED).map_err(Unbuilt::from)?;
        connection.set_db_config(DbConfig::SQLITE_DBCONFIG_RESET_DATABASE, false).map_err(Unbuilt::from)?;
        self.stamped(schema)
    }

    pub(super) fn fresh(schema: &Stamped) -> Result<Opened, StoreFailure> {
        Opened::configured(Connection::open_in_memory().map_err(Unopened::from)?)?.stamped(schema)
    }

    pub(super) fn connection(&self) -> &Connection {
        &self.connection
    }

    pub(super) fn read<Q: Query>(&self, query: Q) -> Result<Q::Rows, StoreFailure> {
        query.rows(&mut Connected::over(&self.connection))
    }

    pub(super) fn transacted<B: Batch>(&self, batch: B) -> Result<B::Done, B::Failure> {
        self.settled(|connected| batch.written(connected))
    }

    pub(super) fn created<R: Record>(&self, created: Created<R>) -> Result<R, StoreFailure> {
        self.settled(|connected| created.stored(connected))
    }

    fn configured(connection: Connection) -> Result<Opened, StoreFailure> {
        connection.execute_batch(Self::SETTINGS).map_err(Unopened::from)?;
        connection.busy_timeout(Self::BUSY).map_err(Unopened::from)?;
        connection.set_prepared_statement_cache_capacity(Self::STATEMENTS);
        Ok(Opened { connection })
    }

    fn stamped(self, schema: &Stamped) -> Result<Opened, StoreFailure> {
        self.settled(|connected| {
            let connection = connected.connection();
            schema
                .schema()
                .statements()
                .try_for_each(|statement| connection.execute_batch(statement.as_ref()))
                .map_err(Unbuilt::from)?;
            connection.pragma_update(None, Self::STAMP, schema.stamp()).map_err(Unbuilt::from)?;
            Ok::<(), StoreFailure>(())
        })?;
        Ok(self)
    }

    fn settled<Done, Failure: From<StoreFailure>>(
        &self,
        writing: impl FnOnce(&mut Connected<'_>) -> Result<Done, Failure>,
    ) -> Result<Done, Failure> {
        let transaction =
            self.connection.unchecked_transaction().map_err(Unexecuted::from).map_err(StoreFailure::from)?;
        let written = writing(&mut Connected::over(&transaction))?;
        transaction.commit().map_err(Unexecuted::from).map_err(StoreFailure::from)?;
        Ok(written)
    }
}
