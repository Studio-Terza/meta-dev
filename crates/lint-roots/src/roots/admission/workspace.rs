use std::collections::{BTreeMap, BTreeSet};
use std::iter;
use std::path::{Path, PathBuf};

use crate::roots::directory::Directory;
use crate::roots::regrouped::Regrouped;
use crate::survey::{Fact, Survey};
use file_ports::Files;
use lint_layout::Failure;
use lint_layout::{Manifest, PathDependencies};

#[derive(Debug)]
pub struct Workspace {
    members: BTreeSet<PathBuf>,
}

impl Workspace {
    pub fn load_many(files: &impl Files, roots: &[PathBuf]) -> Vec<Result<Self, Failure>> {
        let trees = Directory::trees(files, roots);
        let directories: Vec<Vec<PathBuf>> = trees
            .iter()
            .zip(roots)
            .map(|(tree, root)| {
                tree.iter()
                    .flat_map(|below| iter::once(root.as_path()).chain(below.iter().map(AsRef::as_ref)))
                    .map(Path::to_path_buf)
                    .collect()
            })
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
            .map(|(root, manifests)| manifests.map(|manifests| Self::assembled(root, manifests, &named)))
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

    fn assembled(root: &Path, manifests: BTreeMap<PathBuf, Manifest>, named: &Survey) -> Self {
        let kept: Vec<Option<()>> = {
            let listed = manifests
                .iter()
                .filter(|(directory, manifest)| {
                    manifest.package().and(Self::listed(root, directory, &manifests)).is_some()
                })
                .map(|(directory, _)| directory.as_path())
                .collect();
            let canonical = |path: &Path| named.canonical(path).map(Path::to_path_buf);
            let dependencies = manifests
                .iter()
                .map(|(directory, manifest)| (directory.as_path(), manifest.path_dependencies(canonical)))
                .collect();
            let members = Self::with_path_dependencies(listed, &manifests, &dependencies, root);
            manifests.keys().map(|directory| members.get(directory.as_path()).map(|_| ())).collect()
        };
        Self {
            members: manifests.into_keys().zip(kept).filter_map(|(directory, kept)| kept.map(|()| directory)).collect(),
        }
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

    fn with_path_dependencies<'manifests>(
        members: BTreeSet<&'manifests Path>,
        manifests: &'manifests BTreeMap<PathBuf, Manifest>,
        dependencies: &BTreeMap<&Path, PathDependencies>,
        root: &Path,
    ) -> BTreeSet<&'manifests Path> {
        iter::successors(Some(members), |members| {
            let packaged: BTreeSet<&'manifests Path> = members
                .iter()
                .copied()
                .chain([root])
                .filter_map(|directory| dependencies.get(directory))
                .flat_map(|found| found.packaged(manifests))
                .collect();
            let reached: Vec<&'manifests Path> = packaged.difference(members).copied().collect();
            reached.first().map(|_| members.iter().copied().chain(reached.iter().copied()).collect())
        })
        .last()
        .unwrap_or_default()
    }
}
