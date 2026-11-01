mod crate_summary;

use std::collections::BTreeMap;

use lint_core::{CrateFile, Lint, Report, SourceFile};

use crate_summary::CrateSummary;
use lint_item_graph::ItemGraph;

#[derive(Debug)]
pub struct CrateGraph;

impl Lint for CrateGraph {
    fn report(files: &[SourceFile]) -> Report {
        let crates = files.iter().filter(|file| file.is_shipped()).fold(
            BTreeMap::<&CrateFile, Vec<&SourceFile>>::new(),
            |mut crates, file| {
                crates.entry(file.crate_file()).or_default().push(file);
                crates
            },
        );
        Report::new(
            "a crate whose items form one big cycle or two loosely linked groups is split",
            crates
                .into_iter()
                .flat_map(|(crate_file, files)| {
                    CrateSummary::of_graph(crate_file, &ItemGraph::of_files(&files)).violations()
                })
                .collect(),
        )
    }
}
