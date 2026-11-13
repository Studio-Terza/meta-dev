mod crate_summary;

use lint_core::{FileHead, Headline, Lint, Report, SourceFile};
use lint_item_graph::FileNames;

use crate_summary::CrateSummary;

#[derive(Debug)]
pub struct CrateGraph;

impl CrateGraph {
    #[must_use]
    pub fn of_names<'file>(names: &FileNames<'file>) -> Report<'file> {
        Report::new(
            Headline::new("a crate whose items form one big cycle is split"),
            names
                .graphs(FileHead::crate_file)
                .into_iter()
                .flat_map(|(crate_file, graph)| CrateSummary::of_graph(crate_file, &graph).violations())
                .collect(),
        )
    }
}

impl Lint for CrateGraph {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Self::of_names(&FileNames::from(files))
    }
}
