use port_core::Op;
use store_ports::{Query, Statements, StoreFailure};
use transcript_ports::Uuid;

use crate::boundary::{LinkRow, MessageRow, SegmentRow};
use crate::{Annotation, Stored};

#[derive(Clone, Debug, Op)]
pub struct ShownMessage(Uuid);

impl ShownMessage {
    #[must_use]
    pub fn named(uuid: Uuid) -> ShownMessage {
        ShownMessage(uuid)
    }
}

impl Query for ShownMessage {
    type Rows = Option<Stored>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Option<Stored>, StoreFailure> {
        let messages = MessageRow::named(&self.0, statements)?;
        let segments = SegmentRow::stored_for(&self.0, statements)?;
        let links = LinkRow::touching(&self.0, statements)?;
        let annotations = Annotation::stored_for(&self.0, statements)?;
        Ok(messages.into_iter().next().map(|message| Stored::new(message, segments, links, annotations)))
    }
}
