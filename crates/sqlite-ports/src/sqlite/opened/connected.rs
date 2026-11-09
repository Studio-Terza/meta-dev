use std::collections::BTreeSet;

use sqlx::migrate::{Migrate, Migrator};
use sqlx::sqlite::{SqliteArguments, SqliteRow, SqliteValue, SqliteValueRef};
use sqlx::{Arguments, Row, Sqlite, SqliteConnection, Type, Value as _, ValueRef};
use store_ports::{Argument, Bind, Execute, StoreFailure, Value, Version, Written};
use tokio::runtime::Runtime;

use super::super::failed::{Unbound, Unexecuted, Unmigrated};

#[derive(Debug)]
pub(super) struct Connected<'run> {
    connection: &'run mut SqliteConnection,
    runtime: &'run Runtime,
}

#[derive(Debug, Default)]
struct Bound(SqliteArguments);

struct Stored(SqliteValue);

impl<'run> Connected<'run> {
    pub(super) fn over(connection: &'run mut SqliteConnection, runtime: &'run Runtime) -> Connected<'run> {
        Connected { connection, runtime }
    }

    pub(super) fn migrated(&mut self, migrator: &Migrator) -> Result<BTreeSet<Version>, StoreFailure> {
        let runtime = self.runtime;
        runtime.block_on(self.connection.ensure_migrations_table(&migrator.table_name)).map_err(Unmigrated::from)?;
        let listed = runtime
            .block_on(self.connection.list_applied_migrations(&migrator.table_name))
            .map_err(Unmigrated::from)?;
        let applied: BTreeSet<Version> = listed.iter().map(|migration| Version::new(migration.version)).collect();
        runtime.block_on(migrator.run(&mut *self.connection)).map_err(Unmigrated::from)?;
        let declared: BTreeSet<Version> = migrator.iter().map(|migration| Version::new(migration.version)).collect();
        Ok(declared.difference(&applied).copied().collect())
    }
}

impl Bound {
    fn arguments(arguments: Vec<Argument>) -> Result<SqliteArguments, StoreFailure> {
        let mut bound = Bound::default();
        arguments.into_iter().try_for_each(|argument| argument.bound(&mut bound))?;
        Ok(bound.0)
    }

    fn added(
        &mut self,
        value: impl sqlx::Encode<'static, Sqlite> + Type<Sqlite> + 'static,
    ) -> Result<(), StoreFailure> {
        self.0.add(value).map_err(Unbound::from).map_err(StoreFailure::from)
    }
}

impl Bind for Bound {
    fn null(&mut self) -> Result<(), StoreFailure> {
        self.added(Option::<i64>::None)
    }

    fn integer(&mut self, integer: i64) -> Result<(), StoreFailure> {
        self.added(integer)
    }

    fn text(&mut self, text: String) -> Result<(), StoreFailure> {
        self.added(text)
    }

    fn blob(&mut self, blob: Vec<u8>) -> Result<(), StoreFailure> {
        self.added(blob)
    }
}

impl Stored {
    fn row(row: &SqliteRow) -> Result<Vec<Value>, StoreFailure> {
        (0..row.len())
            .map(|index| row.try_get_raw(index).map_err(Unexecuted::from).map_err(StoreFailure::from).map(Stored::from))
            .map(|stored| stored.and_then(Stored::decoded))
            .collect()
    }

    fn decoded(self) -> Result<Value, StoreFailure> {
        let stored = self.0.type_info().into_owned();
        self.0
            .is_null()
            .then(|| Ok(Value::Null))
            .or_else(|| <i64 as Type<Sqlite>>::compatible(&stored).then(|| self.0.try_decode().map(Value::Integer)))
            .or_else(|| <f64 as Type<Sqlite>>::compatible(&stored).then(|| self.0.try_decode().map(Value::Real)))
            .or_else(|| <String as Type<Sqlite>>::compatible(&stored).then(|| self.0.try_decode().map(Value::Text)))
            .unwrap_or_else(|| self.0.try_decode().map(Value::Blob))
            .map_err(|failed| StoreFailure::Statement(Box::new(failed)))
    }
}

impl From<SqliteValueRef<'_>> for Stored {
    fn from(raw: SqliteValueRef<'_>) -> Stored {
        Stored(ValueRef::to_owned(&raw))
    }
}

impl Execute for Connected<'_> {
    fn fetched(&mut self, sql: &'static str, arguments: Vec<Argument>) -> Result<Vec<Vec<Value>>, StoreFailure> {
        let query = sqlx::query_with(sql, Bound::arguments(arguments)?);
        let rows = self.runtime.block_on(query.fetch_all(&mut *self.connection)).map_err(Unexecuted::from)?;
        rows.iter().map(Stored::row).collect()
    }

    fn executed(&mut self, sql: &'static str, arguments: Vec<Argument>) -> Result<Written, StoreFailure> {
        let query = sqlx::query_with(sql, Bound::arguments(arguments)?);
        let done = self.runtime.block_on(query.execute(&mut *self.connection)).map_err(Unexecuted::from)?;
        Ok(Written::new(usize::try_from(done.rows_affected()).unwrap_or(usize::MAX)))
    }
}
