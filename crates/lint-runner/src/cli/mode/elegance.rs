use lint_complexity::Complexity;
use lint_core::{Lint, Report, SourceFile};
use lint_effect_sites::EffectSites;
use lint_linear_flow::FlowShapes;
use lint_transition_graph::{
    Clustered, ConnectedClusters, CoreMutation, DeepClusters, DuplicateTransitions, Extraction, GraphLint,
    InterfaceByConsumers, InterfaceByInternals, LowestCluster, TransitionGraph, TypeSystem,
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
            DuplicateTransitions::report_graph(&system),
            CoreMutation::report_graph(&system),
            <Clustered<ConnectedClusters>>::report_graph(&system),
            <Clustered<InterfaceByConsumers>>::report_graph(&system),
            <Clustered<InterfaceByInternals>>::report_graph(&system),
            <Clustered<LowestCluster>>::report_graph(&system),
            <Clustered<DeepClusters>>::report_graph(&system),
            EffectSites::report(files),
            FlowShapes::report(files),
        ]
    }
}
