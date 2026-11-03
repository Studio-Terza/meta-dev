use std::collections::BTreeSet;
use std::fmt;

use lint_core::{Site, Violation};
use lint_layout::Fix;
use lint_placement::CrateName;

use crate::count::Count;
use crate::coverage::Coverage;
use crate::edge::Edge;
use crate::end::{End, Key, Place};
use crate::workspace_types::WorkspaceTypes;

#[derive(Debug)]
pub(crate) struct CrateTransitions<'found> {
    name: &'found CrateName,
    first: Option<&'found Site>,
    domain: Count,
    interfaces: BTreeSet<&'found End>,
    boxes: BTreeSet<&'found End>,
    pairs: BTreeSet<Key>,
    coverage: Coverage,
}

impl<'found> CrateTransitions<'found> {
    pub(crate) fn measured(
        types: &WorkspaceTypes,
        name: &'found CrateName,
        edges: &[&'found Edge<'found>],
    ) -> CrateTransitions<'found> {
        let ends = || edges.iter().flat_map(|edge| edge.ends().all());
        CrateTransitions {
            name,
            first: edges.iter().map(|edge| AsRef::<Site>::as_ref(edge.transition())).min(),
            domain: types.declared_in(name),
            interfaces: ends().filter(|end| end.place() == Place::Interface).collect(),
            boxes: ends().filter(|end| end.place() == Place::External).collect(),
            pairs: edges.iter().flat_map(|edge| edge.ends().edges()).collect(),
            coverage: Coverage::tallied(edges.iter().copied()),
        }
    }

    pub(crate) fn violation(&self) -> Option<Violation> {
        self.first.map(|site| Violation::new(site.clone(), self))
    }
}

impl fmt::Display for CrateTransitions<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "crate {}: {} domain types, {} interface nodes, {} black boxes, {} domain pairs, {}",
            self.name,
            self.domain,
            self.interfaces.len(),
            self.boxes.len(),
            self.pairs.len(),
            self.coverage
        )
    }
}

impl lint_layout::Finding for CrateTransitions<'_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "give each bare primitive a newtype, an enum or a standard trait, and keep one transition per input-output pair",
        )
    }
}
