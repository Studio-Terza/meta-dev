use std::marker::PhantomData;

use graph::Model;
use lint_core::Report;
use lint_file::ModelLint;

use crate::containment::{ClusterLint, Containment};
use crate::system::TypeSystem;

pub trait GraphLint {
    fn report_graph<'file>(system: &TypeSystem<'_, 'file>) -> Report<'file>;

    fn dirty<'file>(system: &TypeSystem<'_, 'file>) -> Option<Report<'file>> {
        Self::report_graph(system).dirty()
    }
}

#[derive(Debug)]
pub struct Clustered<Check>(PhantomData<Check>);

impl<Check: ClusterLint> GraphLint for Clustered<Check> {
    fn report_graph<'file>(system: &TypeSystem<'_, 'file>) -> Report<'file> {
        system.containment().report::<Check>()
    }
}

impl<Check: ClusterLint> ModelLint for Clustered<Check> {
    fn report(model: &Model) -> Report<'_> {
        Containment::clustered(model).report::<Check>()
    }
}
