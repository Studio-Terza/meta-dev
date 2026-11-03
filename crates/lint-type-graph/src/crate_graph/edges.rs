use std::collections::{BTreeMap, BTreeSet};

use lint_item_graph::ItemGraph;
use lint_layout::{Name, Whole, Word, words};
use type_facts::{Facts, Implementation, TypeFact};

use super::component::Component;
use super::count::Count;

#[derive(Clone, Debug, Default)]
pub(crate) struct Edges<'facts>(BTreeMap<&'facts Name, BTreeSet<&'facts Name>>);

impl<'facts> Edges<'facts> {
    const INTO_SELF: [Word<'static>; 2] = words!["From", "TryFrom"];
    const FROM_SELF: [Word<'static>; 5] = words!["AsRef", "AsMut", "Into", "TryInto", "Borrow"];

    pub(crate) fn with_nodes(nodes: BTreeSet<&'facts Name>) -> Edges<'facts> {
        Edges(nodes.into_iter().map(|node| (node, BTreeSet::new())).collect())
    }

    pub(crate) fn with_type<'file>(
        self,
        facts: &'facts Facts<'file>,
        found: &'facts TypeFact<'file>,
        traits: &BTreeSet<&Name>,
    ) -> Edges<'facts> {
        facts
            .implementations(found)
            .iter()
            .fold(self, |edges, implementation| edges.with_implementation(found, implementation, traits))
    }

    fn with_implementation(
        self,
        found: &'facts TypeFact<'_>,
        implementation: &'facts Implementation,
        traits: &BTreeSet<&Name>,
    ) -> Edges<'facts> {
        let this: &Name = found.as_ref();
        let kind: &Name = implementation.as_ref();
        let into = implementation.argument().filter(|_| kind.matches::<Whole>(&Self::INTO_SELF).is_some());
        let out_of = implementation.argument().filter(|_| kind.matches::<Whole>(&Self::FROM_SELF).is_some());
        let traited = traits.get(kind).map(|_| kind);
        let source = into.or_else(|| out_of.map(|_| this)).or_else(|| traited.map(|_| this));
        let target = into.map(|_| this).or(out_of).or(traited);
        self.linked(source, target)
    }

    pub(crate) fn count(&self) -> Count {
        Count::new(self.0.values().map(BTreeSet::len).sum())
    }

    pub(crate) fn largest_tangle(self) -> Component {
        Component::new(ItemGraph::of_references(self.0).tangles().first().map_or(0, Vec::len))
    }

    fn linked(mut self, from: Option<&Name>, to: Option<&'facts Name>) -> Edges<'facts> {
        let target = to.filter(|to| self.0.contains_key(*to)).filter(|target| Some(*target) != from);
        from.and_then(|from| self.0.get_mut(from)).zip(target).map(|(links, target)| links.insert(target));
        self
    }
}
