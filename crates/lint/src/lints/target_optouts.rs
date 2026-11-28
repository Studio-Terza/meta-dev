use std::sync::Arc;

use pipeline::Measure;

use crate::finding::{Finding, Findings, Found};
use crate::hooks::Hooks;
use crate::lint::{Fact, Lint, Reads};
use crate::source::Source;
use crate::workspace::Package;

#[derive(Debug, Measure)]
pub(crate) struct OptoutFacts(Findings);

#[derive(Debug, Default)]
pub(crate) struct TargetOptouts;

impl TargetOptouts {
    const FIX: &'static str = "declare the target with test = false, doctest = false and bench = false";
}

impl Hooks for TargetOptouts {}

impl<'file> Lint<'file> for TargetOptouts {
    const HEADLINE: &'static str = "every lib and bin target opts out of test, doctest and bench";
    const READS: Reads = Reads::All;

    type Fact = OptoutFacts;

    fn new(_: &'file Source) -> TargetOptouts {
        TargetOptouts
    }

    fn fact(self, _: &'file Source) -> OptoutFacts {
        OptoutFacts::default()
    }
}

impl Fact for OptoutFacts {
    fn findings(self, packages: &[Arc<Package>]) -> Vec<Finding> {
        let manifests = packages.iter().flat_map(|package| {
            package.unopted().iter().map(|target| {
                Found::at(target.line(), target.name(), TargetOptouts::FIX)
                    .in_file(TargetOptouts::HEADLINE, package.manifest())
            })
        });
        self.0.into_iter().chain(manifests).collect()
    }
}
