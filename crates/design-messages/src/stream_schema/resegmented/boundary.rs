use newtype::Counter;
use store_ports::{Batch, Execute, Released, StoreFailure, Written};

use super::{Referenced, Resegmented};
use crate::boundary::{MessageRow, Row, SegmentRow};

impl Batch for Resegmented {
    type Failure = StoreFailure;

    fn written(self, statements: &mut impl Execute) -> Result<Written, StoreFailure> {
        let messages = MessageRow::every(statements)?;
        let derived: Vec<SegmentRow> = messages.iter().flat_map(SegmentRow::within).collect();
        let stale = Resegmented::stale(&derived, SegmentRow::every(statements)?);
        let released =
            stale.iter().map(|row| row.released(statements)).collect::<Result<Vec<Released>, StoreFailure>>()?;
        let cleared = Referenced::cleared(derived, &stale, &released)?;
        let written =
            cleared.iter().map(|row| row.written(statements)).collect::<Result<Vec<Written>, StoreFailure>>()?;
        Ok(Written::total(written.into_iter()))
    }
}
