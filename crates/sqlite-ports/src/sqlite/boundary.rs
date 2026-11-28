use port_core::Pass;
use query_cache::Entry;
use query_ports::{DescribeFiller, Undescribed, describe_ops};
use rusqlite::fallible_iterator::FallibleIterator;
use rusqlite::{Batch, Connection};
use store_ports::{Parses, Schema, Statement, StoreFailure};

use super::failed::{Unbuilt, Unopened};
use super::{Sqlite, SqliteOpener};

impl DescribeFiller for Sqlite {
    fn describe(&self, _pass: Pass<'_, describe_ops::Describe>, query: &str) -> Result<Entry<String>, Undescribed> {
        let opened = self.shared.opened(|opened| Ok::<_, StoreFailure>(opened.described(query)));
        opened.map_err(|failed| Undescribed::told(query, &failed)).and_then(|described| described)
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
