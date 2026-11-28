use port_core::Op;

use crate::{Argument, Columns, Execute, Prepared, StoreFailure};

pub trait Record: Columns + Sized {
    type Key;

    type New: for<'record> From<&'record Self>;

    fn key(&self) -> &Self::Key;

    fn created(new: &Self::New) -> Prepared<Self>;

    fn fetched(key: &Self::Key) -> Prepared<Self>;

    fn updated(&self) -> Prepared<()>;

    fn deleted(key: &Self::Key) -> Prepared<()>;

    fn listed() -> Prepared<Self>;

    fn overwrites(argument: &Argument) -> Vec<Prepared<()>>;
}

#[derive(Debug, Op)]
pub struct Created<R: Record>(R::New);

impl<R: Record> Created<R> {
    #[must_use]
    pub fn new(record: R::New) -> Created<R> {
        Created(record)
    }

    pub fn stored(self, statements: &mut impl Execute) -> Result<R, StoreFailure> {
        statements.one(R::created(&self.0))
    }
}
