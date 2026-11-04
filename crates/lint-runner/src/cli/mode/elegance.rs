use lint_complexity::Complexity;
use lint_core::{Lint, Report, SourceFile};
use lint_effect_sites::EffectSites;
use lint_linear_flow::FlowShapes;
use lint_transition_graph::{
    Clustered, ConnectedClusters, CoreMutation, DeepClusters, DuplicateTransitions, Extraction, InterfaceByConsumers,
    InterfaceByInternals, LowestCluster, TransitionGraph, TypeSystem,
};

#[derive(Debug)]
pub struct Elegance;

impl Elegance {
    #[must_use]
    pub fn reports(files: &[SourceFile]) -> Vec<Report<'_>> {
        let extraction = Extraction::of_files(files);
        let system = TypeSystem::of(files, &extraction);
        vec![
            Complexity::report(files),
            system.report::<TransitionGraph>(),
            system.report::<DuplicateTransitions>(),
            system.report::<CoreMutation>(),
            system.report::<Clustered<ConnectedClusters>>(),
            system.report::<Clustered<InterfaceByConsumers>>(),
            system.report::<Clustered<InterfaceByInternals>>(),
            system.report::<Clustered<LowestCluster>>(),
            system.report::<Clustered<DeepClusters>>(),
            EffectSites::report(files),
            FlowShapes::report(files),
        ]
    }
}
