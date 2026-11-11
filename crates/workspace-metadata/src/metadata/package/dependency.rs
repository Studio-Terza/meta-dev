use serde::Deserialize;

use crate::crate_name::CrateName;

#[derive(Clone, Debug, Deserialize)]
pub struct Dependency {
    name: CrateName,
    rename: Option<CrateName>,
}

impl Dependency {
    pub fn name(&self) -> &CrateName {
        &self.name
    }

    pub fn key_of(&self, required: &CrateName) -> Option<&CrateName> {
        Some(&self.name).filter(|name| *name == required).map(|name| self.rename.as_ref().unwrap_or(name))
    }
}
