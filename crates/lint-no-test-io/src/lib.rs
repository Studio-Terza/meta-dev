mod boundary;
mod effect_paths;
mod finding;

use boundary::TestTarget;
use code_model::Model;
use effect_paths::EffectPaths;
use finding::Finding;
use lint_core::{Gate, Headline, ModelLint, Report, Violation};

#[derive(Debug)]
pub struct NoTestIo;

impl ModelLint for NoTestIo {
    fn report(model: &Model) -> Report<'_> {
        let paths = EffectPaths::from(model);
        Report::new(
            Headline::new("unit tests do no I/O and start no process"),
            model
                .uses()
                .filter(|placed| TestTarget::rooted(placed.root()).is_some())
                .filter(|placed| Gate::PORTS.names_crate(placed.krate()).is_none())
                .filter_map(|placed| {
                    paths.effect(placed.entity()).map(|path| Violation::new(placed.site(), Finding::from(path.clone())))
                })
                .collect(),
        )
    }
}
