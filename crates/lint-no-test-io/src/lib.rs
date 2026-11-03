mod effect_paths;
mod finding;

use effect_paths::EffectPaths;
use finding::Finding;
use lint_core::{Headline, Lint, Report, SourceFile, Violation};
use lint_paths::Paths;

#[derive(Debug)]
pub struct NoTestIo;

impl Lint for NoTestIo {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new("unit tests do no I/O and start no process"),
            Paths::every(files.iter().filter_map(EffectPaths::tested))
                .iter()
                .filter_map(|used| {
                    EffectPaths::effect(used).map(|path| Violation::new(used.place(), Finding::from(path)))
                })
                .collect(),
        )
    }
}
