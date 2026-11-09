use std::collections::{BTreeMap, BTreeSet};

use super::manifest::Kind;
use super::package::Package;
use super::role::Role;
use super::unit_key::UnitKey;

#[derive(Debug)]
pub(super) struct Crates {
    members: BTreeSet<Package>,
    macros: BTreeSet<Package>,
    edges: BTreeMap<Package, Vec<Edge>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Origin {
    Workspace,
    ProcMacro,
    Registry,
}

#[derive(Clone, Debug)]
pub(super) struct Edge {
    kind: Kind,
    package: Package,
}

impl Edge {
    pub(super) fn new(kind: Kind, package: Package) -> Edge {
        Edge { kind, package }
    }

    fn reached(&self, kinds: &[Kind]) -> Option<&Package> {
        kinds.contains(&self.kind).then_some(&self.package)
    }
}

impl Crates {
    pub(super) fn new(
        members: BTreeSet<Package>,
        macros: BTreeSet<Package>,
        edges: BTreeMap<Package, Vec<Edge>>,
    ) -> Crates {
        Crates { members, macros, edges }
    }

    pub(super) fn origin(&self, package: &Package) -> Origin {
        self.members
            .get(package)
            .map(|_| Origin::Workspace)
            .or_else(|| self.macros.get(package).map(|_| Origin::ProcMacro))
            .unwrap_or(Origin::Registry)
    }

    pub(super) fn dependencies(&self, package: &Package, kinds: &[Kind]) -> Vec<&Package> {
        self.edges.get(package).into_iter().flatten().filter_map(|edge| edge.reached(kinds)).collect()
    }

    pub(super) fn linked(&self, package: &Package) -> UnitKey {
        UnitKey::new(package.clone(), self.macros.get(package).map_or(Role::Library, |_| Role::HostBuild))
    }
}
