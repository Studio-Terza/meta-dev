use serde::Deserialize;

use crate::package_id::PackageId;

#[derive(Clone, Debug, Deserialize)]
pub struct Edge {
    pkg: PackageId,
    dep_kinds: Vec<EdgeKind>,
}

#[derive(Clone, Debug, Deserialize)]
struct EdgeKind {
    kind: Option<Kind>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Kind {
    Dev,
    Build,
}

impl Edge {
    pub fn normal(&self) -> Option<&PackageId> {
        self.dep_kinds.iter().find(|kind| kind.kind.is_none()).map(|_| &self.pkg)
    }
}
