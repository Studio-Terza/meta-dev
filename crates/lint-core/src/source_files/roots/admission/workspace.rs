use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::failure::Failure;
use crate::manifest::Manifest;
use crate::source_files::roots::directory::Directory;

#[derive(Debug)]
pub struct Workspace {
    members: BTreeSet<PathBuf>,
}

impl Workspace {
    pub fn load(root: &Path) -> Result<Workspace, Failure> {
        let manifests = Self::manifests_under(root)?;
        let members = manifests
            .iter()
            .filter(|(directory, manifest)| manifest.is_package() && Self::is_listed(root, directory, &manifests))
            .map(|(directory, _)| directory.clone())
            .collect();
        Ok(Workspace { members: Self::with_path_dependencies(members, &manifests, root) })
    }

    #[must_use]
    pub fn admits(&self, directory: &Path) -> bool {
        self.members.contains(directory)
    }

    fn manifests_under(root: &Path) -> Result<BTreeMap<PathBuf, Manifest>, Failure> {
        Directory::new(root.to_path_buf()).tree()?.iter().try_fold(BTreeMap::new(), |found, directory| {
            Ok(found
                .into_iter()
                .chain(
                    Manifest::read(directory.as_ref())?.map(|manifest| (manifest.directory().to_path_buf(), manifest)),
                )
                .collect())
        })
    }

    fn is_listed(root: &Path, directory: &Path, manifests: &BTreeMap<PathBuf, Manifest>) -> bool {
        let relative = directory.strip_prefix(root).unwrap_or(directory);
        directory == root
            || manifests.get(root).and_then(Manifest::workspace).is_some_and(|members| members.admits(relative))
    }

    fn with_path_dependencies(
        members: BTreeSet<PathBuf>,
        manifests: &BTreeMap<PathBuf, Manifest>,
        root: &Path,
    ) -> BTreeSet<PathBuf> {
        let reached: BTreeSet<PathBuf> = members
            .iter()
            .map(PathBuf::as_path)
            .chain([root])
            .filter_map(|directory| manifests.get(directory))
            .flat_map(Manifest::path_dependencies)
            .filter(|path| manifests.get(path).is_some_and(Manifest::is_package) && !members.contains(path))
            .collect();
        if reached.is_empty() {
            members
        } else {
            Self::with_path_dependencies(members.into_iter().chain(reached).collect(), manifests, root)
        }
    }
}
