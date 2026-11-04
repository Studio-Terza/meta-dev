mod share;

use std::collections::BTreeSet;
use std::fmt;

use lint_core::Fix;
use lint_core::{Site, Violation};
use newtype::Counter;

use crate::algebra::{Algebra, Algebras, Totality};
use crate::class::Class;
use crate::count::Count;
use crate::edge::Edge;
use crate::form::Form;
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
        edges: impl Iterator<Item = &'edge Edge<'found, 'file>>,
    ) -> Coverage {
        let edges: Vec<&Edge<'found, 'file>> = edges.collect();
        let sites: BTreeSet<Site<'file>> = edges.iter().map(|edge| edge.site()).collect();
        edges
            .iter()
            .fold(Coverage::default(), |tally, edge| tally.counted(edge.form(), edge.ends().class()))
            .algebraic(system.algebras(), &sites)
    }

    fn counted(self, form: &Form<'_>, class: Class) -> Coverage {
        form.sorted(
            Coverage { written: self.written.incremented(), covered: class.covering(self.covered), ..self },
            Coverage { lifts: self.lifts.incremented(), ..self },
            Coverage { operations: self.operations.incremented(), ..self },
        )
    }

    fn algebraic<'file>(self, algebras: &Algebras<'_, 'file>, sites: &BTreeSet<Site<'file>>) -> Coverage {
        let operations: Vec<_> = algebras
            .types()
            .filter_map(|node| algebras.of_type(node))
            .flat_map(Algebra::operations)
            .filter(|operation| sites.contains(AsRef::<Site<'file>>::as_ref(operation.transition())))
            .collect();
        Coverage {
            algebraic: Count::new(operations.len()),
            partial: Count::new(operations.iter().filter(|operation| operation.partial() == Totality::Partial).count()),
            ..self
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
