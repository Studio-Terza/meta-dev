mod extent;
mod families;
mod kind;
mod node;
mod occurrence;
mod scan;

use lint_core::{Headline, Lint, Report, SourceFile};

use families::{Families, Family};
use scan::Scan;

#[derive(Debug)]
pub struct RepeatedShapes;

impl Lint for RepeatedShapes {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let nodes = files.iter().flat_map(Scan::of_file);
        Report::new(
            Headline::new(
                "a body of 40 tokens or more repeats across the workspace with its names and literals erased; each group says whether its copies are tests, shipped code or both",
            ),
            Families::grouped(nodes).maximal().iter().map(Family::violation).collect(),
        )
    }
}
