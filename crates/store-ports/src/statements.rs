use sqlx::query::{Map, Query};
use sqlx::sqlite::SqliteRow;
use sqlx::{IntoArguments, Sqlite, SqliteConnection};
use tokio::runtime::Runtime;

use crate::{ColumnFailure, Columns, Selected, StoreFailure, Written};

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
        selected: Selected<Row, Map<'_, Sqlite, Mapper, Arguments>>,
    ) -> Result<Vec<Row>, StoreFailure>
    where
        Row: Columns<Raw: Send + Unpin>,
        Mapper: FnMut(SqliteRow) -> Result<Row::Raw, sqlx::Error> + Send,
        Arguments: Send + IntoArguments<Sqlite>,
    {
        let raw = self.runtime.block_on(selected.statement().fetch_all(&mut *self.connection))?;
        Ok(raw.into_iter().map(Row::from_raw).collect::<Result<Vec<Row>, ColumnFailure>>()?)
    }

    pub fn one<Row, Mapper, Arguments>(
        &mut self,
        selected: Selected<Row, Map<'_, Sqlite, Mapper, Arguments>>,
    ) -> Result<Row, StoreFailure>
    where
        Row: Columns<Raw: Send + Unpin>,
        Mapper: FnMut(SqliteRow) -> Result<Row::Raw, sqlx::Error> + Send,
        Arguments: Send + IntoArguments<Sqlite>,
    {
        let raw = self.runtime.block_on(selected.statement().fetch_one(&mut *self.connection))?;
        Ok(Row::from_raw(raw)?)
    }

    pub fn written<Arguments>(&mut self, query: Query<'_, Sqlite, Arguments>) -> Result<Written, StoreFailure>
    where
        Arguments: Send + IntoArguments<Sqlite>,
    {
        let done = self.runtime.block_on(query.execute(&mut *self.connection))?;
        Ok(Written::affected(&done))
    }
}
