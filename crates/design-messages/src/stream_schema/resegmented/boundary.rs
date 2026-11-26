use store_ports::{Batch, Execute, StoreFailure};

use super::{Referenced, Resegmented};
use crate::boundary::{MessageRow, Row, SegmentRow};

impl Batch for Resegmented {
    type Failure = StoreFailure;

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), StoreFailure> {
        let messages = MessageRow::every(statements)?;
        let derived: Vec<SegmentRow> = messages.iter().flat_map(SegmentRow::within).collect();
        let stale = Resegmented::stale(&derived, SegmentRow::every(statements)?);
        let released = statements.released_all(stale.iter().map(SegmentRow::deleted))?;
        let cleared = Referenced::cleared(derived, &stale, &released)?;
        statements.written_all(cleared.iter().map(Row::inserted))
    }
}
