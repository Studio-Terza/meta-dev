use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::crate_name::CrateName;
use crate::crate_root::CrateRoot;
use file_ports::Files;
use lint_layout::Dependencies;
use lint_layout::KeyLines;
use lint_layout::Manifest;
use lint_layout::Untested;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageRoot {
    root: CrateRoot,
    name: CrateName,
    lines: KeyLines,
    untested: Untested,
    dependencies: Dependencies,
}

impl PackageRoot {
    #[must_use]
    pub fn read_many(files: &impl Files, directories: &[impl AsRef<Path>]) -> Vec<PackageRoot> {
        let manifests: Vec<Option<Manifest>> =
            Manifest::read_many(files, directories).into_iter().map(|read| read.ok().flatten()).collect();
        let named: Vec<PathBuf> = manifests.iter().flatten().flat_map(Manifest::named_paths).collect();
        let found: BTreeMap<PathBuf, PathBuf> = named
            .iter()
            .cloned()
            .zip(files.canonical_many(&named))
            .filter_map(|(path, real)| Some((path, real.ok()?)))
            .collect();
        let canonical = |path: &Path| found.get(path).cloned();
        let workspaces = Self::workspaces(files, directories);
        directories
            .iter()
            .zip(&manifests)
            .map(|(directory, manifest)| {
                let root = CrateRoot::of_manifest(directory.as_ref(), manifest.as_ref());
                let workspace = Self::workspace(&workspaces, directory.as_ref(), manifest.as_ref());
                PackageRoot {
                    dependencies: manifest.as_ref().map(|present| present.dependencies(workspace)).unwrap_or_default(),
                    name: CrateName::from(&root),
                    root,
                    lines: manifest.as_ref().map(AsRef::as_ref).cloned().unwrap_or_default(),
                    untested: manifest.as_ref().map(|present| present.untested_targets(canonical)).unwrap_or_default(),
                }
            })
            .collect()
    }

    fn workspaces(files: &impl Files, directories: &[impl AsRef<Path>]) -> BTreeMap<PathBuf, Manifest> {
        let enclosing: Vec<PathBuf> = directories
            .iter()
            .flat_map(|directory| directory.as_ref().ancestors().skip(1))
            .map(Path::to_path_buf)
            .collect::<BTreeSet<PathBuf>>()
            .into_iter()
            .collect();
        enclosing
            .iter()
            .cloned()
            .zip(Manifest::read_many(files, &enclosing))
            .filter_map(|(directory, read)| Some((directory, read.ok().flatten()?)))
            .filter(|(_, manifest)| manifest.workspace().is_some())
            .collect()
    }

    fn workspace<'read>(
        workspaces: &'read BTreeMap<PathBuf, Manifest>,
        directory: &Path,
        manifest: Option<&'read Manifest>,
    ) -> Option<&'read Manifest> {
        let own = manifest.filter(|candidate| candidate.workspace().is_some());
        own.or_else(|| directory.ancestors().skip(1).find_map(|enclosing| workspaces.get(enclosing)))
    }

    #[must_use]
    pub fn unplaced(root: CrateRoot) -> PackageRoot {
        PackageRoot {
            name: CrateName::from(&root),
            root,
            lines: KeyLines::default(),
            untested: Untested::default(),
            dependencies: Dependencies::default(),
        }
    }

    #[must_use]
    pub fn root(&self) -> &CrateRoot {
        &self.root
    }

    #[must_use]
    pub fn crate_name(&self) -> &CrateName {
        &self.name
    }

    #[must_use]
    pub fn key_lines(&self) -> &KeyLines {
        &self.lines
    }

    #[must_use]
    pub fn untested(&self) -> &Untested {
        &self.untested
    }

    #[must_use]
    pub fn dependencies(&self) -> &Dependencies {
        &self.dependencies
    }
}
