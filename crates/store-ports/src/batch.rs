use port_core::Op;

use crate::{Statements, StoreFailure, Written};

pub trait Batch: Op {
    type Failure: From<StoreFailure>;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, Self::Failure>;
}
