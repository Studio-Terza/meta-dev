use lint_complexity::Complexity;
use lint_core::{Lint, Report, SourceFile};
use lint_effect_sites::EffectSites;
use lint_repeated_bodies::RepeatedBodies;
use lint_transition_graph::{
    Bisimilarity, CommutingTriangles, CoreMutation, DuplicateTransitions, GraphLint, HiddenStates, LiveNodes,
    RepeatedMotifs, Tangles, TransitionGraph,
};
use lint_type_graph::TypeGraph;
use lint_unconsumed_traits::UnconsumedTraits;
use lint_walk_similarity::WalkSimilarity;
use transition_facts::Transitions;
use type_facts::{Facts, FactsLint};

#[derive(Debug)]
pub struct Elegance;

impl Elegance {
    #[must_use]
    pub fn reports(files: &[SourceFile]) -> Vec<Report> {
        let facts = Facts::of_files(files);
        let transitions = Transitions::of_files(files);
        vec![
            RepeatedBodies::report(files),
            UnconsumedTraits::report(files),
            Complexity::report(files),
            TypeGraph::report_facts(&facts),
            TransitionGraph::report_graph(&facts, &transitions),
            DuplicateTransitions::report_graph(&facts, &transitions),
            CoreMutation::report_graph(&facts, &transitions),
            HiddenStates::report_graph(&facts, &transitions),
            LiveNodes::report_graph(&facts, &transitions),
            Bisimilarity::report_graph(&facts, &transitions),
            CommutingTriangles::report_graph(&facts, &transitions),
            RepeatedMotifs::report_graph(&facts, &transitions),
            Tangles::report_graph(&facts, &transitions),
            WalkSimilarity::report_facts(&facts),
            EffectSites::report(files),
        ]
    }
}
