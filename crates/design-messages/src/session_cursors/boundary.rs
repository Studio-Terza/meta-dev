use store_ports::{Query, Statements, StoreFailure};

use super::SessionCursors;
use crate::Cursors;
use crate::boundary::CursorRow;

impl Query for SessionCursors {
    type Rows = Cursors;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Cursors, StoreFailure> {
        CursorRow::all(statements).map(|rows| Cursors::resumed(&rows))
    }
}
