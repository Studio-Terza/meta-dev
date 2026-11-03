use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::regrouped::Regrouped;
use crate::roots::directory::Directory;
use crate::survey::{Fact, Survey};
use file_ports::Files;
use lint_layout::Failure;
use lint_layout::Manifest;

#[derive(Debug)]
pub struct Workspace {
    members: BTreeSet<PathBuf>,
}

impl Workspace {
    pub fn load_many(files: &impl Files, roots: &[PathBuf]) -> Vec<Result<Workspace, Failure>> {
        let trees = Directory::trees(files, roots);
        let directories: Vec<Vec<PathBuf>> = trees
            .iter()
            .map(|tree| tree.iter().flatten().map(|directory| directory.as_ref().to_path_buf()).collect())
            .collect();
        let read = Regrouped::like(&directories, Manifest::read_many(files, &directories.concat()));
        let manifests: Vec<Result<BTreeMap<PathBuf, Manifest>, Failure>> =
            trees.into_iter().zip(read).map(|(tree, read)| tree.and_then(|_| Self::keyed(read))).collect();
        let named: Vec<PathBuf> =
            manifests.iter().flatten().flat_map(BTreeMap::values).flat_map(Manifest::named_paths).collect();
        let named = Survey::default().learn(files, Fact::Canonical, &named);
        roots
            .iter()
            .zip(manifests)
            .map(|(root, manifests)| manifests.map(|manifests| Self::assembled(root, &manifests, &named)))
            .collect()
    }

    #[must_use]
    pub fn admits(&self, directory: &Path) -> Option<&Path> {
        self.members.get(directory).map(PathBuf::as_path)
    }

    fn keyed(read: Vec<Result<Option<Manifest>, Failure>>) -> Result<BTreeMap<PathBuf, Manifest>, Failure> {
        let found = read.into_iter().collect::<Result<Vec<Option<Manifest>>, Failure>>()?;
        Ok(found.into_iter().flatten().map(|manifest| (manifest.directory().to_path_buf(), manifest)).collect())
    }

    fn assembled(root: &Path, manifests: &BTreeMap<PathBuf, Manifest>, named: &Survey) -> Workspace {
        let members = manifests
            .iter()
            .filter(|(directory, manifest)| manifest.package().and(Self::listed(root, directory, manifests)).is_some())
            .map(|(directory, _)| directory.clone())
            .collect();
        let canonical = |path: &Path| named.canonical(path).map(Path::to_path_buf);
        let dependencies = manifests
            .iter()
            .map(|(directory, manifest)| (directory.clone(), manifest.path_dependencies(canonical)))
            .collect();
        Workspace { members: Self::with_path_dependencies(members, manifests, &dependencies, root) }
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
        members: BTreeSet<PathBuf>,
        manifests: &BTreeMap<PathBuf, Manifest>,
        dependencies: &BTreeMap<PathBuf, Vec<PathBuf>>,
        root: &Path,
    ) -> BTreeSet<PathBuf> {
        let reached: BTreeSet<PathBuf> = members
            .iter()
            .map(PathBuf::as_path)
            .chain([root])
            .filter_map(|directory| dependencies.get(directory))
            .flatten()
            .filter(|path| manifests.get(*path).and_then(Manifest::package).is_some() && !members.contains(*path))
            .cloned()
            .collect();
        if reached.is_empty() {
            members
        } else {
            Self::with_path_dependencies(members.into_iter().chain(reached).collect(), manifests, dependencies, root)
        }
    }
}
