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
pub(crate) struct CrateEffects<'file>(Vec<EffectSite<'file>>);

impl<'file> CrateEffects<'file> {
    pub(crate) fn found(files: &[&'file SourceFile]) -> CrateEffects<'file> {
        let called = Paths::read(files.iter().copied(), &Vocabulary::FINALS);
        let scans: Vec<FileScan<'file>> = files.iter().map(|file| FileScan::scanned(file)).collect();
        let tested: BTreeSet<Site<'file>> = scans.iter().flat_map(FileScan::tested).copied().collect();
        let mut sites: Vec<EffectSite<'file>> = called
            .iter()
            .filter_map(Vocabulary::site_of)
            .filter(|site| !tested.contains(&site.place()))
            .chain(scans.into_iter().flat_map(FileScan::into_printed))
            .collect();
        sites.sort_by_key(EffectSite::place);
        CrateEffects(sites)
    }

    pub(crate) fn violations(&self, name: &CrateName) -> Vec<Violation<'file>> {
        let summary = self.0.first().map(|first| Violation::new(first.place(), Summary::counted(name, &self.0)));
        summary.into_iter().chain(self.outside_ports()).collect()
    }

    pub(crate) fn outside_ports(&self) -> Vec<Violation<'file>> {
        self.0.iter().filter_map(EffectSite::outside_ports).collect()
    }
}
