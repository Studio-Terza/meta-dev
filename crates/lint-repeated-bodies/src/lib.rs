mod extent;
mod families;
mod node;
mod occurrence;
mod scan;

use lint_core::{Headline, Lint, Report, SourceFile};

use families::{Families, Family};
use scan::Scan;

#[derive(Debug)]
pub struct RepeatedBodies;

impl Lint for RepeatedBodies {
    fn report(files: &[SourceFile]) -> Report {
        let nodes = files.iter().filter(|file| file.is_shipped()).flat_map(Scan::of_file);
        Report::new(
            Headline::new(
                "report only, a code body is repeated with its names and literals erased; the size of each is listed so any floor can be read",
            ),
            Families::grouped(nodes).maximal().iter().map(Family::violation).collect(),
        )
    }
}
