use std::fmt::Debug;
use std::vec;

use pipeline::Measure;
use port_core::Op;

use crate::{Argument, ColumnFailure, Columns, StoreFailure, Value, Written};

pub trait Batch: Op {
    type Failure: From<StoreFailure>;

    type Done: Measure;

    fn written(self, statements: &mut impl Execute) -> Result<Self::Done, Self::Failure>;
}

pub trait Execute: Debug {
    fn fetched(&mut self, sql: &'static str, arguments: Vec<Argument>) -> Result<Vec<Vec<Value>>, StoreFailure>;

    fn executed(&mut self, sql: &'static str, arguments: Vec<Argument>) -> Result<Written, StoreFailure>;

    fn rows<Row: Columns>(&mut self, query: Prepared<Row>) -> Result<Vec<Row>, StoreFailure> {
        let decode = query.decode;
        self.fetched(query.sql, query.arguments)?
            .into_iter()
            .map(|values| decode(&mut Values::from(values)).map_err(StoreFailure::from))
            .collect()
    }

    fn written<B: Batch>(&mut self, batches: impl IntoIterator<Item = B>) -> Result<B::Done, B::Failure>
    where
        Self: Sized,
    {
        let done =
            batches.into_iter().map(|batch| batch.written(self)).collect::<Result<Vec<B::Done>, B::Failure>>()?;
        Ok(B::Done::total(done))
    }
}

#[derive(Debug, Op)]
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

impl Batch for Prepared<()> {
    type Failure = StoreFailure;

    type Done = Written;

    fn written(self, statements: &mut impl Execute) -> Result<Written, StoreFailure> {
        statements.executed(self.sql, self.arguments)
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
