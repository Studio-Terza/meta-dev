use std::fmt;

use design_model::{Facet, NodeId, Ordinal, Prefix, Segment, SegmentRelation, Sense, SpanRelation, Title};
use grammar_directive::Quote;
use newtype::Wrapping;

use crate::verb::{By, New, Refines, Verb};
use crate::{Bind, Directive, Label, NodeRef, SegmentRef, Visit};

#[derive(Clone, Copy, Debug)]
struct Id(NodeId);

#[derive(Clone, Copy, Debug)]
struct Quoted<'title>(&'title Title);

#[derive(Clone, Copy, Debug)]
struct Fresh<'label>(Option<&'label Label>);

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

impl fmt::Display for Label {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_ref())
    }
}

impl fmt::Display for NodeRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.bound(Rendering(formatter))
    }
}

impl Bind for Rendering<'_, '_> {
    type Output = fmt::Result;

    fn stored(self, id: NodeId) -> fmt::Result {
        write!(self.0, "{}", Id(id))
    }

    fn labelled(self, label: &Label) -> fmt::Result {
        write!(self.0, "{label}")
    }
}

impl fmt::Display for Fresh<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        New::New.fmt(formatter)?;
        self.0.iter().try_for_each(|label| write!(formatter, " {label}"))
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

    fn place(self, segment: &SegmentRef, node: &NodeRef, relation: SpanRelation) -> fmt::Result {
        write!(self.0, "{}({segment}, {node}, {relation})", Verb::Place)
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

    fn propose(self, fresh: Option<&Label>, refines: &NodeRef, facet: Facet, title: &Title) -> fmt::Result {
        write!(
            self.0,
            "{}({}, {}({refines}), {facet}, {})",
            Verb::Propose,
            Fresh(fresh),
            Refines::Refines,
            Quoted(title)
        )
    }

    fn supersede(self, old: &NodeRef, by: &NodeRef) -> fmt::Result {
        write!(self.0, "{}({old}, {}({by}))", Verb::Supersede, By::By)
    }

    fn converse(self, segment: &SegmentRef) -> fmt::Result {
        write!(self.0, "{}({segment})", Verb::Conversational)
    }
}
