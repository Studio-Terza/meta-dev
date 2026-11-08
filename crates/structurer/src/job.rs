use design_model::Outline;
use transcript_ports::Body;

use crate::SegmentRef;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Said {
    segment: SegmentRef,
    body: Body,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Job {
    segments: Vec<Said>,
    context: Vec<Outline>,
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
    pub const SEGMENTS: usize = 8;

    #[must_use]
    pub fn new(segments: Vec<Said>, context: Vec<Outline>) -> Job {
        Job { segments, context }
    }

    pub(crate) fn segments(&self) -> &[Said] {
        &self.segments
    }

    pub(crate) fn context(&self) -> &[Outline] {
        &self.context
    }
}
