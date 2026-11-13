mod called;
mod effect;
mod effect_site;
mod file_scan;
mod summary;
mod vocabulary;

use std::collections::{BTreeMap, BTreeSet};

use code_model::{Ground, Model, Reach, Region};
use lint_core::{Site, SourceFile, Violation};
use lint_paths::Imports;
use lint_placement::CrateName;

use file_scan::FileScan;
use summary::Summary;
use vocabulary::Vocabulary;

use effect_site::{Confinement, EffectSite};

#[derive(Debug)]
pub struct CrateEffects<'file>(Vec<EffectSite<'file>>);

impl<'file> CrateEffects<'file> {
    #[must_use]
    pub fn found_in(file: &'file SourceFile, imports: &Imports<'_>) -> CrateEffects<'file> {
        let called = imports.read(file, &Vocabulary::FINALS);
        let scans: Vec<FileScan<'file>> = vec![FileScan::scanned(file)];
        let tested: BTreeSet<Site<'file>> = scans.iter().flat_map(FileScan::tested).copied().collect();
        let reached: Vec<EffectSite<'file>> = called.iter().filter_map(Vocabulary::site_of).collect();
        let placed: BTreeSet<Site<'file>> = reached.iter().map(EffectSite::place).collect();
        let untested: BTreeSet<&Site<'file>> = placed.difference(&tested).collect();
        let mut sites: Vec<EffectSite<'file>> = reached
            .into_iter()
            .filter(|site| untested.contains(&site.place()))
            .chain(scans.into_iter().flat_map(FileScan::into_printed))
            .collect();
        sites.sort_by_key(EffectSite::place);
        CrateEffects(sites)
    }

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

    #[must_use]
    pub fn outside_fillers(&self, fillers: &BTreeSet<Site<'file>>) -> Vec<Violation<'file>> {
        self.0
            .iter()
            .filter(|site| fillers.get(&site.place()).is_none())
            .map(|site| site.violation(Confinement::Fillers))
            .collect()
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
