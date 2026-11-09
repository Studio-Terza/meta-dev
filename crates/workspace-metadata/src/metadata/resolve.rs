mod edge;

use serde::Deserialize;

use crate::package_id::PackageId;
pub use edge::Edge;

#[derive(Clone, Debug, Deserialize)]
pub struct Resolve {
    nodes: Vec<Node>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Node {
    id: PackageId,
    dependencies: Vec<PackageId>,
    #[serde(default)]
    deps: Vec<Edge>,
}

impl Resolve {
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }
}

impl AsRef<PackageId> for Node {
    fn as_ref(&self) -> &PackageId {
        &self.id
    }
}

impl Node {
    pub fn dependencies(&self) -> &[PackageId] {
        &self.dependencies
    }

    pub fn edges(&self) -> &[Edge] {
        &self.deps
    }
}
