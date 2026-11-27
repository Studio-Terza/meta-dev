mod boundary;

use rusqlite::types::{Value as Stored, ValueRef};
use rusqlite::{Connection, Row};
use store_ports::{Argument, ColumnFailure, StoreFailure, Value};

#[derive(Debug)]
pub(in super::super) struct Connected<'run> {
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
            ValueRef::Blob(blob) => Ok(Value::Blob(blob.to_vec())),
        }
    }
}
