use std::marker::PhantomData;

use lint_core::{Lint, Report, SourceFile};
use transition_facts::Transitions;
use type_facts::Facts;

pub trait GraphLint {
    fn report_graph<'file>(facts: &Facts<'file>, transitions: &Transitions<'file>) -> Report<'file>;
}

#[derive(Debug)]
pub struct OnGraph<Graph>(PhantomData<Graph>);

impl<Graph: GraphLint> Lint for OnGraph<Graph> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Graph::report_graph(&Facts::of_files(files), &Transitions::of_files(files))
    }
}
