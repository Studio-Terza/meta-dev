mod called;
mod effect;
mod effect_site;
mod summary;
mod vocabulary;

use std::collections::BTreeMap;

use code_model::{Ground, Model, PathUse, Placed, Reach, Region};
use lint_core::Violation;
use lint_placement::CrateName;

use summary::Summary;
use vocabulary::Vocabulary;

use effect_site::{Confinement, EffectSite};

#[derive(Debug)]
pub struct CrateEffects<'file>(Vec<EffectSite<'file>>);

impl<'file> CrateEffects<'file> {
    #[must_use]
    pub fn of_model(model: &'file Model) -> BTreeMap<CrateName, CrateEffects<'file>> {
        let shipped = model.uses().filter(|placed| AsRef::<Region>::as_ref(placed.entity()).reach() == Reach::Shipped);
        let grouped =
            shipped.fold(BTreeMap::new(), |mut crates: BTreeMap<CrateName, Vec<EffectSite<'file>>>, placed| {
                let found = Vocabulary::placed_site(placed);
                crates.entry(CrateName::from(placed.krate())).or_default().extend(found);
                crates
            });
        grouped
            .into_iter()
            .filter_map(|(krate, sites)| sites.first().map(|_| ()).map(|()| (krate, CrateEffects::sorted(sites))))
            .collect()
    }

    fn sorted(mut sites: Vec<EffectSite<'file>>) -> CrateEffects<'file> {
        sites.sort_by_key(EffectSite::place);
        CrateEffects(sites)
    }

    pub(crate) fn violations(&self, name: &CrateName) -> Vec<Violation<'file>> {
        let summary = self.0.first().map(|first| Violation::new(first.place(), Summary::counted(name, &self.0)));
        summary.into_iter().chain(self.outside_ports()).collect()
    }

    pub(crate) fn outside_ports(&self) -> Vec<Violation<'file>> {
        self.0
            .iter()
            .filter(|site| site.ground() == Ground::Open)
            .map(|site| site.violation(Confinement::Ports))
            .collect()
    }
}

#[derive(Debug)]
pub struct OutsideFillers<'file>(CrateEffects<'file>);

impl<'file> FromIterator<Placed<'file, &'file PathUse>> for OutsideFillers<'file> {
    fn from_iter<Uses: IntoIterator<Item = Placed<'file, &'file PathUse>>>(uses: Uses) -> OutsideFillers<'file> {
        OutsideFillers(CrateEffects::sorted(uses.into_iter().filter_map(Vocabulary::placed_site).collect()))
    }
}

impl<'file> From<OutsideFillers<'file>> for Vec<Violation<'file>> {
    fn from(outside: OutsideFillers<'file>) -> Vec<Violation<'file>> {
        outside.0.0.iter().map(|site| site.violation(Confinement::Fillers)).collect()
    }
}
