use crate::store::{Done, Execute, StoreFailure, Written};
use port_core::{Op, OpName};

pub trait Batch: Op {
    type Failure: From<StoreFailure>;

    type Done: Done;

    fn written(self, statements: &mut impl Execute) -> Result<Self::Done, Self::Failure>;
}

pub trait Counts: Op {
    type Failure: From<StoreFailure>;

    fn counted<D: Done>(self, statements: &mut impl Execute) -> Result<D, Self::Failure>;
}

#[derive(Debug)]
pub struct Counted<B>(B);

impl<B: Counts> Counted<B> {
    #[must_use]
    pub fn new(batch: B) -> Counted<B> {
        Counted(batch)
    }
}

impl<B: Op> Op for Counted<B> {
    const NAME: OpName = B::NAME;
}

impl<B: Counts> Batch for Counted<B> {
    type Failure = B::Failure;

    type Done = Written;

    fn written(self, statements: &mut impl Execute) -> Result<Written, B::Failure> {
        self.0.counted(statements)
    }
}
