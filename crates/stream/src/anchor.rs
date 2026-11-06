use transcript_ports::Uuid;

use crate::SegmentId;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Anchor {
    Message(Uuid),
    Segment(SegmentId),
}
