use serde::Deserialize;

use super::super::crates::Edge;
use super::super::package::Package;
use text_lines::OneLine;

#[derive(Debug, Deserialize)]
pub(super) struct Dependency {
    pkg: OneLine,
    dep_kinds: Vec<Declared>,
}

#[derive(Debug, Deserialize)]
struct Declared {
    kind: Option<Named>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Named {
    Dev,
    Build,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in super::super) enum Kind {
    Normal,
    Dev,
    Build,
}

impl Dependency {
    pub(super) fn package(&self) -> &OneLine {
        &self.pkg
    }

    pub(super) fn edges(&self, package: &Package) -> Vec<Edge> {
        self.dep_kinds.iter().map(|declared| Edge::new(Kind::from(declared.kind), package.clone())).collect()
    }
}

impl From<Option<Named>> for Kind {
    fn from(named: Option<Named>) -> Kind {
        match named {
            None => Kind::Normal,
            Some(Named::Dev) => Kind::Dev,
            Some(Named::Build) => Kind::Build,
        }
    }
}
