use crate::port::Op;
use crate::store::{Execute, StoreFailure};

pub trait Query: Op {
    type Rows;

    fn rows(self, statements: &mut impl Execute) -> Result<Self::Rows, StoreFailure>;
}
