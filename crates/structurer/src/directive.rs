use std::str::FromStr;

use design_model::{Facet, NodeId, SegmentRelation, Sense, SpanRelation, Title};
use grammar::directive::Pieces;

use crate::reading::Reader;
use crate::{Misread, SegmentRef};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Directive {
    Place { segment: SegmentRef, node: NodeId, relation: SpanRelation },
    Sense { segment: SegmentRef, sense: Sense },
    Link { source: SegmentRef, target: SegmentRef, relation: SegmentRelation },
    Propose { refines: NodeId, facet: Facet, title: Title },
    Supersede { old: NodeId, by: NodeId },
    Conversational { segment: SegmentRef },
}

pub trait Visit {
    type Output;

    fn place(self, segment: &SegmentRef, node: NodeId, relation: SpanRelation) -> Self::Output;

    fn sense(self, segment: &SegmentRef, sense: Sense) -> Self::Output;

    fn link(self, source: &SegmentRef, target: &SegmentRef, relation: SegmentRelation) -> Self::Output;

    fn propose(self, refines: NodeId, facet: Facet, title: &Title) -> Self::Output;

    fn supersede(self, old: NodeId, by: NodeId) -> Self::Output;

    fn converse(self, segment: &SegmentRef) -> Self::Output;
}

impl Directive {
    pub fn visited<V: Visit>(&self, visitor: V) -> V::Output {
        match self {
            Directive::Place { segment, node, relation } => visitor.place(segment, *node, *relation),
            Directive::Sense { segment, sense } => visitor.sense(segment, *sense),
            Directive::Link { source, target, relation } => visitor.link(source, target, *relation),
            Directive::Propose { refines, facet, title } => visitor.propose(*refines, *facet, title),
            Directive::Supersede { old, by } => visitor.supersede(*old, *by),
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
