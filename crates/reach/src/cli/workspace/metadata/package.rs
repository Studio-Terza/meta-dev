mod declared;
mod dependency;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::cli::failure::Failure;
use crate::cli::workspace::crate_name::CrateName;
use declared::Declared;
use dependency::Dependency;

#[derive(Debug, Deserialize)]
pub struct Package {
    name: CrateName,
    manifest_path: PathBuf,
    dependencies: Vec<Dependency>,
    metadata: Option<Declared>,
}

impl Package {
    pub fn name(&self) -> &CrateName {
        &self.name
    }

    pub fn readings(&self) -> &[PathBuf] {
        self.metadata.as_ref().map_or(&[], Declared::reads)
    }

    pub fn directory_under(&self, root: &Path) -> Result<PathBuf, Failure> {
        self.manifest_path
            .parent()
            .and_then(|directory| directory.strip_prefix(root).ok())
            .map(Path::to_path_buf)
            .ok_or_else(|| Failure::Outside { manifest: self.manifest_path.clone() })
    }

    pub fn dependencies_among<'members>(
        &'members self,
        members: &'members BTreeSet<CrateName>,
    ) -> impl Iterator<Item = &'members CrateName> {
        self.dependencies.iter().map(Dependency::name).filter(|name| members.contains(*name))
    }
}
