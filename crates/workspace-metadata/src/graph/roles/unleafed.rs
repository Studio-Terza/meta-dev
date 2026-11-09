use std::collections::BTreeSet;
use std::fmt;

use crate::crate_name::CrateName;
use crate::metadata::{Package, PortTable};
use crate::package_id::PackageId;
use crate::role::Role;

use super::super::Graph;
use super::breaching::Breaching;
use super::owned::Owned;

#[derive(Debug)]
pub struct Unleafed<'meta> {
    breaching: Breaching<'meta>,
    dependency: &'meta CrateName,
    owned: &'meta CrateName,
}

impl<'meta> Unleafed<'meta> {
    pub(super) fn found(graph: &Graph<'meta>, owned: &Owned<'_, 'meta>) -> Vec<Unleafed<'meta>> {
        graph
            .declaring(Role::Lexer)
            .into_iter()
            .filter_map(|id| graph.package(id))
            .flat_map(|lexer| {
                lexer
                    .package_id()
                    .into_iter()
                    .flat_map(|id| graph.normal.direct(id))
                    .filter_map(move |required| Self::depending(graph, owned, lexer, required))
            })
            .collect()
    }

    fn depending(
        graph: &Graph<'meta>,
        owned: &Owned<'_, 'meta>,
        lexer: &'meta Package,
        required: &'meta PackageId,
    ) -> Option<Unleafed<'meta>> {
        let dependency = graph.package(required)?;
        let admitted = dependency
            .port()
            .and_then(PortTable::dependents)
            .filter(|role| *role == Role::Lexer)
            .or_else(|| dependency.declares(Role::Lexer));
        let held =
            admitted.map_or_else(|| owned.first_in(&graph.normal_reach(required, &BTreeSet::new())), |_| None)?;
        let breaching = Breaching::new(lexer.name(), lexer.keys(dependency.name()).collect());
        Some(Unleafed { breaching, dependency: dependency.name(), owned: graph.package(held)?.name() })
    }
}

impl<'meta> AsRef<Breaching<'meta>> for Unleafed<'meta> {
    fn as_ref(&self) -> &Breaching<'meta> {
        &self.breaching
    }
}

impl fmt::Display for Unleafed<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "is a lexer and depends on {}, whose closure holds {}, a package a port owns",
            self.dependency, self.owned
        )
    }
}
