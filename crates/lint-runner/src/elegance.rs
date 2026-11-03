use lint_complexity::Complexity;
use lint_core::{Lint, Report, SourceFile};
use lint_effect_sites::EffectSites;
use lint_linear_flow::{FlowShapes, LinearBodies, PlainConditions, PureRouters};
use lint_repeated_shapes::RepeatedShapes;
use lint_transition_graph::{
    AcyclicQuotients, ClusterLint, ConnectedClusters, Containment, CoreMutation, DeepClusters, DuplicateTransitions,
    GraphLint, InterfaceByConsumers, InterfaceByInternals, LowestCluster, TransitionGraph,
};
use transition_facts::Transitions;
use type_facts::Facts;

#[derive(Debug)]
pub struct Elegance;

impl Elegance {
    #[must_use]
    pub fn reports(files: &[SourceFile]) -> Vec<Report<'_>> {
        let facts = Facts::of_files(files);
        let transitions = Transitions::of_files(files);
        let containment = Containment::of(files, &facts, &transitions);
        vec![
            RepeatedShapes::report(files),
            Complexity::report(files),
            TransitionGraph::report_graph(&facts, &transitions),
            DuplicateTransitions::report_graph(&facts, &transitions),
            CoreMutation::report_graph(&facts, &transitions),
            ConnectedClusters::report_clusters(&containment),
            AcyclicQuotients::report_clusters(&containment),
            InterfaceByConsumers::report_clusters(&containment),
            InterfaceByInternals::report_clusters(&containment),
            LowestCluster::report_clusters(&containment),
            DeepClusters::report_clusters(&containment),
            EffectSites::report(files),
            FlowShapes::report(files),
            LinearBodies::report(files),
            PureRouters::report(files),
            PlainConditions::report(files),
        ]
    }
}
