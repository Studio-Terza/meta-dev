mod finding;
mod raw_paths;

use finding::Finding;
use lint_core::{Gate, Lint, Report, SourceFile};
use raw_paths::RawPaths;

#[derive(Debug)]
pub struct RawResources;

impl Lint for RawResources {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "only ports touch raw resources",
            files.iter().filter(|file| !Gate::PORTS.admits(file)).flat_map(RawPaths::found).collect(),
        )
    }
}
