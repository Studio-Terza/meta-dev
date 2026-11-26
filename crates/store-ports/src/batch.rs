use port_core::Op;

use crate::{Execute, StoreFailure, Written};

pub trait Batch: Op {
    type Failure: From<StoreFailure>;

    fn written(self, statements: &mut impl Execute) -> Result<Written, Self::Failure>;
}
