use std::collections::{BTreeMap, BTreeSet};

use lint_layout::Name;
use lint_placement::CrateName;
use transition_facts::{Endpoint, Nature, Transition};
use type_facts::Facts;

use crate::count::Count;
use crate::edge::Edge;
use crate::end::{End, Ends, Node, Place};

#[derive(Debug, Default)]
pub(crate) struct WorkspaceTypes<'found> {
    types: BTreeMap<&'found Name, BTreeSet<&'found CrateName>>,
    traits: BTreeSet<&'found Name>,
}

impl<'found> WorkspaceTypes<'found> {
    pub(crate) fn declared(facts: &'found Facts<'_>) -> WorkspaceTypes<'found> {
        WorkspaceTypes {
            types: facts.types().iter().fold(
                BTreeMap::new(),
                |mut names: BTreeMap<&Name, BTreeSet<&CrateName>>, found| {
                    names.entry(found.as_ref()).or_default().insert(found.as_ref());
                    names
                },
            ),
            traits: facts.traits().iter().map(AsRef::as_ref).collect(),
        }
    }

    pub(crate) fn declared_in(&self, krate: &CrateName) -> Count {
        Count::new(self.types.values().filter(|crates| crates.contains(krate)).count())
    }

    pub(crate) fn edge<'file>(&self, transition: Transition<'found, 'file>) -> Edge<'found, 'file> {
        let fixed_by = transition.context().trait_name().filter(|name| !self.traits.contains(*name));
        let from: &CrateName = transition.as_ref();
        let end = |endpoint: &'found Endpoint| self.end(from, endpoint, fixed_by);
        let ends = Ends::new(transition.takes().map(end).collect(), end(transition.gives()));
        Edge::new(transition, ends, fixed_by)
    }

    fn end(&self, from: &CrateName, endpoint: &'found Endpoint, fixed_by: Option<&'found Name>) -> End<'found> {
        let name: &Name = endpoint.as_ref();
        match (endpoint.nature(), fixed_by) {
            (Nature::Named, _) => self.named(from, name),
            (Nature::Interface, _) => End::placed(Place::Interface, name),
            (Nature::Primitive, Some(fixing)) => End::placed(Place::Interface, fixing),
            (Nature::Primitive, None) => End::placed(Place::Primitive, name),
            (Nature::Boxed, _) => End::placed(Place::External, name),
        }
    }

    fn named(&self, from: &CrateName, name: &'found Name) -> End<'found> {
        let declaring = self.types.get(name);
        let resolved = declaring.map(|crates| Node::resolved(crates, from, name));
        match resolved {
            None => End::placed(Place::External, name),
            Some(None) => End::placed(Place::Shared, name),
            Some(Some(node)) => End::domain(node),
        }
    }
}
