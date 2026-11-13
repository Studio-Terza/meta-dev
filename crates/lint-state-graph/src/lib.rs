mod declarations;
mod declared_graphs;
mod edge;
mod edge_count;
mod edge_scan;
mod field_visibility;
mod finding;
mod function_shape;
mod impl_context;
mod variant_matches;

use declarations::Declarations;
use declared_graphs::DeclaredGraphs;
use edge_scan::EdgeScan;
use field_visibility::FieldVisibility;
use lint_core::{Headline, Lint};
use variant_matches::VariantMatches;

use lint_core::Report;
use lint_core::SourceFile;

#[derive(Debug)]
pub struct StateGraph;

impl Lint for StateGraph {
    fn report(sources: &[SourceFile]) -> Report<'_> {
        let files: Vec<&SourceFile> = sources.iter().filter_map(SourceFile::shipped).collect();
        let declarations = Declarations::from(files.as_slice());
        let edges = files.iter().fold(EdgeScan::new(&declarations), |mut edges, file| {
            edges.scan(file);
            edges
        });
        Report::new(
            Headline::new("state graph violated"),
            files
                .iter()
                .flat_map(|file| {
                    Vec::from(FieldVisibility::from(*file))
                        .into_iter()
                        .chain(VariantMatches::violations(file, &declarations))
                        .chain(Vec::from(DeclaredGraphs::from(*file)))
                })
                .chain(edges.violations())
                .collect(),
        )
    }
}
