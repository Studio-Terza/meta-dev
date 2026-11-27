use std::fmt::Debug;
use std::vec;

use crate::store::{Argument, Batch, ColumnFailure, Columns, Counts, Done, StoreFailure, Value};
use port_core::Op;

pub trait Execute: Debug {
    fn fetched(&mut self, sql: &'static str, arguments: Vec<Argument>) -> Result<Vec<Vec<Value>>, StoreFailure>;

    fn executed<D: Done>(&mut self, sql: &'static str, arguments: Vec<Argument>) -> Result<D, StoreFailure>;

    fn rows<Row: Columns>(&mut self, query: Prepared<Row>) -> Result<Vec<Row>, StoreFailure> {
        let decode = query.decode;
        self.fetched(query.sql, query.arguments)?
            .into_iter()
            .map(|values| decode(&mut Values::from(values)).map_err(StoreFailure::from))
            .collect()
    }

    fn one<Row: Columns>(&mut self, query: Prepared<Row>) -> Result<Row, StoreFailure> {
        self.rows(query)?.into_iter().next().ok_or(StoreFailure::Missing)
    }

    fn written(&mut self, query: Prepared<()>) -> Result<(), StoreFailure> {
        self.done(query)
    }

    fn done<D: Done>(&mut self, query: Prepared<()>) -> Result<D, StoreFailure> {
        self.executed(query.sql, query.arguments)
    }

    fn written_all<B: Batch>(&mut self, batches: impl IntoIterator<Item = B>) -> Result<B::Done, B::Failure>
    where
        Self: Sized,
    {
        let done =
            batches.into_iter().map(|batch| batch.written(self)).collect::<Result<Vec<B::Done>, B::Failure>>()?;
        Ok(Done::total(done.into_iter()))
    }

    fn counted_all<B: Counts, D: Done>(&mut self, batches: impl IntoIterator<Item = B>) -> Result<D, B::Failure>
    where
        Self: Sized,
    {
        let done = batches.into_iter().map(|batch| batch.counted(self)).collect::<Result<Vec<D>, B::Failure>>()?;
        Ok(D::total(done.into_iter()))
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

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), StoreFailure> {
        Execute::written(statements, self)
    }
}

impl Counts for Prepared<()> {
    type Failure = StoreFailure;

    fn counted<D: Done>(self, statements: &mut impl Execute) -> Result<D, StoreFailure> {
        statements.done(self)
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
