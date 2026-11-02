mod component;
mod count;
mod edges;
mod finding;
mod measures;

use std::collections::BTreeSet;

use lint_core::{Site, Violation};
use lint_layout::Name;
use lint_placement::CrateName;
use type_facts::{TraitFact, TypeFact};

use count::Count;
use edges::Edges;
use finding::Finding;
use measures::Measures;

#[derive(Debug, Default)]
pub(crate) struct CrateGraph<'facts> {
    types: Vec<&'facts TypeFact>,
    traits: Vec<&'facts TraitFact>,
}

impl<'facts> CrateGraph<'facts> {
    pub(crate) fn add_type(&mut self, found: &'facts TypeFact) {
        self.types.push(found);
    }

    pub(crate) fn add_trait(&mut self, found: &'facts TraitFact) {
        self.traits.push(found);
    }

    pub(crate) fn violation(&self, name: &CrateName) -> Option<Violation> {
        let trait_names: BTreeSet<Name> =
            self.traits.iter().map(|found| AsRef::<Name>::as_ref(found).clone()).collect();
        let nodes: BTreeSet<Name> = self
            .types
            .iter()
            .map(|found| AsRef::<Name>::as_ref(found).clone())
            .chain(trait_names.iter().cloned())
            .collect();
        let edges =
            self.types.iter().fold(Edges::with_nodes(&nodes), |edges, found| edges.with_type(found, &trait_names));
        let sites = self.types.iter().map(AsRef::<Site>::as_ref).chain(self.traits.iter().map(AsRef::<Site>::as_ref));
        let first = sites.min()?;
        let node_count = Count::new(nodes.len());
        let edge_count = edges.count();
        Some(Violation::new(
            first.clone(),
            Finding::new(name, Measures::counted(node_count, edge_count, edges.largest_tangle())),
        ))
    }
}
