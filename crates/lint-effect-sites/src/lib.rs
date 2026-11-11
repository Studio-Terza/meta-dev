mod crate_effects;

use std::collections::BTreeMap;

use lint_core::{Headline, Lint, Report, SourceFile};
use lint_placement::CrateName;

pub use crate_effects::CrateEffects;

#[derive(Debug)]
pub struct EffectSites;

#[derive(Debug)]
pub struct EffectsInPorts;

impl Lint for EffectsInPorts {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new(
                "effects on files, environment, clock, network, processes, randomness and stdio happen only in ports",
            ),
            EffectSites::crates(files).values().flat_map(|found| CrateEffects::found(found).outside_ports()).collect(),
        )
    }
}

impl EffectSites {
    #[must_use]
    pub fn crates(files: &[SourceFile]) -> BTreeMap<CrateName, Vec<&SourceFile>> {
        files.iter().filter_map(SourceFile::shipped).fold(BTreeMap::new(), |mut crates, file| {
            crates.entry(CrateName::from(file.placement().package().root())).or_insert_with(Vec::new).push(file);
            crates
        })
    }
}

impl Lint for EffectSites {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new(
                "report only, effect sites in shipped code per crate (files, environment, clock, network, processes, randomness, stdio), counted everywhere, listed only outside ports; move each behind a port: a crate whose name ends in ports, or a module named ports",
            ),
            Self::crates(files).iter().flat_map(|(name, found)| CrateEffects::found(found).violations(name)).collect(),
        )
    }
}
