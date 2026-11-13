use std::collections::BTreeSet;

use code_model::{
    Implementation, ItemId, ItemPath, Mention, Model, PathUse, Placed, Reach, Region, Resolved, Spelling,
};
use lint_core::Site;
use lint_layout::Name;

#[derive(Debug)]
pub(super) struct Called<'model> {
    site: Site<'model>,
    krate: &'model Name,
    candidates: Vec<Vec<Name>>,
}

#[derive(Debug)]
pub(super) struct ModelCalls<'model>(Vec<Called<'model>>);

impl<'model> From<&'model Model> for ModelCalls<'model> {
    fn from(model: &'model Model) -> ModelCalls<'model> {
        let named = |id: ItemId| {
            let item = model.declared(id)?;
            Some(vec![item.krate()?.clone(), AsRef::<Name>::as_ref(item).clone()])
        };
        let invoked = model
            .uses()
            .filter(|placed| Self::MACROS.contains(&placed.entity().mention()))
            .filter(|placed| AsRef::<Region>::as_ref(placed.entity()).reach() == Reach::Shipped)
            .map(|placed| Called::invoked(placed, &named));
        let derived = model
            .implementations()
            .filter(|placed| placed.entity().written().is_none())
            .filter(|placed| AsRef::<Region>::as_ref(placed.entity()).reach() == Reach::Shipped)
            .filter_map(|placed| Called::derived(placed, &named));
        ModelCalls(invoked.chain(derived).collect())
    }
}

impl<'model> ModelCalls<'model> {
    const MACROS: [Mention; 2] = [Mention::Invoked, Mention::InMacro];

    pub(super) fn sites(&self, wanted: &[Vec<Name>], admitted: &BTreeSet<Name>) -> Vec<Site<'model>> {
        self.0
            .iter()
            .filter(|called| admitted.get(called.krate).is_none())
            .filter(|called| called.candidates.iter().any(|candidate| wanted.contains(candidate)))
            .map(|called| called.site)
            .collect()
    }
}

impl<'model> Called<'model> {
    fn invoked(
        placed: Placed<'model, &'model PathUse>,
        named: &impl Fn(ItemId) -> Option<Vec<Name>>,
    ) -> Called<'model> {
        let used = placed.entity();
        Called {
            site: placed.site(),
            krate: placed.krate(),
            candidates: Self::candidates(used.written(), used.resolved(), named),
        }
    }

    fn derived(
        placed: Placed<'model, &'model Implementation>,
        named: &impl Fn(ItemId) -> Option<Vec<Name>>,
    ) -> Option<Called<'model>> {
        let interface = placed.entity().interface()?;
        let written = AsRef::<Spelling>::as_ref(interface).written()?;
        Some(Called {
            site: placed.site(),
            krate: placed.krate(),
            candidates: Self::candidates(written, interface.as_ref(), named),
        })
    }

    fn candidates(
        written: &ItemPath,
        resolved: &Resolved,
        named: &impl Fn(ItemId) -> Option<Vec<Name>>,
    ) -> Vec<Vec<Name>> {
        let declared = resolved.crate_item().and_then(named);
        let external = resolved.external().map(|found| AsRef::<[Name]>::as_ref(found.path()).to_vec());
        [Some(AsRef::<[Name]>::as_ref(written).to_vec()), declared, external].into_iter().flatten().collect()
    }
}
