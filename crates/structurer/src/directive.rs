use std::str::FromStr;

use design_model::{Facet, SegmentRelation, Sense, SpanRelation, Title};
use grammar_directive::Pieces;

use crate::reading::Reader;
use crate::{Edge, Label, Misread, NodeRef, Proposal, SegmentRef};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Directive {
    Place { segment: SegmentRef, node: NodeRef, relation: SpanRelation },
    Sense { segment: SegmentRef, sense: Sense },
    Link { edge: Edge<SegmentRef>, relation: SegmentRelation },
    Propose { proposal: Proposal, facet: Facet, title: Title },
    Supersede(Edge<NodeRef>),
    Conversational { segment: SegmentRef },
}

pub trait Visit {
    type Output;

    fn place(self, segment: &SegmentRef, node: &NodeRef, relation: SpanRelation) -> Self::Output;

    fn sense(self, segment: &SegmentRef, sense: Sense) -> Self::Output;

    fn link(self, source: &SegmentRef, target: &SegmentRef, relation: SegmentRelation) -> Self::Output;

    fn propose(self, fresh: Option<&Label>, refines: &NodeRef, facet: Facet, title: &Title) -> Self::Output;

    fn supersede(self, old: &NodeRef, by: &NodeRef) -> Self::Output;

    fn converse(self, segment: &SegmentRef) -> Self::Output;
}

impl Directive {
    pub fn visited<V: Visit>(&self, visitor: V) -> V::Output {
        match self {
            Directive::Place { segment, node, relation } => visitor.place(segment, node, *relation),
            Directive::Sense { segment, sense } => visitor.sense(segment, *sense),
            Directive::Link { edge, relation } => edge.ends(|source, target| visitor.link(source, target, *relation)),
            Directive::Propose { proposal, facet, title } => proposal.visited(visitor, *facet, title),
            Directive::Supersede(edge) => edge.ends(|old, by| visitor.supersede(old, by)),
            Directive::Conversational { segment } => visitor.converse(segment),
        }
    }
}

impl FromStr for Directive {
    type Err = Misread;

    fn from_str(text: &str) -> Result<Directive, Misread> {
        Reader::over(&Pieces::line(text)).directive()
    }
}
