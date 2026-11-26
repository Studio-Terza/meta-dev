use store_ports::{Execute, Query, StoreFailure};

use super::ShownMessage;
use crate::Stored;
use crate::boundary::{LinkRow, MessageRow, SegmentRow};
use design_closure::Annotation;

impl Query for ShownMessage {
    type Rows = Option<Stored>;

    fn rows(self, statements: &mut impl Execute) -> Result<Option<Stored>, StoreFailure> {
        let messages = statements.rows(MessageRow::named(&self.0))?;
        let segments = statements.rows(SegmentRow::stored_for(&self.0))?;
        let links = LinkRow::touching(&self.0, statements)?;
        let annotations = Annotation::stored_for(&self.0, statements)?;
        Ok(messages.into_iter().next().map(|message| Stored::new(message, segments, links, annotations)))
    }
}
