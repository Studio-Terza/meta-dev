use design_model::{AnyNode, Facet, NodeId, Project, Stage, Title};
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
    project: Project,
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
    pub(crate) fn new(node: NodeId, project: Project, facet: Facet, stage: Stage) -> Context {
        Context { node, project, facet, stage, title: None }
    }

    pub(crate) fn titled(self, title: Option<Title>) -> Context {
        Context { title, ..self }
    }

    pub(crate) fn node(&self) -> NodeId {
        self.node
    }

    pub(crate) fn project(&self) -> &Project {
        &self.project
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
        let entry = node.entry();
        Context::new(node.identity(), entry.project().clone(), entry.facet(), node.stage().stage())
            .titled(node.title().cloned())
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
