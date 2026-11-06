use port_core::Op;
use store_ports::{Query, Statements, StoreFailure};
use stream::Earlier;
use transcript_ports::SessionId;

use crate::boundary::{EdgeRow, MessageRow};

#[derive(Clone, Debug, Op)]
pub struct EarlierLines(SessionId);

impl EarlierLines {
    #[must_use]
    pub fn within(session: SessionId) -> EarlierLines {
        EarlierLines(session)
    }
}

impl Query for EarlierLines {
    type Rows = Earlier;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Earlier, StoreFailure> {
        let said = MessageRow::earlier(&self.0, statements)?;
        let skipped = EdgeRow::passing(&self.0, statements)?;
        Ok(Earlier::new(said, skipped))
    }
}
