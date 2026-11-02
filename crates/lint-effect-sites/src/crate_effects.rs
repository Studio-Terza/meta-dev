mod called;
mod effect;
mod effect_site;
mod stdio_macros;
mod summary;
mod vocabulary;

use lint_core::{SourceFile, Violation};
use lint_paths::Paths;
use lint_placement::CrateName;

use effect_site::EffectSite;
use stdio_macros::StdioMacros;
use summary::Summary;
use vocabulary::Vocabulary;

#[derive(Debug)]
pub(crate) struct CrateEffects(Vec<EffectSite>);

impl CrateEffects {
    pub(crate) fn found(files: &[&SourceFile]) -> CrateEffects {
        let called = Paths::read(files.iter().copied(), &Vocabulary::FINALS);
        let mut sites: Vec<EffectSite> = called
            .iter()
            .filter_map(Vocabulary::site_of)
            .chain(files.iter().flat_map(|file| StdioMacros::found(file)))
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
