use lint_complexity::Complexity;
use lint_core::{Lint, Report, SourceFile};
use lint_effect_sites::EffectSites;
use lint_linear_flow::FlowShapes;
use lint_repeated_shapes::NormalisedShapes;
use lint_transition_graph::{
    AlgebraLaws, Clustered, ConnectedClusters, CoreMutation, Covered, Extraction, GraphLint, Homomorphisms,
    InterfaceByConsumers, OneEdge, ParallelStructures, RepeatedRoutes, TransitionGraph, TypeSystem,
};

#[derive(Debug)]
pub struct Elegance;

impl Elegance {
    #[must_use]
    pub fn reports(files: &[SourceFile]) -> Vec<Report<'_>> {
        let extraction = Extraction::from(files);
        let system = TypeSystem::built_from(files, &extraction);
        vec![
            Complexity::report(files),
            TransitionGraph::report_graph(&system),
            <<OneEdge as Covered>::Coverage>::report_graph(&system),
            CoreMutation::report_graph(&system),
            AlgebraLaws::report_graph(&system),
            Homomorphisms::report_graph(&system),
            RepeatedRoutes::report_graph(&system),
            ParallelStructures::report_graph(&system),
            <Clustered<<ConnectedClusters as Covered>::Coverage>>::report_graph(&system),
            <Clustered<<InterfaceByConsumers as Covered>::Coverage>>::report_graph(&system),
            EffectSites::report(files),
            FlowShapes::report(files),
            NormalisedShapes::report(files),
        ]
    }
}
