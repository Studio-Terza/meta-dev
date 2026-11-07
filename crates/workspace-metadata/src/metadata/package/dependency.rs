use serde::Deserialize;

use crate::crate_name::CrateName;

#[derive(Debug, Deserialize)]
pub struct Dependency {
    name: CrateName,
}

impl Dependency {
    pub const fn name(&self) -> &CrateName {
        &self.name
    }
}
