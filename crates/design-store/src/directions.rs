mod boundary;

use design_model::{Node, NodeId, Provenance, Segment, SegmentRelation, Sense, SpanRelation, Supersession, Vague};
use port_core::Op;
use store_ports::Written;

use crate::{Refusal, Unseen};

use boundary::Annotating;
pub(crate) use boundary::Columns;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Directed {
    Placed { segment: Segment, node: NodeId, relation: SpanRelation },
    Sensed { segment: Segment, sense: Sense },
    Related { source: Segment, target: Segment, relation: SegmentRelation },
    Proposed { node: Node<Vague>, parent: NodeId },
    Superseded(Supersession),
    Conversational { segment: Segment },
}

#[derive(Debug, Op)]
pub struct Directions {
    provenance: Provenance,
    file: Unseen,
    directed: Vec<Directed>,
}

impl Directions {
    #[must_use]
    pub fn new(provenance: Provenance, file: Unseen, directed: Vec<Directed>) -> Directions {
        Directions { provenance, file, directed }
    }
}

impl Directed {
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

    #[must_use]
    pub fn covered(&self) -> Vec<&Segment> {
        match self {
            Directed::Placed { segment, .. }
            | Directed::Sensed { segment, .. }
            | Directed::Conversational { segment } => {
                vec![segment]
            }
            Directed::Related { source, target, .. } => vec![source, target],
            Directed::Proposed { .. } | Directed::Superseded(_) => Vec::new(),
        }
    }

    fn written(&self, annotating: &mut Annotating<'_, '_>) -> Result<Written, Refusal> {
        match self {
            Directed::Placed { segment, node, relation } => annotating.placed(segment, *node, *relation),
            Directed::Sensed { segment, sense } => annotating.sensed(segment, *sense),
            Directed::Related { source, target, relation } => annotating.related(source, target, *relation),
            Directed::Proposed { node, parent } => annotating.proposed(node, *parent),
            Directed::Superseded(supersession) => annotating.superseded(*supersession),
            Directed::Conversational { segment } => annotating.conversational(segment),
        }
    }
}
