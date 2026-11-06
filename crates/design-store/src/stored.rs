use crate::boundary::{LinkRow, MessageRow, SegmentRow};

#[derive(Debug)]
pub struct Stored {
    message: MessageRow,
    segments: Vec<SegmentRow>,
    links: Vec<LinkRow>,
}

impl Stored {
    pub(crate) fn new(message: MessageRow, segments: Vec<SegmentRow>, links: Vec<LinkRow>) -> Stored {
        Stored { message, segments, links }
    }

    #[must_use]
    pub fn message(&self) -> &MessageRow {
        &self.message
    }

    #[must_use]
    pub fn segments(&self) -> &[SegmentRow] {
        &self.segments
    }

    #[must_use]
    pub fn links(&self) -> &[LinkRow] {
        &self.links
    }
}
