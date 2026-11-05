use std::marker::PhantomData;

use lint_core::Report;

use crate::containment::ClusterLint;
use crate::system::TypeSystem;

pub trait GraphLint {
    fn report_graph<'file>(system: &TypeSystem<'_, 'file>) -> Report<'file>;

    fn dirty<'file>(system: &TypeSystem<'_, 'file>) -> Option<Report<'file>> {
        Self::report_graph(system).dirty()
    }
}

pub trait Covered {
    type Coverage;
}

#[derive(Debug)]
pub struct Clustered<Check>(PhantomData<Check>);

impl<Check: ClusterLint> GraphLint for Clustered<Check> {
    fn report_graph<'file>(system: &TypeSystem<'_, 'file>) -> Report<'file> {
        system.containment().report::<Check>()
    }
}
