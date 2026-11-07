use std::collections::BTreeMap;

use lint_item_graph::ItemGraph;
use lint_layout::Name;
use lint_placement::CrateName;

#[derive(Debug, Default)]
pub(crate) struct StandIns<'found>(BTreeMap<&'found CrateName, BTreeMap<Name, Vec<Name>>>);

impl<'found> StandIns<'found> {
    pub(crate) fn with_graph(self, krate: &'found CrateName, graph: &ItemGraph) -> Self {
        graph.stand_ins().into_iter().fold(self, |stand_ins, (name, used)| stand_ins.standing(krate, name, &used))
    }

    fn standing(mut self, krate: &'found CrateName, name: &Name, used: &[&Name]) -> Self {
        self.0.entry(krate).or_default().insert(name.clone(), used.iter().copied().cloned().collect());
        self
    }

    pub(crate) fn standing_for(&self, krate: &CrateName, name: &Name) -> &[Name] {
        self.0.get(krate).and_then(|standing| standing.get(name)).map_or(&[], Vec::as_slice)
    }
}
