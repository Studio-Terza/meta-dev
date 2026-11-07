mod owners;
mod port_crate;
mod reserved_names;

use lint_core::{Headline, Lint, Report, SourceFile};
use lint_effect_sites::EffectSites;
use lint_file::FileLint;

use owners::Owners;
use port_crate::PortCrate;
use reserved_names::{CountedName, ReservedNames, SqlxName};

#[derive(Debug)]
pub struct OwnedDependencies;

#[derive(Debug)]
pub struct EffectsInFillers;

#[derive(Debug)]
pub struct CountedCalls;

#[derive(Debug)]
pub struct HiddenSqlx;

impl Lint for OwnedDependencies {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new(
                "a dependency some crate owns through [package.metadata.port] owns has that one owner and is depended on by no other crate, in dependencies and dev-dependencies alike",
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
            files.iter().flat_map(ReservedNames::<CountedName>::violations).collect(),
        )
    }
}

impl Lint for HiddenSqlx {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new(
                "__sqlx is named only in store-ports and store-ports-macros, so every query goes through store_ports::query! and no crate reaches sqlx through the re-export",
            ),
            files.iter().flat_map(ReservedNames::<SqlxName>::violations).collect(),
        )
    }
}
