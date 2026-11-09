use newtype::Counter;
use port_core::Op;
use store_ports::{Batch, Statements, StoreFailure, Version, Written};

use crate::boundary::{MessageRow, Rederived, Row, SegmentRow};

#[derive(Debug, Op)]
pub struct Resegmented;

impl Resegmented {
    pub(super) const BEFORE: Version = Version::new(20_261_108_000_012);
}

impl Batch for Resegmented {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let messages = MessageRow::every(statements)?;
        let rederived: Vec<Rederived> = messages.iter().flat_map(SegmentRow::within).map(Rederived::from).collect();
        let written =
            rederived.iter().map(|row| row.written(statements)).collect::<Result<Vec<Written>, StoreFailure>>()?;
        Ok(Written::total(written.into_iter()))
    }
}
