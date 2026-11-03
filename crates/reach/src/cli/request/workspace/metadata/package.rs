mod declared;
mod dependency;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::cli::request::workspace::crate_name::CrateName;
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

    pub fn directory_under(&self, root: &Path) -> Option<&Path> {
        self.manifest_path.parent().and_then(|directory| directory.strip_prefix(root).ok())
    }

    pub fn into_manifest(self) -> PathBuf {
        self.manifest_path
    }

    pub fn dependencies_among<'package>(
        &'package self,
        members: &BTreeMap<&CrateName, &Path>,
    ) -> impl Iterator<Item = &'package CrateName> {
        self.dependencies.iter().map(Dependency::name).filter(|name| members.contains_key(*name))
    }
}
