use port_core::Op;
use store_ports::{Batch, Execute, StoreFailure};

use super::anchored::Anchored;
use super::resegmented::Resegmented;

#[derive(Debug, Op)]
pub enum Rewrite {
    Resegmented(Resegmented),
    Anchored(Anchored),
}

impl Batch for Rewrite {
    type Failure = StoreFailure;

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), StoreFailure> {
        match self {
            Rewrite::Resegmented(resegmented) => resegmented.written(statements),
            Rewrite::Anchored(anchored) => anchored.written(statements),
        }
    }
}
