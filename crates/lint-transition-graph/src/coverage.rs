mod share;

use std::collections::BTreeSet;
use std::fmt;

use algebra::{Monoid, Semigroup};
use lint_core::Fix;
use lint_core::{Site, Violation};
use newtype::Counter;

use crate::algebra::{Algebra, Algebras, Totality};
use crate::count::Count;
use crate::edge::Edge;
use crate::forms::{Lifted, Stored, Written};
use crate::system::TypeSystem;
pub(crate) use share::Share;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Coverage {
    written: Count,
    covered: Count,
    lifts: Count,
    operations: Count,
    algebraic: Count,
    partial: Count,
}

impl Coverage {
    pub(crate) fn tallied<'edge, 'found: 'edge, 'file: 'found>(
        system: &TypeSystem<'found, 'file>,
        listed: impl Iterator<Item = &'edge Edge<'found, 'file>>,
    ) -> Coverage {
        let edges: Vec<&Edge<'found, 'file>> = listed.collect();
        let sites: BTreeSet<Site<'file>> = edges.iter().map(|edge| edge.site()).collect();
        let written: Vec<&Edge<'found, 'file>> =
            edges.iter().copied().collect::<Written<'edge, 'found, 'file>>().into_iter().collect();
        let counted = Coverage {
            written: Count::new(written.len()),
            covered: Count::total(written.iter().map(|edge| edge.ends().class().covered())),
            lifts: Count::new(edges.iter().copied().collect::<Lifted<'edge, 'found, 'file>>().into_iter().count()),
            operations: Count::new(edges.iter().copied().collect::<Stored<'edge, 'found, 'file>>().into_iter().count()),
            ..Coverage::default()
        };
        counted.combine(Coverage::algebraic(system.algebras(), &sites))
    }

    fn algebraic<'file>(algebras: &Algebras<'_, 'file>, sites: &BTreeSet<Site<'file>>) -> Coverage {
        let operations: Vec<_> = algebras
            .types()
            .filter_map(|node| algebras.of_type(node))
            .flat_map(Algebra::operations)
            .filter(|operation| sites.contains(AsRef::<Site<'file>>::as_ref(operation.transition())))
            .collect();
        Coverage {
            algebraic: Count::new(operations.len()),
            partial: Count::new(operations.iter().filter(|operation| operation.partial() == Totality::Partial).count()),
            ..Coverage::default()
        }
    }

    pub(crate) fn violation(self, first: Option<Site<'_>>) -> Option<Violation<'_>> {
        first.map(|site| Violation::new(site, Workspace(self)))
    }
}

impl Semigroup for Coverage {
    fn combine(self, other: Coverage) -> Coverage {
        Coverage {
            written: self.written.plus(other.written),
            covered: self.covered.plus(other.covered),
            lifts: self.lifts.plus(other.lifts),
            operations: self.operations.plus(other.operations),
            algebraic: self.algebraic.plus(other.algebraic),
            partial: self.partial.plus(other.partial),
        }
    }
}

impl Monoid for Coverage {}

impl fmt::Display for Coverage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} written edges, coverage {} of {}, share {}, {} touch a bare primitive; of which {} algebra operations, {} partial; not written: {} derived lifts, {} store operations",
            self.written,
            self.covered,
            self.written,
            Share::between(self.covered, self.written),
            Count::new(self.written.count().saturating_sub(self.covered.count())),
            self.algebraic,
            self.partial,
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

impl lint_core::Finding for Workspace {
    fn fix(&self) -> Fix {
        Fix::new(
            "give each bare primitive a newtype, an enum or a standard trait, so every written edge joins nodes the graph can see",
        )
    }
}
