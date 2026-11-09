use rusqlite::types::{Value as Stored, ValueRef};
use rusqlite::{Connection, Row, params_from_iter};
use store_ports::{Argument, Bind, ColumnFailure, Execute, StoreFailure, Value, Written};

use super::super::failed::Unexecuted;

#[derive(Debug)]
pub(super) struct Connected<'run> {
    connection: &'run Connection,
}

#[derive(Debug, Default)]
struct Bound(Vec<Stored>);

impl<'run> Connected<'run> {
    pub(super) fn over(connection: &'run Connection) -> Connected<'run> {
        Connected { connection }
    }

    pub(super) fn connection(&self) -> &'run Connection {
        self.connection
    }

    fn row(row: &Row<'_>) -> rusqlite::Result<Result<Vec<Value>, StoreFailure>> {
        let count = row.as_ref().column_count();
        (0..count)
            .map(|index| row.get_ref(index).map(Bound::read))
            .collect::<rusqlite::Result<Result<Vec<Value>, StoreFailure>>>()
    }
}

impl Bound {
    fn arguments(arguments: Vec<Argument>) -> Result<Vec<Stored>, StoreFailure> {
        let mut bound = Bound::default();
        arguments.into_iter().try_for_each(|argument| argument.bound(&mut bound))?;
        Ok(bound.0)
    }

    fn read(stored: ValueRef<'_>) -> Result<Value, StoreFailure> {
        match stored {
            ValueRef::Null => Ok(Value::Null),
            ValueRef::Integer(integer) => Ok(Value::Integer(integer)),
            ValueRef::Real(real) => Ok(Value::Real(real)),
            ValueRef::Text(text) => String::from_utf8(text.to_vec())
                .map(Value::Text)
                .map_err(|_unreadable| StoreFailure::from(ColumnFailure::Mistyped)),
            ValueRef::Blob(_) => Err(StoreFailure::from(ColumnFailure::Mistyped)),
        }
    }
}

impl Bind for Bound {
    fn null(&mut self) -> Result<(), StoreFailure> {
        self.0.push(Stored::Null);
        Ok(())
    }

    fn integer(&mut self, integer: i64) -> Result<(), StoreFailure> {
        self.0.push(Stored::Integer(integer));
        Ok(())
    }

    fn text(&mut self, text: String) -> Result<(), StoreFailure> {
        self.0.push(Stored::Text(text));
        Ok(())
    }

    fn blob(&mut self, blob: Vec<u8>) -> Result<(), StoreFailure> {
        self.0.push(Stored::Blob(blob));
        Ok(())
    }
}

impl Execute for Connected<'_> {
    fn fetched(&mut self, sql: &'static str, arguments: Vec<Argument>) -> Result<Vec<Vec<Value>>, StoreFailure> {
        let mut statement = self.connection.prepare_cached(sql).map_err(Unexecuted::from)?;
        let rows = statement
            .query_map(params_from_iter(Bound::arguments(arguments)?), Connected::row)
            .map_err(Unexecuted::from)?;
        rows.map(|row| row.map_err(Unexecuted::from).map_err(StoreFailure::from).and_then(|values| values)).collect()
    }

    fn executed(&mut self, sql: &'static str, arguments: Vec<Argument>) -> Result<Written, StoreFailure> {
        let mut statement = self.connection.prepare_cached(sql).map_err(Unexecuted::from)?;
        let changed = statement.execute(params_from_iter(Bound::arguments(arguments)?)).map_err(Unexecuted::from)?;
        Ok(Written::new(changed))
    }
}
