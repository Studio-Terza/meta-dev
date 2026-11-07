use std::fmt;

use design_model::{
    AnyNode, AnyStage, Dropped, Entered, NodeEdge, NodeId, Ordinal, Provenance, Segment, SegmentRelation, Sense,
    SpanRelation, Staging, Terminal,
};
use design_store::{Annotation, AppliedId, Earlier, Placed, Placement, Remarking};
use newtype::{Counter, Newtype};
use structurer::SegmentRef;
use transcript_ports::Authorship;

use super::request::Applied;

#[derive(Clone, Copy, Debug)]
pub struct Id(NodeId);

#[derive(Clone, Copy, Debug)]
pub struct Edge(NodeEdge);

#[derive(Clone, Copy, Debug)]
pub struct Line<'node>(&'node AnyNode);

#[derive(Clone, Copy, Debug)]
struct Details<'stage>(&'stage AnyStage);

#[derive(Clone, Copy, Debug)]
pub struct Neighbourhood<'placed>(&'placed Placed);

#[derive(Clone, Copy, Debug)]
pub struct Annotated<'annotation>(&'annotation Annotation);

#[derive(Clone, Copy, Debug)]
struct Placing<'placement>(&'placement Placement);

#[derive(Clone, Copy, Debug)]
struct Words(Sense);

#[derive(Clone, Copy, Debug)]
pub struct Repeat<'earlier>(&'earlier Earlier);

struct Writing<'out, 'formatter>(&'out mut fmt::Formatter<'formatter>);

impl From<NodeId> for Id {
    fn from(id: NodeId) -> Id {
        Id(id)
    }
}

impl From<NodeEdge> for Edge {
    fn from(edge: NodeEdge) -> Edge {
        Edge(edge)
    }
}

impl<'node> From<&'node AnyNode> for Line<'node> {
    fn from(node: &'node AnyNode) -> Line<'node> {
        Line(node)
    }
}

impl<'placed> From<&'placed Placed> for Neighbourhood<'placed> {
    fn from(placed: &'placed Placed) -> Neighbourhood<'placed> {
        Neighbourhood(placed)
    }
}

impl<'annotation> From<&'annotation Annotation> for Annotated<'annotation> {
    fn from(annotation: &'annotation Annotation) -> Annotated<'annotation> {
        Annotated(annotation)
    }
}

impl<'earlier> From<&'earlier Earlier> for Repeat<'earlier> {
    fn from(earlier: &'earlier Earlier) -> Repeat<'earlier> {
        Repeat(earlier)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "n{}", self.0.into_inner())
    }
}

impl fmt::Display for Edge {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [source, target] = self.0.ends();
        write!(formatter, "{} {} {}", Id(source), self.0.relation(), Id(target))
    }
}

impl fmt::Display for Line<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entry = self.0.entry();
        write!(
            formatter,
            "{} {} {} {} {} entered {}",
            Id(self.0.identity()),
            entry.project(),
            entry.facet(),
            self.0.stage().stage(),
            self.0.casting().role(),
            Entered::from(entry.provenance())
        )?;
        self.0.title().iter().try_for_each(|title| write!(formatter, " {:?}", title.to_string()))
    }
}

impl fmt::Display for Neighbourhood<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{}", Line(self.0.node()))?;
        write!(formatter, "{}", Details(self.0.node().stage()))?;
        self.0
            .relatives()
            .iter()
            .try_for_each(|relative| writeln!(formatter, "  {} {}", relative.kin(), Line(relative.node())))?;
        self.0.links().iter().try_for_each(|link| {
            writeln!(formatter, "  {} entered {}", Edge(link.edge()), Entered::from(link.provenance()))
        })?;
        let casting = self.0.node().casting();
        writeln!(formatter, "  role {} entered {}", casting.role(), Entered::from(casting.provenance()))?;
        self.0.placements().iter().try_for_each(|placement| writeln!(formatter, "  {}", Placing(placement)))
    }
}

impl fmt::Display for Placing<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let author: &Authorship = self.0.as_ref();
        let provenance: &Provenance = self.0.as_ref();
        write!(formatter, "{} {}.{} {author}", self.0.relation(), self.0.prefix(), Ordinal::from(self.0))?;
        self.0.sense().iter().try_for_each(|sense| write!(formatter, " {}", Words(*sense)))?;
        write!(formatter, " entered {} {:?}", Entered::from(*provenance), self.0.body().to_string())
    }
}

impl fmt::Display for Words {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sense = self.0;
        write!(formatter, "{} {} {} {}", sense.kind(), sense.specificity(), sense.confidence(), sense.explicitness())
    }
}

impl fmt::Display for Details<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.staged(Writing(formatter)).unwrap_or(Ok(()))
    }
}

impl Staging for Writing<'_, '_> {
    type Output = fmt::Result;

    fn terminal(self, terminal: &Terminal) -> fmt::Result {
        writeln!(self.0, "  artifact {}", terminal.artifact())?;
        writeln!(self.0, "  condition {:?}", terminal.condition().to_string())
    }

    fn dropped(self, dropped: &Dropped) -> fmt::Result {
        writeln!(self.0, "  reason {:?}", dropped.reason().to_string())
    }
}

impl fmt::Display for Annotated<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.remark().visited(Writing(&mut *formatter))?;
        write!(formatter, " {} entered {}", self.0.standing(), Entered::from(self.0.provenance()))
    }
}

impl Remarking for Writing<'_, '_> {
    type Output = fmt::Result;

    fn placed(self, segment: &Segment, node: NodeId, relation: SpanRelation) -> fmt::Result {
        write!(self.0, "place {} {relation} {}", SegmentRef::from(segment), Id(node))
    }

    fn sensed(self, segment: &Segment, sense: Sense) -> fmt::Result {
        write!(self.0, "sense {} {}", SegmentRef::from(segment), Words(sense))
    }

    fn related(self, source: &Segment, target: &Segment, relation: SegmentRelation) -> fmt::Result {
        write!(self.0, "link {} {relation} {}", SegmentRef::from(source), SegmentRef::from(target))
    }

    fn conversational(self, segment: &Segment) -> fmt::Result {
        write!(self.0, "conversational {}", SegmentRef::from(segment))
    }
}

#[derive(Clone, Copy, Debug)]
struct Verdict(Option<NodeId>);

impl fmt::Display for Verdict {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(id) => write!(formatter, "proposed {} by", Id(id)),
            None => formatter.write_str("applied"),
        }
    }
}

impl fmt::Display for Applied {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.echoes()
            .iter()
            .try_for_each(|echo| writeln!(formatter, "{} {}", Verdict(echo.proposed()), echo.directive()))?;
        self.uncovered()
            .iter()
            .try_for_each(|segment| writeln!(formatter, "uncovered {}", SegmentRef::from(segment)))?;
        writeln!(formatter, "written {}", self.written().count())
    }
}

impl fmt::Display for Repeat<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let applied: AppliedId = self.0.identity();
        writeln!(formatter, "already applied as file {} at {}; nothing written", applied.into_inner(), self.0.stamp())
    }
}
