use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::roots::directory::Directory;
use file_ports::Files;
use lint_layout::Failure;
use lint_layout::Manifest;

#[derive(Debug)]
pub struct Workspace {
    members: BTreeSet<PathBuf>,
}

impl Workspace {
    pub fn load(files: &impl Files, root: &Path) -> Result<Workspace, Failure> {
        let manifests = Self::manifests_under(files, root)?;
        let members = manifests
            .iter()
            .filter(|(directory, manifest)| manifest.package().and(Self::listed(root, directory, &manifests)).is_some())
            .map(|(directory, _)| directory.clone())
            .collect();
        Ok(Workspace { members: Self::with_path_dependencies(files, members, &manifests, root) })
    }

    #[must_use]
    pub fn admits(&self, directory: &Path) -> Option<&Path> {
        self.members.get(directory).map(PathBuf::as_path)
    }

    fn manifests_under(files: &impl Files, root: &Path) -> Result<BTreeMap<PathBuf, Manifest>, Failure> {
        Directory::new(root.to_path_buf()).tree(files)?.iter().try_fold(BTreeMap::new(), |found, directory| {
            Ok(found
                .into_iter()
                .chain(
                    Manifest::read(files, directory.as_ref())?
                        .map(|manifest| (manifest.directory().to_path_buf(), manifest)),
                )
                .collect())
        })
    }

    fn listed<'path>(
        root: &Path,
        directory: &'path Path,
        manifests: &BTreeMap<PathBuf, Manifest>,
    ) -> Option<&'path Path> {
        let relative = directory.strip_prefix(root).unwrap_or(directory);
        (directory == root).then_some(directory).or_else(|| {
            manifests
                .get(root)
                .and_then(Manifest::workspace)
                .and_then(|members| members.admits(relative))
                .map(|_| directory)
        })
    }

    fn with_path_dependencies(
        files: &impl Files,
        members: BTreeSet<PathBuf>,
        manifests: &BTreeMap<PathBuf, Manifest>,
        root: &Path,
    ) -> BTreeSet<PathBuf> {
        let reached: BTreeSet<PathBuf> = members
            .iter()
            .map(PathBuf::as_path)
            .chain([root])
            .filter_map(|directory| manifests.get(directory))
            .flat_map(|manifest| manifest.path_dependencies(files))
            .filter(|path| manifests.get(path).and_then(Manifest::package).is_some() && !members.contains(path))
            .collect();
        if reached.is_empty() {
            members
        } else {
            Self::with_path_dependencies(files, members.into_iter().chain(reached).collect(), manifests, root)
        }
    }
}
