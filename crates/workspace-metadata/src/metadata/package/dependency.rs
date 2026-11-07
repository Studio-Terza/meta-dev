use serde::Deserialize;

use crate::crate_name::CrateName;

#[derive(Clone, Debug, Deserialize)]
pub struct Dependency {
    name: CrateName,
}

impl Dependency {
    pub fn name(&self) -> &CrateName {
        &self.name
    }
}
