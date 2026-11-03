use std::collections::BTreeMap;

use lint_item_graph::{ItemGraph, NodeIndex};
use lint_layout::Name;
use lint_placement::CrateName;

#[derive(Debug, Default)]
pub(crate) struct StandIns<'found>(BTreeMap<&'found CrateName, BTreeMap<Name, Vec<Name>>>);

impl<'found> StandIns<'found> {
    pub(crate) fn with_graph(self, krate: &'found CrateName, graph: &ItemGraph) -> StandIns<'found> {
        graph.stand_ins().iter().fold(self, |stand_ins, index| stand_ins.standing(krate, graph, *index))
    }

    fn standing(mut self, krate: &'found CrateName, graph: &ItemGraph, index: NodeIndex) -> StandIns<'found> {
        let used = graph.links().get(index.into_inner()).into_iter().flatten();
        let named: Vec<Name> = used.filter_map(|used| graph.name_at(*used).cloned()).collect();
        let standing = graph.name_at(index).cloned().map(|name| BTreeMap::from([(name, named)]));
        self.0.entry(krate).or_default().extend(standing.unwrap_or_default());
        self
    }

    pub(crate) fn standing_for(&self, krate: &CrateName, name: &Name) -> &[Name] {
        self.0.get(krate).and_then(|standing| standing.get(name)).map_or(&[], Vec::as_slice)
    }
}
