use std::borrow::Borrow;
use std::collections::BTreeMap;

use code_model::{ItemKind, Model, References};
use lint_layout::Name;
use lint_placement::CrateName;

#[derive(Debug)]
struct Standing<'found> {
    krate: &'found Name,
    name: &'found Name,
    used: Vec<&'found Name>,
}

#[derive(Debug, Default)]
pub(crate) struct StandIns<'found>(BTreeMap<&'found Name, BTreeMap<&'found Name, Vec<&'found Name>>>);

impl<'found> FromIterator<Standing<'found>> for StandIns<'found> {
    fn from_iter<Found: IntoIterator<Item = Standing<'found>>>(found: Found) -> StandIns<'found> {
        found.into_iter().fold(StandIns::default(), |mut stand_ins, standing| {
            stand_ins.0.entry(standing.krate).or_default().insert(standing.name, standing.used);
            stand_ins
        })
    }
}

impl<'found> StandIns<'found> {
    pub(crate) const STANDING: [ItemKind; 3] = [ItemKind::Alias, ItemKind::Const, ItemKind::Macro];

    pub(crate) fn of_model(model: &'found Model, references: &References<'found>) -> StandIns<'found> {
        model
            .items()
            .filter_map(|placed| {
                let item = *placed.entity();
                let declared = model.declared(item).filter(|found| Self::STANDING.contains(&found.kind()))?;
                let targets = references.targets().get(&item).into_iter().flatten();
                let used = targets.filter_map(|target| model.declared(*target)).map(AsRef::<Name>::as_ref).collect();
                Some(Standing { krate: placed.krate(), name: declared.as_ref(), used })
            })
            .collect()
    }

    pub(crate) fn standing_for(&self, krate: &CrateName, name: &Name) -> &[&'found Name] {
        let crate_name: &Name = krate.borrow();
        self.0.get(crate_name).and_then(|standing| standing.get(name)).map_or(&[], Vec::as_slice)
    }
}
