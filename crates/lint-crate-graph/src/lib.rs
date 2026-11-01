mod crate_summary;
mod item_graph;

use std::collections::BTreeMap;

use lint_core::{CrateRoot, Lint, Report, SourceFile};

use crate_summary::CrateSummary;
use item_graph::ItemGraph;

#[derive(Debug)]
pub struct CrateGraph;

impl Lint for CrateGraph {
    fn report(files: &[SourceFile]) -> Report {
        let crates = files.iter().filter(|file| file.is_shipped()).fold(
            BTreeMap::<&CrateRoot, Vec<&SourceFile>>::new(),
            |mut crates, file| {
                crates.entry(file.placement().crate_root()).or_default().push(file);
                crates
            },
        );
        Report::new(
            "a crate whose items form one big cycle or two loosely linked groups is split",
            crates
                .into_iter()
                .flat_map(|(root, files)| CrateSummary::of_graph(root, &ItemGraph::of_files(&files)).violations())
                .collect(),
        )
    }
}
