use port_core::Op;

use crate::{Columns, Prepared, Statements, StoreFailure};

pub trait Record: Columns + Sized {
    type Key;

    type New: for<'record> From<&'record Self>;

    fn key(&self) -> &Self::Key;

    #[must_use]
    fn revised(&self, new: &Self::New) -> Self;

    fn created(new: &Self::New) -> Prepared<Self>;

    fn fetched(key: &Self::Key) -> Prepared<Self>;

    fn updated(&self) -> Prepared<()>;

    fn deleted(key: &Self::Key) -> Prepared<()>;
}

#[derive(Debug, Op)]
pub struct Created<R: Record>(R::New);

impl<R: Record> Created<R> {
    #[must_use]
    pub fn new(record: R::New) -> Created<R> {
        Created(record)
    }

    pub fn stored(self, statements: &mut Statements<'_>) -> Result<R, StoreFailure> {
        statements.one(R::created(&self.0))
    }
}
