mod anchored;
mod resegmented;

use port_core::Op;
use store_ports::{Batch, Migrator, Schema, Statements, StoreFailure, Version, Written};

use anchored::Anchored;
use resegmented::Resegmented;

static MIGRATOR: Migrator = store_ports::migrate!();

#[derive(Clone, Copy, Debug)]
pub struct StreamSchema;

#[derive(Debug, Op)]
pub enum Rewrite {
    Resegmented(Resegmented),
    Anchored(Anchored),
}

impl Schema for StreamSchema {
    type Rewrite = Rewrite;

    fn migrator() -> &'static Migrator {
        &MIGRATOR
    }

    fn rewrite(pending: Version) -> Option<Rewrite> {
        (pending == Resegmented::PENDING)
            .then_some(Rewrite::Resegmented(Resegmented))
            .or_else(|| (pending == Anchored::PENDING).then_some(Rewrite::Anchored(Anchored)))
    }
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
