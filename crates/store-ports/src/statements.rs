use std::collections::BTreeSet;

use sqlx::error::ErrorKind;
use sqlx::migrate::{Migrate, Migrator};
use sqlx::query::{Map, Query};
use sqlx::sqlite::SqliteRow;
use sqlx::{Error, IntoArguments, Sqlite, SqliteConnection};
use tokio::runtime::Runtime;

use crate::{Columns, StoreFailure, Version, Written};

mod boundary;

#[derive(Debug)]
pub struct Statements<'run> {
    connection: &'run mut SqliteConnection,
    runtime: &'run Runtime,
}

impl<'run> Statements<'run> {
    pub(crate) fn over(connection: &'run mut SqliteConnection, runtime: &'run Runtime) -> Statements<'run> {
        Statements { connection, runtime }
    }

    pub fn rows<Row, Mapper, Arguments>(
        &mut self,
        query: Map<'_, Sqlite, Mapper, Arguments>,
    ) -> Result<Vec<Row>, StoreFailure>
    where
        Row: Columns + Send + Unpin,
        Mapper: FnMut(SqliteRow) -> Result<Row, sqlx::Error> + Send,
        Arguments: Send + IntoArguments<Sqlite>,
    {
        Ok(self.runtime.block_on(query.fetch_all(&mut *self.connection))?)
    }

    pub fn one<Row, Mapper, Arguments>(
        &mut self,
        query: Map<'_, Sqlite, Mapper, Arguments>,
    ) -> Result<Row, StoreFailure>
    where
        Row: Columns + Send + Unpin,
        Mapper: FnMut(SqliteRow) -> Result<Row, sqlx::Error> + Send,
        Arguments: Send + IntoArguments<Sqlite>,
    {
        Ok(self.runtime.block_on(query.fetch_one(&mut *self.connection))?)
    }

    pub fn written<Arguments>(&mut self, query: Query<'_, Sqlite, Arguments>) -> Result<Written, StoreFailure>
    where
        Arguments: Send + IntoArguments<Sqlite>,
    {
        let done = self.runtime.block_on(query.execute(&mut *self.connection))?;
        Ok(Written::affected(&done))
    }

    pub fn written_unless_referenced<Arguments>(
        &mut self,
        query: Query<'_, Sqlite, Arguments>,
    ) -> Result<Option<Written>, StoreFailure>
    where
        Arguments: Send + IntoArguments<Sqlite>,
    {
        match self.runtime.block_on(query.execute(&mut *self.connection)) {
            Ok(done) => Ok(Some(Written::affected(&done))),
            Err(Error::Database(failed)) if failed.kind() == ErrorKind::ForeignKeyViolation => Ok(None),
            Err(failed) => Err(StoreFailure::from(failed)),
        }
    }

    pub(crate) fn migrated(&mut self, migrator: &Migrator) -> Result<BTreeSet<Version>, StoreFailure> {
        self.runtime.block_on(self.connection.ensure_migrations_table(&migrator.table_name))?;
        let applied = self.runtime.block_on(self.connection.list_applied_migrations(&migrator.table_name))?;
        let applied: BTreeSet<Version> = applied.iter().map(|migration| Version::new(migration.version)).collect();
        self.runtime.block_on(migrator.run(&mut *self.connection))?;
        let declared: BTreeSet<Version> = migrator.iter().map(|migration| Version::new(migration.version)).collect();
        Ok(declared.difference(&applied).copied().collect())
    }
}
