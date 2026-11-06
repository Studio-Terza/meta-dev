mod boundary;

use design_model::{Node, NodeId, Provenance, Segment, SegmentRelation, Sense, SpanRelation};
use port_core::Op;
use store_ports::{StoreFailure, Written};

use boundary::Annotating;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Directed {
    Placed { segment: Segment, node: NodeId, relation: SpanRelation },
    Sensed { segment: Segment, sense: Sense },
    Related { source: Segment, target: Segment, relation: SegmentRelation },
    Proposed { node: Node, parent: NodeId },
    Superseded { old: NodeId, by: NodeId },
    Conversational { segment: Segment },
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
            Directed::Proposed { .. } | Directed::Superseded { .. } => Vec::new(),
        }
    }

    #[must_use]
    pub fn proposed(&self) -> Option<NodeId> {
        match self {
            Directed::Proposed { node, .. } => Some(node.identity()),
            Directed::Placed { .. }
            | Directed::Sensed { .. }
            | Directed::Related { .. }
            | Directed::Superseded { .. }
            | Directed::Conversational { .. } => None,
        }
    }

    fn written(&self, annotating: &mut Annotating<'_, '_>) -> Result<Written, StoreFailure> {
        match self {
            Directed::Placed { segment, node, relation } => annotating.placed(segment, *node, *relation),
            Directed::Sensed { segment, sense } => annotating.sensed(segment, *sense),
            Directed::Related { source, target, relation } => annotating.related(source, target, *relation),
            Directed::Proposed { node, parent } => annotating.proposed(node, *parent),
            Directed::Superseded { old, by } => annotating.superseded(*old, *by),
            Directed::Conversational { segment } => annotating.conversational(segment),
        }
    }
}
