use port_core::Op;
use store_ports::{Batch, Statements, StoreFailure, Written};

use super::anchored::Anchored;
use super::resegmented::Resegmented;

#[derive(Debug, Op)]
pub enum Rewrite {
    Resegmented(Resegmented),
    Anchored(Anchored),
}

impl Batch for Rewrite {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        match self {
            Rewrite::Resegmented(resegmented) => resegmented.written(statements),
            Rewrite::Anchored(anchored) => anchored.written(statements),
        }
    }
}
