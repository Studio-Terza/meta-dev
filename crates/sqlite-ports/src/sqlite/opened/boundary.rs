use query_cache::{Described, Entry, Undescribed};
use rusqlite::Statement;
use sha2::{Digest, Sha256};
use store_ports::Schema;

use super::Opened;

#[derive(Clone, Copy, Debug)]
pub(in super::super) struct Stamped {
    sql: &'static str,
    stamp: i32,
}

impl Stamped {
    pub(in super::super) fn of<S: Schema>() -> Stamped {
        let digest = Sha256::digest(S::SQL.as_bytes());
        let [first, second, third, fourth, ..]: [u8; 32] = digest.into();
        Stamped { sql: S::SQL, stamp: i32::from_be_bytes([first, second, third, fourth]) }
    }

    pub(super) fn sql(self) -> &'static str {
        self.sql
    }

    pub(super) fn stamp(self) -> i32 {
        self.stamp
    }
}

impl Opened {
    pub(in super::super) fn described(&self, query: &str) -> Result<Entry<String>, Undescribed> {
        let told = |failed: rusqlite::Error| Undescribed::told(query, &failed);
        let statement = self.connection.prepare(query).map_err(told)?;
        let columns = (0..statement.column_count())
            .map(|at| Opened::column(&statement, at))
            .collect::<rusqlite::Result<Vec<Described<String>>>>()
            .map_err(told)?;
        Ok(Entry::described(query, columns, statement.parameter_count()))
    }

    fn column(statement: &Statement<'_>, at: usize) -> rusqlite::Result<Described<String>> {
        let name = statement.column_name(at)?.to_owned();
        let origin = statement.column_metadata(at)?;
        let declared =
            origin.and_then(|(_, _, _, declared, _, _, _, _)| declared).map(|spelled| spelled.to_string_lossy());
        let nullable = origin.map(|(_, _, _, _, _, not_null, _, _)| !not_null);
        Ok(Described::new(name, declared.as_deref(), nullable))
    }
}
