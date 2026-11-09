use std::collections::BTreeSet;
use std::fmt;

use crate::crate_name::CrateName;
use crate::metadata::Package;
use crate::role::Role;

use super::super::Graph;
use super::breaching::Breaching;
use super::owned::Owned;

#[derive(Debug)]
pub struct Unported<'meta> {
    breaching: Breaching<'meta>,
    owned: &'meta CrateName,
    role: Option<Role>,
}

impl<'meta> Unported<'meta> {
    pub(super) fn found(graph: &Graph<'meta>, owned: &Owned<'_, 'meta>) -> Vec<Unported<'meta>> {
        graph
            .members
            .iter()
            .filter_map(|id| graph.package(id)?.proc_macro())
            .flat_map(|package| Self::reached(graph, owned, package))
            .collect()
    }

    pub(super) fn new(breaching: Breaching<'meta>, owned: &'meta CrateName, role: Option<Role>) -> Unported<'meta> {
        Unported { breaching, owned, role }
    }

    fn reached(graph: &Graph<'meta>, owned: &Owned<'_, 'meta>, package: &'meta Package) -> Vec<Unported<'meta>> {
        let reach = package.package_id().map(|id| graph.normal_reach(id, &BTreeSet::new())).unwrap_or_default();
        owned.within(&reach).into_iter().filter_map(|owner| owner.reached_by(graph, package)).collect()
    }
}

impl<'meta> AsRef<Breaching<'meta>> for Unported<'meta> {
    fn as_ref(&self) -> &Breaching<'meta> {
        &self.breaching
    }
}

impl fmt::Display for Unported<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self.role.map(|role| format!(" and not through a crate declaring [package.metadata.{role}]"));
        write!(
            formatter,
            "is a proc-macro whose closure reaches {}, which a port owns, outside the owning unit{}",
            self.owned,
            path.unwrap_or_default()
        )
    }
}
