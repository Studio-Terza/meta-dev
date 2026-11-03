use std::collections::{BTreeMap, BTreeSet};

use lint_layout::Name;
use lint_placement::CrateName;
use transition_facts::{Endpoint, Implementer, Nature, Transition, Transitions};
use type_facts::Facts;

use crate::count::Count;
use crate::edge::Edge;
use crate::end::{End, Ends, Node, Place};

#[derive(Debug, Default)]
pub(crate) struct WorkspaceTypes<'found> {
    types: BTreeMap<&'found Name, BTreeSet<&'found CrateName>>,
    traits: BTreeSet<&'found Name>,
    crates: BTreeSet<&'found CrateName>,
    ambiguous: BTreeMap<&'found CrateName, BTreeSet<&'found Name>>,
}

impl<'found> WorkspaceTypes<'found> {
    pub(crate) fn declared(facts: &'found Facts<'_>, transitions: &'found Transitions<'_>) -> WorkspaceTypes<'found> {
        WorkspaceTypes {
            crates: transitions
                .all()
                .map(|found| AsRef::<CrateName>::as_ref(found.context()))
                .chain(facts.types().iter().map(AsRef::<CrateName>::as_ref))
                .collect(),
            ambiguous: Self::ambiguous(facts),
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

    fn ambiguous(facts: &'found Facts<'_>) -> BTreeMap<&'found CrateName, BTreeSet<&'found Name>> {
        let mut seen: BTreeMap<&CrateName, BTreeSet<&Name>> = BTreeMap::new();
        facts.types().iter().fold(
            BTreeMap::new(),
            |mut twice: BTreeMap<&'found CrateName, BTreeSet<&'found Name>>, found| {
                let krate: &'found CrateName = found.as_ref();
                let name: &'found Name = found.as_ref();
                let fresh = seen.entry(krate).or_default().insert(name);
                twice.entry(krate).or_default().extend((!fresh).then_some(name));
                twice
            },
        )
    }

    pub(crate) fn declared_in(&self, krate: &CrateName) -> Count {
        Count::new(self.types.values().filter(|crates| crates.contains(krate)).count())
    }

    pub(crate) fn edge<'file>(&self, transition: Transition<'found, 'file>) -> Edge<'found, 'file> {
        let fixed_by = transition.context().trait_name().filter(|name| !self.traits.contains(*name));
        let from: &CrateName = transition.as_ref();
        let end = |endpoint: &'found Endpoint| self.end(from, endpoint, fixed_by);
        let implementer = transition.implementer().map(Implementer::endpoint).map(end);
        let ends =
            Ends::new(transition.takes().map(end).collect(), end(transition.gives())).implemented_by(implementer);
        Edge::new(transition, ends, fixed_by)
    }

    fn end(&self, from: &CrateName, endpoint: &'found Endpoint, fixed_by: Option<&'found Name>) -> End<'found> {
        let name: &Name = endpoint.as_ref();
        match (endpoint.nature(), fixed_by) {
            (Nature::Named, _) => self.named(endpoint.origin().unwrap_or(from), name),
            (Nature::Interface, _) => End::placed(Place::Interface, name),
            (Nature::Primitive, Some(fixing)) => End::placed(Place::Interface, fixing),
            (Nature::Primitive, None) => End::placed(Place::Primitive, name),
            (Nature::Boxed, _) => End::placed(Place::External, name),
        }
    }

    fn named(&self, from: &CrateName, name: &'found Name) -> End<'found> {
        let declaring = self.types.get(name).filter(|_| self.crates.contains(from));
        let resolved = declaring.map(|crates| Node::resolved(crates, from, name));
        match resolved.map(|found| found.and_then(|node| self.unambiguous(node))) {
            None => End::placed(Place::External, name),
            Some(None) => End::placed(Place::Shared, name),
            Some(Some(node)) => End::domain(node),
        }
    }

    fn unambiguous(&self, node: Node<'found>) -> Option<Node<'found>> {
        let twice = self.ambiguous.get(AsRef::<CrateName>::as_ref(&node));
        match twice.and_then(|names| names.get(AsRef::<Name>::as_ref(&node))) {
            Some(_) => None,
            None => Some(node),
        }
    }
}
