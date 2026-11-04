use std::marker::PhantomData;

use lint_core::{Lint, Report, SourceFile};

use crate::containment::ClusterLint;
use crate::system::{Extraction, TypeSystem};

pub trait GraphLint {
    fn report_graph<'file>(system: &TypeSystem<'_, 'file>) -> Report<'file>;
}

#[derive(Debug)]
pub struct OnGraph<Graph>(PhantomData<Graph>);

impl<Graph: GraphLint> Lint for OnGraph<Graph> {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let extraction = Extraction::of_files(files);
        TypeSystem::of(files, &extraction).report::<Graph>()
    }
}

#[derive(Debug)]
pub struct Clustered<Check>(PhantomData<Check>);

impl<Check: ClusterLint> GraphLint for Clustered<Check> {
    fn report_graph<'file>(system: &TypeSystem<'_, 'file>) -> Report<'file> {
        system.containment().report::<Check>()
    }
}
