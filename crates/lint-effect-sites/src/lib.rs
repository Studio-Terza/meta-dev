mod crate_effects;

use std::collections::BTreeMap;

use lint_core::{Headline, Lint, Report, SourceFile};
use lint_placement::CrateName;

use crate_effects::CrateEffects;

#[derive(Debug)]
pub struct EffectSites;

impl EffectSites {
    fn crates(files: &[SourceFile]) -> BTreeMap<CrateName, Vec<&SourceFile>> {
        files.iter().filter_map(SourceFile::shipped).fold(BTreeMap::new(), |mut crates, file| {
            crates.entry(CrateName::from(file.placement().package().root())).or_insert_with(Vec::new).push(file);
            crates
        })
    }
}

impl Lint for EffectSites {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            Headline::new(
                "report only, effect sites in shipped code per crate (files, environment, clock, network, processes, randomness, stdio), counted everywhere, listed only outside ports; move each behind a port: a crate whose name ends in ports, or a module named ports",
            ),
            Self::crates(files).iter().flat_map(|(name, found)| CrateEffects::found(found).violations(name)).collect(),
        )
    }
}
