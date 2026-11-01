use lint_complexity::Complexity;
use lint_core::{Lint, Report, SourceFile};
use lint_repeated_bodies::RepeatedBodies;
use lint_type_graph::TypeGraph;
use lint_unconsumed_traits::UnconsumedTraits;
use lint_walk_similarity::WalkSimilarity;

#[derive(Debug)]
pub struct Elegance;

impl Elegance {
    #[must_use]
    pub fn reports(files: &[SourceFile]) -> Vec<Report> {
        vec![
            RepeatedBodies::report(files),
            UnconsumedTraits::report(files),
            Complexity::report(files),
            TypeGraph::report(files),
            WalkSimilarity::report(files),
        ]
    }
}
