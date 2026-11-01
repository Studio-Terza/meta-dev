use serde::Deserialize;

use crate::cli::workspace::crate_name::CrateName;

#[derive(Debug, Deserialize)]
pub struct Dependency {
    name: CrateName,
}

impl Dependency {
    pub fn name(&self) -> &CrateName {
        &self.name
    }
}
