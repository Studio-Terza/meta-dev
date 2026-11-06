use std::fmt;

use design_model::{Facet, NodeId, Ordinal, Prefix, Segment, SegmentRelation, Sense, SpanRelation, Title};
use grammar::directive::Quote;
use newtype::Newtype;

use crate::verb::{By, New, Refines, Verb};
use crate::{Directive, SegmentRef, Visit};

#[derive(Clone, Copy, Debug)]
struct Id(NodeId);

#[derive(Clone, Copy, Debug)]
struct Quoted<'title>(&'title Title);

struct Rendering<'out, 'formatter>(&'out mut fmt::Formatter<'formatter>);

impl From<&Segment> for SegmentRef {
    fn from(segment: &Segment) -> SegmentRef {
        SegmentRef::new(Prefix::from(segment), Ordinal::from(segment))
    }
}

impl fmt::Display for SegmentRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix: &Prefix = self.as_ref();
        write!(formatter, "{prefix}.{}", Ordinal::from(self))
    }
}

impl fmt::Display for Id {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "n{}", self.0.into_inner())
    }
}

impl fmt::Display for Quoted<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&Quote::quoting(&self.0.to_string()))
    }
}

impl fmt::Display for Directive {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.visited(Rendering(formatter))
    }
}

impl Visit for Rendering<'_, '_> {
    type Output = fmt::Result;

    fn place(self, segment: &SegmentRef, node: NodeId, relation: SpanRelation) -> fmt::Result {
        write!(self.0, "{}({segment}, {}, {relation})", Verb::Place, Id(node))
    }

    fn sense(self, segment: &SegmentRef, sense: Sense) -> fmt::Result {
        write!(
            self.0,
            "{}({segment}, {}, {}, {}, {})",
            Verb::Sense,
            sense.kind(),
            sense.specificity(),
            sense.confidence(),
            sense.explicitness()
        )
    }

    fn link(self, source: &SegmentRef, target: &SegmentRef, relation: SegmentRelation) -> fmt::Result {
        write!(self.0, "{}({source}, {target}, {relation})", Verb::Link)
    }

    fn propose(self, refines: NodeId, facet: Facet, title: &Title) -> fmt::Result {
        write!(
            self.0,
            "{}({}, {}({}), {facet}, {})",
            Verb::Propose,
            New::New,
            Refines::Refines,
            Id(refines),
            Quoted(title)
        )
    }

    fn supersede(self, old: NodeId, by: NodeId) -> fmt::Result {
        write!(self.0, "{}({}, {}({}))", Verb::Supersede, Id(old), By::By, Id(by))
    }

    fn converse(self, segment: &SegmentRef) -> fmt::Result {
        write!(self.0, "{}({segment})", Verb::Conversational)
    }
}
