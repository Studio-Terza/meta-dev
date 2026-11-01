mod dependency;

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::cli::failure::Failure;
use crate::cli::workspace::crate_name::CrateName;
use dependency::Dependency;

#[derive(Debug, Deserialize)]
pub struct Package {
    name: CrateName,
    manifest_path: PathBuf,
    dependencies: Vec<Dependency>,
}

impl Package {
    pub fn name(&self) -> &CrateName {
        &self.name
    }

    pub fn directory_under(&self, root: &Path) -> Result<PathBuf, Failure> {
        self.manifest_path
            .parent()
            .and_then(|directory| directory.strip_prefix(root).ok())
            .map(Path::to_path_buf)
            .ok_or_else(|| Failure::Outside { manifest: self.manifest_path.clone() })
    }

    pub fn local_dependencies(&self) -> impl Iterator<Item = &CrateName> {
        self.dependencies.iter().filter_map(Dependency::local_name)
    }
}
