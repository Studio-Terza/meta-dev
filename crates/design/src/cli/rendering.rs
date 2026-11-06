use std::fmt;

use design_model::{Entered, Node, NodeId};
use design_store::Placed;
use newtype::{Counter, Newtype};
use structurer::SegmentRef;

use super::request::Applied;

#[derive(Clone, Copy, Debug)]
pub struct Id(NodeId);

#[derive(Clone, Copy, Debug)]
pub struct Line<'node>(&'node Node);

#[derive(Clone, Copy, Debug)]
pub struct Neighbourhood<'placed>(&'placed Placed);

impl From<NodeId> for Id {
    fn from(id: NodeId) -> Id {
        Id(id)
    }
}

impl<'node> From<&'node Node> for Line<'node> {
    fn from(node: &'node Node) -> Line<'node> {
        Line(node)
    }
}

impl<'placed> From<&'placed Placed> for Neighbourhood<'placed> {
    fn from(placed: &'placed Placed) -> Neighbourhood<'placed> {
        Neighbourhood(placed)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "n{}", self.0.into_inner())
    }
}

impl fmt::Display for Line<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entry = self.0.entry();
        write!(
            formatter,
            "{} {} {} {} entered {}",
            Id(self.0.identity()),
            entry.project(),
            entry.facet(),
            entry.stage(),
            Entered::from(entry.provenance())
        )?;
        self.0.title().iter().try_for_each(|title| write!(formatter, " {:?}", title.to_string()))
    }
}

impl fmt::Display for Neighbourhood<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{}", Line(self.0.node()))?;
        self.0
            .relatives()
            .iter()
            .try_for_each(|relative| writeln!(formatter, "  {} {}", relative.kin(), Line(relative.node())))?;
        self.0.links().iter().try_for_each(|link| writeln!(formatter, "  {} {}", link.relation(), Id(link.target())))
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
