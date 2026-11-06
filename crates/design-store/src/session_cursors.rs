use port_core::Op;
use store_ports::{Query, Statements, StoreFailure};

use crate::Cursors;
use crate::boundary::CursorRow;

#[derive(Clone, Copy, Debug, Op)]
pub struct SessionCursors;

impl Query for SessionCursors {
    type Rows = Cursors;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Cursors, StoreFailure> {
        CursorRow::all(statements).map(|rows| Cursors::resumed(&rows))
    }
}
