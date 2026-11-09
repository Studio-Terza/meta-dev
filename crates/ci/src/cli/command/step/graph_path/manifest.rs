mod dependency;

use std::collections::BTreeMap;

use serde::Deserialize;

use super::crates::Crates;
use super::package::Package;
use dependency::Dependency;
pub(super) use dependency::Kind;
use text_lines::OneLine;

#[derive(Debug, Deserialize)]
pub(super) struct Manifest {
    packages: Vec<Listed>,
    workspace_members: Vec<OneLine>,
    resolve: Resolve,
}

#[derive(Debug, Deserialize)]
struct Listed {
    id: OneLine,
    name: OneLine,
    version: OneLine,
    targets: Vec<Target>,
}

#[derive(Debug, Deserialize)]
struct Target {
    kind: Vec<TargetKind>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum TargetKind {
    ProcMacro,
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
struct Resolve {
    nodes: Vec<Node>,
}

#[derive(Debug, Deserialize)]
struct Node {
    id: OneLine,
    deps: Vec<Dependency>,
}

impl Listed {
    fn proc_macro(&self) -> Option<&Listed> {
        self.targets.iter().flat_map(|target| &target.kind).find(|kind| **kind == TargetKind::ProcMacro).map(|_| self)
    }
}

impl Manifest {
    pub(super) fn crates(self) -> Crates {
        let packages: BTreeMap<&OneLine, Package> = self
            .packages
            .iter()
            .map(|listed| (&listed.id, Package::new(listed.name.clone(), listed.version.clone())))
            .collect();
        let found = |id: &OneLine| packages.get(id).cloned();
        let members = self.workspace_members.iter().filter_map(found).collect();
        let macros =
            self.packages.iter().filter_map(Listed::proc_macro).filter_map(|listed| found(&listed.id)).collect();
        let edges = self
            .resolve
            .nodes
            .iter()
            .filter_map(|node| {
                let on = node
                    .deps
                    .iter()
                    .filter_map(|dependency| Some(dependency.edges(&found(dependency.package())?)))
                    .flatten()
                    .collect();
                Some((found(&node.id)?, on))
            })
            .collect();
        Crates::new(members, macros, edges)
    }
}
