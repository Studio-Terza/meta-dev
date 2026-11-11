use serde::Deserialize;

use crate::crate_name::CrateName;
use crate::package_id::PackageId;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum DependencyKind {
    Dev,
    Build,
    #[serde(other)]
    Other,
}

#[derive(Clone, Debug, Deserialize)]
struct Linkage {
    kind: Option<DependencyKind>,
}

impl Linkage {
    fn normal(&self) -> Option<&Linkage> {
        match self.kind {
            None => Some(self),
            Some(DependencyKind::Dev | DependencyKind::Build | DependencyKind::Other) => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct NodeDependency {
    name: CrateName,
    pkg: PackageId,
    #[serde(default)]
    dep_kinds: Vec<Linkage>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Node {
    id: PackageId,
    #[serde(default)]
    dependencies: Vec<PackageId>,
    #[serde(default)]
    deps: Vec<NodeDependency>,
}

impl NodeDependency {
    pub fn krate(&self) -> &CrateName {
        &self.name
    }

    pub fn package_id(&self) -> &PackageId {
        &self.pkg
    }
}

impl Node {
    pub fn package_id(&self) -> &PackageId {
        &self.id
    }

    pub fn dependencies(&self) -> &[PackageId] {
        &self.dependencies
    }

    pub fn normal(&self) -> impl Iterator<Item = &NodeDependency> {
        self.deps.iter().filter(|dependency| dependency.dep_kinds.iter().find_map(Linkage::normal).is_some())
    }
}
