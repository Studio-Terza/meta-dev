use std::marker::PhantomData;

use lint_core::{Lint, Report, SourceFile};
use transition_facts::Transitions;

use crate::Containment;
use crate::containment::ClusterLint;
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

#[derive(Debug)]
pub struct OnContainment<Check>(PhantomData<Check>);

impl<Check: ClusterLint> Lint for OnContainment<Check> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let facts = Facts::of_files(files);
        let transitions = Transitions::of_files(files);
        Containment::clustered(files, &facts, &transitions).report::<Check>()
    }
}
