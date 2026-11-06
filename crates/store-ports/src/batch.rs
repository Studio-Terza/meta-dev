use port_core::Op;

use crate::{Statements, StoreFailure, Written};

pub trait Batch: Op {
    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure>;
}
