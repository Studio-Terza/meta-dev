use std::fmt;

use transcript_ports::Uuid;

use crate::segment_id::SegmentId;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Anchor {
    Message(Uuid),
    Segment(SegmentId),
}

impl Anchor {
    #[must_use]
    pub fn message(&self) -> &Uuid {
        match self {
            Anchor::Message(uuid) => uuid,
            Anchor::Segment(id) => id.message(),
        }
    }
}

impl fmt::Display for Anchor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Anchor::Message(uuid) => write!(formatter, "{uuid}"),
            Anchor::Segment(id) => write!(formatter, "{id}"),
        }
    }
}
