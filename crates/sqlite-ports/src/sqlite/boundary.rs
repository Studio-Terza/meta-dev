use port_core::Pass;
use query_cache::{Described, Entry};
use query_ports::{DescribeFiller, Undescribed, describe_ops};
use rusqlite::fallible_iterator::FallibleIterator;
use rusqlite::{Batch, Connection};
use store_ports::{Parses, Schema, Statement, StoreFailure};

use super::failed::{Unbuilt, Unopened};
use super::{Sqlite, SqliteOpener};

impl DescribeFiller for Sqlite {
    fn describe(&self, _pass: Pass<'_, describe_ops::Describe>, query: &str) -> Result<Entry<String>, Undescribed> {
        let opened = self.shared.opened(|opened| Ok::<_, StoreFailure>(Sqlite::described(opened.connection(), query)));
        opened.map_err(|failed| Undescribed::told(query, &failed)).and_then(|described| described)
    }
}

impl Sqlite {
    fn described(connection: &Connection, query: &str) -> Result<Entry<String>, Undescribed> {
        let told = |failed: rusqlite::Error| Undescribed::told(query, &failed);
        let statement = connection.prepare(query).map_err(told)?;
        let columns = (0..statement.column_count())
            .map(|at| Sqlite::column(&statement, at))
            .collect::<rusqlite::Result<Vec<Described<String>>>>()
            .map_err(told)?;
        Ok(Entry::described(query, columns, statement.parameter_count()))
    }

    fn column(statement: &rusqlite::Statement<'_>, at: usize) -> rusqlite::Result<Described<String>> {
        let name = statement.column_name(at)?.to_owned();
        let origin = statement.column_metadata(at)?;
        let declared =
            origin.and_then(|(_, _, _, declared, _, _, _, _)| declared).map(|spelled| spelled.to_string_lossy());
        let nullable = origin.map(|(_, _, _, _, _, not_null, _, _)| !not_null);
        Ok(Described::new(name, declared.as_deref(), nullable))
    }
}

impl Parses for SqliteOpener {
    fn parsed(&self, sql: &str) -> Result<Schema, StoreFailure> {
        let scratch = Connection::open_in_memory().map_err(Unopened::from)?;
        Batch::new(&scratch, sql)
            .map(|mut statement| {
                statement.execute([])?;
                statement.expanded_sql().map(Statement::new).ok_or(rusqlite::Error::InvalidQuery)
            })
            .collect()
            .map_err(Unbuilt::from)
            .map_err(StoreFailure::from)
    }
}
