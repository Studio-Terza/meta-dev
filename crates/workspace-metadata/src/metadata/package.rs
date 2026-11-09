mod declared;
mod dependency;
mod target;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::crate_name::CrateName;
use crate::format::Format;
use crate::package_id::PackageId;
use crate::role::Role;
use declared::Declared;
pub use declared::{Expanders, PortTable};
use dependency::Dependency;
use target::{Kind, Target};

#[derive(Clone, Debug, Deserialize)]
pub struct Package {
    id: Option<PackageId>,
    name: CrateName,
    manifest_path: PathBuf,
    dependencies: Vec<Dependency>,
    metadata: Option<Declared>,
    #[serde(default)]
    targets: Vec<Target>,
}

impl Package {
    pub fn name(&self) -> &CrateName {
        &self.name
    }

    pub fn identity(&self) -> Option<&PackageId> {
        self.id.as_ref()
    }

    pub fn proc_macro(&self) -> Option<&Package> {
        self.targets.iter().flat_map(AsRef::as_ref).find(|kind| **kind == Kind::ProcMacro).map(|_| self)
    }

    pub fn port(&self) -> Option<&PortTable> {
        self.metadata.as_ref().and_then(Declared::port)
    }

    pub fn declares(&self, role: Role) -> Option<Role> {
        self.metadata.as_ref().and_then(|declared| declared.declares(role))
    }

    pub fn format(&self) -> Option<&Format> {
        self.metadata.as_ref().and_then(Declared::format)
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

    pub fn keys<'package>(
        &'package self,
        required: &'package CrateName,
    ) -> impl Iterator<Item = &'package CrateName> + use<'package> {
        self.dependencies.iter().filter_map(|dependency| dependency.key_of(required))
    }

    pub fn dependencies_among<'package>(
        &'package self,
        members: &BTreeMap<&CrateName, &Path>,
    ) -> impl Iterator<Item = &'package CrateName> {
        self.dependencies.iter().map(Dependency::name).filter(|name| members.contains_key(*name))
    }
}
