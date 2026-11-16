mod tangle;

use graph::{ItemGraph, Model};
use lint_core::{Headline, Report, Violation};
use lint_file::ModelLint;

use tangle::Tangle;

#[derive(Debug)]
pub struct CrateGraph;

impl ModelLint for CrateGraph {
    fn report(model: &Model) -> Report<'_> {
        let references = model.references();
        let found: Vec<Violation<'_>> = ItemGraph::of_references(references.targets().clone())
            .tangles()
            .into_iter()
            .filter_map(|members| Tangle::flagged(model, references, members))
            .collect();
        Report::new(Headline::new("a crate whose items form one big cycle is split"), found)
    }
}
