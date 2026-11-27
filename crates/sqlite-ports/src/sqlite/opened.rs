mod connected;

use std::path::Path;

use rusqlite::Connection;
use sha2::{Digest, Sha256};
use store_ports::{Batch, Created, Query, Record, Schema, StoreFailure};

use super::failed::{Unbuilt, Unexecuted, Unopened};
pub(super) use connected::Connected;

#[derive(Debug)]
pub(super) struct Opened {
    connection: Connection,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Stamped {
    sql: &'static str,
    stamp: i32,
}

impl Stamped {
    pub(super) fn of<S: Schema>() -> Stamped {
        let digest = Sha256::digest(S::SQL.as_bytes());
        let [first, second, third, fourth, ..]: [u8; 32] = digest.into();
        Stamped { sql: S::SQL, stamp: i32::from_be_bytes([first, second, third, fourth]) }
    }
}

impl Opened {
    const SETTINGS: &'static str = "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = OFF;";
    const STAMP: &'static str = "user_version";
    const BUSY: std::time::Duration = std::time::Duration::from_secs(5);
    const STATEMENTS: usize = 100;

    pub(super) fn filed(path: &Path, schema: Stamped) -> Result<Option<Opened>, StoreFailure> {
        let opened = Opened::configured(Connection::open(path).map_err(Unopened::from)?)?;
        let stamp: i32 =
            opened.connection.pragma_query_value(None, Self::STAMP, |row| row.get(0)).map_err(Unopened::from)?;
        Ok((stamp == schema.stamp).then_some(opened))
    }

    pub(super) fn built(path: &Path, schema: Stamped) -> Result<Opened, StoreFailure> {
        Opened::configured(Connection::open(path).map_err(Unopened::from)?)?.stamped(schema)
    }

    pub(super) fn fresh(schema: Stamped) -> Result<Opened, StoreFailure> {
        Opened::configured(Connection::open_in_memory().map_err(Unopened::from)?)?.stamped(schema)
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

    fn stamped(self, schema: Stamped) -> Result<Opened, StoreFailure> {
        self.settled(|connected| {
            let connection = connected.connection();
            connection.execute_batch(schema.sql).map_err(Unbuilt::from)?;
            connection.pragma_update(None, Self::STAMP, schema.stamp).map_err(Unbuilt::from)?;
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
