use sqlx::query::{Map, Query};
use sqlx::sqlite::SqliteRow;
use sqlx::{IntoArguments, Sqlite, SqliteConnection};
use tokio::runtime::Runtime;

use crate::{Columns, StoreFailure, Written};

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
}
