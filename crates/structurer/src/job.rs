use design_model::{JobId, Node};
use transcript_ports::Body;

use crate::SegmentRef;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Said {
    segment: SegmentRef,
    body: Body,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Job {
    id: JobId,
    segments: Vec<Said>,
    context: Vec<Node>,
}

impl Said {
    #[must_use]
    pub fn new(segment: SegmentRef, body: Body) -> Said {
        Said { segment, body }
    }

    pub(crate) fn segment(&self) -> &SegmentRef {
        &self.segment
    }

    pub(crate) fn body(&self) -> &Body {
        &self.body
    }
}

impl Job {
    #[must_use]
    pub fn new(id: JobId, segments: Vec<Said>, context: Vec<Node>) -> Job {
        Job { id, segments, context }
    }

    #[must_use]
    pub fn identity(&self) -> JobId {
        self.id
    }

    pub(crate) fn segments(&self) -> &[Said] {
        &self.segments
    }

    pub(crate) fn context(&self) -> &[Node] {
        &self.context
    }
}
