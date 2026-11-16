mod raw_paths;

use graph::Model;
use lint_core::{Headline, Report};
use lint_file::ModelLint;
use raw_paths::RawPaths;

#[derive(Debug)]
pub struct RawResources;

impl ModelLint for RawResources {
    fn report(model: &Model) -> Report<'_> {
        Report::new(Headline::new("only ports touch raw resources"), RawPaths::found(model))
    }
}
