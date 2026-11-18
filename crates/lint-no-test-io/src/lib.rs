mod effect_paths;
mod finding;

use effect_paths::EffectPaths;
use finding::Finding;
use graph::{Harness, Model};
use lint_core::{Gate, Headline, Report, Violation};
use lint_file::ModelLint;

#[derive(Debug)]
pub struct NoTestIo;

impl ModelLint for NoTestIo {
    fn report(model: &Model) -> Report<'_> {
        let paths = EffectPaths::from(model);
        Report::new(
            Headline::new("unit tests do no I/O and start no process"),
            model
                .uses()
                .filter(|placed| placed.harness() == Harness::TestTarget)
                .filter_map(|placed| Gate::PORTS.names_crate(placed.krate(), None).unwrap_or(Some(placed)))
                .filter_map(|placed| {
                    paths.effect(placed.entity()).map(|path| Violation::new(placed.site(), Finding::from(path.clone())))
                })
                .collect(),
        )
    }
}
