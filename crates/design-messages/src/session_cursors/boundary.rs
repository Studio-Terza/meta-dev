use ports::store::{Execute, Query, StoreFailure};

use super::SessionCursors;
use crate::Cursors;
use crate::boundary::CursorRow;

impl Query for SessionCursors {
    type Rows = Cursors;

    fn rows(self, statements: &mut impl Execute) -> Result<Cursors, StoreFailure> {
        CursorRow::all(statements).map(|rows| Cursors::resumed(&rows))
    }
}
