mod counted_names;
mod owners;
mod port_crate;

use lint_core::{Headline, Lint, Report, SourceFile};
use lint_effect_sites::EffectSites;

use counted_names::CountedNames;
use owners::Owners;
use port_crate::PortCrate;

#[derive(Debug)]
pub struct OwnedDependencies;

#[derive(Debug)]
pub struct EffectsInFillers;

#[derive(Debug)]
pub struct CountedCalls;

impl Lint for OwnedDependencies {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new(
                "a dependency some crate owns through [package.metadata.port] owns is depended on only by its owners, in dependencies and dev-dependencies alike",
            ),
            Owners::declared(files).foreign(),
        )
    }
}

impl Lint for EffectsInFillers {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new(
                "in a crate that declares a #[port] trait, effects on files, environment, clock, network, processes, randomness and stdio happen only inside impl <Trait>Filler and impl Release blocks",
            ),
            EffectSites::crates(files)
                .values()
                .filter_map(|found| PortCrate::declared(found))
                .flat_map(|ported| ported.stray_effects())
                .collect(),
        )
    }
}

impl Lint for CountedCalls {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new(
                "counted_call and counted_acquire are named only in port-core and port-derive, so every op is counted through a generated filler",
            ),
            files.iter().filter_map(CountedNames::outside_port_core).flat_map(CountedNames::named).collect(),
        )
    }
}
