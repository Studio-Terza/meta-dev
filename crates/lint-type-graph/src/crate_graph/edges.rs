use std::collections::{BTreeMap, BTreeSet};

use lint_item_graph::ItemGraph;
use lint_layout::{Name, Whole, Word, words};
use type_facts::{Implementation, TypeFact};

use super::component::Component;
use super::count::Count;

#[derive(Clone, Debug, Default)]
pub(crate) struct Edges(BTreeMap<Name, BTreeSet<Name>>);

impl Edges {
    const INTO_SELF: [Word<'static>; 2] = words!["From", "TryFrom"];
    const FROM_SELF: [Word<'static>; 5] = words!["AsRef", "AsMut", "Into", "TryInto", "Borrow"];

    pub(crate) fn with_nodes(nodes: &BTreeSet<Name>) -> Edges {
        Edges(nodes.iter().map(|node| (node.clone(), BTreeSet::new())).collect())
    }

    pub(crate) fn with_type(self, found: &TypeFact, traits: &BTreeSet<Name>) -> Edges {
        found
            .implementations()
            .iter()
            .fold(self, |edges, implementation| edges.with_implementation(found, implementation, traits))
    }

    fn with_implementation(self, found: &TypeFact, implementation: &Implementation, traits: &BTreeSet<Name>) -> Edges {
        let this: &Name = found.as_ref();
        let kind: &Name = implementation.as_ref();
        match implementation.argument() {
            Some(other) if kind.matches::<Whole>(&Self::INTO_SELF) => self.linked(other, this),
            Some(other) if kind.matches::<Whole>(&Self::FROM_SELF) => self.linked(this, other),
            _ if traits.contains(kind) => self.linked(this, kind),
            _ => self,
        }
    }

    pub(crate) fn count(&self) -> Count {
        Count::new(self.0.values().map(BTreeSet::len).sum())
    }

    pub(crate) fn largest_tangle(&self) -> Component {
        let references: BTreeMap<Name, Vec<Name>> =
            self.0.iter().map(|(from, to)| (from.clone(), to.iter().cloned().collect())).collect();
        Component::new(ItemGraph::of_references(&references).tangles().first().map_or(0, Vec::len))
    }

    fn linked(mut self, from: &Name, to: &Name) -> Edges {
        let target = self.0.contains_key(to).then(|| to.clone()).filter(|target| target != from);
        self.0.get_mut(from).zip(target).map(|(links, target)| links.insert(target));
        self
    }
}
