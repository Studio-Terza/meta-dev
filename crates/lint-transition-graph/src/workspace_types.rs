use std::collections::{BTreeMap, BTreeSet};

use lint_layout::Name;
use lint_placement::CrateName;
use transition_facts::{Endpoint, Nature, Transition};
use type_facts::Facts;

use crate::count::Count;
use crate::edge::Edge;
use crate::end::{End, Ends, Node, Place};

#[derive(Debug, Default)]
pub(crate) struct WorkspaceTypes {
    types: BTreeMap<Name, BTreeSet<CrateName>>,
    traits: BTreeSet<Name>,
}

impl WorkspaceTypes {
    pub(crate) fn declared(facts: &Facts<'_>) -> WorkspaceTypes {
        WorkspaceTypes {
            types: facts.types().iter().fold(
                BTreeMap::new(),
                |mut names: BTreeMap<Name, BTreeSet<CrateName>>, found| {
                    names
                        .entry(AsRef::<Name>::as_ref(found).clone())
                        .or_default()
                        .insert(AsRef::<CrateName>::as_ref(found).clone());
                    names
                },
            ),
            traits: facts.traits().iter().map(|found| AsRef::<Name>::as_ref(found).clone()).collect(),
        }
    }

    pub(crate) fn declared_in(&self, krate: &CrateName) -> Count {
        Count::new(self.types.values().filter(|crates| crates.contains(krate)).count())
    }

    pub(crate) fn edge<'found, 'file>(&self, transition: &'found Transition<'file>) -> Edge<'found, 'file> {
        let fixed_by = transition.filler().trait_name().filter(|name| !self.traits.contains(*name));
        let from: &CrateName = transition.as_ref();
        let end = |endpoint: &Endpoint| self.end(from, endpoint, fixed_by);
        let takes = transition.takes().iter().map(end).collect();
        Edge::new(transition, Ends::new(takes, end(transition.gives())), fixed_by)
    }

    fn end(&self, from: &CrateName, endpoint: &Endpoint, fixed_by: Option<&Name>) -> End {
        let name: &Name = endpoint.as_ref();
        match (endpoint.nature(), fixed_by) {
            (Nature::Named, _) => self.named(from, name),
            (Nature::Interface, _) => End::placed(Place::Interface, name),
            (Nature::Primitive, Some(fixing)) => End::placed(Place::Interface, fixing),
            (Nature::Primitive, None) => End::placed(Place::Primitive, name),
            (Nature::Boxed, _) => End::placed(Place::External, name),
        }
    }

    fn named(&self, from: &CrateName, name: &Name) -> End {
        let declaring = self.types.get(name);
        let resolved = declaring.map(|crates| Node::resolved(crates, from, name));
        match resolved {
            None => End::placed(Place::External, name),
            Some(None) => End::placed(Place::Shared, name),
            Some(Some(node)) => End::domain(node),
        }
    }
}
