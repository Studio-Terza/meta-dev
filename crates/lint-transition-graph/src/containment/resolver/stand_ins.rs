use std::collections::BTreeMap;

use lint_item_graph::ItemGraph;
use lint_layout::Name;

use super::super::unit::Unit;

#[derive(Debug, Default)]
pub(crate) struct StandIns<'found>(BTreeMap<Unit<'found>, BTreeMap<Name, Vec<Name>>>);

impl<'found> StandIns<'found> {
    pub(crate) fn with_graph(self, unit: Unit<'found>, graph: &ItemGraph) -> StandIns<'found> {
        graph.stand_ins().into_iter().fold(self, |stand_ins, (name, used)| stand_ins.standing(unit, name, &used))
    }

    fn standing(mut self, unit: Unit<'found>, name: &Name, used: &[&Name]) -> StandIns<'found> {
        self.0.entry(unit).or_default().insert(name.clone(), used.iter().copied().cloned().collect());
        self
    }

    pub(crate) fn standing_for(&self, unit: Unit<'found>, name: &Name) -> &[Name] {
        self.0.get(&unit).and_then(|standing| standing.get(name)).map_or(&[], Vec::as_slice)
    }
}
