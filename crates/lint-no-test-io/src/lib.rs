mod effect_paths;
mod finding;

use effect_paths::EffectPaths;
use finding::Finding;
use lint_core::{Gate, Headline, SourceFile, TestFile, Violation};
use lint_file::{FileLint, Reads};
use lint_paths::Paths;

#[derive(Debug)]
pub struct NoTestIo;

impl FileLint for NoTestIo {
    const HEADLINE: Headline = Headline::new("unit tests do no I/O and start no process");
    const READS: Reads = Reads::Every;

    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        let tested = TestFile::tested(file).filter(|test| Gate::PORTS.owns(test.file().head().placement()).is_none());
        Paths::every(tested.map(TestFile::file))
            .iter()
            .filter_map(|used| EffectPaths::effect(used).map(|path| Violation::new(used.place(), Finding::from(path))))
            .collect()
    }
}
