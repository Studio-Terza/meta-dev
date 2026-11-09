use std::fmt;

use crate::crate_name::CrateName;
use crate::package_id::PackageId;
use crate::role::Role;

use super::super::Graph;
use super::breaching::Breaching;

#[derive(Debug)]
pub struct Unroled<'meta> {
    breaching: Breaching<'meta>,
    port: &'meta CrateName,
    role: Role,
}

impl<'meta> Unroled<'meta> {
    pub(super) fn found(graph: &Graph<'meta>) -> Vec<Unroled<'meta>> {
        graph
            .members
            .iter()
            .flat_map(|dependent| {
                graph.requires.direct(dependent).filter_map(move |required| Self::depending(graph, dependent, required))
            })
            .collect()
    }

    fn depending(graph: &Graph<'meta>, dependent: &PackageId, required: &PackageId) -> Option<Unroled<'meta>> {
        let port = graph.package(required)?;
        let role = port.port()?.dependents()?;
        let package = graph.package(dependent)?;
        let breaching = Breaching::new(package.name(), package.keys(port.name()).collect());
        package.declares(role).map_or(Some(Unroled { breaching, port: port.name(), role }), |_| None)
    }
}

impl<'meta> AsRef<Breaching<'meta>> for Unroled<'meta> {
    fn as_ref(&self) -> &Breaching<'meta> {
        &self.breaching
    }
}

impl fmt::Display for Unroled<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "depends on {}, which only a crate declaring [package.metadata.{}] depends on",
            self.port, self.role
        )
    }
}
