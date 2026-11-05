use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use newtype::{Arbitrary, Newtype};

use crate::manifest::Manifest;

#[derive(Newtype, Arbitrary, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathDependencies(Vec<PathBuf>);

impl PathDependencies {
    pub fn packaged<'manifests>(
        &self,
        manifests: &'manifests BTreeMap<PathBuf, Manifest>,
    ) -> impl Iterator<Item = &'manifests Path> {
        self.0
            .iter()
            .filter_map(|path| manifests.get_key_value(path))
            .filter(|(_, manifest)| manifest.package().is_some())
            .map(|(path, _)| path.as_path())
    }
}
