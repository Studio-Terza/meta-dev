use store_ports::{Execute, Query, StoreFailure};
use stream::Earlier;

use super::EarlierLines;
use crate::boundary::{EdgeRow, MessageRow};

impl Query for EarlierLines {
    type Rows = Earlier;

    fn rows(self, statements: &mut impl Execute) -> Result<Earlier, StoreFailure> {
        let said = MessageRow::earlier(&self.0, statements)?;
        let skipped = EdgeRow::passing(&self.0, statements)?;
        Ok(Earlier::new(said, skipped))
    }
}
