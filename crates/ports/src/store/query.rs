use crate::store::{Execute, StoreFailure};
use port_core::Op;

pub trait Query: Op {
    type Rows;

    fn rows(self, statements: &mut impl Execute) -> Result<Self::Rows, StoreFailure>;
}
