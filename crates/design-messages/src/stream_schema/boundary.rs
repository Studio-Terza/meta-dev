use port_core::Op;
use store_ports::{Batch, Execute, StoreFailure};

#[derive(Debug, Op)]
pub enum Rewrite {}

impl Batch for Rewrite {
    type Failure = StoreFailure;

    type Done = ();

    fn written(self, _: &mut impl Execute) -> Result<(), StoreFailure> {
        match self {}
    }
}
