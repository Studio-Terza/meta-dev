use std::fmt::Debug;
use std::vec;

use crate::{Argument, ColumnFailure, Columns, Released, StoreFailure, Value, Written};

pub trait Execute: Debug {
    fn fetched(&mut self, sql: &'static str, arguments: Vec<Argument>) -> Result<Vec<Vec<Value>>, StoreFailure>;

    fn executed(&mut self, sql: &'static str, arguments: Vec<Argument>) -> Result<Written, StoreFailure>;
}

#[derive(Debug)]
pub struct Prepared<Row> {
    sql: &'static str,
    arguments: Vec<Argument>,
    decode: fn(&mut Values) -> Result<Row, ColumnFailure>,
}

#[derive(Debug)]
pub struct Values(vec::IntoIter<Value>);

impl Prepared<()> {
    #[must_use]
    pub fn written(sql: &'static str, arguments: Vec<Argument>) -> Prepared<()> {
        Prepared { sql, arguments, decode: |_values| Ok(()) }
    }
}

impl<Row: Columns> Prepared<Row> {
    #[must_use]
    pub fn read(
        sql: &'static str,
        arguments: Vec<Argument>,
        decode: fn(&mut Values) -> Result<Row, ColumnFailure>,
    ) -> Prepared<Row> {
        Prepared { sql, arguments, decode }
    }
}

impl Values {
    pub fn taken(&mut self) -> Result<Value, ColumnFailure> {
        self.0.next().ok_or(ColumnFailure::Absent)
    }
}

impl From<Vec<Value>> for Values {
    fn from(values: Vec<Value>) -> Values {
        Values(values.into_iter())
    }
}

impl dyn Execute + '_ {
    pub fn rows<Row: Columns>(&mut self, query: Prepared<Row>) -> Result<Vec<Row>, StoreFailure> {
        let decode = query.decode;
        self.fetched(query.sql, query.arguments)?
            .into_iter()
            .map(|values| decode(&mut Values::from(values)).map_err(StoreFailure::from))
            .collect()
    }

    pub fn one<Row: Columns>(&mut self, query: Prepared<Row>) -> Result<Row, StoreFailure> {
        self.rows(query)?.into_iter().next().ok_or(StoreFailure::Missing)
    }

    pub fn written(&mut self, query: Prepared<()>) -> Result<Written, StoreFailure> {
        self.executed(query.sql, query.arguments)
    }

    pub fn released(&mut self, query: Prepared<()>) -> Result<Released, StoreFailure> {
        self.written(query).map(|_written| Released::Deleted).or_else(StoreFailure::released)
    }
}
