use std::collections::BTreeMap;

use transcript_ports::{Cursor, SessionId};

use crate::boundary::CursorRow;

#[derive(Debug, Default)]
pub struct Cursors(BTreeMap<SessionId, Cursor>);

impl Cursors {
    #[must_use]
    pub fn resuming(&self, session: &SessionId) -> Cursor {
        self.0.get(session).copied().unwrap_or_default()
    }

    pub(crate) fn resumed(rows: &[CursorRow]) -> Cursors {
        Cursors(rows.iter().map(|row| (row.session(), row.cursor())).collect())
    }
}
