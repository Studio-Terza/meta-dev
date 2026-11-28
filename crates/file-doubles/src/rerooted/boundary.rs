use sqlite_ports::SqliteOpener;
use store_ports::{Parses, Schema, StoreFailure};

use super::Rerooted;

impl Parses for Rerooted {
    fn parsed(&self, sql: &str) -> Result<Schema, StoreFailure> {
        SqliteOpener.parsed(sql)
    }
}
