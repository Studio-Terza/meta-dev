use lint_core::Report;
use transition_facts::Transitions;
use type_facts::Facts;

pub trait GraphLint {
    fn report_graph(facts: &Facts, transitions: &Transitions) -> Report;
}
