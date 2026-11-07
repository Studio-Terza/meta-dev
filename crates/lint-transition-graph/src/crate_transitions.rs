use std::collections::BTreeSet;
use std::fmt;

use lint_core::Fix;
use lint_core::{Site, Violation};
use lint_placement::CrateName;

use crate::count::Count;
use crate::coverage::Coverage;
use crate::edge::Edge;
use crate::end::{End, Hop, Place};
use crate::workspace_types::WorkspaceTypes;

#[derive(Debug)]
pub(crate) struct CrateTransitions<'found, 'file> {
    name: &'found CrateName,
    first: Option<Site<'file>>,
    domain: Count,
    interfaces: BTreeSet<&'found End<'found>>,
    boxes: BTreeSet<&'found End<'found>>,
    pairs: Count,
    coverage: Coverage,
}

impl<'found, 'file> CrateTransitions<'found, 'file> {
    pub(crate) fn measured(
        types: &WorkspaceTypes,
        name: &'found CrateName,
        edges: &[&'found Edge<'found, 'file>],
        coverage: Coverage,
    ) -> Self {
        let ends = |place| edges.iter().flat_map(move |edge| edge.ends().placed(place));
        CrateTransitions {
            name,
            first: edges.iter().map(|edge| edge.site()).min(),
            domain: types.declared_in(name),
            interfaces: ends(Place::Interface).collect(),
            boxes: ends(Place::External).collect(),
            pairs: Hop::distinct(edges.iter().map(|edge| edge.ends())),
            coverage,
        }
    }

    pub(crate) fn violation(&self) -> Option<Violation<'file>> {
        self.first.map(|site| Violation::new(site, self))
    }
}

impl fmt::Display for CrateTransitions<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "crate {}: {} domain types, {} interface nodes, {} black boxes, {} domain pairs, {}",
            self.name,
            self.domain,
            self.interfaces.len(),
            self.boxes.len(),
            self.pairs,
            self.coverage
        )
    }
}

impl lint_core::Finding for CrateTransitions<'_, '_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "give each bare primitive a newtype, an enum or a standard trait, and keep one transition per input-output pair",
        )
    }
}
