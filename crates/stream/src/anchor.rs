mod boundary;

use std::fmt;

use newtype::Spelled;
use transcript_ports::Uuid;

use crate::segment_id::SegmentId;

#[derive(Spelled, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Anchor {
    Message(Uuid),
    Segment(SegmentId),
}

pub trait Anchors {
    type Output;

    fn message(self, message: &Uuid) -> Self::Output;

    fn segment(self, segment: &SegmentId) -> Self::Output;
}

impl Anchor {
    #[must_use]
    pub fn message(&self) -> &Uuid {
        match self {
            Anchor::Message(uuid) => uuid,
            Anchor::Segment(id) => id.message(),
        }
    }

    pub fn visited<A: Anchors>(&self, anchors: A) -> A::Output {
        match self {
            Anchor::Message(uuid) => anchors.message(uuid),
            Anchor::Segment(id) => anchors.segment(id),
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
