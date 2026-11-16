use serde::Deserialize;

use crate::crate_name::CrateName;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Edge {
    Dev,
    Build,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Dependency {
    name: CrateName,
    rename: Option<CrateName>,
    kind: Option<Edge>,
}

impl Dependency {
    pub fn shipped(&self) -> Option<&CrateName> {
        self.kind.filter(|edge| *edge == Edge::Dev).map_or(Some(&self.name), |_| None)
    }

    pub fn key_of(&self, required: &CrateName) -> Option<&CrateName> {
        Some(&self.name).filter(|name| *name == required).map(|name| self.rename.as_ref().unwrap_or(name))
    }
}

impl AsRef<CrateName> for Dependency {
    fn as_ref(&self) -> &CrateName {
        &self.name
    }
}
