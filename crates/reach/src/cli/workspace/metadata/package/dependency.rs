use std::path::PathBuf;

use serde::Deserialize;

use crate::cli::workspace::crate_name::CrateName;

#[derive(Debug, Deserialize)]
pub struct Dependency {
    name: CrateName,
    path: Option<PathBuf>,
}

impl Dependency {
    pub fn local_name(&self) -> Option<&CrateName> {
        self.path.as_ref().map(|_| &self.name)
    }
}
