mod boundary;

use design_model::{Node, NodeId, Provenance, Segment, SegmentRelation, Sense, SpanRelation, Supersession};
use port_core::Op;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Directed {
    Placed { segment: Segment, node: NodeId, relation: SpanRelation },
    Sensed { segment: Segment, sense: Sense },
    Related { source: Segment, target: Segment, relation: SegmentRelation },
    Proposed { node: Node, parent: NodeId },
    Superseded(Supersession),
    Conversational { segment: Segment },
}

pub trait Directing {
    type Output;

    fn placed(self, segment: &Segment, node: NodeId, relation: SpanRelation) -> Self::Output;

    fn sensed(self, segment: &Segment, sense: Sense) -> Self::Output;

    fn related(self, source: &Segment, target: &Segment, relation: SegmentRelation) -> Self::Output;

    fn proposed(self, node: &Node, parent: NodeId) -> Self::Output;

    fn superseded(self, supersession: Supersession) -> Self::Output;

    fn conversational(self, segment: &Segment) -> Self::Output;
}

#[derive(Debug, Op)]
pub struct Directions {
    provenance: Provenance,
    directed: Vec<Directed>,
}

impl Directions {
    #[must_use]
    pub fn new(provenance: Provenance, directed: Vec<Directed>) -> Directions {
        Directions { provenance, directed }
    }
}

impl Directed {
    #[must_use]
    pub fn segments(&self) -> Vec<&Segment> {
        match self {
            Directed::Placed { segment, .. }
            | Directed::Sensed { segment, .. }
            | Directed::Conversational { segment } => vec![segment],
            Directed::Related { source, target, .. } => vec![source, target],
            Directed::Proposed { .. } | Directed::Superseded(_) => Vec::new(),
        }
    }

    #[must_use]
    pub fn proposed(&self) -> Option<NodeId> {
        match self {
            Directed::Proposed { node, .. } => Some(node.identity()),
            Directed::Placed { .. }
            | Directed::Sensed { .. }
            | Directed::Related { .. }
            | Directed::Superseded(_)
            | Directed::Conversational { .. } => None,
        }
    }

    pub fn visited<D: Directing>(&self, directing: D) -> D::Output {
        match self {
            Directed::Placed { segment, node, relation } => directing.placed(segment, *node, *relation),
            Directed::Sensed { segment, sense } => directing.sensed(segment, *sense),
            Directed::Related { source, target, relation } => directing.related(source, target, *relation),
            Directed::Proposed { node, parent } => directing.proposed(node, *parent),
            Directed::Superseded(supersession) => directing.superseded(*supersession),
            Directed::Conversational { segment } => directing.conversational(segment),
        }
    }
}
