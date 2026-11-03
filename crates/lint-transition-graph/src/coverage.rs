mod share;

use std::fmt;

use lint_core::{Site, Violation};
use lint_layout::Fix;
use newtype::Counter;

use crate::count::Count;
use crate::edge::{Edge, Role};
use crate::end::Class;
use share::Share;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Coverage {
    written: Count,
    covered: Count,
    lifts: Count,
    operations: Count,
}

impl Coverage {
    pub(crate) fn tallied<'edge, 'found: 'edge, 'file: 'found>(
        edges: impl Iterator<Item = &'edge Edge<'found, 'file>>,
    ) -> Coverage {
        edges.fold(Coverage::default(), |tally, edge| tally.counted(edge.role(), edge.ends().class()))
    }

    fn counted(self, role: Role, class: Class) -> Coverage {
        match role {
            Role::Written => {
                Coverage { written: self.written.incremented(), covered: class.covering(self.covered), ..self }
            }
            Role::Lift => Coverage { lifts: self.lifts.incremented(), ..self },
            Role::Operation => Coverage { operations: self.operations.incremented(), ..self },
        }
    }

    pub(crate) fn violation(self, first: Option<Site<'_>>) -> Option<Violation<'_>> {
        first.map(|site| Violation::new(site, Workspace(self)))
    }
}

impl fmt::Display for Coverage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} written edges, coverage {} of {}, share {}, {} touch a bare primitive; not written: {} derived lifts, {} store operations",
            self.written,
            self.covered,
            self.written,
            Share::between(self.covered, self.written),
            Count::new(self.written.count().saturating_sub(self.covered.count())),
            self.lifts,
            self.operations
        )
    }
}

#[derive(Debug)]
struct Workspace(Coverage);

impl fmt::Display for Workspace {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "workspace: {}", self.0)
    }
}

impl lint_layout::Finding for Workspace {
    fn fix(&self) -> Fix {
        Fix::new(
            "give each bare primitive a newtype, an enum or a standard trait, so every written edge joins nodes the graph can see",
        )
    }
}
