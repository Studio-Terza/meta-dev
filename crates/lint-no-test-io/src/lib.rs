mod effect_paths;
mod finding;

use effect_paths::EffectPaths;
use finding::Finding;
use lint_core::{Headline, Lint, Report, SourceFile, Violation};
use lint_paths::{Ground, Paths};

#[derive(Debug)]
pub struct NoTestIo;

impl Lint for NoTestIo {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            Headline::new("unit tests do no I/O and start no process"),
            Paths::every(files.iter().filter_map(EffectPaths::tested))
                .iter()
                .filter(|used| used.ground() == Ground::Open)
                .filter_map(|used| {
                    EffectPaths::effect(used).map(|path| Violation::new(used.place().clone(), Finding::from(path)))
                })
                .collect(),
        )
    }
}
