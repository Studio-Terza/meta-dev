mod called;
mod effect;
mod effect_site;
mod file_scan;
mod summary;
mod vocabulary;

use std::collections::BTreeSet;

use lint_core::{Site, SourceFile, Violation};
use lint_paths::Paths;
use lint_placement::CrateName;

use effect_site::EffectSite;
use file_scan::FileScan;
use summary::Summary;
use vocabulary::Vocabulary;

#[derive(Debug)]
pub(crate) struct CrateEffects(Vec<EffectSite>);

impl CrateEffects {
    pub(crate) fn found(files: &[&SourceFile]) -> CrateEffects {
        let called = Paths::read(files.iter().copied(), &Vocabulary::FINALS);
        let scans: Vec<FileScan<'_>> = files.iter().map(|file| FileScan::scanned(file)).collect();
        let tested: BTreeSet<&Site> = scans.iter().flat_map(FileScan::tested).collect();
        let mut sites: Vec<EffectSite> = called
            .iter()
            .filter_map(Vocabulary::site_of)
            .filter(|site| !tested.contains(site.place()))
            .chain(scans.iter().flat_map(FileScan::printed).cloned())
            .collect();
        sites.sort_by(|left, right| left.place().cmp(right.place()));
        CrateEffects(sites)
    }

    pub(crate) fn violations(&self, name: &CrateName) -> Vec<Violation> {
        let summary =
            self.0.first().map(|first| Violation::new(first.place().clone(), Summary::counted(name, &self.0)));
        summary.into_iter().chain(self.0.iter().filter_map(EffectSite::outside_ports)).collect()
    }
}
