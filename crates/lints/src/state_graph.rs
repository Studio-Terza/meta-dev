mod declarations;
mod edge_scan;
mod field_visibility;
mod finding;
mod function_shape;
mod impl_context;
mod variant_matches;

use crate::Lint;
use declarations::Declarations;
use edge_scan::EdgeScan;
use field_visibility::FieldVisibility;
use variant_matches::VariantMatches;

use lint_core::Most;
use lint_core::Report;
use lint_core::SourceFile;

#[derive(Debug)]
pub struct StateGraph;

impl Lint for StateGraph {
    fn report(files: &[SourceFile]) -> Report {
        let files: Vec<&SourceFile> = files.iter().filter(|file| file.is_shipped()).collect();
        let declarations = Declarations::from(files.as_slice());
        let edges = files.iter().fold(EdgeScan::new(&declarations), |mut edges, file| {
            edges.scan(file);
            edges
        });
        Report::new(
            "state graph violated",
            files
                .iter()
                .flat_map(|file| {
                    FieldVisibility::violations(file).into_iter().chain(VariantMatches::violations(file, &declarations))
                })
                .chain(edges.violations())
                .collect(),
        )
    }
}

impl StateGraph {
    const EDGES_PER_VERTEX: Most = Most::new(4);
}
