use design_model::{AnyNode, Facet, NodeId, Stage, Title};
use transcript_ports::Body;

use crate::{Owed, SegmentRef};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Said {
    segment: SegmentRef,
    body: Body,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Context {
    node: NodeId,
    facet: Facet,
    stage: Stage,
    title: Option<Title>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Job {
    segments: Vec<Said>,
    context: Vec<Context>,
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

impl Context {
    pub(crate) fn new(node: NodeId, facet: Facet, stage: Stage, title: Option<Title>) -> Context {
        Context { node, facet, stage, title }
    }

    pub(crate) fn node(&self) -> NodeId {
        self.node
    }

    pub(crate) fn facet(&self) -> Facet {
        self.facet
    }

    pub(crate) fn stage(&self) -> Stage {
        self.stage
    }

    pub(crate) fn title(&self) -> Option<&Title> {
        self.title.as_ref()
    }
}

impl From<&AnyNode> for Context {
    fn from(node: &AnyNode) -> Context {
        Context::new(node.identity(), node.entry().facet(), node.stage().stage(), node.title().cloned())
    }
}

impl Job {
    pub const SEGMENTS: usize = 8;

    #[must_use]
    pub fn new(segments: Vec<Said>, nodes: &[AnyNode]) -> Job {
        Job { segments, context: nodes.iter().map(Context::from).collect() }
    }

    #[must_use]
    pub fn owed(&self) -> Owed {
        self.segments.iter().map(|said| said.segment.clone()).collect()
    }

    pub(crate) fn segments(&self) -> &[Said] {
        &self.segments
    }

    pub(crate) fn context(&self) -> &[Context] {
        &self.context
    }
}
