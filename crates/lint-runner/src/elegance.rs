use lint_complexity::Complexity;
use lint_core::{Lint, Report, SourceFile};
use lint_effect_sites::EffectSites;
use lint_linear_flow::{FlowShapes, LinearBodies, PlainConditions, PureRouters};
use lint_transition_graph::{
    AcyclicQuotients, ConnectedClusters, Containment, CoreMutation, DeepClusters, DuplicateTransitions, GraphLint,
    InterfaceByConsumers, InterfaceByInternals, LowestCluster, TransitionGraph,
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
        let containment = Containment::clustered(files, &facts, &transitions);
        vec![
            Complexity::report(files),
            TransitionGraph::report_graph(&facts, &transitions),
            DuplicateTransitions::report_graph(&facts, &transitions),
            CoreMutation::report_graph(&facts, &transitions),
            containment.report::<ConnectedClusters>(),
            containment.report::<AcyclicQuotients>(),
            containment.report::<InterfaceByConsumers>(),
            containment.report::<InterfaceByInternals>(),
            containment.report::<LowestCluster>(),
            containment.report::<DeepClusters>(),
            EffectSites::report(files),
            FlowShapes::report(files),
            LinearBodies::report(files),
            PureRouters::report(files),
            PlainConditions::report(files),
        ]
    }
}
